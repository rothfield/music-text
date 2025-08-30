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

/// Generate VexFlow JavaScript code from FSM Item output
pub fn generate_vexflow_js(
    elements: &Vec<Item>,
    _metadata: &Metadata
) -> Result<String, String> {
    let mut notes = Vec::new();
    let mut slurs = Vec::new();
    let mut slur_start_pending = false;
    let mut slur_end_pending = false;

    eprintln!("VEXFLOW DEBUG: Processing {} elements", elements.len());
    
    for (i, element) in elements.iter().enumerate() {
        eprintln!("VEXFLOW DEBUG: Element {}: {:?}", i, element);
        
        match element {
            Item::SlurStart => {
                eprintln!("VEXFLOW DEBUG: SlurStart - marking pending for next beat");
                slur_start_pending = true;
            },
            Item::SlurEnd => {
                eprintln!("VEXFLOW DEBUG: SlurEnd - marking pending for next beat");
                slur_end_pending = true;
            },
            Item::Beat(beat) => {
                eprintln!("VEXFLOW DEBUG: Processing beat with {} elements", beat.elements.len());
                
                let beat_start_index = notes.len();
                let mut beat_note_count = 0;
                
                // Process all notes in the beat
                for beat_element in &beat.elements {
                    if beat_element.is_note() {
                        let vf_note = convert_beat_element_to_vexflow_note(beat_element)?;
                        let note_index = notes.len();
                        eprintln!("VEXFLOW DEBUG: Added note {} at index {}: {:?}", note_index, note_index, vf_note);
                        notes.push(vf_note);
                        beat_note_count += 1;
                    }
                }
                
                // Handle pending slur markers - apply them to this beat's notes
                if slur_start_pending && slur_end_pending && beat_note_count > 1 {
                    // SlurStart and SlurEnd both pending: slur covers all notes in this beat
                    let start_idx = beat_start_index;
                    let end_idx = beat_start_index + beat_note_count - 1;
                    
                    eprintln!("VEXFLOW DEBUG: Creating slur from note {} to note {} (full beat)", start_idx, end_idx);
                    slurs.push(VexFlowSlur {
                        from_note: start_idx,
                        to_note: end_idx,
                    });
                    
                    slur_start_pending = false;
                    slur_end_pending = false;
                } else if slur_start_pending || slur_end_pending {
                    eprintln!("VEXFLOW DEBUG: Partial slur markers - need more complex logic (not implemented)");
                    slur_start_pending = false;
                    slur_end_pending = false;
                }
            },
            _ => {
                // Skip other items like Tonic, Barline, etc.
                eprintln!("VEXFLOW DEBUG: Skipping element: {:?}", element);
            }
        }
    }

    eprintln!("VEXFLOW DEBUG: Final results - {} notes, {} slurs", notes.len(), slurs.len());
    for (i, slur) in slurs.iter().enumerate() {
        eprintln!("VEXFLOW DEBUG: Slur {}: from note {} to note {}", i, slur.from_note, slur.to_note);
    }

    let measure = VexFlowMeasure { notes, slurs };
    
    // Generate JavaScript code
    let js_code = format!(
        "console.log('VexFlow: Processing {} notes and {} slurs'); \
        const notes = {}; \
        const slurs = {}; \
        console.log('VexFlow: Generated notes:', notes); \
        console.log('VexFlow: Generated slurs:', slurs);",
        measure.notes.len(),
        measure.slurs.len(),
        serde_json::to_string(&measure.notes).map_err(|e| format!("JSON serialization error: {}", e))?,
        serde_json::to_string(&measure.slurs).map_err(|e| format!("JSON serialization error: {}", e))?
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
        
        // Verify that JavaScript code contains expected elements
        assert!(js_code.contains("4 notes"), "Should process 4 notes");
        assert!(js_code.contains("1 slurs"), "Should create 1 slur");
        assert!(js_code.contains("from_note"), "Should contain slur from_note");
        assert!(js_code.contains("to_note"), "Should contain slur to_note");
        
        // Verify note generation
        assert!(js_code.contains("c/4"), "Should contain C note");
        assert!(js_code.contains("d/4"), "Should contain D note");
        assert!(js_code.contains("e/4"), "Should contain E note");
        assert!(js_code.contains("f/4"), "Should contain F note");
        
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
        assert!(js_code.contains("0 notes"));
        assert!(js_code.contains("0 slurs")); // Should create no slurs without notes
    }
}