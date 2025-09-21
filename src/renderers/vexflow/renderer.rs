// VexFlow renderer - generates self-executing JavaScript
use crate::parse::model::{Document, DocumentElement};
use crate::models::pitch::Degree;
use super::js_generator::VexFlowJSGenerator;

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
    
    /// Render VexFlow data directly from Document structure - generates self-executing JavaScript
    pub fn render_data_from_document(&self, document: &Document) -> serde_json::Value {
        let mut js_generator = VexFlowJSGenerator::new();
        let mut generated_js = String::new();

        // Generate JavaScript for each stave
        let mut stave_count = 0;
        for element in &document.elements {
            if let DocumentElement::Stave(stave) = element {
                let stave_js = js_generator.generate_for_stave(stave, "vexflow-output");
                generated_js = stave_js; // For now, just use the last stave
                stave_count += 1;
            }
        }

        // If no staves, generate empty stave JavaScript
        if stave_count == 0 {
            generated_js = self.generate_empty_stave_js();
        }

        serde_json::json!({
            "vexflow_js": generated_js,
            "title": document.title.as_ref().or_else(|| document.directives.get("title")),
            "author": document.author.as_ref().or_else(|| document.directives.get("author")),
            "stave_count": stave_count
        })
    }

    fn generate_empty_stave_js(&self) -> String {
        format!(
            "(function() {{\n\
               const container = document.getElementById('vexflow-output');\n\
               if (!container) return;\n\
               container.innerHTML = '';\n\
               \n\
               const {{ Renderer, Stave }} = Vex.Flow;\n\
               \n\
               const renderer = new Renderer(container, Renderer.Backends.SVG);\n\
               renderer.resize(800, 200);\n\
               const context = renderer.getContext();\n\
               context.scale(0.9, 0.9);\n\
               \n\
               const stave = new Stave(10, 40, 700);\n\
               stave.addClef('treble');\n\
               stave.setContext(context);\n\
               stave.draw();\n\
             }})();\n"
        )
    }
    
}

impl Default for VexFlowRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Process stave to VexFlow notes array using current data structures
fn process_stave_to_vexflow(stave: &crate::parse::model::Stave) -> Vec<serde_json::Value> {
    let mut notes = Vec::new();

    for line in &stave.lines {
        if let crate::parse::model::StaveLine::ContentLine(content_line) = line {
            for element in &content_line.elements {
                match element {
                    crate::parse::model::ContentElement::Beat(beat) => {
                        let beat_notes = process_beat_to_vexflow(beat);

                        // Check if this beat is a tuplet
                        if beat.is_tuplet == Some(true) && !beat_notes.is_empty() {
                            // Wrap notes in a Tuplet object for JavaScript - use old working format
                            let divisions = beat.divisions.unwrap_or(3);
                            let (num_notes, notes_occupied) = beat.tuplet_ratio.unwrap_or((3, 2));

                            notes.push(serde_json::json!({
                                "type": "Tuplet",
                                "divisions": divisions,
                                "ratio": [num_notes, notes_occupied],
                                "notes": beat_notes
                            }));
                        } else {
                            // Add notes directly
                            notes.extend(beat_notes);
                        }
                    }
                    crate::parse::model::ContentElement::Barline(barline) => {
                        let barline_type = match barline.barline_type {
                            crate::rhythm::converters::BarlineType::Single => "single",
                            crate::rhythm::converters::BarlineType::Double => "double",
                            crate::rhythm::converters::BarlineType::RepeatStart => "repeat-start",
                            crate::rhythm::converters::BarlineType::RepeatEnd => "repeat-end",
                            crate::rhythm::converters::BarlineType::RepeatBoth => "repeat-both",
                        };
                        notes.push(serde_json::json!({
                            "type": "BarLine",
                            "bar_type": barline_type
                        }));
                    }
                    crate::parse::model::ContentElement::Whitespace(_) => {
                        // Skip whitespace
                    }
                }
            }
        }
    }

    notes
}

/// Process a beat to VexFlow note elements using current Beat structure
fn process_beat_to_vexflow(beat: &crate::parse::model::Beat) -> Vec<serde_json::Value> {
    let mut elements = Vec::new();
    let mut beamable_indices = Vec::new();

    // Handle empty beat as rest
    if beat.elements.is_empty() {
        let duration = beat.total_duration.clone()
            .unwrap_or_else(|| fraction::Fraction::from(1) / fraction::Fraction::from(4));
        let (vexflow_duration, dots) = convert_fraction_to_vexflow(duration);

        let mut rest = serde_json::json!({
            "type": "Rest",
            "duration": vexflow_duration
        });
        if dots > 0 {
            rest["dots"] = dots.into();
        }
        elements.push(rest);
        return elements;
    }

    // Process each element in the beat
    for (idx, beat_element) in beat.elements.iter().enumerate() {
        match beat_element {
            crate::parse::model::BeatElement::Note(note) => {
                // Use the duration from the note or beat (analyzer already set these)
                // Use simple numerator/denominator from note
                let numer = note.numerator.unwrap_or(1);
                let denom = note.denominator.unwrap_or(4);
                let note_duration = fraction::Fraction::new(numer, denom);
                let (vexflow_duration, dots) = convert_fraction_to_vexflow(note_duration);

                // Check if beamable (eighth notes and smaller)
                if matches!(vexflow_duration.as_str(), "8" | "16" | "32" | "64") {
                    beamable_indices.push(idx);
                }

                // Convert pitch to VexFlow format
                let degree = pitch_code_to_degree(note.pitch_code);
                let (key, accidentals) = degree_to_vexflow_key(degree, note.octave);

                let mut note_obj = serde_json::json!({
                    "type": "Note",
                    "keys": [key],
                    "duration": vexflow_duration
                });

                if dots > 0 {
                    note_obj["dots"] = dots.into();
                }

                if !accidentals.is_empty() {
                    note_obj["accidentals"] = accidentals.into();
                }

                elements.push(note_obj);
            }
            crate::parse::model::BeatElement::Dash(_) => {
                let duration = beat.total_duration.clone()
                    .unwrap_or_else(|| fraction::Fraction::from(1) / fraction::Fraction::from(4));
                let (vexflow_duration, dots) = convert_fraction_to_vexflow(duration);

                let mut rest = serde_json::json!({
                    "type": "Rest",
                    "duration": vexflow_duration
                });
                if dots > 0 {
                    rest["dots"] = dots.into();
                }
                elements.push(rest);
            }
            crate::parse::model::BeatElement::BreathMark(_) => {
                elements.push(serde_json::json!({
                    "type": "Symbol",
                    "symbol": "breathmark"
                }));
            }
        }
    }

    // Add beaming info if we have consecutive beamable notes
    if beamable_indices.len() >= 2 {
        // Check if indices are consecutive
        let mut consecutive = true;
        for i in 1..beamable_indices.len() {
            if beamable_indices[i] != beamable_indices[i-1] + 1 {
                consecutive = false;
                break;
            }
        }

        if consecutive {
            // Mark first note as beam start
            if let Some(first_idx) = beamable_indices.first() {
                if let Some(note) = elements.get_mut(*first_idx) {
                    if let Some(obj) = note.as_object_mut() {
                        obj.insert("beam_start".to_string(), serde_json::Value::Bool(true));
                    }
                }
            }
            // Mark last note as beam end
            if let Some(last_idx) = beamable_indices.last() {
                if let Some(note) = elements.get_mut(*last_idx) {
                    if let Some(obj) = note.as_object_mut() {
                        obj.insert("beam_end".to_string(), serde_json::Value::Bool(true));
                    }
                }
            }
        }
    }

    elements
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
        
        // Tuplet durations - render as appropriate visual note values
        (1, 12) => ("8".to_string(), 0),    // triplet eighth (3 in time of 2)
        (1, 20) => ("16".to_string(), 0),   // quintuplet sixteenth (5 in time of 4)
        (1, 24) => ("16".to_string(), 0),   // sextuplet sixteenth (6 in time of 4)
        (1, 28) => ("16".to_string(), 0),   // septuplet sixteenth (7 in time of 4)
        (1, 36) => ("32".to_string(), 0),   // nonuplet thirty-second (9 in time of 8)
        (1, 40) => ("32".to_string(), 0),   // 10-tuplet thirty-second
        (1, 44) => ("32".to_string(), 0),   // 11-tuplet thirty-second
        (1, 48) => ("32".to_string(), 0),   // triplet thirty-second (12 in time of 8)
        (1, 52) => ("32".to_string(), 0),   // 13-tuplet thirty-second
        (1, 56) => ("32".to_string(), 0),   // 14-tuplet thirty-second
        (1, 60) => ("32".to_string(), 0),   // 15-tuplet thirty-second
        (1, 68) => ("64".to_string(), 0),   // 17-tuplet sixty-fourth
        (1, 72) => ("64".to_string(), 0),   // 18-tuplet sixty-fourth
        (1, 76) => ("64".to_string(), 0),   // 19-tuplet sixty-fourth
        (1, 80) => ("64".to_string(), 0),   // 20-tuplet sixty-fourth
        (1, 84) => ("64".to_string(), 0),   // 21-tuplet sixty-fourth
        (1, 88) => ("64".to_string(), 0),   // 22-tuplet sixty-fourth
        (1, 92) => ("64".to_string(), 0),   // 23-tuplet sixty-fourth
        (1, 96) => ("64".to_string(), 0),   // 24-tuplet sixty-fourth
        (1, 100) => ("64".to_string(), 0),  // 25-tuplet sixty-fourth
        (1, 104) => ("64".to_string(), 0),  // 26-tuplet sixty-fourth
        (1, 108) => ("64".to_string(), 0),  // 27-tuplet sixty-fourth
        (1, 112) => ("64".to_string(), 0),  // 28-tuplet sixty-fourth
        (1, 116) => ("64".to_string(), 0),  // 29-tuplet sixty-fourth
        (1, 120) => ("64".to_string(), 0),  // 30-tuplet sixty-fourth
        (1, 124) => ("64".to_string(), 0),  // 31-tuplet sixty-fourth
        (1, 128) => ("64".to_string(), 0),  // 32-tuplet sixty-fourth

        // Fallback for any other tuplet durations using algorithmic approach
        _ => {
            // For very large denominators, map to appropriate note value
            if den >= 96 { ("64".to_string(), 0) }    // 64th notes for very dense tuplets
            else if den >= 48 { ("32".to_string(), 0) }  // 32nd notes
            else if den >= 24 { ("16".to_string(), 0) }  // 16th notes
            else if den >= 12 { ("8".to_string(), 0) }   // 8th notes
            else { ("32".to_string(), 0) }               // thirty-second notes (default)
        }
    }
}

/// Convert degree to VexFlow key with octave
fn degree_to_vexflow_key(degree: Degree, octave: i8) -> (String, Vec<serde_json::Value>) {
    use crate::models::pitch::Degree::*;

    let (base_note, accidental) = match degree {
        // Scale degree 1 (Do/Sa/C)
        N1bb => ("C", Some("bb")),  N1b => ("C", Some("b")),   N1 => ("C", None),
        N1s => ("C", Some("#")),    N1ss => ("C", Some("##")),
        // Scale degree 2 (Re/D)
        N2bb => ("D", Some("bb")),  N2b => ("D", Some("b")),   N2 => ("D", None),
        N2s => ("D", Some("#")),    N2ss => ("D", Some("##")),
        // Scale degree 3 (Mi/Ga/E)
        N3bb => ("E", Some("bb")),  N3b => ("E", Some("b")),   N3 => ("E", None),
        N3s => ("E", Some("#")),    N3ss => ("E", Some("##")),
        // Scale degree 4 (Fa/Ma/F)
        N4bb => ("F", Some("bb")),  N4b => ("F", Some("b")),   N4 => ("F", None),
        N4s => ("F", Some("#")),    N4ss => ("F", Some("##")),
        // Scale degree 5 (Sol/Pa/G)
        N5bb => ("G", Some("bb")),  N5b => ("G", Some("b")),   N5 => ("G", None),
        N5s => ("G", Some("#")),    N5ss => ("G", Some("##")),
        // Scale degree 6 (La/Dha/A)
        N6bb => ("A", Some("bb")),  N6b => ("A", Some("b")),   N6 => ("A", None),
        N6s => ("A", Some("#")),    N6ss => ("A", Some("##")),
        // Scale degree 7 (Ti/Ni/B)
        N7bb => ("B", Some("bb")),  N7b => ("B", Some("b")),   N7 => ("B", None),
        N7s => ("B", Some("#")),    N7ss => ("B", Some("##")),
    };

    let vexflow_octave = 4 + octave;
    let key = format!("{}/{}", base_note, vexflow_octave);

    let accidentals = if let Some(acc) = accidental {
        vec![serde_json::json!(acc)]
    } else {
        vec![]
    };

    (key, accidentals)
}

/// Convert PitchCode to Degree (they have identical variants)
fn pitch_code_to_degree(pitch_code: crate::parse::model::PitchCode) -> crate::models::pitch::Degree {
    use crate::parse::model::PitchCode::*;
    use crate::models::pitch::Degree;

    match pitch_code {
        N1bb => Degree::N1bb, N1b => Degree::N1b, N1 => Degree::N1, N1s => Degree::N1s, N1ss => Degree::N1ss,
        N2bb => Degree::N2bb, N2b => Degree::N2b, N2 => Degree::N2, N2s => Degree::N2s, N2ss => Degree::N2ss,
        N3bb => Degree::N3bb, N3b => Degree::N3b, N3 => Degree::N3, N3s => Degree::N3s, N3ss => Degree::N3ss,
        N4bb => Degree::N4bb, N4b => Degree::N4b, N4 => Degree::N4, N4s => Degree::N4s, N4ss => Degree::N4ss,
        N5bb => Degree::N5bb, N5b => Degree::N5b, N5 => Degree::N5, N5s => Degree::N5s, N5ss => Degree::N5ss,
        N6bb => Degree::N6bb, N6b => Degree::N6b, N6 => Degree::N6, N6s => Degree::N6s, N6ss => Degree::N6ss,
        N7bb => Degree::N7bb, N7b => Degree::N7b, N7 => Degree::N7, N7s => Degree::N7s, N7ss => Degree::N7ss,
    }
}
