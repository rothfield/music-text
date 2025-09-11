use serde::{Deserialize, Serialize};

// Notation system types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotationSystem {
    Number,     // 1 2 3 4 5 6 7 (numeric system)
    Western,    // C D E F G A B (standard western notes)  
    Sargam,     // S R G M P D N (Indian classical music)
    Bhatkhande, // स रे ग म प ध नि (Devanagari script)
    Tabla,      // dha dhin ta ka taka trkt ge (tabla bols/percussion syllables)
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
    pub line: usize,
    pub column: usize,
}

// Source information tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub value: String,    // Original source text
    pub position: Position, // Line/column position
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

impl PitchCode {
    /// Convert complete source pitch token to normalized pitch code  
    /// Context-aware version that handles ambiguous characters based on notation system
    pub fn from_source_with_context(source_pitch: &str, notation_system: NotationSystem) -> Self {
        // Handle ambiguous characters based on context
        match (source_pitch, notation_system) {
            ("G", NotationSystem::Sargam) => PitchCode::N3,  // Sargam Ga
            ("G", NotationSystem::Western) => PitchCode::N5,  // Western G
            ("D", NotationSystem::Sargam) => PitchCode::N6,   // Sargam Dha  
            ("D", NotationSystem::Western) => PitchCode::N2,  // Western D
            ("R", NotationSystem::Sargam) => PitchCode::N2,   // Sargam Re
            ("R", NotationSystem::Western) => PitchCode::N2,  // R not standard Western, default to N2
            ("M", NotationSystem::Sargam) => PitchCode::N4s,  // Sargam Ma tivra
            ("M", NotationSystem::Western) => PitchCode::N4s, // M not standard Western, default to N4s
            ("P", NotationSystem::Sargam) => PitchCode::N5,   // Sargam Pa
            ("P", NotationSystem::Western) => PitchCode::N5,  // P not standard Western, default to N5
            ("N", NotationSystem::Sargam) => PitchCode::N7,   // Sargam Ni
            ("N", NotationSystem::Western) => PitchCode::N7,  // N not standard Western, default to N7
            _ => Self::from_source(source_pitch), // Fall back to original method
        }
    }
    
    /// Convert complete source pitch token to normalized pitch code  
    /// Now handles all 35 combinations explicitly
    pub fn from_source(source_pitch: &str) -> Self {
        match source_pitch {
            // Number notation - all 35 combinations
            "1bb" => PitchCode::N1bb, "1b" => PitchCode::N1b, "1" => PitchCode::N1, "1#" => PitchCode::N1s, "1##" => PitchCode::N1ss,
            "2bb" => PitchCode::N2bb, "2b" => PitchCode::N2b, "2" => PitchCode::N2, "2#" => PitchCode::N2s, "2##" => PitchCode::N2ss,
            "3bb" => PitchCode::N3bb, "3b" => PitchCode::N3b, "3" => PitchCode::N3, "3#" => PitchCode::N3s, "3##" => PitchCode::N3ss,
            "4bb" => PitchCode::N4bb, "4b" => PitchCode::N4b, "4" => PitchCode::N4, "4#" => PitchCode::N4s, "4##" => PitchCode::N4ss,
            "5bb" => PitchCode::N5bb, "5b" => PitchCode::N5b, "5" => PitchCode::N5, "5#" => PitchCode::N5s, "5##" => PitchCode::N5ss,
            "6bb" => PitchCode::N6bb, "6b" => PitchCode::N6b, "6" => PitchCode::N6, "6#" => PitchCode::N6s, "6##" => PitchCode::N6ss,
            "7bb" => PitchCode::N7bb, "7b" => PitchCode::N7b, "7" => PitchCode::N7, "7#" => PitchCode::N7s, "7##" => PitchCode::N7ss,
            
            // Western notation - all 35 combinations
            "Cbb" => PitchCode::N1bb, "Cb" => PitchCode::N1b, "C" => PitchCode::N1, "C#" => PitchCode::N1s, "C##" => PitchCode::N1ss,
            "Dbb" => PitchCode::N2bb, "Db" => PitchCode::N2b, "D" => PitchCode::N2, "D#" => PitchCode::N2s, "D##" => PitchCode::N2ss,
            "Ebb" => PitchCode::N3bb, "Eb" => PitchCode::N3b, "E" => PitchCode::N3, "E#" => PitchCode::N3s, "E##" => PitchCode::N3ss,
            "Fbb" => PitchCode::N4bb, "Fb" => PitchCode::N4b, "F" => PitchCode::N4, "F#" => PitchCode::N4s, "F##" => PitchCode::N4ss,
            "Gbb" => PitchCode::N5bb, "Gb" => PitchCode::N5b, "G" => PitchCode::N5, "G#" => PitchCode::N5s, "G##" => PitchCode::N5ss,
            "Abb" => PitchCode::N6bb, "Ab" => PitchCode::N6b, "A" => PitchCode::N6, "A#" => PitchCode::N6s, "A##" => PitchCode::N6ss,
            "Bbb" => PitchCode::N7bb, "Bb" => PitchCode::N7b, "B" => PitchCode::N7, "B#" => PitchCode::N7s, "B##" => PitchCode::N7ss,
            
            // Sargam notation - including all pitch variants
            // Sa (tonic)
            "Sbb" => PitchCode::N1bb, "Sb" => PitchCode::N1b, "S" => PitchCode::N1, "s" => PitchCode::N1, "S#" => PitchCode::N1s, "S##" => PitchCode::N1ss,
            // Re (second) - komal/shuddha system
            "r" => PitchCode::N2b, "R" => PitchCode::N2, "Rbb" => PitchCode::N2bb, "R#" => PitchCode::N2s, "R##" => PitchCode::N2ss,
            // Ga (third) - komal/shuddha system (lowercase g only - G already handled in Western)
            "g" => PitchCode::N3b,
            // Ma (fourth) - shuddha/tivra system  
            "m" => PitchCode::N4, "M" => PitchCode::N4s, 
            "mbb" => PitchCode::N4bb, "mb" => PitchCode::N4b, "m#" => PitchCode::N4s, "m##" => PitchCode::N4ss,
            "M#" => PitchCode::N4ss, "M##" => PitchCode::N4ss, "Mbb" => PitchCode::N4bb, "Mb" => PitchCode::N4b,
            // Pa (fifth)
            "Pbb" => PitchCode::N5bb, "Pb" => PitchCode::N5b, "P" => PitchCode::N5, "p" => PitchCode::N5, "P#" => PitchCode::N5s, "P##" => PitchCode::N5ss,
            // Dha (sixth) - komal/shuddha system (lowercase d only - D already handled in Western)
            "d" => PitchCode::N6b,
            // Ni (seventh) - komal/shuddha system
            "n" => PitchCode::N7b, "N" => PitchCode::N7, "Nbb" => PitchCode::N7bb, "N#" => PitchCode::N7s, "N##" => PitchCode::N7ss,
            
            // Tabla notation - all tabla bols map to N1 (tonic) as requested
            "dha" => PitchCode::N1, "dhin" => PitchCode::N1, "ta" => PitchCode::N1, "ka" => PitchCode::N1,
            "taka" => PitchCode::N1, "trkt" => PitchCode::N1, "ge" => PitchCode::N1,
            "Dha" => PitchCode::N1, "Dhin" => PitchCode::N1, "Ta" => PitchCode::N1, "Ka" => PitchCode::N1,
            "Taka" => PitchCode::N1, "Trkt" => PitchCode::N1, "Ge" => PitchCode::N1,
            "DHA" => PitchCode::N1, "DHIN" => PitchCode::N1, "TA" => PitchCode::N1, "KA" => PitchCode::N1,
            "TAKA" => PitchCode::N1, "TRKT" => PitchCode::N1, "GE" => PitchCode::N1,
            
            // Bhatkhande Devanagari notation - basic support (extended variants rare in practice)
            "स" => PitchCode::N1, "रे" => PitchCode::N2, "र" => PitchCode::N2b, "ग" => PitchCode::N3,
            "म" => PitchCode::N4, "प" => PitchCode::N5, "ध" => PitchCode::N6, "द" => PitchCode::N6b,
            "नि" => PitchCode::N7, "न" => PitchCode::N7b,
            
            // Return error for unrecognized input instead of silent fallback
            _ => panic!("Unrecognized pitch: {}", source_pitch), // Will be replaced with proper error handling
        }
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
    pub source: Source,
    pub in_slur: bool,
    pub in_beat_group: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Space {
    pub count: usize,
    pub source: Source,
    pub in_slur: bool,
    pub in_beat_group: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dash {
    pub source: Source,
    pub in_slur: bool,
    pub in_beat_group: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlurBegin {
    pub source: Source,
    pub in_slur: bool,
    pub in_beat_group: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlurEnd {
    pub source: Source,
    pub in_slur: bool,
    pub in_beat_group: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeatGroupBegin {
    pub source: Source,
    pub in_slur: bool,
    pub in_beat_group: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeatGroupEnd {
    pub source: Source,
    pub in_slur: bool,
    pub in_beat_group: bool,
}

// Note object with pitchString attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub pitch_string: PitchString,       // Raw pitch string
    pub octave: i8,                     // Octave -4..4
    pub pitch_code: PitchCode,          // Normalized pitch code
    pub notation_system: NotationSystem, // Which notation system this note uses
    pub in_slur: bool,                  // Whether this note is within a slur
    pub in_beat_group: bool,            // Whether this note is within a beat group
}

// Directive structure for key:value pairs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directive {
    pub key: String,
    pub value: String,
    pub source: Source,
}

// Document structure types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub directives: Vec<Directive>,
    pub staves: Vec<Stave>,
    pub source: Source,
}

impl Document {
    /// Get unique notation systems detected across all staves
    pub fn get_detected_notation_systems(&self) -> Vec<NotationSystem> {
        use std::collections::HashSet;
        
        let mut systems = HashSet::new();
        for stave in &self.staves {
            systems.insert(stave.notation_system);
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stave {
    pub text_lines_before: Vec<TextLine>,
    pub content_line: Vec<crate::rhythm::types::ParsedElement>, // Direct ParsedElement from parseMainLine
    pub upper_lines: Vec<UpperLine>,   // Spatial annotations above content
    pub lower_lines: Vec<LowerLine>,   // Spatial annotations below content
    pub lyrics_lines: Vec<LyricsLine>, // Syllables for assignment to notes
    pub text_lines_after: Vec<TextLine>,
    pub notation_system: NotationSystem,
    pub source: Source,
    pub begin_multi_stave: bool,  // True if this stave begins a multi-stave group
    pub end_multi_stave: bool,    // True if this stave ends a multi-stave group
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextLine {
    pub content: String,
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentLine {
    pub elements: Vec<ContentElement>,
    pub source: Source,
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

// UpperLine elements from specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpperElement {
    UpperOctaveMarker {
        marker: String,  // "." or ":"
        source: Source,
    },
    UpperUnderscores {
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
    Space {
        count: usize,
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
    LowerUnderscores {
        value: String,  // "___" for beat grouping
        source: Source,
    },
    FlatMarker {
        marker: String,  // "_" flat marker, Bhatkande notation only (🚧 planned)
        source: Source,
    },
    Space {
        count: usize,
        source: Source,
    },
}

// LyricsLine elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Syllable {
    pub content: String,  // "he-llo", "world", etc.
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentElement {
    PitchString(PitchString),
    Barline(Barline),
    Space(Space),
    Dash(Dash),
    SlurBegin(SlurBegin),
    SlurEnd(SlurEnd),
    BeatGroupBegin(BeatGroupBegin),
    BeatGroupEnd(BeatGroupEnd),
}