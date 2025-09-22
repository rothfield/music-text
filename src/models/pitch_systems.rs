use crate::models::pitch::{Degree, Notation};
use crate::parse::model::{NotationSystem, PitchCode};
use once_cell::sync::Lazy;
use regex::Regex;

// Import all pitch system modules
pub mod western;
pub mod number;
pub mod sargam;
pub mod bhatkhande;
pub mod tabla;

/// Type conversion between core Notation and parser NotationSystem
fn notation_to_system(notation: Notation) -> NotationSystem {
    match notation {
        Notation::Western => NotationSystem::Western,
        Notation::Number => NotationSystem::Number,
        Notation::Sargam => NotationSystem::Sargam,
        Notation::Bhatkhande => NotationSystem::Bhatkhande,
        Notation::Tabla => NotationSystem::Tabla,
    }
}

fn system_to_notation(system: NotationSystem) -> Notation {
    match system {
        NotationSystem::Western => Notation::Western,
        NotationSystem::Number => Notation::Number,
        NotationSystem::Sargam => Notation::Sargam,
        NotationSystem::Bhatkhande => Notation::Bhatkhande,
        NotationSystem::Tabla => Notation::Tabla,
    }
}

/// Convert Degree (core models) to PitchCode (parser models)
pub fn degree_to_pitch_code(degree: Degree) -> PitchCode {
    match degree {
        // 1 series
        Degree::N1bb => PitchCode::N1bb, Degree::N1b => PitchCode::N1b, Degree::N1 => PitchCode::N1,
        Degree::N1s => PitchCode::N1s, Degree::N1ss => PitchCode::N1ss,
        // 2 series
        Degree::N2bb => PitchCode::N2bb, Degree::N2b => PitchCode::N2b, Degree::N2 => PitchCode::N2,
        Degree::N2s => PitchCode::N2s, Degree::N2ss => PitchCode::N2ss,
        // 3 series
        Degree::N3bb => PitchCode::N3bb, Degree::N3b => PitchCode::N3b, Degree::N3 => PitchCode::N3,
        Degree::N3s => PitchCode::N3s, Degree::N3ss => PitchCode::N3ss,
        // 4 series
        Degree::N4bb => PitchCode::N4bb, Degree::N4b => PitchCode::N4b, Degree::N4 => PitchCode::N4,
        Degree::N4s => PitchCode::N4s, Degree::N4ss => PitchCode::N4ss,
        // 5 series
        Degree::N5bb => PitchCode::N5bb, Degree::N5b => PitchCode::N5b, Degree::N5 => PitchCode::N5,
        Degree::N5s => PitchCode::N5s, Degree::N5ss => PitchCode::N5ss,
        // 6 series
        Degree::N6bb => PitchCode::N6bb, Degree::N6b => PitchCode::N6b, Degree::N6 => PitchCode::N6,
        Degree::N6s => PitchCode::N6s, Degree::N6ss => PitchCode::N6ss,
        // 7 series
        Degree::N7bb => PitchCode::N7bb, Degree::N7b => PitchCode::N7b, Degree::N7 => PitchCode::N7,
        Degree::N7s => PitchCode::N7s, Degree::N7ss => PitchCode::N7ss,
    }
}

/// Convert PitchCode (parser models) to Degree (core models)
pub fn pitch_code_to_degree(pitch_code: PitchCode) -> Degree {
    match pitch_code {
        // 1 series
        PitchCode::N1bb => Degree::N1bb, PitchCode::N1b => Degree::N1b, PitchCode::N1 => Degree::N1,
        PitchCode::N1s => Degree::N1s, PitchCode::N1ss => Degree::N1ss,
        // 2 series
        PitchCode::N2bb => Degree::N2bb, PitchCode::N2b => Degree::N2b, PitchCode::N2 => Degree::N2,
        PitchCode::N2s => Degree::N2s, PitchCode::N2ss => Degree::N2ss,
        // 3 series
        PitchCode::N3bb => Degree::N3bb, PitchCode::N3b => Degree::N3b, PitchCode::N3 => Degree::N3,
        PitchCode::N3s => Degree::N3s, PitchCode::N3ss => Degree::N3ss,
        // 4 series
        PitchCode::N4bb => Degree::N4bb, PitchCode::N4b => Degree::N4b, PitchCode::N4 => Degree::N4,
        PitchCode::N4s => Degree::N4s, PitchCode::N4ss => Degree::N4ss,
        // 5 series
        PitchCode::N5bb => Degree::N5bb, PitchCode::N5b => Degree::N5b, PitchCode::N5 => Degree::N5,
        PitchCode::N5s => Degree::N5s, PitchCode::N5ss => Degree::N5ss,
        // 6 series
        PitchCode::N6bb => Degree::N6bb, PitchCode::N6b => Degree::N6b, PitchCode::N6 => Degree::N6,
        PitchCode::N6s => Degree::N6s, PitchCode::N6ss => Degree::N6ss,
        // 7 series
        PitchCode::N7bb => Degree::N7bb, PitchCode::N7b => Degree::N7b, PitchCode::N7 => Degree::N7,
        PitchCode::N7s => Degree::N7s, PitchCode::N7ss => Degree::N7ss,
    }
}

/// Compiled regex patterns for each notation system
static TABLA_RE: Lazy<Regex> = Lazy::new(|| build_regex_for_system(NotationSystem::Tabla));
static SARGAM_RE: Lazy<Regex> = Lazy::new(|| build_regex_for_system(NotationSystem::Sargam));
static NUMBER_RE: Lazy<Regex> = Lazy::new(|| build_regex_for_system(NotationSystem::Number));
static WESTERN_RE: Lazy<Regex> = Lazy::new(|| build_regex_for_system(NotationSystem::Western));
static BHATKHANDE_RE: Lazy<Regex> = Lazy::new(|| build_regex_for_system(NotationSystem::Bhatkhande));

/// Build regex pattern for a specific notation system
fn build_regex_for_system(system: NotationSystem) -> Regex {
    let symbols = match system {
        NotationSystem::Tabla => tabla::get_all_symbols(),
        NotationSystem::Sargam => sargam::get_all_symbols(),
        NotationSystem::Number => number::get_all_symbols(),
        NotationSystem::Western => western::get_all_symbols(),
        NotationSystem::Bhatkhande => bhatkhande::get_all_symbols(),
    };

    // Escape special regex characters and join with alternation
    let escaped_symbols: Vec<String> = symbols.iter()
        .map(|s| regex::escape(s))
        .collect();

    let pattern = format!("(?i)({})", escaped_symbols.join("|"));
    Regex::new(&pattern).expect("Failed to compile regex pattern")
}

/// Get the compiled regex for a specific notation system
pub fn get_regex_for_system(system: NotationSystem) -> &'static Regex {
    match system {
        NotationSystem::Tabla => &*TABLA_RE,
        NotationSystem::Sargam => &*SARGAM_RE,
        NotationSystem::Number => &*NUMBER_RE,
        NotationSystem::Western => &*WESTERN_RE,
        NotationSystem::Bhatkhande => &*BHATKHANDE_RE,
    }
}

/// Classify input text and return best matching notation system with confidence score
pub fn classify_notation_system(input: &str) -> (NotationSystem, f32) {
    let systems = [
        (NotationSystem::Tabla, &*TABLA_RE),
        (NotationSystem::Sargam, &*SARGAM_RE),
        (NotationSystem::Number, &*NUMBER_RE),
        (NotationSystem::Western, &*WESTERN_RE),
        (NotationSystem::Bhatkhande, &*BHATKHANDE_RE),
    ];

    let mut best_system = NotationSystem::Number; // Default
    let mut best_score = 0.0;

    for (system, regex) in &systems {
        let matches: Vec<_> = regex.find_iter(input).collect();
        let matched_chars: usize = matches.iter().map(|m| m.as_str().len()).sum();
        let total_musical_chars = input.chars().filter(|c| !c.is_whitespace() && *c != '|').count();

        let score = if total_musical_chars > 0 {
            matched_chars as f32 / total_musical_chars as f32
        } else {
            0.0
        };

        if score > best_score {
            best_score = score;
            best_system = *system;
        }
    }

    (best_system, best_score)
}

/// Main dispatcher for pitch lookups across all notation systems
///
/// This function routes symbol lookups to the appropriate notation system
/// and maintains backward compatibility with the original lookup_pitch API.
pub fn lookup_pitch(symbol: &str, notation: Notation) -> Option<Degree> {
    match notation {
        Notation::Western => western::lookup(symbol),
        Notation::Number => number::lookup(symbol),
        Notation::Sargam => sargam::lookup(symbol),
        Notation::Bhatkhande => bhatkhande::lookup(symbol),
        Notation::Tabla => tabla::lookup(symbol),
    }
}

/// Convert a Degree (pitch) to its string representation in the given notation system
///
/// This is the reverse operation of lookup_pitch - takes a degree and notation system
/// and returns the string representation of that pitch in that notation.
pub fn degree_to_string(degree: Degree, notation: Notation) -> Option<String> {
    match notation {
        Notation::Western => western::degree_to_string(degree),
        Notation::Number => number::degree_to_string(degree),
        Notation::Sargam => sargam::degree_to_string(degree),
        Notation::Bhatkhande => bhatkhande::degree_to_string(degree),
        Notation::Tabla => tabla::degree_to_string(degree),
    }
}

/// Convert a PitchCode directly to its string representation in the given notation system
///
/// This is a direct mapping that bypasses the Degree abstraction for efficiency.
/// Each pitch system has its own direct PitchCode → String mapping.
pub fn pitchcode_to_string(pitchcode: PitchCode, notation: Notation) -> Option<String> {
    match notation {
        Notation::Western => western::pitchcode_to_string(pitchcode),
        Notation::Number => number::pitchcode_to_string(pitchcode),
        Notation::Sargam => sargam::pitchcode_to_string(pitchcode),
        Notation::Bhatkhande => bhatkhande::pitchcode_to_string(pitchcode),
        Notation::Tabla => tabla::pitchcode_to_string(pitchcode),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatcher_western() {
        assert_eq!(lookup_pitch("C", Notation::Western), Some(Degree::N1));
        assert_eq!(lookup_pitch("Bb", Notation::Western), Some(Degree::N7b));
        assert_eq!(lookup_pitch("A#", Notation::Western), Some(Degree::N6s));
    }

    #[test]
    fn test_dispatcher_number() {
        assert_eq!(lookup_pitch("1", Notation::Number), Some(Degree::N1));
        assert_eq!(lookup_pitch("7b", Notation::Number), Some(Degree::N7b));
        assert_eq!(lookup_pitch("4#", Notation::Number), Some(Degree::N4s));
    }

    #[test]
    fn test_dispatcher_sargam() {
        assert_eq!(lookup_pitch("S", Notation::Sargam), Some(Degree::N1));
        assert_eq!(lookup_pitch("r", Notation::Sargam), Some(Degree::N2b));
        assert_eq!(lookup_pitch("n", Notation::Sargam), Some(Degree::N7b));
    }

    #[test]
    fn test_dispatcher_bhatkhande() {
        assert_eq!(lookup_pitch("स", Notation::Bhatkhande), Some(Degree::N1));
        assert_eq!(lookup_pitch("S", Notation::Bhatkhande), Some(Degree::N1));
        assert_eq!(lookup_pitch("M#", Notation::Bhatkhande), Some(Degree::N4s));
    }

    #[test]
    fn test_dispatcher_tabla() {
        assert_eq!(lookup_pitch("dha", Notation::Tabla), Some(Degree::N1));
        assert_eq!(lookup_pitch("ge", Notation::Tabla), Some(Degree::N1));
        assert_eq!(lookup_pitch("unknown", Notation::Tabla), None);
    }

    #[test]
    fn test_dispatcher_invalid() {
        assert_eq!(lookup_pitch("X", Notation::Sargam), None);
        assert_eq!(lookup_pitch("", Notation::Western), None);
    }
}