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
            
            // Bhatkhande Devanagari notation - basic support (extended variants rare in practice)
            "स" => PitchCode::N1, "रे" => PitchCode::N2, "र" => PitchCode::N2b, "ग" => PitchCode::N3,
            "म" => PitchCode::N4, "प" => PitchCode::N5, "ध" => PitchCode::N6, "द" => PitchCode::N6b,
            "नि" => PitchCode::N7, "न" => PitchCode::N7b,
            
            // Return error for unrecognized input instead of silent fallback
            _ => panic!("Unrecognized pitch: {}", source_pitch), // Will be replaced with proper error handling
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
    pub source: Source,         // Source tracking (includes complete pitch token in value)
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