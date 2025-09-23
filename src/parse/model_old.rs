use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use fraction::Fraction;

/// Trait for elements that have position and value information
pub trait HasPosition {
    fn char_index(&self) -> usize;
    fn value(&self) -> Option<&String>;
    fn consumed_elements(&self) -> &[ConsumedElement];
    fn type_name(&self) -> &'static str;
}

// Implementations for enum variants
impl HasPosition for ConsumedElement {
    fn char_index(&self) -> usize {
        match self {
            ConsumedElement::UpperOctaveMarker { char_index, .. } => *char_index,
            ConsumedElement::LowerOctaveMarker { char_index, .. } => *char_index,
            ConsumedElement::SlurIndicator { char_index, .. } => *char_index,
        }
    }

    fn value(&self) -> Option<&String> {
        match self {
            ConsumedElement::UpperOctaveMarker { value, .. } => value.as_ref(),
            ConsumedElement::LowerOctaveMarker { value, .. } => value.as_ref(),
            ConsumedElement::SlurIndicator { value, .. } => value.as_ref(),
        }
    }

    fn consumed_elements(&self) -> &[ConsumedElement] {
        &[] // ConsumedElements don't have their own consumed elements
    }

    fn type_name(&self) -> &'static str {
        match self {
            ConsumedElement::UpperOctaveMarker { .. } => "ConsumedUpperOctaveMarker",
            ConsumedElement::LowerOctaveMarker { .. } => "ConsumedLowerOctaveMarker",
            ConsumedElement::SlurIndicator { .. } => "ConsumedSlurIndicator",
        }
    }
}

impl HasPosition for BeatElement {
    fn char_index(&self) -> usize {
        match self {
            BeatElement::Note(note) => note.char_index,
            BeatElement::Dash(dash) => dash.char_index,
            BeatElement::BreathMark(breath) => breath.char_index,
            BeatElement::Rest(rest) => rest.char_index,
        }
    }

    fn value(&self) -> Option<&String> {
        match self {
            BeatElement::Note(note) => note.value.as_ref(),
            BeatElement::Dash(dash) => dash.value.as_ref(),
            BeatElement::BreathMark(breath) => breath.value.as_ref(),
            BeatElement::Rest(rest) => rest.value.as_ref(),
        }
    }

    fn consumed_elements(&self) -> &[ConsumedElement] {
        match self {
            BeatElement::Note(note) => &note.consumed_elements,
            BeatElement::Dash(dash) => &dash.consumed_elements,
            BeatElement::BreathMark(breath) => &breath.consumed_elements,
            BeatElement::Rest(rest) => &rest.consumed_elements,
        }
    }

    fn type_name(&self) -> &'static str {
        match self {
            BeatElement::Note(_) => "Note",
            BeatElement::Dash(_) => "Dash",
            BeatElement::BreathMark(_) => "BreathMark",
            BeatElement::Rest(_) => "Rest",
        }
    }
}

impl HasPosition for UpperElement {
    fn char_index(&self) -> usize {
        match self {
            UpperElement::UpperOctaveMarker { char_index, .. } => *char_index,
            UpperElement::SlurIndicator { char_index, .. } => *char_index,
            UpperElement::UpperHashes { char_index, .. } => *char_index,
            UpperElement::Ornament { char_index, .. } => *char_index,
            UpperElement::Chord { char_index, .. } => *char_index,
            UpperElement::Mordent { char_index, .. } => *char_index,
            UpperElement::Space { char_index, .. } => *char_index,
            UpperElement::Unknown { char_index, .. } => *char_index,
            UpperElement::Newline { char_index, .. } => *char_index,
        }
    }

    fn value(&self) -> Option<&String> {
        match self {
            UpperElement::UpperOctaveMarker { value, .. } => value.as_ref(),
            UpperElement::SlurIndicator { value, .. } => value.as_ref(),
            UpperElement::UpperHashes { value, .. } => value.as_ref(),
            UpperElement::Ornament { value, .. } => value.as_ref(),
            UpperElement::Chord { value, .. } => value.as_ref(),
            UpperElement::Mordent { value, .. } => value.as_ref(),
            UpperElement::Space { value, .. } => value.as_ref(),
            UpperElement::Unknown { value, .. } => value.as_ref(),
            UpperElement::Newline { value, .. } => value.as_ref(),
        }
    }

    fn consumed_elements(&self) -> &[ConsumedElement] {
        &[] // Upper elements typically don't have consumed elements
    }

    fn type_name(&self) -> &'static str {
        match self {
            UpperElement::UpperOctaveMarker { .. } => "UpperOctaveMarker",
            UpperElement::SlurIndicator { .. } => "UpperSlurIndicator",
            UpperElement::UpperHashes { .. } => "UpperHashes",
            UpperElement::Ornament { .. } => "Ornament",
            UpperElement::Chord { .. } => "Chord",
            UpperElement::Mordent { .. } => "Mordent",
            UpperElement::Space { .. } => "UpperSpace",
            UpperElement::Unknown { .. } => "UpperUnknown",
            UpperElement::Newline { .. } => "UpperNewline",
        }
    }
}

impl HasPosition for LowerElement {
    fn char_index(&self) -> usize {
        match self {
            LowerElement::LowerOctaveMarker { char_index, .. } => *char_index,
            LowerElement::BeatGroupIndicator { char_index, .. } => *char_index,
            LowerElement::Syllable { char_index, .. } => *char_index,
            LowerElement::Space { char_index, .. } => *char_index,
            LowerElement::Unknown { char_index, .. } => *char_index,
            LowerElement::Newline { char_index, .. } => *char_index,
            LowerElement::EndOfInput { char_index, .. } => *char_index,
        }
    }

    fn value(&self) -> Option<&String> {
        match self {
            LowerElement::LowerOctaveMarker { value, .. } => value.as_ref(),
            LowerElement::BeatGroupIndicator { value, .. } => value.as_ref(),
            LowerElement::Syllable { value, .. } => value.as_ref(),
            LowerElement::Space { value, .. } => value.as_ref(),
            LowerElement::Unknown { value, .. } => value.as_ref(),
            LowerElement::Newline { value, .. } => value.as_ref(),
            LowerElement::EndOfInput { value, .. } => value.as_ref(),
        }
    }

    fn consumed_elements(&self) -> &[ConsumedElement] {
        &[] // Lower elements typically don't have consumed elements
    }

    fn type_name(&self) -> &'static str {
        match self {
            LowerElement::LowerOctaveMarker { .. } => "LowerOctaveMarker",
            LowerElement::BeatGroupIndicator { .. } => "BeatGroupIndicator",
            LowerElement::Syllable { .. } => "Syllable",
            LowerElement::Space { .. } => "LowerSpace",
            LowerElement::Unknown { .. } => "LowerUnknown",
            LowerElement::Newline { .. } => "LowerNewline",
            LowerElement::EndOfInput { .. } => "EndOfInput",
        }
    }
}

/// Position of a note within slur markings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlurPosition {
    None,       // Not part of any slur
    Start,      // Starts a slur
    Middle,     // Inside a slur (not start or end)
    End,        // Ends a slur
    StartEnd,   // Both starts and ends a slur (single-note slur)
}

impl Default for SlurPosition {
    fn default() -> Self {
        SlurPosition::None
    }
}

/// Position information with old field structure (transitional)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub index_in_line: usize,
    pub index_in_doc: usize,
}

/// Attributes structure for parser elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attributes {
    pub slur_position: SlurPosition,
    pub value: Option<String>,
    pub position: Position,
}


// Consumed elements that have been moved to notes (follows ContentElement pattern)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsumedElement {
    UpperOctaveMarker {
        value: Option<String>,
        char_index: usize, // was: line, column, index_in_line, index_in_doc
    },
    LowerOctaveMarker {
        value: Option<String>,
        char_index: usize, // was: line, column, index_in_line, index_in_doc
    },
    SlurIndicator {
        value: Option<String>,
        char_index: usize, // Position of the slur indicator in the document
    },
}

// Notation system types
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

// Common helper functions for creating elements with position info
pub fn create_position(line: usize, column: usize, index_in_line: usize, index_in_doc: usize) -> (usize, usize, usize, usize) {
    (line, column, index_in_line, index_in_doc)
}

// Normalized pitch codes - matches old Degree enum with complete pitch coverage
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

/// Lookup pitch from symbol and notation system (ported from old system)
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

// Raw pitch string object for ContentLine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PitchString {
    pub value: Option<String>,      // Raw pitch string ("1", "S", "C", etc.)
    pub line: usize,
    pub column: usize,
    pub index_in_line: usize,
    pub index_in_doc: usize,
}

// Individual barline types matching grammar productions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleBarline {
    pub value: Option<String>,
    pub char_index: usize, // was: line, column, index_in_line, index_in_doc
    pub consumed_elements: Vec<ConsumedElement>, // Elements consumed by this barline via 2D spatial rules
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoubleBarline {
    pub value: Option<String>,
    pub char_index: usize, // was: line, column, index_in_line, index_in_doc
    pub consumed_elements: Vec<ConsumedElement>, // Elements consumed by this barline via 2D spatial rules
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalBarline {
    pub value: Option<String>,
    pub char_index: usize, // was: line, column, index_in_line, index_in_doc
    pub consumed_elements: Vec<ConsumedElement>, // Elements consumed by this barline via 2D spatial rules
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeatStartBarline {
    pub value: Option<String>,
    pub char_index: usize, // was: line, column, index_in_line, index_in_doc
    pub consumed_elements: Vec<ConsumedElement>, // Elements consumed by this barline via 2D spatial rules
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeatEndBarline {
    pub value: Option<String>,
    pub char_index: usize, // was: line, column, index_in_line, index_in_doc
    pub consumed_elements: Vec<ConsumedElement>, // Elements consumed by this barline via 2D spatial rules
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeatBothBarline {
    pub value: Option<String>,
    pub char_index: usize, // was: line, column, index_in_line, index_in_doc
    pub consumed_elements: Vec<ConsumedElement>, // Elements consumed by this barline via 2D spatial rules
}

// Unified barline enum for ContentElement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Barline {
    Single(SingleBarline),
    Double(DoubleBarline),
    Final(FinalBarline),
    RepeatStart(RepeatStartBarline),
    RepeatEnd(RepeatEndBarline),
    RepeatBoth(RepeatBothBarline),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Space {
    pub value: Option<String>,
    pub char_index: usize, // was: line, column, index_in_line, index_in_doc
    pub count: usize,
    pub consumed_elements: Vec<ConsumedElement>, // Elements consumed by this space via 2D spatial rules
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dash {
    pub value: Option<String>,
    pub char_index: usize, // was: line, column, index_in_line, index_in_doc
    // Duration fields populated by rhythm analyzer
    pub numerator: Option<u32>,
    pub denominator: Option<u32>,
    pub consumed_elements: Vec<ConsumedElement>, // Elements consumed by this dash via 2D spatial rules
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Newline {
    pub value: Option<String>,
    pub char_index: usize, // was: line, column, index_in_line, index_in_doc
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndOfInput {
    pub value: Option<String>,
    pub char_index: usize, // was: line, column, index_in_line, index_in_doc
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreathMark {
    pub value: Option<String>,
    pub char_index: usize, // was: line, column, index_in_line, index_in_doc
    pub consumed_elements: Vec<ConsumedElement>, // Elements consumed by this breath mark via 2D spatial rules
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rest {
    pub value: Option<String>,
    pub char_index: usize, // was: line, column, index_in_line, index_in_doc
    pub consumed_elements: Vec<ConsumedElement>, // Elements consumed by this rest via 2D spatial rules
    // Duration fields populated by rhythm analyzer
    pub numerator: Option<u32>,
    pub denominator: Option<u32>,
}

// Note object - consistent with other elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    // Common fields
    pub value: Option<String>,          // Raw pitch string
    pub char_index: usize,              // was: line, column, index_in_line, index_in_doc
    // Note-specific fields
    pub octave: i8,                     // Octave -4..4
    pub pitch_code: PitchCode,          // Normalized pitch code
    pub notation_system: NotationSystem, // Which notation system this note uses
    pub numerator: Option<u32>,         // Simple duration numerator
    pub denominator: Option<u32>,       // Simple duration denominator
    pub consumed_elements: Vec<ConsumedElement>, // Elements consumed by this note via 2D spatial rules
}

impl Note {
    /// Factory function to create a new Note with consistent default values
    pub fn new(
        value: Option<String>,
        char_index: usize,
        pitch_code: PitchCode,
        notation_system: NotationSystem,
    ) -> Self {
        Self {
            value,
            char_index,
            octave: 0,                      // Default octave
            pitch_code,
            notation_system,
            numerator: None,                // Will be populated by rhythm analysis
            denominator: None,              // Will be populated by rhythm analysis
            consumed_elements: Vec::new()   // Will be populated during spatial analysis
        }
    }
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


// Document element - either blank lines or stave
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentElement {
    BlankLines(BlankLines),
    Stave(Stave),
}

// Blank lines structure (newline (whitespace* newline)+)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlankLines {
    pub value: Option<String>, // The complete blank lines content
    pub char_index: usize, // Converted from line/column for consistency
    pub line: usize,
    pub column: usize,
    pub index_in_line: usize,
    pub index_in_doc: usize,
}

// Document structure types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub value: Option<String>,
    pub char_index: usize, // was: line, column, index_in_line, index_in_doc
    pub title: Option<String>,
    pub author: Option<String>,
    pub directives: HashMap<String, String>, // key -> value
    pub elements: Vec<DocumentElement>, // Document as sequence of elements
}

impl Document {
    /// Get unique notation systems detected across all staves
    pub fn get_detected_notation_systems(&self) -> Vec<NotationSystem> {
        use std::collections::HashSet;

        let mut systems = HashSet::new();
        for element in &self.elements {
            if let DocumentElement::Stave(stave) = element {
                systems.insert(stave.notation_system);
            }
        }

        let mut result: Vec<NotationSystem> = systems.into_iter().collect();
        result.sort_by_key(|system| match system {
            NotationSystem::Number => 0,
            NotationSystem::Western => 1,
            NotationSystem::Sargam => 2,
            NotationSystem::Bhatkhande => 3,
            NotationSystem::Tabla => 4,
        });
        result
    }
}

// Enum for different types of lines in a stave
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StaveLine {
    Text(TextLine),
    Upper(UpperLine),
    Content(Vec<crate::rhythm::types::ParsedElement>), // Keep for backward compat
    ContentLine(ContentLine),  // New: elements parsed directly
    Lower(LowerLine),
    Lyrics(LyricsLine),
    Whitespace(WhitespaceLine),
    BlankLines(BlankLines),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stave {
    pub value: Option<String>,
    pub char_index: usize, // Converted from line/column for consistency
    pub notation_system: NotationSystem,
    pub line: usize,
    pub column: usize,
    pub index_in_line: usize,
    pub index_in_doc: usize,
    pub lines: Vec<StaveLine>,  // All lines in order
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextLine {
    pub value: Option<String>, // The text content
    pub char_index: usize, // was: line, column, index_in_line, index_in_doc
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentElement {
    Barline(Barline),
    Whitespace(Whitespace),
    Beat(Beat),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Whitespace {
    pub value: Option<String>, // The whitespace content
    pub char_index: usize, // was: line, column, index_in_line, index_in_doc
    pub consumed_elements: Vec<ConsumedElement>, // Elements consumed by this whitespace via 2D spatial rules
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentLine {
    pub elements: Vec<ContentElement>,  // Mixed elements: barlines, whitespace, beats
    pub value: Option<String>,
    pub char_index: usize, // was: line, column, index_in_line, index_in_doc
    pub consumed_elements: Vec<ConsumedElement>, // Elements consumed by this content line via 2D spatial rules
}

// Beat structure - a sequence of beat elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Beat {
    pub value: Option<String>,
    pub char_index: usize, // was: line, column, index_in_line, index_in_doc
    pub divisions: Option<usize>,        // Number of divisions in this beat (e.g., 12 for 12 sixteenths)
    pub is_tuplet: Option<bool>,         // Whether this beat is a tuplet (3, 5, 6, 7, etc. divisions)
    pub tuplet_ratio: Option<(usize, usize)>, // Tuplet ratio (e.g., (3, 2) for triplet)
    pub tied_to_previous: Option<bool>,  // Whether this beat's first note is tied to the previous beat's last note
    pub total_duration: Option<Fraction>, // Total duration of this beat (e.g., 1/4 for quarter note beat)
    pub elements: Vec<BeatElement>,
    pub consumed_elements: Vec<ConsumedElement>, // Elements consumed by this beat via 2D spatial rules
}

// Elements that can appear in a beat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BeatElement {
    Note(Note),
    Dash(Dash),
    BreathMark(BreathMark),
    Rest(Rest),
}

// Spatial annotation lines per MUSIC_TEXT_SPECIFICATION.md

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpperLine {
    pub value: Option<String>,
    pub char_index: usize, // was: line, column, index_in_line, index_in_doc
    pub elements: Vec<UpperElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LowerLine {
    pub elements: Vec<LowerElement>,
    pub value: Option<String>,
    pub char_index: usize, // was: line, column, index_in_line, index_in_doc
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricsLine {
    pub value: Option<String>,
    pub char_index: usize, // was: line, column, index_in_line, index_in_doc
    pub syllables: Vec<Syllable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitespaceLine {
    pub char_index: usize, // was: line, column, index_in_line, index_in_doc
    pub elements: Vec<crate::rhythm::types::ParsedElement>, // Whitespace elements and optional newline
    pub value: Option<String>,
}

// UpperLine elements from specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpperElement {
    UpperOctaveMarker {
        marker: String,  // "." or ":"
        value: Option<String>,
        char_index: usize, // was: line, column, index_in_line, index_in_doc
    },
    SlurIndicator {
        indicator_value: String,  // "_____" for slurs
        value: Option<String>,
        char_index: usize, // was: line, column, index_in_line, index_in_doc
    },
    UpperHashes {
        value: Option<String>,
        char_index: usize, // was: line, column, index_in_line, index_in_doc
        hash_value: String,  // "###" for multi-stave markers
    },
    Ornament {
        value: Option<String>,
        char_index: usize, // was: line, column, index_in_line, index_in_doc
        pitches: Vec<String>,  // 123, <456> grace notes/melismas (🚧 planned)
    },
    Chord {
        value: Option<String>,
        chord: String,  // [Am] chord symbols (🚧 planned)
        char_index: usize, // was: line, column, index_in_line, index_in_doc
    },
    Mordent {
        value: Option<String>,
        char_index: usize, // was: line, column, index_in_line, index_in_doc
    },
    Space {
        char_index: usize, // was: line, column, index_in_line, index_in_doc
        count: usize,
        value: Option<String>,
    },
    Unknown {
        unknown_value: String,
        value: Option<String>,
        char_index: usize, // was: line, column, index_in_line, index_in_doc
    },
    /// Newline token - explicit line terminator (upper lines cannot have EOI)
    Newline {
        value: Option<String>,
        char_index: usize, // was: line, column, index_in_line, index_in_doc
        newline_value: String,
    },
}

// LowerLine elements from specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LowerElement {
    LowerOctaveMarker {
        value: Option<String>,
        char_index: usize, // was: line, column, index_in_line, index_in_doc
        marker: String,  // "." or ":"
    },
    BeatGroupIndicator {
        value: Option<String>,
        char_index: usize, // was: line, column, index_in_line, index_in_doc
        indicator_value: String,  // "___" for beat grouping
    },
    Syllable {
        value: Option<String>,
        char_index: usize, // was: line, column, index_in_line, index_in_doc
        content: String,  // syllables like "dha", "he-llo"
    },
    Space {
        value: Option<String>,
        char_index: usize, // was: line, column, index_in_line, index_in_doc
        count: usize,
    },
    Unknown {
        value: Option<String>,
        char_index: usize, // was: line, column, index_in_line, index_in_doc
        unknown_value: String,
    },
    /// Newline token - explicit line terminator
    Newline {
        newline_value: String,
        value: Option<String>,
        char_index: usize, // was: line, column, index_in_line, index_in_doc
    },
    /// End of input token - explicit EOF terminator
    EndOfInput {
        value: Option<String>,
        char_index: usize, // was: line, column, index_in_line, index_in_doc
    },
}

// HasPosition implementations for barline structs
impl HasPosition for SingleBarline {
    fn char_index(&self) -> usize { self.char_index }
    fn value(&self) -> Option<&String> { self.value.as_ref() }
    fn consumed_elements(&self) -> &[ConsumedElement] { &self.consumed_elements }
    fn type_name(&self) -> &'static str { "SingleBarline" }
}

impl HasPosition for DoubleBarline {
    fn char_index(&self) -> usize { self.char_index }
    fn value(&self) -> Option<&String> { self.value.as_ref() }
    fn consumed_elements(&self) -> &[ConsumedElement] { &self.consumed_elements }
    fn type_name(&self) -> &'static str { "DoubleBarline" }
}

impl HasPosition for FinalBarline {
    fn char_index(&self) -> usize { self.char_index }
    fn value(&self) -> Option<&String> { self.value.as_ref() }
    fn consumed_elements(&self) -> &[ConsumedElement] { &self.consumed_elements }
    fn type_name(&self) -> &'static str { "FinalBarline" }
}

impl HasPosition for RepeatStartBarline {
    fn char_index(&self) -> usize { self.char_index }
    fn value(&self) -> Option<&String> { self.value.as_ref() }
    fn consumed_elements(&self) -> &[ConsumedElement] { &self.consumed_elements }
    fn type_name(&self) -> &'static str { "RepeatStartBarline" }
}

impl HasPosition for RepeatEndBarline {
    fn char_index(&self) -> usize { self.char_index }
    fn value(&self) -> Option<&String> { self.value.as_ref() }
    fn consumed_elements(&self) -> &[ConsumedElement] { &self.consumed_elements }
    fn type_name(&self) -> &'static str { "RepeatEndBarline" }
}

impl HasPosition for RepeatBothBarline {
    fn char_index(&self) -> usize { self.char_index }
    fn value(&self) -> Option<&String> { self.value.as_ref() }
    fn consumed_elements(&self) -> &[ConsumedElement] { &self.consumed_elements }
    fn type_name(&self) -> &'static str { "RepeatBothBarline" }
}

impl HasPosition for Barline {
    fn char_index(&self) -> usize {
        match self {
            Barline::Single(b) => b.char_index,
            Barline::Double(b) => b.char_index,
            Barline::Final(b) => b.char_index,
            Barline::RepeatStart(b) => b.char_index,
            Barline::RepeatEnd(b) => b.char_index,
            Barline::RepeatBoth(b) => b.char_index,
        }
    }

    fn value(&self) -> Option<&String> {
        match self {
            Barline::Single(b) => b.value.as_ref(),
            Barline::Double(b) => b.value.as_ref(),
            Barline::Final(b) => b.value.as_ref(),
            Barline::RepeatStart(b) => b.value.as_ref(),
            Barline::RepeatEnd(b) => b.value.as_ref(),
            Barline::RepeatBoth(b) => b.value.as_ref(),
        }
    }

    fn consumed_elements(&self) -> &[ConsumedElement] {
        match self {
            Barline::Single(b) => &b.consumed_elements,
            Barline::Double(b) => &b.consumed_elements,
            Barline::Final(b) => &b.consumed_elements,
            Barline::RepeatStart(b) => &b.consumed_elements,
            Barline::RepeatEnd(b) => &b.consumed_elements,
            Barline::RepeatBoth(b) => &b.consumed_elements,
        }
    }

    fn type_name(&self) -> &'static str {
        match self {
            Barline::Single(_) => "SingleBarline",
            Barline::Double(_) => "DoubleBarline",
            Barline::Final(_) => "FinalBarline",
            Barline::RepeatStart(_) => "RepeatStartBarline",
            Barline::RepeatEnd(_) => "RepeatEndBarline",
            Barline::RepeatBoth(_) => "RepeatBothBarline",
        }
    }
}

impl HasPosition for Beat {
    fn char_index(&self) -> usize { self.char_index }
    fn value(&self) -> Option<&String> { self.value.as_ref() }
    fn consumed_elements(&self) -> &[ConsumedElement] { &self.consumed_elements }
    fn type_name(&self) -> &'static str { "Beat" }
}

// LyricsLine elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Syllable {
    pub value: Option<String>, // "he-llo", "world", etc.
    pub char_index: usize, // was: line, column, index_in_line, index_in_doc
}

