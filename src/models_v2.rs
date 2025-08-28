// New AST Models - Parser Output Only
// This module defines type-safe enums for parser output, replacing the monolithic Node struct
// for the flat AST level (before FSM processing)

use serde::{Deserialize, Serialize};
use crate::models::{Node, Metadata}; // Keep using existing for compatibility
use crate::pitch::Degree;

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedDocument {
    pub metadata: Metadata, // Reuse existing metadata structure
    pub elements: Vec<ParsedElement>,
    pub notation_system: Option<String>,
}

/// Complete document structure using ParsedElement (replacement for Document)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentV2 {
    pub metadata: Metadata, // Reuse existing metadata structure
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

/// Convert ParsedElement back to legacy Node for compatibility with FSM and converters
/// This allows incremental refactoring without breaking existing code
impl From<ParsedElement> for Node {
    fn from(element: ParsedElement) -> Self {
        match element {
            ParsedElement::Note { degree, octave, value, position, children, duration } => {
                let mut node = Node::new(
                    "PITCH".to_string(),
                    value,
                    position.row,
                    position.col,
                );
                node.degree = Some(degree);
                node.octave = Some(octave);
                
                // Store duration if present
                if let Some((num, denom)) = duration {
                    node.duration_fraction = Some(format!("{}/{}", num, denom));
                }
                
                // Convert children to child nodes
                for child in children {
                    let child_node = match child {
                        ParsedChild::OctaveMarker { symbol, .. } => {
                            Node::new("OCTAVE_MARKER".to_string(), symbol, position.row, position.col)
                        },
                        ParsedChild::Ornament { kind, .. } => {
                            let ornament_type = match kind {
                                OrnamentType::Mordent => "MORDENT",
                                OrnamentType::Trill => "TRILL", 
                                OrnamentType::Turn => "TURN",
                                OrnamentType::Grace => "GRACE",
                            };
                            Node::new(ornament_type.to_string(), kind.to_string(), position.row, position.col)
                        },
                        ParsedChild::Syllable { text, .. } => {
                            Node::new("SYL".to_string(), text, position.row, position.col)
                        },
                    };
                    node.nodes.push(child_node);
                }
                
                node
            },
            
            ParsedElement::Rest { value, position, duration } => {
                let mut node = Node::new("REST".to_string(), value, position.row, position.col);
                if let Some((num, denom)) = duration {
                    node.duration_fraction = Some(format!("{}/{}", num, denom));
                }
                node
            },
            
            ParsedElement::Dash { degree, octave, position, duration } => {
                let mut node = Node::new("DASH".to_string(), "-".to_string(), position.row, position.col);
                node.degree = degree;
                node.octave = octave;
                if let Some((num, denom)) = duration {
                    node.duration_fraction = Some(format!("{}/{}", num, denom));
                }
                node
            },
            
            ParsedElement::Barline { style, position } => {
                Node::new("BARLINE".to_string(), style, position.row, position.col)
            },
            
            ParsedElement::SlurStart { position } => {
                Node::new("SLUR_START".to_string(), "(".to_string(), position.row, position.col)
            },
            
            ParsedElement::SlurEnd { position } => {
                Node::new("SLUR_END".to_string(), ")".to_string(), position.row, position.col)
            },
            
            ParsedElement::Whitespace { width, position } => {
                Node::new("WHITESPACE".to_string(), " ".repeat(width), position.row, position.col)
            },
            
            ParsedElement::Newline { position } => {
                Node::new("NEWLINE".to_string(), "\n".to_string(), position.row, position.col)
            },
            
            ParsedElement::Word { text, position } => {
                Node::new("WORD".to_string(), text, position.row, position.col)
            },
            
            ParsedElement::Symbol { value, position } => {
                Node::new("SYMBOLS".to_string(), value, position.row, position.col)
            },
            
            ParsedElement::Unknown { value, position } => {
                Node::new("UNKNOWN".to_string(), value, position.row, position.col)
            },
        }
    }
}

/// Convert a vector of ParsedElements to Nodes for compatibility
pub fn parsed_elements_to_nodes(elements: Vec<ParsedElement>) -> Vec<Node> {
    elements.into_iter().map(|e| e.into()).collect()
}

/// Convert DocumentV2 to legacy Document for WASM compatibility
impl From<DocumentV2> for crate::Document {
    fn from(doc_v2: DocumentV2) -> Self {
        crate::Document {
            metadata: doc_v2.metadata,
            nodes: parsed_elements_to_nodes(doc_v2.elements),
            notation_system: doc_v2.notation_system,
        }
    }
}