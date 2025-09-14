// Core rhythm processing types - extracted from old_models.rs
// These types are used by the rhythm FSM for processing musical elements

use serde::{Deserialize, Serialize};

// Pitch degree representation for rhythm processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Degree {
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

/// Position information for parsed elements
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Position {
    pub row: usize,
    pub col: usize,
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
}

/// Role of a note in a slur phrase
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SlurRole {
    Start,
    Middle,
    End,
}

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
        slur: Option<SlurRole>, // Boundary information (Start/Middle/End)
        beat_group: Option<BeatGroupRole>, // Boundary information (Start/Middle/End)  
        in_slur: bool, // Convenience flag: slur.is_some()
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
            slur: None,
            beat_group: None,
            in_slur: false,
            in_beat_group: false,
        }
    }
}