use crate::parser_v2_fsm::{Item, Beat};
use crate::models::{Degree, BarlineType, RhythmConverter};
use crate::ast::Document;
use crate::renderers::vexflow::renderer::VexFlowOutput;
use crate::{renderers::vexflow::renderer as vexflow, ast_to_parsed};
use crate::structure_preserving_fsm::ProcessedDocument;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedVexFlowOutput {
    pub staves: Vec<EnhancedVexFlowStave>,
    pub key_signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedVexFlowStave {
    pub notes: Vec<EnhancedVexFlowElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EnhancedVexFlowElement {
    Note {
        keys: Vec<String>,
        duration: String,
        dots: u8,
        tied: bool,
    },
    Rest {
        duration: String,
        dots: u8,
    },
    BarLine {
        bar_type: String,
    },
    Tuplet {
        notes: Vec<EnhancedVexFlowElement>,
        ratio: (u8, u8), // (3, 2) for triplets, etc.
    },
    Breathe,
}

/// Convert FSM items directly to VexFlow output (unified architecture)
pub fn convert_fsm_items_to_enhanced_vexflow(fsm_items: &Vec<Item>) -> Result<EnhancedVexFlowOutput, String> {
    let mut staves = Vec::new();
    let mut current_stave = EnhancedVexFlowStave { notes: Vec::new() };
    
    for item in fsm_items {
        match item {
            Item::Beat(beat) => {
                let beat_notes = process_beat_to_vexflow(beat)?;
                
                if beat.is_tuplet {
                    let (tuplet_num, tuplet_den) = beat.tuplet_ratio.unwrap_or((3, 2));
                    current_stave.notes.push(EnhancedVexFlowElement::Tuplet {
                        notes: beat_notes,
                        ratio: (tuplet_num as u8, tuplet_den as u8),
                    });
                } else {
                    current_stave.notes.extend(beat_notes);
                }
            },
            Item::Barline(barline_type, _tala) => {
                current_stave.notes.push(EnhancedVexFlowElement::BarLine {
                    bar_type: barline_to_vexflow_string(barline_type),
                });
            },
            Item::Tonic(_) => {
                // Tonic affects transposition, handled elsewhere
            },
            Item::Breathmark => {
                current_stave.notes.push(EnhancedVexFlowElement::Breathe);
            },
        }
    }
    
    // If no notes were generated, add a default rest
    if current_stave.notes.is_empty() {
        current_stave.notes.push(EnhancedVexFlowElement::Rest {
            duration: "q".to_string(),
            dots: 0,
        });
    }
    
    staves.push(current_stave);
    
    Ok(EnhancedVexFlowOutput { 
        staves, 
        key_signature: None // TODO: Extract from metadata
    })
}


fn process_beat_to_vexflow(beat: &Beat) -> Result<Vec<EnhancedVexFlowElement>, String> {
    let mut elements = Vec::new();
    
    for beat_element in &beat.elements {
        match &beat_element.event {
            crate::parser_v2_fsm::Event::Note { degree, octave, .. } => {
                let vexflow_key = degree_to_vexflow_key(*degree, *octave);
                
                // Convert duration using rhythm converter
                let duration_fractions = RhythmConverter::fraction_to_vexflow(
                    beat_element.tuplet_display_duration.unwrap_or(beat_element.tuplet_duration)
                );
                
                for (i, (duration_str, dots)) in duration_fractions.iter().enumerate() {
                    let tied = i < duration_fractions.len() - 1; // Tie if more durations follow
                    
                    elements.push(EnhancedVexFlowElement::Note {
                        keys: vec![vexflow_key.clone()],
                        duration: duration_str.clone(),
                        dots: *dots,
                        tied,
                    });
                }
            },
            crate::parser_v2_fsm::Event::Rest => {
                let duration_fractions = RhythmConverter::fraction_to_vexflow(
                    beat_element.tuplet_display_duration.unwrap_or(beat_element.tuplet_duration)
                );
                
                for (duration_str, dots) in duration_fractions {
                    elements.push(EnhancedVexFlowElement::Rest {
                        duration: duration_str,
                        dots,
                    });
                }
            },
        }
    }
    
    Ok(elements)
}

fn degree_to_vexflow_key(degree: Degree, octave: i8) -> String {
    let note_name = degree_to_note_name(degree);
    let vexflow_octave = octave + 4; // VexFlow uses 4 as middle octave
    format!("{}/{}", note_name, vexflow_octave)
}

fn degree_to_note_name(degree: Degree) -> String {
    use Degree::*;
    match degree {
        // Scale degree 1 (Do/Sa/C)
        N1bb => "cbb".to_string(), N1b => "cb".to_string(), N1 => "c".to_string(),
        N1s => "c#".to_string(), N1ss => "c##".to_string(),
        // Scale degree 2 (Re/D)  
        N2bb => "dbb".to_string(), N2b => "db".to_string(), N2 => "d".to_string(),
        N2s => "d#".to_string(), N2ss => "d##".to_string(),
        // Scale degree 3 (Mi/Ga/E)
        N3bb => "ebb".to_string(), N3b => "eb".to_string(), N3 => "e".to_string(),
        N3s => "e#".to_string(), N3ss => "e##".to_string(),
        // Scale degree 4 (Fa/Ma/F)
        N4bb => "fbb".to_string(), N4b => "fb".to_string(), N4 => "f".to_string(),
        N4s => "f#".to_string(), N4ss => "f##".to_string(),
        // Scale degree 5 (Sol/Pa/G)
        N5bb => "gbb".to_string(), N5b => "gb".to_string(), N5 => "g".to_string(),
        N5s => "g#".to_string(), N5ss => "g##".to_string(),
        // Scale degree 6 (La/Dha/A)
        N6bb => "abb".to_string(), N6b => "ab".to_string(), N6 => "a".to_string(),
        N6s => "a#".to_string(), N6ss => "a##".to_string(),
        // Scale degree 7 (Ti/Ni/B)
        N7bb => "bbb".to_string(), N7b => "bb".to_string(), N7 => "b".to_string(),
        N7s => "b#".to_string(), N7ss => "b##".to_string(),
    }
}

fn barline_to_vexflow_string(barline_type: &BarlineType) -> String {
    match barline_type {
        BarlineType::Single => "single".to_string(),
        BarlineType::Double => "double".to_string(),
        BarlineType::Final => "end".to_string(),
        BarlineType::RepeatStart => "repeat-begin".to_string(),
        BarlineType::RepeatEnd => "repeat-end".to_string(),
        BarlineType::RepeatBoth => "double-repeat".to_string(),
    }
}

/// Convert ProcessedDocument (structure-preserving) to VexFlow output
pub fn convert_processed_document_to_vexflow(processed_doc: &ProcessedDocument) -> Result<EnhancedVexFlowOutput, String> {
    let mut staves = Vec::new();
    
    for stave in &processed_doc.staves {
        // Convert each stave's items to VexFlow using existing function
        let stave_output = convert_fsm_items_to_enhanced_vexflow(&stave.items)?;
        
        // Since convert_fsm_items_to_enhanced_vexflow returns a single-stave output,
        // we need to extract the first stave and add it to our collection
        if let Some(first_stave) = stave_output.staves.into_iter().next() {
            staves.push(first_stave);
        }
    }
    
    Ok(EnhancedVexFlowOutput {
        staves,
        key_signature: None, // TODO: Extract key signature from ProcessedDocument if needed
    })
}