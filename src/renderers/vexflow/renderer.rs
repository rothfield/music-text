use crate::parse::model::{Document, DocumentElement};
use crate::rhythm::types::Degree;
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

        // Convert each stave using rhythm analysis results
        for element in &document.elements {
            if let DocumentElement::Stave(stave) = element {
                let notes = if let Some(rhythm_items) = &stave.rhythm_items {
                    process_items(rhythm_items)
                } else {
                    Vec::new() // No rhythm items available
                };

                staves_data.push(serde_json::json!({
                    "notes": notes,
                    "key_signature": "C"
                }));
            }
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
}

impl Default for VexFlowRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Process rhythm items to VexFlow JSON
fn process_items(rhythm_items: &[crate::rhythm::analyzer::Item]) -> Vec<serde_json::Value> {
    let mut notes = Vec::new();

    for item in rhythm_items {
        match item {
            crate::rhythm::analyzer::Item::Beat(beat) => {
                if beat.is_tuplet {
                    // Handle tuplet as a group
                    let tuplet_notes = convert_beat_to_vexflow_elements(beat);
                    notes.push(serde_json::json!({
                        "type": "Tuplet",
                        "divisions": beat.divisions,
                        "ratio": beat.tuplet_ratio,
                        "notes": tuplet_notes
                    }));
                } else {
                    // Regular beat - add elements directly
                    let beat_elements = convert_beat_to_vexflow_elements(beat);
                    notes.extend(beat_elements);
                }
            },
            crate::rhythm::analyzer::Item::Barline(barline_type, _) => {
                notes.push(serde_json::json!({
                    "type": "BarLine",
                    "bar_type": format!("{:?}", barline_type)
                }));
            },
            crate::rhythm::analyzer::Item::Breathmark => {
                notes.push(serde_json::json!({
                    "type": "Breathmark"
                }));
            },
            crate::rhythm::analyzer::Item::Tonic(_) => {
                // Tonic doesn't generate visual elements
            }
        }
    }

    notes
}

/// Convert a beat to VexFlow elements with beaming
fn convert_beat_to_vexflow_elements(beat: &crate::rhythm::analyzer::Beat) -> Vec<serde_json::Value> {
    let mut elements = Vec::new();

    // Apply beaming logic: beam consecutive beamable notes within beat
    let beaming_info = analyze_beat_for_beaming(beat);

    for (element_index, beat_element) in beat.elements.iter().enumerate() {
        match &beat_element.event {
            crate::rhythm::analyzer::Event::Note { degree, octave, .. } => {
                let (key, accidentals) = degree_to_vexflow_key(*degree, *octave);
                let (duration, dots) = convert_fraction_to_vexflow(beat_element.tuplet_duration);

                // Determine beaming for this note
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
            crate::rhythm::analyzer::Event::Rest => {
                let (duration, dots) = convert_fraction_to_vexflow(beat_element.tuplet_duration);

                elements.push(serde_json::json!({
                    "type": "Rest",
                    "duration": duration,
                    "dots": dots
                }));
            },
            crate::rhythm::analyzer::Event::Unknown { .. } => {
                // Skip unknown elements
            }
        }
    }

    elements
}

/// Analyze beat for beaming decisions
fn analyze_beat_for_beaming(beat: &crate::rhythm::analyzer::Beat) -> BeamingInfo {
    let mut beamable_notes = Vec::new();

    // Find consecutive beamable notes (eighth notes or smaller)
    for (index, element) in beat.elements.iter().enumerate() {
        if matches!(&element.event, crate::rhythm::analyzer::Event::Note { .. }) {
            let (duration, _) = convert_fraction_to_vexflow(element.tuplet_duration);
            if is_beamable_duration(&duration) {
                beamable_notes.push(index);
            }
        }
    }

    let should_beam = beamable_notes.len() >= 2;

    BeamingInfo {
        should_beam,
        beamable_notes,
    }
}

/// Check if a duration is beamable (eighth note or smaller)
fn is_beamable_duration(duration: &str) -> bool {
    matches!(duration, "8" | "16" | "32" | "64")
}

/// Convert degree to VexFlow key with octave
fn degree_to_vexflow_key(degree: Degree, octave: i8) -> (String, Vec<serde_json::Value>) {
    use crate::rhythm::types::Degree::*;

    let (base_note, accidental) = match degree {
        N1 => ("C", None),
        N1s => ("C", Some("#")),
        N1b => ("C", Some("b")),
        N2 => ("D", None),
        N2s => ("D", Some("#")),
        N2b => ("D", Some("b")),
        N3 => ("E", None),
        N3s => ("E", Some("#")),
        N3b => ("E", Some("b")),
        N4 => ("F", None),
        N4s => ("F", Some("#")),
        N4b => ("F", Some("b")),
        N5 => ("G", None),
        N5s => ("G", Some("#")),
        N5b => ("G", Some("b")),
        N6 => ("A", None),
        N6s => ("A", Some("#")),
        N6b => ("A", Some("b")),
        N7 => ("B", None),
        N7s => ("B", Some("#")),
        N7b => ("B", Some("b")),
        _ => ("C", None), // Default for other variants
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