// Core rhythm processing types - extracted from old_models.rs
// These types are used by the rhythm FSM for processing musical elements

use serde::{Deserialize, Serialize};
pub use crate::models::Degree;

/// Position information for parsed elements
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Position {
    pub row: usize,
    pub col: usize,
    pub char_index: usize,  // Zero-based character index into whole document
}

/// Types of musical ornaments that can be attached to notes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrnamentType {
    Mordent,
    Trill,
    Turn,
    Grace,
}

/// Child elements that can be attached to notes (vertical spatial relationships)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParsedChild {
    /// Octave markers like dots, colons, apostrophes
    OctaveMarker { 
        symbol: String,
        distance: i8, // Vertical distance from parent note (-1 = above, +1 = below)
    },
    /// Musical ornaments like mordents, trills
    Ornament { 
        kind: OrnamentType,
        distance: i8,
    },
    /// Syllable/lyric text
    Syllable {
        text: String,
        distance: i8,
    },
    /// Beat group indicator consumed by start note
    BeatGroupIndicator {
        symbol: String,  // Original underscore pattern ("__", "___", etc.)
        span: usize,     // Length of the underscore sequence
    },
}

/// Role of a note in a slur phrase
/// Role of a note in a beat group
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BeatGroupRole {
    Start,
    Middle,
    End,
}

/// Parsed elements - what the parser extracts from raw text (flat structure)
/// These are the working types for rhythm FSM processing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParsedElement {
    /// Musical note with pitch
    Note { 
        degree: Degree,
        octave: i8, // Calculated from octave markers
        value: String, // Original text value (e.g. "G", "S", "1")
        position: Position,
        children: Vec<ParsedChild>, // Attached ornaments, octave markers, lyrics
        duration: Option<(usize, usize)>, // Duration fraction (numerator, denominator) from FSM
        beat_group: Option<BeatGroupRole>, // Boundary information (Start/Middle/End)
        slur_position: crate::parse::model::SlurPosition, // Position within slurs
        in_beat_group: bool, // Convenience flag: beat_group.is_some()
    },
    
    /// Rest note
    Rest { 
        value: String, // Original text value
        position: Position,
        duration: Option<(usize, usize)>, // Duration fraction (numerator, denominator) from FSM
    },
    
    /// Note extension (dash) - inherits pitch from preceding note
    Dash { 
        degree: Option<Degree>, // Inherited from preceding note
        octave: Option<i8>, // Inherited or calculated from local octave markers
        position: Position,
        duration: Option<(usize, usize)>, // Duration fraction (numerator, denominator) from FSM
    },
    
    /// Bar line separator
    Barline { 
        style: String, // "|", "||", etc.
        position: Position,
        tala: Option<u8>, // Associated tala marker (0-6)
    },
    
    /// Whitespace/beat separator
    Whitespace { 
        value: String,
        position: Position,
    },
    
    /// Breath mark
    Symbol { 
        value: String,
        position: Position,
    },
    
    /// Unrecognized consecutive characters
    Unknown {
        value: String,      // The consecutive unrecognized characters
        position: Position, // Start position in source
    },
    
    /// Newline token - explicit line terminator
    Newline {
        value: String,      // The newline character(s)
        position: Position,
    },
    
    /// End of input token - explicit EOF terminator
    EndOfInput {
        position: Position,
    },
}

impl ParsedElement {
    /// Factory method for creating a Note with default spatial values
    pub fn new_note(
        degree: Degree,
        octave: i8,
        value: String,
        position: Position,
    ) -> Self {
        ParsedElement::Note {
            degree,
            octave,
            value,
            position,
            children: vec![],
            duration: None,
            beat_group: None,
            slur_position: crate::parse::model::SlurPosition::None,
            in_beat_group: false,
        }
    }

}