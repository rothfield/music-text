// Structure-preserving FSM that maintains Document → Staves → Items hierarchy
use crate::ast::{Document, Stave, Measure, Beat};
use crate::models::{ParsedElement, Degree, Position};
use crate::parser_v2_fsm::Item;
use crate::rhythm_fsm;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProcessedDocument {
    pub staves: Vec<ProcessedStave>,
    pub notation_system: crate::ast::NotationSystem,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProcessedStave {
    pub items: Vec<Item>,  // FSM output for this stave only
    pub original_stave_index: usize,
}

impl ProcessedDocument {
    /// Convert Document to ProcessedDocument by running FSM on each stave individually
    pub fn from_document(document: &Document) -> Self {
        let mut processed_staves = Vec::new();
        
        for (stave_index, stave) in document.staves.iter().enumerate() {
            let stave_elements = convert_stave_to_parsed_elements(stave, &document.notation_system);
            let fsm_items = rhythm_fsm::convert_parsed_to_fsm_output(&stave_elements);
            
            processed_staves.push(ProcessedStave {
                items: fsm_items,
                original_stave_index: stave_index,
            });
        }
        
        ProcessedDocument {
            staves: processed_staves,
            notation_system: document.notation_system.clone(),
        }
    }
}

/// Convert a single stave to ParsedElements (no flattening across staves)
fn convert_stave_to_parsed_elements(stave: &Stave, notation_system: &crate::ast::NotationSystem) -> Vec<ParsedElement> {
    let mut elements = Vec::new();
    
    for measure in &stave.content_line.measures {
        // Add start barline if present
        if let Some(barline) = &measure.start_barline {
            elements.push(ParsedElement::Barline {
                style: crate::ast_to_parsed::convert_barline_to_string(barline),
                position: Position { row: 0, col: 0, char_index: 0 },
                tala: None,
            });
        }
        
        // Convert beats to parsed elements
        for (i, beat) in measure.beats.iter().enumerate() {
            let beat_elements = &beat.elements;
            
            for element in beat_elements {
                if let Some(parsed_elem) = crate::ast_to_parsed::convert_beat_element_to_parsed(element, notation_system) {
                    elements.push(parsed_elem);
                }
            }
            
            // Add space between beats (beat separator), but not after the last beat
            if i < measure.beats.len() - 1 {
                elements.push(ParsedElement::Whitespace {
                    position: Position { row: 0, col: 0, char_index: 0 },
                    width: 1,
                });
            }
        }
        
        // Add end barline if present
        if let Some(barline) = &measure.end_barline {
            elements.push(ParsedElement::Barline {
                style: crate::ast_to_parsed::convert_barline_to_string(barline),
                position: Position { row: 0, col: 0, char_index: 0 },
                tala: None,
            });
        }
    }
    
    elements
}
