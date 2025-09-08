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
    let mut base_pitch = String::new();
    let mut accidentals = String::new();
    let mut notation_system = context_notation_system;
    
    // Extract base pitch and accidentals from inner rules
    for inner in pitch_pair.into_inner() {
        match inner.as_rule() {
            Rule::number_pitch => {
                notation_system = NotationSystem::Number;
                base_pitch = inner.as_str().to_string();
            },
            Rule::western_pitch => {
                notation_system = NotationSystem::Western;
                base_pitch = inner.as_str().to_string();
            },
            Rule::sargam_pitch => {
                notation_system = NotationSystem::Sargam;
                base_pitch = inner.as_str().to_string();
            },
            Rule::bhatkhande_pitch => {
                notation_system = NotationSystem::Bhatkhande;
                base_pitch = inner.as_str().to_string();
            },
            Rule::tabla_pitch => {
                // Tabla bols all map to N1 for now
                notation_system = NotationSystem::Number;
                base_pitch = "1".to_string();
            },
            // Handle composite pitch structure (base_pitch + accidentals)
            _ => {
                for sub_inner in inner.into_inner() {
                    match sub_inner.as_rule() {
                        Rule::number_pitch => {
                            notation_system = NotationSystem::Number;
                            base_pitch = sub_inner.as_str().to_string();
                        },
                        Rule::western_pitch => {
                            notation_system = NotationSystem::Western;
                            base_pitch = sub_inner.as_str().to_string();
                        },
                        Rule::sargam_pitch => {
                            notation_system = NotationSystem::Sargam;
                            base_pitch = sub_inner.as_str().to_string();
                        },
                        Rule::bhatkhande_pitch => {
                            notation_system = NotationSystem::Bhatkhande;
                            base_pitch = sub_inner.as_str().to_string();
                        },
                        Rule::tabla_pitch => {
                            // Tabla bols all map to N1 for now
                            notation_system = NotationSystem::Number;
                            base_pitch = "1".to_string();
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
    
    // Construct complete pitch string
    let complete_pitch = if accidentals.is_empty() {
        base_pitch.clone()
    } else {
        format!("{}{}", base_pitch, accidentals)
    };
    
    // Use the full span text which should include accidentals  
    let pitch_code = PitchCode::from_source(full_pitch_str);
    
    // Extract base note for display (remove modifiers for syllable field)
    let base_pitch = extract_base_pitch(full_pitch_str);
    
    let note = Note {
        syllable: base_pitch, // Base note without modifiers for display
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