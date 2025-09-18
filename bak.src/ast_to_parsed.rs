// Direct AST to ParsedElement conversion
// Replaces the bridge FSM with simple conversion to main codebase types

use crate::ast::{Document, Beat, BeatElement, NotationSystem};
use crate::models::{Degree, ParsedElement, ParsedChild, Position};

pub fn convert_ast_to_parsed_elements(document: &Document) -> Vec<ParsedElement> {
    let mut elements = Vec::new();
    
    for (stave_index, stave) in document.staves.iter().enumerate() {
        // Add newline between staves (except before the first one)
        if stave_index > 0 {
            elements.push(ParsedElement::Newline {
                position: Position { row: stave_index, col: 0, char_index: 0 },
            });
        }
        
        for measure in &stave.content_line.measures {
            // Add start barline if present
            if let Some(barline) = &measure.start_barline {
                elements.push(ParsedElement::Barline {
                    style: convert_barline_to_string(barline),
                    position: Position { row: 0, col: 0, char_index: 0 },
                    tala: None,
                });
            }
            
            // Convert beats to parsed elements
            for (i, beat) in measure.beats.iter().enumerate() {
                let beat_elements = &beat.elements;
                
                for element in beat_elements {
                    if let Some(parsed_elem) = convert_beat_element_to_parsed(element, &document.notation_system) {
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
                    style: convert_barline_to_string(barline),
                    position: Position { row: 0, col: 0, char_index: 0 },
                    tala: None,
                });
            }
        }
    }
    
    elements
}

pub fn convert_beat_element_to_parsed(element: &BeatElement, notation_system: &NotationSystem) -> Option<ParsedElement> {
    match element {
        BeatElement::Pitch { value, accidental, octave, .. } => {
            let degree = convert_pitch_to_degree(value, accidental, *octave, notation_system).ok()?;
            Some(ParsedElement::Note { 
                degree, 
                octave: *octave, 
                children: Vec::new(),
                position: Position { row: 0, col: 0, char_index: 0 },
                value: value.clone(),
                duration: None,
                slur: None,
            })
        },
        BeatElement::Dash { position } => {
            let pos = position.clone().unwrap_or(Position { row: 0, col: 0, char_index: 0 });
            Some(ParsedElement::Dash {
                degree: None,
                octave: None,
                position: pos,
                duration: None,
            })
        },
        BeatElement::Space { position: _ } => {
            // Skip spaces within beats - they don't become parsed elements
            None
        },
        BeatElement::SlurStart { position } => {
            let pos = position.clone().unwrap_or(Position { row: 0, col: 0, char_index: 0 });
            Some(ParsedElement::Symbol {
                value: "(".to_string(),
                position: pos,
            })
        },
        BeatElement::SlurEnd { position } => {
            let pos = position.clone().unwrap_or(Position { row: 0, col: 0, char_index: 0 });
            Some(ParsedElement::Symbol {
                value: ")".to_string(), 
                position: pos,
            })
        },
        BeatElement::Rest { .. } => {
            Some(ParsedElement::Rest {
                value: "r".to_string(),
                position: Position { row: 0, col: 0, char_index: 0 },
                duration: None,
            })
        },
        BeatElement::BreathMark { position } => {
            let pos = position.clone().unwrap_or(Position { row: 0, col: 0, char_index: 0 });
            Some(ParsedElement::Symbol {
                value: "'".to_string(),
                position: pos,
            })
        },
    }
}

fn convert_pitch_to_degree(value: &str, accidental: &Option<String>, _octave: i8, notation_system: &NotationSystem) -> Result<Degree, String> {
    let base_degree = match notation_system {
        NotationSystem::Sargam => {
            match value.trim().to_uppercase().as_str() {
                "S" => Degree::N1,
                "R" => Degree::N2,
                "G" => Degree::N3,
                "M" => Degree::N4,
                "P" => Degree::N5,
                "D" => Degree::N6,
                "N" => Degree::N7,
                _ => return Err(format!("Unknown sargam note: {}", value)),
            }
        },
        NotationSystem::Number => {
            match value.trim() {
                "1" => Degree::N1,
                "2" => Degree::N2,
                "3" => Degree::N3,
                "4" => Degree::N4,
                "5" => Degree::N5,
                "6" => Degree::N6,
                "7" => Degree::N7,
                _ => return Err(format!("Unknown number note: {}", value)),
            }
        },
        NotationSystem::Western => {
            let upper = value.trim().to_uppercase();
            match upper.as_str() {
                "C" => Degree::N1, "D" => Degree::N2, "E" => Degree::N3, "F" => Degree::N4,
                "G" => Degree::N5, "A" => Degree::N6, "B" => Degree::N7,
                _ => return Err(format!("Unknown western note: {}", value)),
            }
        },
        NotationSystem::Doremi => {
            let lower = value.trim().to_lowercase();
            match lower.as_str() {
                "d" => Degree::N1, "r" => Degree::N2, "m" => Degree::N3, "f" => Degree::N4,
                "s" => Degree::N5, "l" => Degree::N6, "t" => Degree::N7,
                _ => return Err(format!("Unknown doremi note: {}", value)),
            }
        },
        _ => {
            // Mixed notation system - try all systems
            // First try number notation
            match value.trim() {
                "1" => Degree::N1,
                "2" => Degree::N2, 
                "3" => Degree::N3,
                "4" => Degree::N4,
                "5" => Degree::N5,
                "6" => Degree::N6,
                "7" => Degree::N7,
                _ => {
                    // Try sargam
                    match value.trim().to_uppercase().as_str() {
                        "S" => Degree::N1,
                        "R" => Degree::N2,
                        "G" => Degree::N3,
                        "M" => Degree::N4,
                        "P" => Degree::N5,
                        "D" => Degree::N6,
                        "N" => Degree::N7,
                        _ => {
                            // Try western
                            let upper = value.trim().to_uppercase();
                            match upper.as_str() {
                                "C" => Degree::N1, "D" => Degree::N2, "E" => Degree::N3, "F" => Degree::N4,
                                "G" => Degree::N5, "A" => Degree::N6, "B" => Degree::N7,
                                _ => return Err(format!("Unknown note (tried all systems): {}", value)),
                            }
                        }
                    }
                }
            }
        },
    };
    
    // Apply accidentals
    let degree = if let Some(acc) = accidental {
        match acc.as_str() {
            "#" | "s" => apply_sharp(base_degree),
            "b" => apply_flat(base_degree),
            _ => base_degree,
        }
    } else {
        base_degree
    };
    
    Ok(degree)
}

fn apply_sharp(degree: Degree) -> Degree {
    use Degree::*;
    match degree {
        N1 => N1s, N2 => N2s, N3 => N3s, N4 => N4s,
        N5 => N5s, N6 => N6s, N7 => N7s,
        _ => degree, // Already sharp or other
    }
}

fn apply_flat(degree: Degree) -> Degree {
    use Degree::*;
    match degree {
        N1 => N1b, N2 => N2b, N3 => N3b, N4 => N4b,
        N5 => N5b, N6 => N6b, N7 => N7b,
        _ => degree, // Already flat or other
    }
}

pub fn convert_barline_to_string(barline: &crate::ast::Barline) -> String {
    match barline {
        crate::ast::Barline::Single => "|".to_string(),
        crate::ast::Barline::Double => "||".to_string(),
        crate::ast::Barline::Final => "|]".to_string(),
        crate::ast::Barline::ReverseFinal => "[|".to_string(),
        crate::ast::Barline::LeftRepeat => "|:".to_string(),
        crate::ast::Barline::RightRepeat => ":|".to_string(),
    }
}
