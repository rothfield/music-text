use serde::{Deserialize, Serialize};
use std::fmt;

// Unified notation system models

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotationSystem {
    Number,     // 1 2 3 4 5 6 7 (numeric system)
    Western,    // C D E F G A B (standard western notes)
    Sargam,     // S R G M P D N (Indian classical music)
    Bhatkhande, // स रे ग म प ध नि (Devanagari script)
    Tabla,      // dha dhin ta ka taka trkt ge (tabla bols/percussion syllables)
}

// Alternative notation enum for legacy compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Notation {
    Western,
    Number,
    Sargam,
    Tabla,
    Bhatkhande,
}

impl fmt::Display for Notation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Notation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Notation::Western => "Western",
            Notation::Number => "Number",
            Notation::Sargam => "Sargam",
            Notation::Tabla => "Tabla",
            Notation::Bhatkhande => "Bhatkhande",
        }
    }
}

impl From<NotationSystem> for Notation {
    fn from(system: NotationSystem) -> Self {
        match system {
            NotationSystem::Western => Notation::Western,
            NotationSystem::Number => Notation::Number,
            NotationSystem::Sargam => Notation::Sargam,
            NotationSystem::Tabla => Notation::Tabla,
            NotationSystem::Bhatkhande => Notation::Bhatkhande,
        }
    }
}

impl NotationSystem {
    /// Detect notation system from syllable (complete pitch token)
    pub fn from_syllable(syllable: &str) -> Self {
        // Extract base note from complete pitch token
        let base_syllable = if syllable.len() > 1 {
            if syllable.ends_with("##") { &syllable[..syllable.len()-2] }
            else if syllable.ends_with('#') || syllable.ends_with('b') { &syllable[..syllable.len()-1] }
            else if syllable.ends_with("bb") { &syllable[..syllable.len()-2] }
            else { syllable }
        } else { syllable };

        match base_syllable {
            // Number notation
            "1" | "2" | "3" | "4" | "5" | "6" | "7" => NotationSystem::Number,
            // Unambiguous Western notation
            "C" | "E" | "A" | "B" => NotationSystem::Western,
            // Unambiguous Sargam notation
            "s" | "S" | "r" | "g" | "m" | "n" | "d" | "p" => NotationSystem::Sargam,
            // Bhatkhande Devanagari notation
            "स" | "रे" | "र" | "ग" | "म" | "प" | "ध" | "द" | "नि" | "न" => NotationSystem::Bhatkhande,
            // Tabla notation (all cases)
            "dha" | "dhin" | "ta" | "ka" | "taka" | "trkt" | "ge" |
            "Dha" | "Dhin" | "Ta" | "Ka" | "Taka" | "Trkt" | "Ge" |
            "DHA" | "DHIN" | "TA" | "KA" | "TAKA" | "TRKT" | "GE" => NotationSystem::Tabla,
            // Ambiguous letters (case-specific disambiguation)
            "F" => NotationSystem::Western,  // F doesn't exist in Sargam, clearly Western
            "R" | "G" | "M" | "P" | "D" | "N" => NotationSystem::Sargam, // More commonly Sargam
            // Default to Number if unrecognized
            _ => NotationSystem::Number,
        }
    }
}

// Unified pitch codes - matches old Degree enum with complete pitch coverage
// Also known as Degree in legacy code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PitchCode {
    // 1 series (Do/Sa/C)
    N1bb, N1b, N1, N1s, N1ss,
    // 2 series (Re/D)
    N2bb, N2b, N2, N2s, N2ss,
    // 3 series (Mi/Ga/E)
    N3bb, N3b, N3, N3s, N3ss,
    // 4 series (Fa/Ma/F)
    N4bb, N4b, N4, N4s, N4ss,
    // 5 series (Sol/Pa/G)
    N5bb, N5b, N5, N5s, N5ss,
    // 6 series (La/Dha/A)
    N6bb, N6b, N6, N6s, N6ss,
    // 7 series (Ti/Ni/B)
    N7bb, N7b, N7, N7s, N7ss,
}

// Legacy alias for compatibility
pub type Degree = PitchCode;

/// Lookup pitch from symbol and notation system
pub fn lookup_pitch(symbol: &str, notation: Notation) -> Option<PitchCode> {
    match notation {
        Notation::Western => match symbol {
            // Natural notes
            "C" => Some(PitchCode::N1),
            "D" => Some(PitchCode::N2),
            "E" => Some(PitchCode::N3),
            "F" => Some(PitchCode::N4),
            "G" => Some(PitchCode::N5),
            "A" => Some(PitchCode::N6),
            "B" => Some(PitchCode::N7),
            // Sharps
            "C#" => Some(PitchCode::N1s),
            "D#" => Some(PitchCode::N2s),
            "E#" => Some(PitchCode::N3s),
            "F#" => Some(PitchCode::N4s),
            "G#" => Some(PitchCode::N5s),
            "A#" => Some(PitchCode::N6s),
            "B#" => Some(PitchCode::N7s),
            // Flats
            "Cb" => Some(PitchCode::N1b),
            "Db" => Some(PitchCode::N2b),
            "Eb" => Some(PitchCode::N3b),
            "Fb" => Some(PitchCode::N4b),
            "Gb" => Some(PitchCode::N5b),
            "Ab" => Some(PitchCode::N6b),
            "Bb" => Some(PitchCode::N7b),
            // Double sharps
            "C##" => Some(PitchCode::N1ss),
            "D##" => Some(PitchCode::N2ss),
            "E##" => Some(PitchCode::N3ss),
            "F##" => Some(PitchCode::N4ss),
            "G##" => Some(PitchCode::N5ss),
            "A##" => Some(PitchCode::N6ss),
            "B##" => Some(PitchCode::N7ss),
            // Double flats
            "Cbb" => Some(PitchCode::N1bb),
            "Dbb" => Some(PitchCode::N2bb),
            "Ebb" => Some(PitchCode::N3bb),
            "Fbb" => Some(PitchCode::N4bb),
            "Gbb" => Some(PitchCode::N5bb),
            "Abb" => Some(PitchCode::N6bb),
            "Bbb" => Some(PitchCode::N7bb),
            _ => None,
        },
        Notation::Number => match symbol {
            "1" => Some(PitchCode::N1),
            "2" => Some(PitchCode::N2),
            "3" => Some(PitchCode::N3),
            "4" => Some(PitchCode::N4),
            "5" => Some(PitchCode::N5),
            "6" => Some(PitchCode::N6),
            "7" => Some(PitchCode::N7),
            // With sharps and flats
            "1#" => Some(PitchCode::N1s), "1b" => Some(PitchCode::N1b),
            "2#" => Some(PitchCode::N2s), "2b" => Some(PitchCode::N2b),
            "3#" => Some(PitchCode::N3s), "3b" => Some(PitchCode::N3b),
            "4#" => Some(PitchCode::N4s), "4b" => Some(PitchCode::N4b),
            "5#" => Some(PitchCode::N5s), "5b" => Some(PitchCode::N5b),
            "6#" => Some(PitchCode::N6s), "6b" => Some(PitchCode::N6b),
            "7#" => Some(PitchCode::N7s), "7b" => Some(PitchCode::N7b),
            // Double sharps and flats
            "1##" => Some(PitchCode::N1ss), "1bb" => Some(PitchCode::N1bb),
            "2##" => Some(PitchCode::N2ss), "2bb" => Some(PitchCode::N2bb),
            "3##" => Some(PitchCode::N3ss), "3bb" => Some(PitchCode::N3bb),
            "4##" => Some(PitchCode::N4ss), "4bb" => Some(PitchCode::N4bb),
            "5##" => Some(PitchCode::N5ss), "5bb" => Some(PitchCode::N5bb),
            "6##" => Some(PitchCode::N6ss), "6bb" => Some(PitchCode::N6bb),
            "7##" => Some(PitchCode::N7ss), "7bb" => Some(PitchCode::N7bb),
            _ => None,
        },
        Notation::Sargam => match symbol {
            // Basic sargam notes (uppercase and lowercase)
            "S" | "s" => Some(PitchCode::N1),  // Sa
            "R" => Some(PitchCode::N2),         // Re (shuddha)
            "r" => Some(PitchCode::N2b),        // Re (komal)
            "G" => Some(PitchCode::N3),         // Ga (shuddha)
            "g" => Some(PitchCode::N3b),        // Ga (komal)
            "M" => Some(PitchCode::N4s),        // Ma (tivra)
            "m" => Some(PitchCode::N4),         // Ma (shuddha)
            "P" | "p" => Some(PitchCode::N5),   // Pa
            "D" => Some(PitchCode::N6),         // Dha (shuddha)
            "d" => Some(PitchCode::N6b),        // Dha (komal)
            "N" => Some(PitchCode::N7),         // Ni (shuddha)
            "n" => Some(PitchCode::N7b),        // Ni (komal)
            // With explicit sharps and flats
            "S#" | "s#" => Some(PitchCode::N1s),
            "R#" => Some(PitchCode::N2s), "Rb" => Some(PitchCode::N2b),
            "G#" => Some(PitchCode::N3s), "Gb" => Some(PitchCode::N3b),
            "M#" => Some(PitchCode::N4ss), "Mb" => Some(PitchCode::N4b),
            "P#" | "p#" => Some(PitchCode::N5s), "Pb" | "pb" => Some(PitchCode::N5b),
            "D#" => Some(PitchCode::N6s), "Db" => Some(PitchCode::N6b),
            "N#" => Some(PitchCode::N7s), "Nb" => Some(PitchCode::N7b),
            _ => None,
        },
        Notation::Tabla => match symbol {
            // All tabla bols map to N1 (tonic) since tabla is percussion
            "dha" | "dhin" | "ta" | "ka" | "taka" | "trkt" | "ge" |
            "Dha" | "Dhin" | "Ta" | "Ka" | "Taka" | "Trkt" | "Ge" |
            "DHA" | "DHIN" | "TA" | "KA" | "TAKA" | "TRKT" | "GE" => Some(PitchCode::N1),
            _ => None,
        },
        Notation::Bhatkhande => match symbol {
            // Devanagari script
            "स" => Some(PitchCode::N1), "रे" => Some(PitchCode::N2), "र" => Some(PitchCode::N2b),
            "ग" => Some(PitchCode::N3), "म" => Some(PitchCode::N4), "प" => Some(PitchCode::N5),
            "ध" => Some(PitchCode::N6), "द" => Some(PitchCode::N6b), "नि" => Some(PitchCode::N7), "न" => Some(PitchCode::N7b),
            // Roman equivalents
            "S" => Some(PitchCode::N1), "R" => Some(PitchCode::N2), "G" => Some(PitchCode::N3),
            "M" => Some(PitchCode::N4), "P" => Some(PitchCode::N5), "D" => Some(PitchCode::N6), "N" => Some(PitchCode::N7),
            // With accidentals
            "स#" | "S#" => Some(PitchCode::N1s), "म#" | "M#" => Some(PitchCode::N4s),
            "धb" | "Db" => Some(PitchCode::N6b),
            _ => None,
        },
    }
}

impl PitchCode {
    /// Convert complete source pitch token to normalized pitch code
    /// Context-aware version that handles ambiguous characters based on notation system
    pub fn from_source_with_context(source_pitch: &str, notation_system: NotationSystem) -> Option<Self> {
        // Use the new lookup_pitch function
        lookup_pitch(source_pitch, notation_system.into())
    }

    /// Convert complete source pitch token to normalized pitch code
    /// Legacy method - tries all notation systems
    pub fn from_source(source_pitch: &str) -> Option<Self> {
        // Try each notation system in order of likelihood
        if let Some(pitch) = lookup_pitch(source_pitch, Notation::Number) {
            return Some(pitch);
        }
        if let Some(pitch) = lookup_pitch(source_pitch, Notation::Sargam) {
            return Some(pitch);
        }
        if let Some(pitch) = lookup_pitch(source_pitch, Notation::Western) {
            return Some(pitch);
        }
        if let Some(pitch) = lookup_pitch(source_pitch, Notation::Tabla) {
            return Some(pitch);
        }
        if let Some(pitch) = lookup_pitch(source_pitch, Notation::Bhatkhande) {
            return Some(pitch);
        }
        None // Unknown pitch
    }
}

// Common helper functions for creating elements with position info
pub fn create_position(line: usize, column: usize, index_in_line: usize, index_in_doc: usize) -> (usize, usize, usize, usize) {
    (line, column, index_in_line, index_in_doc)
}

// Raw pitch string object for ContentLine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PitchString {
    pub value: Option<String>,      // Raw pitch string ("1", "S", "C", etc.)
    pub line: usize,
    pub column: usize,
    pub index_in_line: usize,
    pub index_in_doc: usize,
}

// Directive structure for key:value pairs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directive {
    pub key: String,
    pub directive_value: String,    // Renamed to avoid conflict with common 'value' field
    pub value: Option<String>,      // Raw source text
    pub line: usize,
    pub column: usize,
    pub index_in_line: usize,
    pub index_in_doc: usize,
}