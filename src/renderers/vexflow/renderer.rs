// VexFlow renderer - generates self-executing JavaScript
use crate::parse::model::{Document, DocumentElement};
use crate::models::Degree;
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
                        let barline_type = match barline {
                            crate::parse::model::Barline::Single(_) => "single",
                            crate::parse::model::Barline::Double(_) => "double",
                            crate::parse::model::Barline::Final(_) => "end",
                            crate::parse::model::Barline::RepeatStart(_) => "repeat-start",
                            crate::parse::model::Barline::RepeatEnd(_) => "repeat-end",
                            crate::parse::model::Barline::RepeatBoth(_) => "repeat-both",
                        };
                        notes.push(serde_json::json!({
                            "type": "BarLine",
                            "bar_type": barline_type
                        }));
                    }
                    crate::parse::model::ContentElement::Whitespace(_) => {
                        // Skip whitespace
                    }
                    crate::parse::model::ContentElement::UnknownToken(_) => {
                        // Skip unknown tokens (behave like whitespace)
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
            crate::parse::model::BeatElement::Dash(dash) => {
                // Only process dashes that have rhythm data (starting dashes)
                // Extending dashes have no rhythm data and should be skipped
                if let (Some(numer), Some(denom)) = (dash.numerator, dash.denominator) {
                    let note_duration = fraction::Fraction::new(numer, denom);
                    let (vexflow_duration, dots) = convert_fraction_to_vexflow(note_duration);

                    let mut rest = serde_json::json!({
                        "type": "Rest",
                        "duration": vexflow_duration
                    });
                    if dots > 0 {
                        rest["dots"] = dots.into();
                    }
                    elements.push(rest);
                }
                // Skip dashes without rhythm data (extenders)
            }
            crate::parse::model::BeatElement::BreathMark(_) => {
                elements.push(serde_json::json!({
                    "type": "Symbol",
                    "symbol": "breathmark"
                }));
            }
            crate::parse::model::BeatElement::Rest(_) => {
                // TODO: Handle Rest elements
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

/// Convert Fraction to VexFlow duration using shared RhythmConverter
fn convert_fraction_to_vexflow(duration: fraction::Fraction) -> (String, u8) {
    use crate::models::RhythmConverter;

    // Use the existing rhythm converter to get VexFlow durations
    let vexflow_durations = RhythmConverter::fraction_to_vexflow(duration);

    if let Some((vexflow_duration, dots)) = vexflow_durations.first() {
        (vexflow_duration.clone(), *dots)
    } else {
        // Fallback to quarter note
        ("q".to_string(), 0)
    }
}

/// Convert degree to VexFlow key with octave
fn degree_to_vexflow_key(degree: Degree, octave: i8) -> (String, Vec<serde_json::Value>) {
    use crate::models::Degree::*;

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
fn pitch_code_to_degree(pitch_code: crate::models::PitchCode) -> crate::models::Degree {
    use crate::models::PitchCode::*;
    use crate::models::Degree;

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
