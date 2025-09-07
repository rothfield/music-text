use serde::{Deserialize, Serialize};

// Notation system types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotationSystem {
    Number,     // 1 2 3 4 5 6 7 (numeric system)
    Western,    // C D E F G A B (standard western notes)  
    Sargam,     // S R G M P D N (Indian classical music)
    Bhatkhande, // स रे ग म प ध नि (Devanagari script)
}

impl NotationSystem {
    /// Detect notation system from syllable
    pub fn from_syllable(syllable: &str) -> Self {
        // Remove accidentals to get base syllable
        let base_syllable = syllable.trim_end_matches('#').trim_end_matches('b');
        
        match base_syllable {
            // Number notation
            "1" | "2" | "3" | "4" | "5" | "6" | "7" => NotationSystem::Number,
            // Unambiguous Western notation
            "C" | "E" | "A" | "B" => NotationSystem::Western,
            // Unambiguous Sargam notation  
            "s" | "S" | "r" | "g" | "m" | "n" | "d" | "p" => NotationSystem::Sargam,
            // Bhatkhande Devanagari notation
            "स" | "रे" | "र" | "ग" | "म" | "प" | "ध" | "द" | "नि" | "न" => NotationSystem::Bhatkhande,
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

// Normalized pitch codes - matches old Degree enum with complete accidental coverage
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
    // Convert source pitch to normalized pitch code
    pub fn from_source(source_pitch: &str) -> Self {
        // Remove accidentals to get base pitch
        let base_pitch = source_pitch.trim_end_matches('#').trim_end_matches('b');
        
        match base_pitch {
            // Number notation
            "1" => PitchCode::N1,
            "2" => PitchCode::N2, 
            "3" => PitchCode::N3,
            "4" => PitchCode::N4,
            "5" => PitchCode::N5,
            "6" => PitchCode::N6,
            "7" => PitchCode::N7,
            // Sargam notation (case-sensitive chromatic system)
            "s" => PitchCode::N1,           // Sa (tonic) - lowercase variant
            "S" => PitchCode::N1,           // Sa (tonic) - uppercase
            "r" => PitchCode::N2b,          // komal Re (flat second)  
            "R" => PitchCode::N2,           // shuddha Re (natural second)
            "g" => PitchCode::N3b,          // komal Ga (flat third)
            "G" => PitchCode::N3,           // shuddha Ga (natural third) 
            "m" => PitchCode::N4,           // shuddha Ma (natural fourth)
            "M" => PitchCode::N4s,          // tivra Ma (sharp fourth)
            "p" => PitchCode::N5,           // Pa (fifth) - lowercase
            "P" => PitchCode::N5,           // Pa (fifth) - uppercase
            "d" => PitchCode::N6b,          // komal Dha (flat sixth)
            "D" => PitchCode::N6,           // shuddha Dha (natural sixth)
            "n" => PitchCode::N7b,          // komal Ni (flat seventh)
            "N" => PitchCode::N7,           // shuddha Ni (natural seventh)
            // Western notation  
            "C" => PitchCode::N1,
            // "D" already handled by Sargam above
            "E" => PitchCode::N3,
            "F" => PitchCode::N4,
            // "G" already handled by Sargam above  
            "A" => PitchCode::N6,
            "B" => PitchCode::N7,
            // Bhatkhande Devanagari notation
            "स" => PitchCode::N1,           // Sa (स)
            "रे" => PitchCode::N2,          // shuddha Re (रे)
            "र" => PitchCode::N2b,          // komal Re (र)
            "ग" => PitchCode::N3,           // shuddha Ga (ग) 
            // Note: komal Ga would need different character - checking user specification
            "म" => PitchCode::N4,           // shuddha Ma (म) - M = N4 as specified
            "प" => PitchCode::N5,           // Pa (प)
            "ध" => PitchCode::N6,           // shuddha Dha (ध)
            "द" => PitchCode::N6b,          // komal Dha (द)
            "नि" => PitchCode::N7,          // shuddha Ni (नि)
            "न" => PitchCode::N7b,          // komal Ni (न)
            _ => PitchCode::N1, // Default fallback
        }
    }
}

// Note object with syllable and octave
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub syllable: String,        // Original syllable (1, 2, C, D, etc.)
    pub octave: i8,             // Octave -4..4
    pub pitch_code: PitchCode,  // Normalized pitch code
    pub notation_system: NotationSystem, // Which notation system this note uses
    pub source: Source,         // Source tracking (includes accidentals in value)
    pub in_slur: bool,          // Whether this note is within a slur
    pub in_beat_group: bool,    // Whether this note is within a beat group
}

// Document structure types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
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
        });
        result
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stave {
    pub text_lines_before: Vec<TextLine>,
    pub content_line: ContentLine,
    pub text_lines_after: Vec<TextLine>,
    pub notation_system: NotationSystem,
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextLine {
    pub content: String,
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentLine {
    pub elements: Vec<MusicalElement>,
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MusicalElement {
    Note(Note),
    Barline {
        source: Source,
        in_slur: bool,
        in_beat_group: bool,
    },
    Space { 
        count: usize,
        source: Source,
        in_slur: bool,
        in_beat_group: bool,
    },
    SlurBegin {
        source: Source,
        in_slur: bool,
        in_beat_group: bool,
    },
    SlurEnd {
        source: Source,
        in_slur: bool,
        in_beat_group: bool,
    },
    BeatGroupBegin {
        source: Source,
        in_slur: bool,
        in_beat_group: bool,
    },
    BeatGroupEnd {
        source: Source,
        in_slur: bool,
        in_beat_group: bool,
    },
}