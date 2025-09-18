use crate::parse::model::{Document, DocumentElement};
use crate::models::pitch::Degree;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
struct BeamingInfo {
    should_beam: bool,
    beamable_notes: Vec<usize>,
}

/// VexFlow renderer that works directly with Documents using rhythm analysis
pub struct VexFlowRenderer;

impl VexFlowRenderer {
    pub fn new() -> Self {
        Self
    }

    /// Render VexFlow data directly from Document structure
    pub fn render_data_from_document(&self, document: &Document) -> serde_json::Value {
        let mut staves_data = Vec::new();

        // Convert each stave using ContentLine beats
        for element in &document.elements {
            if let DocumentElement::Stave(stave) = element {
                let notes = extract_beats_from_stave(stave);

                staves_data.push(serde_json::json!({
                    "notes": notes,
                    "key_signature": "C"
                }));
            }
        }

        // Ensure there's always at least one empty stave if no content
        if staves_data.is_empty() {
            staves_data.push(serde_json::json!({
                "notes": [],
                "key_signature": "C"
            }));
        }

        serde_json::json!({
            "staves": staves_data,
            "title": document.title.as_ref().or_else(|| document.directives.get("title")),
            "author": document.author.as_ref().or_else(|| document.directives.get("author")),
            "time_signature": "4/4",
            "clef": "treble",
            "key_signature": "C"
        })
    }
}

impl Default for VexFlowRenderer {
    fn default() -> Self {
        Self::new()
    }
}



/// Convert degree to VexFlow key with octave
fn degree_to_vexflow_key(degree: Degree, octave: i8) -> (String, Vec<serde_json::Value>) {
    use crate::models::pitch::Degree::*;

    let (base_note, accidental) = match degree {
        // Scale degree 1 (Do/Sa/C) - all 5 variants
        N1bb => ("C", Some("bb")),  N1b => ("C", Some("b")),   N1 => ("C", None),
        N1s => ("C", Some("#")),    N1ss => ("C", Some("##")),
        // Scale degree 2 (Re/D) - all 5 variants
        N2bb => ("D", Some("bb")),  N2b => ("D", Some("b")),   N2 => ("D", None),
        N2s => ("D", Some("#")),    N2ss => ("D", Some("##")),
        // Scale degree 3 (Mi/Ga/E) - all 5 variants
        N3bb => ("E", Some("bb")),  N3b => ("E", Some("b")),   N3 => ("E", None),
        N3s => ("E", Some("#")),    N3ss => ("E", Some("##")),
        // Scale degree 4 (Fa/Ma/F) - all 5 variants
        N4bb => ("F", Some("bb")),  N4b => ("F", Some("b")),   N4 => ("F", None),
        N4s => ("F", Some("#")),    N4ss => ("F", Some("##")),
        // Scale degree 5 (Sol/Pa/G) - all 5 variants
        N5bb => ("G", Some("bb")),  N5b => ("G", Some("b")),   N5 => ("G", None),
        N5s => ("G", Some("#")),    N5ss => ("G", Some("##")),
        // Scale degree 6 (La/Dha/A) - all 5 variants
        N6bb => ("A", Some("bb")),  N6b => ("A", Some("b")),   N6 => ("A", None),
        N6s => ("A", Some("#")),    N6ss => ("A", Some("##")),
        // Scale degree 7 (Ti/Ni/B) - all 5 variants
        N7bb => ("B", Some("bb")),  N7b => ("B", Some("b")),   N7 => ("B", None),
        N7s => ("B", Some("#")),    N7ss => ("B", Some("##")),
    };

    let vexflow_octave = 4 + octave; // Default to 4th octave
    let key = format!("{}/{}", base_note, vexflow_octave);

    let accidentals = if let Some(acc) = accidental {
        vec![serde_json::json!(acc)]
    } else {
        vec![]
    };

    (key, accidentals)
}

/// Convert fraction to VexFlow duration
fn convert_fraction_to_vexflow(duration: fraction::Fraction) -> (String, u8) {
    let num = *duration.numer().unwrap() as usize;
    let den = *duration.denom().unwrap() as usize;

    match (num, den) {
        (1, 1) => ("w".to_string(), 0),        // whole note
        (1, 2) => ("h".to_string(), 0),        // half note
        (1, 4) => ("q".to_string(), 0),        // quarter note
        (1, 8) => ("8".to_string(), 0),        // eighth note
        (1, 16) => ("16".to_string(), 0),      // sixteenth note
        (1, 32) => ("32".to_string(), 0),      // thirty-second note
        // Single-dotted durations
        (3, 2) => ("w".to_string(), 1),        // dotted whole
        (3, 4) => ("h".to_string(), 1),        // dotted half
        (3, 8) => ("q".to_string(), 1),        // dotted quarter
        (3, 16) => ("8".to_string(), 1),       // dotted eighth
        (3, 32) => ("16".to_string(), 1),      // dotted sixteenth
        // Double-dotted durations
        (7, 4) => ("w".to_string(), 2),        // double-dotted whole
        (7, 8) => ("h".to_string(), 2),        // double-dotted half
        (7, 16) => ("q".to_string(), 2),       // double-dotted quarter
        (7, 32) => ("8".to_string(), 2),       // double-dotted eighth
        (7, 64) => ("16".to_string(), 2),      // double-dotted sixteenth
        _ => ("q".to_string(), 0),             // default to quarter note
    }
}

/// Extract beats from a stave's ContentLine elements
fn extract_beats_from_stave(stave: &crate::parse::model::Stave) -> Vec<serde_json::Value> {
    let mut notes = Vec::new();

    for line in &stave.lines {
        if let crate::parse::model::StaveLine::ContentLine(content_line) = line {
            for element in &content_line.elements {
                if let crate::parse::model::ContentElement::Beat(beat) = element {
                    // Convert parse::model::Beat to VexFlow format
                    for beat_element in &beat.elements {
                        match beat_element {
                            crate::parse::model::BeatElement::Note(note) => {
                                // Convert PitchCode to Degree (they have identical variants)
                                let degree = pitch_code_to_degree(note.pitch_code);
                                let (pitch_name, accidentals) = degree_to_vexflow_key(degree, note.octave);
                                notes.push(serde_json::json!({
                                    "type": "Note",
                                    "pitch": pitch_name,
                                    "accidentals": accidentals,
                                    "duration": "q" // Default to quarter note for now
                                }));
                            }
                            crate::parse::model::BeatElement::Dash(_) => {
                                notes.push(serde_json::json!({
                                    "type": "Rest",
                                    "duration": "q" // Default to quarter rest
                                }));
                            }
                            crate::parse::model::BeatElement::BreathMark(_) => {
                                // Breath marks could be rendered as symbols
                                notes.push(serde_json::json!({
                                    "type": "Symbol",
                                    "symbol": "breathmark"
                                }));
                            }
                        }
                    }
                }
            }
        }
    }

    notes
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