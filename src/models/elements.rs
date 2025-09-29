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
    // Note-specific fields
    pub octave: i8,                     // Octave -4..4
    pub pitch_code: super::notation::PitchCode,          // Normalized pitch code
    pub notation_system: super::notation::NotationSystem, // Which notation system this note uses
    pub numerator: Option<u32>,         // Simple duration numerator
    pub denominator: Option<u32>,       // Simple duration denominator
}

impl Note {
    /// Factory function to create a new Note with consistent default values
    pub fn new(
        value: Option<String>,
        pitch_code: super::notation::PitchCode,
        notation_system: super::notation::NotationSystem,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),             // Generate unique ID
            value,
            octave: 0,                      // Default octave
            pitch_code,
            notation_system,
            numerator: None,                // Will be populated by rhythm analysis
            denominator: None,              // Will be populated by rhythm analysis
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dash {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: Option<String>,
    // Duration fields populated by rhythm analyzer
    pub numerator: Option<u32>,
    pub denominator: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreathMark {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rest {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: Option<String>,
    // Duration fields populated by rhythm analyzer
    pub numerator: Option<u32>,
    pub denominator: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Space {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: Option<String>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Whitespace {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Newline {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndOfInput {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: Option<String>,
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
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: Option<String>,
    pub divisions: Option<usize>,        // Number of divisions in this beat (e.g., 12 for 12 sixteenths)
    pub is_tuplet: Option<bool>,         // Whether this beat is a tuplet (3, 5, 6, 7, etc. divisions)
    pub tuplet_ratio: Option<(usize, usize)>, // Tuplet ratio (e.g., (3, 2) for triplet)
    pub tied_to_previous: Option<bool>,  // Whether this beat's first note is tied to the previous beat's last note
    pub total_duration: Option<Fraction>, // Total duration of this beat (e.g., 1/4 for quarter note beat)
    pub elements: Vec<BeatElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownToken {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: Option<String>,
    pub token_value: String,
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
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub elements: Vec<ContentElement>,  // Mixed elements: barlines, whitespace, beats
    pub value: Option<String>,
}

// Non-spatial annotation lines

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricsLine {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: Option<String>,
    pub syllables: Vec<Syllable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitespaceLine {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub elements: Vec<crate::rhythm::types::ParsedElement>, // Whitespace elements and optional newline
    pub value: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Syllable {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: Option<String>,
    pub content: String,
}