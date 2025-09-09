// New AST Models - Parser Output Only
// This module defines type-safe enums for parser output, replacing the monolithic Node struct
// for the flat AST level (before FSM processing)

use serde::{Deserialize, Serialize};
// Removed legacy compatibility imports - using greenfield approach
use crate::models::Degree;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SlurRole {
    Start,
    Middle,
    End,
    StartEnd,
}

/// Shared position information for all parsed elements
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

impl std::fmt::Display for OrnamentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrnamentType::Mordent => write!(f, "mordent"),
            OrnamentType::Trill => write!(f, "trill"),
            OrnamentType::Turn => write!(f, "turn"),
            OrnamentType::Grace => write!(f, "grace"),
        }
    }
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

/// Parsed elements - what the parser extracts from raw text (flat structure)
/// This represents the notation as it appears spatially, without musical interpretation
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
        slur: Option<SlurRole>, // Assigned by vertical_parser
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
    
    /// Start of slur (created by spatial analysis of overlines)
    SlurStart { 
        position: Position,
    },
    
    /// End of slur (created by spatial analysis of overlines)
    SlurEnd { 
        position: Position,
    },
    
    /// Whitespace (preserved for spatial layout)
    Whitespace { 
        width: usize, // Number of spaces/characters
        position: Position,
    },
    
    /// Line break
    Newline { 
        position: Position,
    },
    
    /// Text word (lyrics, titles, etc.)
    Word { 
        text: String,
        position: Position,
    },
    
    /// Tala marker (beat numbers 0-6)
    Tala { 
        number: u8, // 0-6
        position: Position,
    },
    
    /// Generic symbols not otherwise classified
    Symbol { 
        value: String,
        position: Position,
    },
    
    /// Unknown/unrecognized element
    Unknown { 
        value: String,
        position: Position,
    },
}

/// Document structure using new parsed elements
/// Note: This is for the parser output level, before FSM processing
/// Complete document structure using ParsedElement (replacement for Document)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedDocument {
    pub elements: Vec<ParsedElement>, // Final processed elements after FSM
    pub notation_system: Option<String>,
}

impl ParsedElement {
    /// Get the position of any parsed element
    pub fn position(&self) -> &Position {
        match self {
            ParsedElement::Note { position, .. } => position,
            ParsedElement::Rest { position, .. } => position,
            ParsedElement::Dash { position, .. } => position,
            ParsedElement::Barline { position, .. } => position,
            ParsedElement::SlurStart { position } => position,
            ParsedElement::SlurEnd { position } => position,
            ParsedElement::Whitespace { position, .. } => position,
            ParsedElement::Newline { position } => position,
            ParsedElement::Word { position, .. } => position,
            ParsedElement::Tala { position, .. } => position,
            ParsedElement::Symbol { position, .. } => position,
            ParsedElement::Unknown { position, .. } => position,
        }
    }
    
    /// Get the original text value if available
    pub fn value(&self) -> String {
        match self {
            ParsedElement::Note { value, .. } => value.clone(),
            ParsedElement::Rest { value, .. } => value.clone(),
            ParsedElement::Dash { .. } => "-".to_string(),
            ParsedElement::Barline { style, .. } => style.clone(),
            ParsedElement::SlurStart { .. } => "(".to_string(),
            ParsedElement::SlurEnd { .. } => ")".to_string(),
            ParsedElement::Whitespace { width, .. } => " ".repeat(*width),
            ParsedElement::Newline { .. } => "\n".to_string(),
            ParsedElement::Word { text, .. } => text.clone(),
            ParsedElement::Tala { number, .. } => number.to_string(),
            ParsedElement::Symbol { value, .. } => value.clone(),
            ParsedElement::Unknown { value, .. } => value.clone(),
        }
    }
    
    /// Check if this element represents a musical pitch
    pub fn is_musical_note(&self) -> bool {
        matches!(self, 
            ParsedElement::Note { .. } | 
            ParsedElement::Rest { .. } | 
            ParsedElement::Dash { .. }
        )
    }
    
    /// Check if this element is a slur marker
    pub fn is_slur_marker(&self) -> bool {
        matches!(self, 
            ParsedElement::SlurStart { .. } | 
            ParsedElement::SlurEnd { .. }
        )
    }
}

impl Position {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}
