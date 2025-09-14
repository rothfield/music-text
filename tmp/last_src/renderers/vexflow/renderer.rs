use crate::parse::model::{Directive, Document};
use crate::rhythm::types::{Degree};

#[derive(Debug, Clone)]
struct NoteData {
    element_type: String,
    key: String,
    duration: String,
    dots: u8,
    accidentals: Vec<serde_json::Value>,
}

/// VexFlow renderer that works directly with Documents (like LilyPond does)
pub struct VexFlowRenderer {
}

impl VexFlowRenderer {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Render VexFlow data directly from Document structure (follows lilypond pattern)
    pub fn render_data_from_document(&self, document: &Document) -> serde_json::Value {
        let mut staves_data = Vec::new();
        
        // Convert each stave using Beat structures for tuplets
        for stave in &document.staves {
            let notes = if let Some(rhythm_items) = &stave.rhythm_items {
                process_items(rhythm_items)
            } else {
                // Fallback to old method if no rhythm_items
                process_stave_with_beaming(&stave.content_line)
            };
            
            staves_data.push(serde_json::json!({
                "notes": notes,
                "key_signature": "C"
            }));
        }
        
        serde_json::json!({
            "staves": staves_data,
            "title": document.directives.iter().find_map(|d| {
                if d.key == "title" { Some(&d.value) } else { None }
            }),
            "author": document.directives.iter().find_map(|d| {
                if d.key == "author" { Some(&d.value) } else { None }
            }),
            "time_signature": "4/4",
            "clef": "treble",
            "key_signature": "C"
        })
    }
    
    /// Render VexFlow SVG directly from Document structure (follows lilypond pattern)
    pub fn render_svg_from_document(&self, document: &Document) -> String {
        render_vexflow_svg_from_document_with_directives(document, &document.directives)
    }
}

impl Default for VexFlowRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Process rhythm items to VexFlow JSON (follows old working pattern)
fn process_items(rhythm_items: &[crate::rhythm::Item]) -> Vec<serde_json::Value> {
    let mut notes = Vec::new();
    
    for item in rhythm_items {
        match item {
            crate::rhythm::Item::Beat(beat) => {
                if beat.is_tuplet {
                    // Handle tuplet as a group (same as old code)
                    let tuplet_notes = convert_beat_to_vexflow_elements(beat);
                    notes.push(serde_json::json!({
                        "type": "Tuplet",
                        "divisions": beat.divisions,
                        "ratio": beat.tuplet_ratio,
                        "notes": tuplet_notes
                    }));
                } else {
                    // Regular beat - add elements directly (same as old code)
                    let beat_elements = convert_beat_to_vexflow_elements(beat);
                    notes.extend(beat_elements);
                }
            },
            crate::rhythm::Item::Barline(barline_type, _) => {
                notes.push(serde_json::json!({
                    "type": "BarLine", 
                    "bar_type": format!("{:?}", barline_type)
                }));
            },
            crate::rhythm::Item::Breathmark => {
                notes.push(serde_json::json!({
                    "type": "Breathmark"
                }));
            },
            crate::rhythm::Item::Tonic(_) => {
                // Tonic doesn't generate visual elements
            }
        }
    }
    
    notes
}

/// Convert a beat to VexFlow elements with beaming (follows old working code)
fn convert_beat_to_vexflow_elements(beat: &crate::rhythm::Beat) -> Vec<serde_json::Value> {
    let mut elements = Vec::new();
    
    // Apply beaming logic: beam consecutive beamable notes within beat
    let beaming_info = analyze_beat_for_beaming(beat);
    
    for (element_index, beat_element) in beat.elements.iter().enumerate() {
        match &beat_element.event {
            crate::rhythm::Event::Note { degree, octave, .. } => {
                let (key, accidentals) = degree_to_vexflow_key(*degree, *octave);
                let (duration, dots) = convert_fraction_to_vexflow(beat_element.tuplet_duration);
                
                // Determine beaming for this note (from old logic)
                let is_beamable_note = beaming_info.beamable_notes.contains(&element_index);
                let beam_start = beaming_info.should_beam && is_beamable_note && 
                                Some(element_index) == beaming_info.beamable_notes.first().copied();
                let beam_end = beaming_info.should_beam && is_beamable_note && 
                              Some(element_index) == beaming_info.beamable_notes.last().copied();
                
                elements.push(serde_json::json!({
                    "type": "Note",
                    "keys": [key],
                    "duration": duration,
                    "dots": dots,
                    "accidentals": accidentals,
                    "tied": false,
                    "beam_start": beam_start,
                    "beam_end": beam_end
                }));
            },
            crate::rhythm::Event::Rest => {
                let (duration, dots) = convert_fraction_to_vexflow(beat_element.tuplet_duration);
                
                elements.push(serde_json::json!({
                    "type": "Rest",
                    "duration": duration,
                    "dots": dots
                }));
            },
            crate::rhythm::Event::Unknown { .. } => {
                // Skip unknown tokens
            }
        }
    }
    
    elements
}

/// Beaming information for notes within a beat (from old working code)
#[derive(Debug)]
struct BeamingInfo {
    beamable_notes: Vec<usize>, // indices of beamable notes in this beat
    should_beam: bool,          // whether these notes should form a beam
}

/// Analyze beat for beaming (from old working code)
fn analyze_beat_for_beaming(beat: &crate::rhythm::Beat) -> BeamingInfo {
    let mut beamable_notes = Vec::new();
    
    // Find beamable notes (8th, 16th, 32nd notes)
    for (index, element) in beat.elements.iter().enumerate() {
        match &element.event {
            crate::rhythm::Event::Note { .. } => {
                let (duration_str, _) = convert_fraction_to_vexflow(element.tuplet_duration);
                let is_beamable = matches!(duration_str.as_str(), "8" | "16" | "32");
                if is_beamable {
                    beamable_notes.push(index);
                }
            },
            _ => {
                // Non-notes don't affect beaming within the beat
            }
        }
    }
    
    // Need at least 2 beamable notes for a beam
    let should_beam = beamable_notes.len() >= 2;
    
    BeamingInfo {
        beamable_notes,
        should_beam,
    }
}

/// Convert Fraction to VexFlow duration using shared function
fn convert_fraction_to_vexflow(duration: fraction::Fraction) -> (String, u8) {
    let num = *duration.numer().unwrap() as usize;
    let den = *duration.denom().unwrap() as usize;
    
    // Use shared fraction conversion logic (same as LilyPond)
    match (num, den) {
        // Basic durations
        (1, 1) => ("w".to_string(), 0),     // whole
        (1, 2) => ("h".to_string(), 0),     // half
        (1, 4) => ("q".to_string(), 0),     // quarter
        (1, 8) => ("8".to_string(), 0),     // eighth
        (1, 16) => ("16".to_string(), 0),   // sixteenth
        (1, 32) => ("32".to_string(), 0),   // thirty-second
        (1, 64) => ("64".to_string(), 0),   // sixty-fourth
        
        // Single-dotted durations (3/2 of basic duration)
        (3, 2) => ("w".to_string(), 1),     // dotted whole
        (3, 4) => ("h".to_string(), 1),     // dotted half
        (3, 8) => ("q".to_string(), 1),     // dotted quarter
        (3, 16) => ("8".to_string(), 1),    // dotted eighth
        (3, 32) => ("16".to_string(), 1),   // dotted sixteenth
        (3, 64) => ("32".to_string(), 1),   // dotted thirty-second
        
        // Double-dotted durations (7/4 of basic duration)
        (7, 4) => ("w".to_string(), 2),     // double-dotted whole
        (7, 8) => ("h".to_string(), 2),     // double-dotted half  
        (7, 16) => ("q".to_string(), 2),    // double-dotted quarter
        (7, 32) => ("8".to_string(), 2),    // double-dotted eighth
        (7, 64) => ("16".to_string(), 2),   // double-dotted sixteenth
        
        // Default to quarter note for unknown fractions
        _ => ("q".to_string(), 0),
    }
}

/// Process stave content with beaming analysis (following old vexflow logic)
fn process_stave_with_beaming(content_line: &[crate::rhythm::types::ParsedElement]) -> Vec<serde_json::Value> {
    let mut notes = Vec::new();
    let mut note_indices = Vec::new(); // Track which elements are notes for beaming
    
    // First pass: convert elements and track note positions
    for (index, element) in content_line.iter().enumerate() {
        match element {
            crate::rhythm::types::ParsedElement::Note { degree, octave, duration, .. } => {
                let (key, accidentals) = degree_to_vexflow_key(*degree, *octave);
                let (vf_duration, dots) = if let Some((num, den)) = duration {
                    let frac = fraction::Fraction::new(*num as u64, *den as u64);
                    convert_fraction_to_vexflow(frac)
                } else {
                    ("q".to_string(), 0)
                };
                
                notes.push(serde_json::json!({
                    "type": "Note",
                    "keys": [key],
                    "duration": vf_duration,
                    "dots": dots,
                    "accidentals": accidentals,
                    "tied": false,
                    "beam_start": false,
                    "beam_end": false
                }));
                note_indices.push(index);
            },
            crate::rhythm::types::ParsedElement::Rest { duration, .. } => {
                let (vf_duration, dots) = if let Some((num, den)) = duration {
                    let frac = fraction::Fraction::new(*num as u64, *den as u64);
                    convert_fraction_to_vexflow(frac)
                } else {
                    ("q".to_string(), 0)
                };
                
                notes.push(serde_json::json!({
                    "type": "Rest",
                    "duration": vf_duration,
                    "dots": dots
                }));
            },
            crate::rhythm::types::ParsedElement::Barline { style, .. } => {
                notes.push(serde_json::json!({
                    "type": "BarLine",
                    "bar_type": style
                }));
            },
            _ => {} // Skip whitespace, unknown tokens, etc.
        }
    }
    
    // Second pass: analyze beaming (following old vexflow logic)
    let beamable_groups = analyze_beaming(&notes);
    
    // Third pass: apply beaming flags
    for group in beamable_groups {
        if group.len() >= 2 {
            // Set beam_start on first note, beam_end on last note
            if let Some(first_idx) = group.first() {
                if let Some(note) = notes.get_mut(*first_idx) {
                    if let Some(obj) = note.as_object_mut() {
                        obj.insert("beam_start".to_string(), serde_json::Value::Bool(true));
                    }
                }
            }
            if let Some(last_idx) = group.last() {
                if let Some(note) = notes.get_mut(*last_idx) {
                    if let Some(obj) = note.as_object_mut() {
                        obj.insert("beam_end".to_string(), serde_json::Value::Bool(true));
                    }
                }
            }
        }
    }
    
    notes
}

/// Analyze notes for beaming groups (following old vexflow logic)
fn analyze_beaming(notes: &[serde_json::Value]) -> Vec<Vec<usize>> {
    let mut beaming_groups = Vec::new();
    let mut current_group = Vec::new();
    
    for (index, note) in notes.iter().enumerate() {
        if let Some(note_obj) = note.as_object() {
            if note_obj.get("type").and_then(|v| v.as_str()) == Some("Note") {
                if let Some(duration) = note_obj.get("duration").and_then(|v| v.as_str()) {
                    // Old code beamed 8th, 16th, 32nd notes  
                    let is_beamable = matches!(duration, "8" | "16" | "32");
                    
                    if is_beamable {
                        current_group.push(index);
                    } else {
                        // Non-beamable note ends current group
                        if current_group.len() >= 2 {
                            beaming_groups.push(current_group.clone());
                        }
                        current_group.clear();
                    }
                }
            } else {
                // Non-note (rest, barline) ends current group
                if current_group.len() >= 2 {
                    beaming_groups.push(current_group.clone());
                }
                current_group.clear();
            }
        }
    }
    
    // Don't forget the last group
    if current_group.len() >= 2 {
        beaming_groups.push(current_group);
    }
    
    beaming_groups
}

/// Convert document to VexFlow SVG with title/author from directives
fn render_vexflow_svg_from_document_with_directives(document: &Document, directives: &[Directive]) -> String {
    // Extract title and author from directives
    let mut title = None;
    let mut author = None;
    for directive in directives {
        match directive.key.as_str() {
            "title" => title = Some(directive.value.clone()),
            "author" => author = Some(directive.value.clone()),
            _ => {}
        }
    }
    
    // Use the VexFlow JSON to render SVG
    let mut svg = String::new();
    svg.push_str(r#"<svg width="800" height="300" xmlns="http://www.w3.org/2000/svg">"#);
    svg.push_str("\n");
    
    // Background
    svg.push_str("  <rect width=\"800\" height=\"300\" fill=\"#fafafa\" stroke=\"#333\" stroke-width=\"1\"/>");
    svg.push_str("\n");
    
    // Title and Author
    if let Some(title_text) = title {
        svg.push_str(&format!("  <text x=\"400\" y=\"25\" font-family=\"serif\" font-size=\"18\" font-weight=\"bold\" fill=\"#333\" text-anchor=\"middle\">{}</text>", 
            html_escape(&title_text)));
        svg.push_str("\n");
    }
    if let Some(author_text) = author {
        svg.push_str(&format!("  <text x=\"400\" y=\"45\" font-family=\"serif\" font-size=\"14\" fill=\"#666\" text-anchor=\"middle\">{}</text>", 
            html_escape(&author_text)));
        svg.push_str("\n");
    }
    
    // Staff lines
    let staff_y = 80;
    let staff_width = 700;
    let staff_x = 50;
    
    for i in 0..5 {
        let y = staff_y + i * 10;
        svg.push_str(&format!("  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#333\" stroke-width=\"1\"/>", 
            staff_x, y, staff_x + staff_width, y));
        svg.push_str("\n");
    }
    
    // Treble clef
    svg.push_str(&format!("  <text x=\"{}\" y=\"{}\" font-family=\"serif\" font-size=\"40\" fill=\"#333\">𝄞</text>", 
        staff_x + 10, staff_y + 25));
    svg.push_str("\n");
    
    // Time signature
    svg.push_str(&format!("  <text x=\"{}\" y=\"{}\" font-family=\"serif\" font-size=\"16\" fill=\"#333\">4</text>", 
        staff_x + 50, staff_y));
    svg.push_str("\n");
    svg.push_str(&format!("  <text x=\"{}\" y=\"{}\" font-family=\"serif\" font-size=\"16\" fill=\"#333\">4</text>", 
        staff_x + 50, staff_y + 20));
    svg.push_str("\n");
    
    // Enhanced rendering notice
    svg.push_str("  <text x=\"20\" y=\"50\" font-family=\"Arial\" font-size=\"12\" fill=\"#666\">Enhanced VexFlow data available in JSON - use JavaScript renderer for full features</text>");
    svg.push_str("\n");
    
    // Footer
    svg.push_str("  <text x=\"20\" y=\"280\" font-family=\"serif\" font-size=\"10\" fill=\"#999\">Generated by Music Text Parser • Professional VexFlow rendering via JSON</text>");
    svg.push_str("\n");
    
    svg.push_str("</svg>");
    svg
}

/// Escape HTML special characters
fn html_escape(s: &str) -> String {
    s.chars().map(|c| match c {
        '&' => "&amp;".to_string(),
        '<' => "&lt;".to_string(),
        '>' => "&gt;".to_string(),
        '"' => "&quot;".to_string(),
        '\'' => "&#39;".to_string(),
        _ => c.to_string(),
    }).collect()
}

/// Convert Degree and octave to VexFlow key format
fn degree_to_vexflow_key(degree: Degree, octave: i8) -> (String, Vec<serde_json::Value>) {
    use Degree::*;
    
    // VexFlow octave: 4 = middle C
    let vexflow_octave = octave + 4;
    
    let (note_name, accidental_str) = match degree {
        // Scale degree 1 (Do/Sa/C)
        N1bb => ("c", Some("bb")), N1b => ("c", Some("b")), N1 => ("c", None),
        N1s => ("c", Some("#")), N1ss => ("c", Some("##")),
        // Scale degree 2 (Re/D)  
        N2bb => ("d", Some("bb")), N2b => ("d", Some("b")), N2 => ("d", None),
        N2s => ("d", Some("#")), N2ss => ("d", Some("##")),
        // Scale degree 3 (Mi/Ga/E)
        N3bb => ("e", Some("bb")), N3b => ("e", Some("b")), N3 => ("e", None),
        N3s => ("e", Some("#")), N3ss => ("e", Some("##")),
        // Scale degree 4 (Fa/Ma/F)
        N4bb => ("f", Some("bb")), N4b => ("f", Some("b")), N4 => ("f", None),
        N4s => ("f", Some("#")), N4ss => ("f", Some("##")),
        // Scale degree 5 (Sol/Pa/G)
        N5bb => ("g", Some("bb")), N5b => ("g", Some("b")), N5 => ("g", None),
        N5s => ("g", Some("#")), N5ss => ("g", Some("##")),
        // Scale degree 6 (La/Dha/A)
        N6bb => ("a", Some("bb")), N6b => ("a", Some("b")), N6 => ("a", None),
        N6s => ("a", Some("#")), N6ss => ("a", Some("##")),
        // Scale degree 7 (Ti/Ni/B)
        N7bb => ("b", Some("bb")), N7b => ("b", Some("b")), N7 => ("b", None),
        N7s => ("b", Some("#")), N7ss => ("b", Some("##")),
    };
    
    let key = if let Some(acc) = accidental_str {
        format!("{}{}/{}", note_name, acc, vexflow_octave)
    } else {
        format!("{}/{}", note_name, vexflow_octave)
    };
    
    // Create VexFlow accidental objects for visual rendering
    let accidentals = if let Some(acc) = accidental_str {
        vec![serde_json::json!({
            "index": 0,
            "accidental": acc
        })]
    } else {
        vec![]
    };
    
    (key, accidentals)
}

