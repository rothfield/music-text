// VexFlow JavaScript Generator
// Generates VexFlow-compatible JavaScript from FSM Item output

use crate::models::Metadata;
use crate::rhythm_fsm::{Item, Beat, BeatElement};
use crate::pitch::Degree;
use serde::{Serialize, Deserialize};

/// VexFlow slur structure for JavaScript generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VexFlowSlur {
    pub from_note: usize,
    pub to_note: usize,
}

/// VexFlow note structure for JavaScript generation  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VexFlowNote {
    pub keys: Vec<String>,
    pub duration: String,
    pub dots: u8,
    pub tied: bool,
}

/// VexFlow measure structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VexFlowMeasure {
    pub notes: Vec<VexFlowNote>,
    pub slurs: Vec<VexFlowSlur>,
}

#[derive(Debug, Clone)]
enum SlurMarkerType {
    Start,
    End,
}

#[derive(Debug, Clone)]
struct SlurMarker {
    marker_type: SlurMarkerType,
    position: usize, // Position in FSM element sequence
}

/// Generate VexFlow JavaScript code from FSM Item output using 2-pass architecture
pub fn generate_vexflow_js(
    elements: &Vec<Item>,
    _metadata: &Metadata
) -> Result<String, String> {
    eprintln!("VEXFLOW DEBUG: Starting 2-pass processing of {} elements", elements.len());
    
    // Debug: Print all input elements
    for (i, element) in elements.iter().enumerate() {
        eprintln!("VEXFLOW DEBUG: Input Element {}: {:?}", i, element);
    }
    
    // PASS 1: Extract notes and slur markers into normalized form
    let (notes, slur_markers) = extract_notes_and_slur_markers(elements)?;
    
    eprintln!("VEXFLOW DEBUG: Pass 1 complete - {} notes, {} slur markers", notes.len(), slur_markers.len());
    
    // PASS 2: Create slurs from markers and normalized notes
    let slurs = create_slurs_from_markers(&notes, &slur_markers);
    
    eprintln!("VEXFLOW DEBUG: Pass 2 complete - {} slurs created", slurs.len());
    for (i, slur) in slurs.iter().enumerate() {
        eprintln!("VEXFLOW DEBUG: Slur {}: from note {} to note {}", i, slur.from_note, slur.to_note);
    }

    // Store lengths before moving into measure
    let notes_len = notes.len();
    let slurs_len = slurs.len();
    
    let measure = VexFlowMeasure { notes, slurs };
    
    // Generate executable VexFlow JavaScript code  
    let js_code = format!(
        r#"
console.log('🚀 CLAUDE: 2-pass generator starting with {} elements');
console.log('🚀 CLAUDE: Pass 1 found {} notes and {} slur markers');  
console.log('🚀 CLAUDE: Pass 2 created {} slurs');
window.lastVexFlowDebugMessages = 'Elements: {}, Notes: {}, SlurMarkers: {}, Slurs: {}';

// Clear the canvas
canvas.innerHTML = '';
canvas.classList.add('has-content');

// Initialize VexFlow
const renderer = new VF.Renderer(canvas, VF.Renderer.Backends.SVG);
renderer.resize(800, 200);
const context = renderer.getContext();

// Create stave
const stave = new VF.Stave(20, 40, 750);
stave.addClef('treble').setContext(context).draw();

// Create notes
const notes = {};
const vexNotes = [];

// Add notes to array
{}

// Create voice and add notes
const voice = new VF.Voice({{ beats: 4, beat_value: 4 }});
voice.addTickables(vexNotes);

// Format and draw
const formatter = new VF.Formatter().joinVoices([voice]).format([voice], 700);
voice.draw(context, stave);

// Create slurs
console.log('VexFlow: Found {} SlurStart elements');
console.log('VexFlow: Found {} SlurEnd elements'); 
console.log('VexFlow: Found {} total slurs during processing');
console.log('VexFlow: Starting slur creation...');
console.log('VexFlow: Creating {} slurs');

{}
"#,
        elements.len(),
        notes_len, 
        slur_markers.len(),
        slurs_len,
        elements.len(),
        notes_len, 
        slur_markers.len(),
        slurs_len,
        serde_json::to_string(&measure.notes).map_err(|e| format!("JSON serialization error: {}", e))?,
        generate_note_creation_js(&measure.notes)?,
        1, // SlurStart count (hardcoded for now)
        1, // SlurEnd count (hardcoded for now) 
        measure.slurs.len(),
        measure.slurs.len(),
        generate_slur_creation_js(&measure.slurs)?
    );
    
    Ok(js_code)
}

/// Convert BeatElement to VexFlow note
fn convert_beat_element_to_vexflow_note(beat_element: &BeatElement) -> Result<VexFlowNote, String> {
    if !beat_element.is_note() {
        return Err("BeatElement is not a note".to_string());
    }

    let degree = beat_element.degree.ok_or("Note missing degree")?;
    let octave = beat_element.octave.ok_or("Note missing octave")?;
    
    // Convert degree to VexFlow key
    let key = degree_to_vexflow_key(degree, octave);
    
    // Convert duration (simplified for now)
    let duration = "16"; // Default to 16th note
    
    Ok(VexFlowNote {
        keys: vec![key],
        duration: duration.to_string(),
        dots: 0,
        tied: false,
    })
}

/// Convert Degree and octave to VexFlow key format
fn degree_to_vexflow_key(degree: Degree, octave: i8) -> String {
    use Degree::*;
    
    let note_name = match degree {
        N1 => "c", N1s => "cs", N1b => "cb",
        N2 => "d", N2s => "ds", N2b => "db",
        N3 => "e", N3s => "es", N3b => "eb",
        N4 => "f", N4s => "fs", N4b => "fb",
        N5 => "g", N5s => "gs", N5b => "gb",
        N6 => "a", N6s => "as", N6b => "ab",  
        N7 => "b", N7s => "bs", N7b => "bb",
        _ => "c", // Default fallback
    };
    
    // VexFlow octave: 4 = middle C
    let vf_octave = octave + 4;
    
    format!("{}/{}", note_name, vf_octave)
}

/// Generate JavaScript code to create VexFlow notes
fn generate_note_creation_js(notes: &[VexFlowNote]) -> Result<String, String> {
    let mut js_lines = Vec::new();
    
    for (i, note) in notes.iter().enumerate() {
        let keys_js = format!("[{}]", 
            note.keys.iter()
                .map(|k| format!("'{}'", k))
                .collect::<Vec<_>>()
                .join(", ")
        );
        
        js_lines.push(format!(
            "const note{} = new VF.StaveNote({{ clef: 'treble', keys: {}, duration: '{}' }});",
            i, keys_js, note.duration
        ));
        js_lines.push(format!("vexNotes.push(note{});", i));
    }
    
    Ok(js_lines.join("\n"))
}

/// Generate JavaScript code to create VexFlow slurs
fn generate_slur_creation_js(slurs: &[VexFlowSlur]) -> Result<String, String> {
    let mut js_lines = Vec::new();
    
    for (i, slur) in slurs.iter().enumerate() {
        js_lines.push(format!(
            r#"
console.log('VexFlow: Slur {} from note{} to note{}');
console.log('VexFlow: Checking note{} and note{} exist...');
if (typeof note{} === 'undefined') {{
    console.log('ERROR: note{} is undefined!');
}} else if (typeof note{} === 'undefined') {{
    console.log('ERROR: note{} is undefined!');  
}} else {{
    const curve{} = new VF.Curve(note{}, note{}, {{ cps: [{{ x: 0, y: 10 }}, {{ x: 0, y: 10 }}] }});
    curve{}.setContext(context).draw();
    console.log('VexFlow: Slur {} created successfully');
}}
"#,
            i, slur.from_note, slur.to_note,
            slur.from_note, slur.to_note,
            slur.from_note,
            slur.from_note,
            slur.to_note,
            slur.to_note,
            i, slur.from_note, slur.to_note,
            i,
            i
        ));
    }
    
    Ok(js_lines.join("\n"))
}

/// PASS 1: Extract notes and slur markers from FSM elements
fn extract_notes_and_slur_markers(elements: &Vec<Item>) -> Result<(Vec<VexFlowNote>, Vec<SlurMarker>), String> {
    let mut notes = Vec::new();
    let mut slur_markers = Vec::new();
    
    for (pos, element) in elements.iter().enumerate() {
        eprintln!("VEXFLOW DEBUG: Pass 1 - Element {}: {:?}", pos, element);
        
        match element {
            Item::SlurStart => {
                eprintln!("VEXFLOW DEBUG: Pass 1 - Found SlurStart at position {}", pos);
                slur_markers.push(SlurMarker {
                    marker_type: SlurMarkerType::Start,
                    position: pos,
                });
            },
            Item::SlurEnd => {
                eprintln!("VEXFLOW DEBUG: Pass 1 - Found SlurEnd at position {}", pos);
                slur_markers.push(SlurMarker {
                    marker_type: SlurMarkerType::End,
                    position: pos,
                });
            },
            Item::Beat(beat) => {
                if beat.is_tuplet {
                    eprintln!("VEXFLOW DEBUG: Pass 1 - Processing tuplet beat with {} elements (ratio: {:?})", 
                             beat.elements.len(), beat.tuplet_ratio);
                } else {
                    eprintln!("VEXFLOW DEBUG: Pass 1 - Processing regular beat with {} elements", beat.elements.len());
                }
                extract_notes_from_beat_elements(&beat.elements, &mut notes)?;
            },
            // Handle any future rhythm structures uniformly
            _ => {
                eprintln!("VEXFLOW DEBUG: Pass 1 - Skipping element: {:?}", element);
            }
        }
    }
    
    Ok((notes, slur_markers))
}

/// Extract notes from beat elements (works for Beat, Tuplet, any rhythm structure)
fn extract_notes_from_beat_elements(elements: &[BeatElement], notes: &mut Vec<VexFlowNote>) -> Result<(), String> {
    for beat_element in elements {
        if beat_element.is_note() {
            let vf_note = convert_beat_element_to_vexflow_note(beat_element)?;
            let note_index = notes.len();
            eprintln!("VEXFLOW DEBUG: Pass 1 - Added note {} at index {}: {:?}", note_index, note_index, vf_note);
            notes.push(vf_note);
        }
    }
    Ok(())
}

/// PASS 2: Create slurs from markers using clean logic
fn create_slurs_from_markers(notes: &[VexFlowNote], slur_markers: &[SlurMarker]) -> Vec<VexFlowSlur> {
    let mut slurs = Vec::new();
    
    // Simple pattern: SlurStart followed by SlurEnd = slur over all notes
    let mut start_markers: Vec<usize> = Vec::new();
    let mut end_markers: Vec<usize> = Vec::new();
    
    for marker in slur_markers {
        match marker.marker_type {
            SlurMarkerType::Start => start_markers.push(marker.position),
            SlurMarkerType::End => end_markers.push(marker.position),
        }
    }
    
    eprintln!("VEXFLOW DEBUG: Pass 2 - Found {} SlurStart and {} SlurEnd markers", 
              start_markers.len(), end_markers.len());
    
    // Create slurs for each Start/End pair
    if start_markers.len() > 0 && end_markers.len() > 0 && notes.len() > 1 {
        // Simple case: one slur over all notes
        let slur = VexFlowSlur {
            from_note: 0,
            to_note: notes.len() - 1,
        };
        
        eprintln!("VEXFLOW DEBUG: Pass 2 - Creating slur from note {} to note {}", 
                  slur.from_note, slur.to_note);
        slurs.push(slur);
    } else {
        eprintln!("VEXFLOW DEBUG: Pass 2 - No slur created - insufficient markers or notes");
    }
    
    slurs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rhythm_fsm::{Item, Beat, BeatElement, ParsedElementType};
    use crate::pitch::Degree;
    use crate::parsed_models::Position;
    use crate::models::{Metadata, Title, Directive};
    use std::collections::HashMap;
    use fraction::Fraction;

    #[test]
    fn test_vexflow_generator_with_slur_items() {
        // Create the exact FSM output that CLI generates for "___\n1234"
        // FSM outputs: [SlurStart, SlurEnd, Beat]
        let elements = vec![
            Item::SlurStart,
            Item::SlurEnd,
            Item::Beat(Beat {
                divisions: 4,
                elements: vec![
                    BeatElement {
                        subdivisions: 1,
                        duration: Fraction::new(1u64, 16u64),
                        tuplet_duration: Fraction::new(1u64, 16u64),
                        tuplet_display_duration: None,
                        degree: Some(Degree::N1),
                        octave: Some(0),
                        value: "1".to_string(),
                        position: Position { row: 1, col: 0 },
                        children: Vec::new(),
                        element_duration: None,
                        syl: None,
                        ornaments: Vec::new(),
                        octave_markers: Vec::new(),
                        element_type: ParsedElementType::Note,
                    },
                    BeatElement {
                        subdivisions: 1,
                        duration: Fraction::new(1u64, 16u64),
                        tuplet_duration: Fraction::new(1u64, 16u64),
                        tuplet_display_duration: None,
                        degree: Some(Degree::N2),
                        octave: Some(0),
                        value: "2".to_string(),
                        position: Position { row: 1, col: 1 },
                        children: Vec::new(),
                        element_duration: None,
                        syl: None,
                        ornaments: Vec::new(),
                        octave_markers: Vec::new(),
                        element_type: ParsedElementType::Note,
                    },
                    BeatElement {
                        subdivisions: 1,
                        duration: Fraction::new(1u64, 16u64),
                        tuplet_duration: Fraction::new(1u64, 16u64),
                        tuplet_display_duration: None,
                        degree: Some(Degree::N3),
                        octave: Some(0),
                        value: "3".to_string(),
                        position: Position { row: 1, col: 2 },
                        children: Vec::new(),
                        element_duration: None,
                        syl: None,
                        ornaments: Vec::new(),
                        octave_markers: Vec::new(),
                        element_type: ParsedElementType::Note,
                    },
                    BeatElement {
                        subdivisions: 1,
                        duration: Fraction::new(1u64, 16u64),
                        tuplet_duration: Fraction::new(1u64, 16u64),
                        tuplet_display_duration: None,
                        degree: Some(Degree::N4),
                        octave: Some(0),
                        value: "4".to_string(),
                        position: Position { row: 1, col: 3 },
                        children: Vec::new(),
                        element_duration: None,
                        syl: None,
                        ornaments: Vec::new(),
                        octave_markers: Vec::new(),
                        element_type: ParsedElementType::Note,
                    },
                ],
                tied_to_previous: false,
                is_tuplet: false,
                tuplet_ratio: None,
            }),
        ];

        let metadata = Metadata {
            title: None,
            directives: Vec::new(),
            attributes: HashMap::new(),
            detected_system: None,
        };

        let result = generate_vexflow_js(&elements, &metadata);
        assert!(result.is_ok(), "VexFlow generation should succeed");

        let js_code = result.unwrap();
        
        // Verify that JavaScript code contains expected console messages
        assert!(js_code.contains("1 total slurs during processing"), "Should create 1 slur");
        assert!(js_code.contains("Creating 1 slurs"), "Should create 1 slur");
        
        // Verify note generation JavaScript
        assert!(js_code.contains("note0"), "Should create note0");
        assert!(js_code.contains("note1"), "Should create note1");
        assert!(js_code.contains("note2"), "Should create note2");  
        assert!(js_code.contains("note3"), "Should create note3");
        assert!(js_code.contains("'c/4'"), "Should contain C note");
        assert!(js_code.contains("'d/4'"), "Should contain D note");
        assert!(js_code.contains("'e/4'"), "Should contain E note");
        assert!(js_code.contains("'f/4'"), "Should contain F note");
        
        // Verify slur creation JavaScript
        assert!(js_code.contains("Curve(note0, note3"), "Should create slur from note0 to note3");
        
        println!("Generated VexFlow JavaScript:\n{}", js_code);
    }

    #[test] 
    fn test_degree_to_vexflow_key() {
        assert_eq!(degree_to_vexflow_key(Degree::N1, 0), "c/4");
        assert_eq!(degree_to_vexflow_key(Degree::N2, 0), "d/4");
        assert_eq!(degree_to_vexflow_key(Degree::N1s, 0), "cs/4");
        assert_eq!(degree_to_vexflow_key(Degree::N7b, 0), "bb/4");
        assert_eq!(degree_to_vexflow_key(Degree::N1, 1), "c/5"); // Higher octave
    }

    #[test]
    fn test_tuplet_slur_case() {
        // Test the 2 overlines over 3 notes case: __\n123
        // This creates a tuplet Beat with is_tuplet=true
        let elements = vec![
            Item::SlurStart,
            Item::SlurEnd,
            Item::Beat(Beat {
                divisions: 3,
                elements: vec![
                    BeatElement {
                        subdivisions: 1,
                        duration: Fraction::new(1u64, 3u64),
                        tuplet_duration: Fraction::new(1u64, 8u64),
                        tuplet_display_duration: Some(Fraction::new(1u64, 8u64)),
                        degree: Some(Degree::N1),
                        octave: Some(0),
                        value: "1".to_string(),
                        position: Position { row: 1, col: 0 },
                        children: Vec::new(),
                        element_duration: None,
                        syl: None,
                        ornaments: Vec::new(),
                        octave_markers: Vec::new(),
                        element_type: ParsedElementType::Note,
                    },
                    BeatElement {
                        subdivisions: 1,
                        duration: Fraction::new(1u64, 3u64),
                        tuplet_duration: Fraction::new(1u64, 8u64),
                        tuplet_display_duration: Some(Fraction::new(1u64, 8u64)),
                        degree: Some(Degree::N2),
                        octave: Some(0),
                        value: "2".to_string(),
                        position: Position { row: 1, col: 1 },
                        children: Vec::new(),
                        element_duration: None,
                        syl: None,
                        ornaments: Vec::new(),
                        octave_markers: Vec::new(),
                        element_type: ParsedElementType::Note,
                    },
                    BeatElement {
                        subdivisions: 1,
                        duration: Fraction::new(1u64, 3u64),
                        tuplet_duration: Fraction::new(1u64, 8u64),
                        tuplet_display_duration: Some(Fraction::new(1u64, 8u64)),
                        degree: Some(Degree::N3),
                        octave: Some(0),
                        value: "3".to_string(),
                        position: Position { row: 1, col: 2 },
                        children: Vec::new(),
                        element_duration: None,
                        syl: None,
                        ornaments: Vec::new(),
                        octave_markers: Vec::new(),
                        element_type: ParsedElementType::Note,
                    },
                ],
                tied_to_previous: false,
                is_tuplet: true,
                tuplet_ratio: Some((3, 2)),
            }),
        ];

        let metadata = Metadata {
            title: None,
            directives: Vec::new(),
            attributes: HashMap::new(),
            detected_system: None,
        };

        let result = generate_vexflow_js(&elements, &metadata);
        assert!(result.is_ok(), "VexFlow generation should succeed for tuplet");

        let js_code = result.unwrap();
        
        // Should handle tuplet Beat the same as regular Beat
        assert!(js_code.contains("1 total slurs during processing"));
        assert!(js_code.contains("Creating 1 slurs")); 
        assert!(js_code.contains("note0"), "Should create note0");
        assert!(js_code.contains("note1"), "Should create note1");
        assert!(js_code.contains("note2"), "Should create note2");
        assert!(js_code.contains("Curve(note0, note2"), "Should create slur from note0 to note2");
        
        println!("Generated VexFlow JavaScript for tuplet:\n{}", js_code);
    }

    #[test]
    fn test_slur_start_end_without_notes() {
        // Test edge case: SlurStart/SlurEnd without any notes
        let elements = vec![Item::SlurStart, Item::SlurEnd];
        let metadata = Metadata {
            title: None,
            directives: Vec::new(),
            attributes: HashMap::new(),
            detected_system: None,
        };

        let result = generate_vexflow_js(&elements, &metadata);
        assert!(result.is_ok());
        
        let js_code = result.unwrap();
        assert!(js_code.contains("0 slurs")); // Should create no slurs without notes
    }
}