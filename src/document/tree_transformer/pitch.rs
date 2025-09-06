use crate::document::pest_interface::{Pair, Rule};
use crate::document::model::{MusicalElement, PitchCode, Note, NotationSystem};
use super::helpers::source_from_span;

pub(super) fn resolve_notation_system(syllable: &str, context_system: NotationSystem) -> NotationSystem {
    // Remove accidentals to get base syllable
    let base_syllable = syllable.trim_end_matches('#').trim_end_matches('b');
    
    match base_syllable {
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
    let mut syllable = String::new();
    let source = source_from_span(pitch_pair.as_span());
    
    for inner_pair in pitch_pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::base_pitch => {
                for base_inner in inner_pair.into_inner() {
                    match base_inner.as_rule() {
                        Rule::number_pitch | Rule::letter_pitch | Rule::sargam_pitch | Rule::bhatkhande_pitch => {
                            syllable = base_inner.as_str().to_string();
                        }
                        _ => {}
                    }
                }
            }
            Rule::accidentals => {
                // Accidentals are now captured in source.value, no separate field
            }
            _ => {}
        }
    }
    
    let pitch_code = PitchCode::from_source(&syllable);
    
    let notation_system = resolve_notation_system(&syllable, context_notation_system);
    
    let note = Note {
        syllable,
        octave: 0, // Default octave for now
        pitch_code,
        notation_system,
        source, // Contains full text including accidentals
        in_slur,
        in_beat_group,
    };
    
    Ok(MusicalElement::Note(note))
}