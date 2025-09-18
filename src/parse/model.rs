use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use fraction::Fraction;

// Spatial assignment types that can be applied to notes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpatialAssignment {
    OctaveMarker {
        octave_value: i8,
        marker_symbol: String,
        is_upper: bool,
    },
    Slur {
        start_pos: usize,
        end_pos: usize,
    },
    Syllable {
        content: String,
    },
    BeatGroup {
        start_pos: usize,
        end_pos: usize,
        underscore_count: usize,
    },
    Mordent,
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

// Position information for source tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    // 1-based line number and column for human-readable diagnostics
    pub line: usize,
    pub column: usize,
    // Zero-based character offsets for precise indexing
    pub index_in_line: usize, // offset from start of line
    pub index_in_doc: usize,  // offset from start of document
}

// Source information tracking with move semantics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub value: Option<String>,  // Original source text (None when moved/consumed)
    pub position: Position,     // Line/column position
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
    pub source: Source,         // Raw pitch string ("1", "S", "C", etc.) + position
}

// ContentElement struct types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Barline {
    pub barline_type: crate::rhythm::converters::BarlineType,
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Space {
    pub count: usize,
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dash {
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Newline {
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndOfInput {
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreathMark {
    pub source: Source,
}

// Note object with pitchString attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub pitch_string: PitchString,       // Raw pitch string
    pub octave: i8,                     // Octave -4..4
    pub pitch_code: PitchCode,          // Normalized pitch code
    pub notation_system: NotationSystem, // Which notation system this note uses
    pub spatial_assignments: Vec<SpatialAssignment>, // Spatial elements assigned to this note
    pub duration: Option<Fraction>,     // Duration fraction (1/4, 1/8, etc.) - added by rhythm analysis
}

// Directive structure for key:value pairs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directive {
    pub key: String,
    pub value: String,
    pub source: Source,
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
    pub content: String, // The complete blank lines content
    pub source: Source,
}

// Document structure types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub title: Option<String>,
    pub author: Option<String>,
    pub directives: HashMap<String, String>, // key -> value
    pub elements: Vec<DocumentElement>, // Document as sequence of elements
    pub source: Source,
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
    pub lines: Vec<StaveLine>,  // All lines in order
    pub notation_system: NotationSystem,
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextLine {
    pub content: String,
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentElement {
    Barline(Barline),
    Whitespace(Whitespace),
    Beat(Beat),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Whitespace {
    pub content: String,
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentLine {
    pub elements: Vec<ContentElement>,  // Mixed elements: barlines, whitespace, beats
    pub source: Source,
}

// Beat structure - a sequence of beat elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Beat {
    pub elements: Vec<BeatElement>,
    pub source: Source,
    pub divisions: Option<usize>,        // Number of divisions in this beat (e.g., 12 for 12 sixteenths)
    pub total_duration: Option<Fraction>, // Total duration of this beat (e.g., 1/4 for quarter note beat)
}

// Elements that can appear in a beat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BeatElement {
    Note(Note),
    Dash(Dash),
    BreathMark(BreathMark),
}

// Spatial annotation lines per MUSIC_TEXT_SPECIFICATION.md

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpperLine {
    pub elements: Vec<UpperElement>,
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LowerLine {
    pub elements: Vec<LowerElement>,
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricsLine {
    pub syllables: Vec<Syllable>,
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitespaceLine {
    pub elements: Vec<crate::rhythm::types::ParsedElement>, // Whitespace elements and optional newline
    pub source: Source,
}

// UpperLine elements from specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpperElement {
    UpperOctaveMarker {
        marker: String,  // "." or ":"
        source: Source,
    },
    SlurIndicator {
        value: String,  // "_____" for slurs
        source: Source,
    },
    UpperHashes {
        value: String,  // "###" for multi-stave markers
        source: Source,
    },
    Ornament {
        pitches: Vec<String>,  // 123, <456> grace notes/melismas (🚧 planned)
        source: Source,
    },
    Chord {
        chord: String,  // [Am] chord symbols (🚧 planned)
        source: Source,
    },
    Mordent {
        source: Source,
    },
    Space {
        count: usize,
        source: Source,
    },
    Unknown {
        value: String,
        source: Source,
    },
    /// Newline token - explicit line terminator (upper lines cannot have EOI)
    Newline {
        value: String,
        source: Source,
    },
}

// LowerLine elements from specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LowerElement {
    LowerOctaveMarker {
        marker: String,  // "." or ":"
        source: Source,
    },
    BeatGroupIndicator {
        value: String,  // "___" for beat grouping
        source: Source,
    },
    Syllable {
        content: String,  // syllables like "dha", "he-llo"
        source: Source,
    },
    Space {
        count: usize,
        source: Source,
    },
    Unknown {
        value: String,
        source: Source,
    },
    /// Newline token - explicit line terminator
    Newline {
        value: String,
        source: Source,
    },
    /// End of input token - explicit EOF terminator
    EndOfInput {
        source: Source,
    },
}

// LyricsLine elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Syllable {
    pub content: String,  // "he-llo", "world", etc.
    pub source: Source,
}

