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
    let full_pitch_str = source.value.as_str(); // Complete pitch token
    
    // With atomic grammar, we can directly extract the notation system from the rule type
    let notation_system = match pitch_pair.into_inner().next() {
        Some(inner) => match inner.as_rule() {
            Rule::number_pitch => NotationSystem::Number,
            Rule::western_pitch => NotationSystem::Western,
            Rule::sargam_pitch => NotationSystem::Sargam,
            Rule::bhatkhande_pitch => NotationSystem::Bhatkhande,
            _ => context_notation_system, // fallback
        }
        None => context_notation_system,
    };
    
    // Direct conversion from complete pitch token to PitchCode
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