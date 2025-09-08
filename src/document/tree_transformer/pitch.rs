use crate::document::pest_interface::{Pair, Rule};
use crate::document::model::{MusicalElement, PitchCode, Note, NotationSystem};
use super::helpers::source_from_span;

pub(super) fn resolve_notation_system(syllable: &str, context_system: NotationSystem) -> NotationSystem {
    // Extract base note from complete pitch token
    let base_syllable = extract_base_pitch(syllable);
    
    match base_syllable.as_str() {
        // Unambiguous cases - ignore context
        "1" | "2" | "3" | "4" | "5" | "6" | "7" => NotationSystem::Number,
        "C" | "E" | "A" | "B" => NotationSystem::Western,
        "S" | "r" | "m" | "n" | "d" | "g" => NotationSystem::Sargam,
        "स" | "रे" | "र" | "ग" | "म" | "प" | "ध" | "द" | "नि" | "न" => NotationSystem::Bhatkhande,
        "dha" | "dhin" | "ta" | "ka" | "taka" | "trkt" | "ge" |
        "Dha" | "Dhin" | "Ta" | "Ka" | "Taka" | "Trkt" | "Ge" |
        "DHA" | "DHIN" | "TA" | "KA" | "TAKA" | "TRKT" | "GE" => NotationSystem::Tabla,
        
        // Ambiguous cases - use context system
        "D" | "F" | "G" | "R" | "M" | "P" | "N" => context_system,
        
        // Default fallback
        _ => NotationSystem::Number,
    }
}

pub(super) fn transform_pitch(pitch_pair: Pair<Rule>, in_slur: bool, in_beat_group: bool, context_notation_system: NotationSystem) -> Result<MusicalElement, String> {
    let source = source_from_span(pitch_pair.as_span());
    let full_pitch_str = source.value.as_str(); // Complete pitch token including accidentals
    
    // Parse the pitch components from the PEST tree
    let mut accidentals = String::new();
    let mut notation_system = context_notation_system;
    
    // Extract notation system and accidentals from inner rules
    for inner in pitch_pair.into_inner() {
        match inner.as_rule() {
            Rule::number_pitch => {
                notation_system = NotationSystem::Number;
            },
            Rule::western_pitch => {
                notation_system = NotationSystem::Western;
            },
            Rule::sargam_pitch => {
                notation_system = NotationSystem::Sargam;
            },
            Rule::bhatkhande_pitch => {
                notation_system = NotationSystem::Bhatkhande;
            },
            Rule::tabla_pitch => {
                notation_system = NotationSystem::Tabla;
            },
            // Handle composite pitch structure (base_pitch + accidentals)
            _ => {
                for sub_inner in inner.into_inner() {
                    match sub_inner.as_rule() {
                        Rule::number_pitch => {
                            notation_system = NotationSystem::Number;
                        },
                        Rule::western_pitch => {
                            notation_system = NotationSystem::Western;
                        },
                        Rule::sargam_pitch => {
                            notation_system = NotationSystem::Sargam;
                        },
                        Rule::bhatkhande_pitch => {
                            notation_system = NotationSystem::Bhatkhande;
                        },
                        Rule::tabla_pitch => {
                            notation_system = NotationSystem::Tabla;
                        },
                        // Capture accidentals if they exist as separate rules
                        _ => {
                            let text = sub_inner.as_str();
                            if text == "#" || text == "b" || text == "##" || text == "bb" {
                                accidentals.push_str(text);
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Use the full span text which should include accidentals  
    let pitch_code = PitchCode::from_source(full_pitch_str);
    
    // For tabla notation, use the original source text as syllable
    let syllable = if notation_system == NotationSystem::Tabla {
        full_pitch_str.to_string() // Use original tabla bol as syllable
    } else {
        // For other notation systems, don't set syllables (they don't need lyrics)
        "".to_string() // Empty syllable means no lyrics will be generated
    };
    
    let note = Note {
        syllable, // Tabla bols use source text, others use base pitch
        octave: 0, // Default octave for now  
        pitch_code, // Complete pitch information
        notation_system,
        source, // Contains complete pitch token
        in_slur,
        in_beat_group,
    };
    
    Ok(MusicalElement::Note(note))
}

/// Extract base note from complete pitch token (remove modifiers)
fn extract_base_pitch(full_pitch: &str) -> String {
    // Remove pitch modifiers to get base note for display
    full_pitch
        .trim_end_matches("##")
        .trim_end_matches("#")
        .trim_end_matches("bb")
        .trim_end_matches("b")
        .to_string()
}