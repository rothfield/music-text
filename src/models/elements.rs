use serde::{Deserialize, Serialize};
use fraction::Fraction;
use uuid::Uuid;

// Core music elements - the fundamental building blocks of musical notation

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,                       // Unique identifier
    // Common fields
    pub value: Option<String>,          // Raw pitch string
    pub char_index: usize,              // Position in source
    // Note-specific fields
    pub octave: i8,                     // Octave -4..4
    pub pitch_code: super::notation::PitchCode,          // Normalized pitch code
    pub notation_system: super::notation::NotationSystem, // Which notation system this note uses
    pub numerator: Option<u32>,         // Simple duration numerator
    pub denominator: Option<u32>,       // Simple duration denominator
    pub consumed_elements: Vec<super::position::ConsumedElement>, // Elements consumed by this note via 2D spatial rules
}

impl Note {
    /// Factory function to create a new Note with consistent default values
    pub fn new(
        value: Option<String>,
        char_index: usize,
        pitch_code: super::notation::PitchCode,
        notation_system: super::notation::NotationSystem,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),             // Generate unique ID
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dash {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: Option<String>,
    pub char_index: usize,
    // Duration fields populated by rhythm analyzer
    pub numerator: Option<u32>,
    pub denominator: Option<u32>,
    pub consumed_elements: Vec<super::position::ConsumedElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreathMark {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: Option<String>,
    pub char_index: usize,
    pub consumed_elements: Vec<super::position::ConsumedElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rest {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: Option<String>,
    pub char_index: usize,
    pub consumed_elements: Vec<super::position::ConsumedElement>,
    // Duration fields populated by rhythm analyzer
    pub numerator: Option<u32>,
    pub denominator: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Space {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: Option<String>,
    pub char_index: usize,
    pub count: usize,
    pub consumed_elements: Vec<super::position::ConsumedElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Whitespace {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: Option<String>,
    pub char_index: usize,
    pub consumed_elements: Vec<super::position::ConsumedElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Newline {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: Option<String>,
    pub char_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndOfInput {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: Option<String>,
    pub char_index: usize,
}

// Elements that can appear in a beat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BeatElement {
    Note(Note),
    Dash(Dash),
    BreathMark(BreathMark),
    Rest(Rest),
}

// Beat structure - a sequence of beat elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Beat {
    pub value: Option<String>,
    pub char_index: usize,
    pub divisions: Option<usize>,        // Number of divisions in this beat (e.g., 12 for 12 sixteenths)
    pub is_tuplet: Option<bool>,         // Whether this beat is a tuplet (3, 5, 6, 7, etc. divisions)
    pub tuplet_ratio: Option<(usize, usize)>, // Tuplet ratio (e.g., (3, 2) for triplet)
    pub tied_to_previous: Option<bool>,  // Whether this beat's first note is tied to the previous beat's last note
    pub total_duration: Option<Fraction>, // Total duration of this beat (e.g., 1/4 for quarter note beat)
    pub elements: Vec<BeatElement>,
    pub consumed_elements: Vec<super::position::ConsumedElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownToken {
    pub value: Option<String>,
    pub char_index: usize,
    pub token_value: String,
    pub consumed_elements: Vec<super::position::ConsumedElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentElement {
    Barline(super::barlines::Barline),
    Whitespace(Whitespace),
    Beat(Beat),
    UnknownToken(UnknownToken),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentLine {
    pub elements: Vec<ContentElement>,  // Mixed elements: barlines, whitespace, beats
    pub value: Option<String>,
    pub char_index: usize,
    pub consumed_elements: Vec<super::position::ConsumedElement>,
}

// Spatial annotation lines per MUSIC_TEXT_SPECIFICATION.md

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpperLine {
    pub value: Option<String>,
    pub char_index: usize,
    pub elements: Vec<UpperElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LowerLine {
    pub elements: Vec<LowerElement>,
    pub value: Option<String>,
    pub char_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricsLine {
    pub value: Option<String>,
    pub char_index: usize,
    pub syllables: Vec<Syllable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitespaceLine {
    pub char_index: usize,
    pub elements: Vec<crate::rhythm::types::ParsedElement>, // Whitespace elements and optional newline
    pub value: Option<String>,
}

// UpperLine elements from specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpperElement {
    UpperOctaveMarker {
        marker: String,  // "." or ":"
        value: Option<String>,
        char_index: usize,
    },
    SlurIndicator {
        indicator_value: String,  // "_____" for slurs
        value: Option<String>,
        char_index: usize,
    },
    UpperHashes {
        value: Option<String>,
        char_index: usize,
        hash_value: String,  // "###" for multi-stave markers
    },
    Ornament {
        value: Option<String>,
        char_index: usize,
        pitches: Vec<String>,  // 123, <456> grace notes/melismas (🚧 planned)
    },
    Chord {
        value: Option<String>,
        chord: String,  // [Am] chord symbols (🚧 planned)
        char_index: usize,
    },
    Mordent {
        value: Option<String>,
        char_index: usize,
    },
    Space {
        char_index: usize,
        count: usize,
        value: Option<String>,
    },
    Unknown {
        unknown_value: String,
        value: Option<String>,
        char_index: usize,
    },
    /// Newline token - explicit line terminator (upper lines cannot have EOI)
    Newline {
        value: Option<String>,
        char_index: usize,
        newline_value: String,
    },
}

// LowerLine elements from specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LowerElement {
    LowerOctaveMarker {
        value: Option<String>,
        char_index: usize,
        marker: String,  // "." or ":"
    },
    BeatGroupIndicator {
        value: Option<String>,
        char_index: usize,
        indicator_value: String,  // "___" for beat grouping
    },
    Syllable {
        value: Option<String>,
        char_index: usize,
        content: String,  // syllables like "dha", "he-llo"
    },
    Space {
        value: Option<String>,
        char_index: usize,
        count: usize,
    },
    Unknown {
        unknown_value: String,
        value: Option<String>,
        char_index: usize,
    },
    Newline {
        value: Option<String>,
        char_index: usize,
        newline_value: String,
    },
    EndOfInput {
        value: Option<String>,
        char_index: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Syllable {
    pub value: Option<String>,
    pub char_index: usize,
    pub content: String,
}