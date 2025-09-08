// Document parser module - clean separation of concerns
// - grammar.pest: Defines the syntax
// - pest_interface.rs: Pest parser generation and interface
// - tree_transformer.rs: Transforms parse tree into AST
// - model.rs: Domain types (Document, Stave, etc.)
// - compact_notation_preprocessor.rs: Handles compact notation like "SRG"

pub mod pest_interface;
pub mod tree_transformer;
pub mod model;
pub mod compact_notation_preprocessor;

use serde::{Serialize, Deserialize};

// Re-export key types and functions for convenience
pub use model::{Document, Stave, ContentLine, MusicalElement, TextLine, Position, PitchCode};
pub use tree_transformer::build_document;
pub use pest_interface::{parse, Rule, Error};

// Convenience function that combines preprocessing, parsing and transformation
pub fn parse_document(input: &str) -> Result<Document, String> {
    // Step 1: Check for consecutive notation first (higher priority than grammar parsing)
    if let Some(stave) = try_create_consecutive_stave(input) {
        // Create document with consecutive stave directly
        let document_source = model::Source {
            value: input.to_string(),
            position: model::Position { line: 1, column: 1 },
        };
        return Ok(Document { 
            content: vec![model::DocumentElement::SingleStave(stave)], 
            source: document_source 
        });
    }
    
    // Step 2: Try normal parsing if consecutive detection fails
    let document = build_document(input)?;
    
    Ok(document)
}

/// Try creating stave directly from consecutive characters
fn try_create_consecutive_stave(input: &str) -> Option<Stave> {
    // Only for single-line input (trim first to ignore trailing newlines)
    let trimmed_input = input.trim();
    if trimmed_input.contains('\n') {
        return None;
    }
    
    // Must be 3+ consecutive characters
    let chars: Vec<char> = trimmed_input.chars().collect();
    if chars.len() < 3 {
        return None;
    }
    
    // All characters must be from same musical system
    let system = detect_system_from_consecutive(&chars)?;
    
    // Create stave directly
    use model::{Source, Position, ContentLine, MusicalElement, Note};
    
    let source = Source {
        value: trimmed_input.to_string(),
        position: Position { line: 1, column: 1 },
    };
    
    // Convert chars to musical elements
    let mut elements = Vec::new();
    for (i, &ch) in chars.iter().enumerate() {
        if i > 0 {
            // Add space between notes
            elements.push(MusicalElement::Space {
                count: 1,
                in_slur: false,
                in_beat_group: false,
                source: source.clone(),
            });
        }
        
        // Convert character to pitch
        if let Some(pitch_code) = char_to_pitchcode(ch, system) {
            elements.push(MusicalElement::Note(Note {
                syllable: ch.to_string(),
                octave: 0,
                pitch_code,
                notation_system: match system {
                    NotationSystem::Number => model::NotationSystem::Number,
                    NotationSystem::Sargam => model::NotationSystem::Sargam,
                    NotationSystem::Western => model::NotationSystem::Western,
                    NotationSystem::Bhatkhande => model::NotationSystem::Bhatkhande,
                    NotationSystem::Tabla => model::NotationSystem::Tabla,
                },
                source: source.clone(),
                in_slur: false,
                in_beat_group: false,
            }));
        }
    }
    
    let content_line = ContentLine {
        elements,
        source: source.clone(),
    };
    
    Some(Stave {
        text_lines_before: Vec::new(),
        content_line,
        text_lines_after: Vec::new(),
        notation_system: match system {
            NotationSystem::Number => model::NotationSystem::Number,
            NotationSystem::Sargam => model::NotationSystem::Sargam,
            NotationSystem::Western => model::NotationSystem::Western,
            NotationSystem::Bhatkhande => model::NotationSystem::Bhatkhande,
            NotationSystem::Tabla => model::NotationSystem::Tabla,
        },
        source,
    })
}

/// Convert character to PitchCode based on system
fn char_to_pitchcode(ch: char, system: NotationSystem) -> Option<PitchCode> {
    match system {
        NotationSystem::Number => match ch {
            '1' => Some(PitchCode::N1),
            '2' => Some(PitchCode::N2),
            '3' => Some(PitchCode::N3),
            '4' => Some(PitchCode::N4),
            '5' => Some(PitchCode::N5),
            '6' => Some(PitchCode::N6),
            '7' => Some(PitchCode::N7),
            _ => None,
        },
        NotationSystem::Sargam => match ch {
            'S' | 's' => Some(PitchCode::N1), // Sa
            'R' | 'r' => Some(PitchCode::N2), // Re  
            'G' | 'g' => Some(PitchCode::N3), // Ga
            'M' | 'm' => Some(PitchCode::N4), // Ma
            'P' | 'p' => Some(PitchCode::N5), // Pa
            'D' | 'd' => Some(PitchCode::N6), // Dha
            'N' | 'n' => Some(PitchCode::N7), // Ni
            _ => None,
        },
        NotationSystem::Western => match ch {
            'C' => Some(PitchCode::N1),
            'D' => Some(PitchCode::N2),
            'E' => Some(PitchCode::N3),
            'F' => Some(PitchCode::N4),
            'G' => Some(PitchCode::N5),
            'A' => Some(PitchCode::N6),
            'B' => Some(PitchCode::N7),
            _ => None,
        },
        NotationSystem::Bhatkhande => None, // Not implemented
        NotationSystem::Tabla => None, // Not implemented for single characters (tabla bols are multi-character)
    }
}

/// Detect system from consecutive characters
fn detect_system_from_consecutive(chars: &[char]) -> Option<NotationSystem> {
    // All must be numbers
    if chars.iter().all(|&c| matches!(c, '1'..='7')) {
        return Some(NotationSystem::Number);
    }
    
    // All must be Sargam
    if chars.iter().all(|&c| matches!(c, 
        'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N' |           // Sargam upper
        's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n'             // Sargam lower
    )) {
        return Some(NotationSystem::Sargam);
    }
    
    // All must be Western
    if chars.iter().all(|&c| matches!(c, 'C' | 'D' | 'E' | 'F' | 'G' | 'A' | 'B')) {
        return Some(NotationSystem::Western);
    }
    
    None
}


#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
enum NotationSystem {
    Number,
    Western, 
    Sargam,
    Bhatkhande,
    Tabla,
}