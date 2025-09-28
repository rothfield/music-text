use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use crate::models::ui_state::UIState;

// Core document structure models
// These represent the fundamental structure of a music document

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentElement {
    BlankLines(BlankLines),
    Stave(Stave),
}

impl DocumentElement {
    pub fn as_stave_mut(&mut self) -> Option<&mut Stave> {
        match self {
            DocumentElement::Stave(s) => Some(s),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Document {
    #[serde(rename = "documentUUID")]
    pub document_uuid: Option<String>,  // Document's unique identifier
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,                      // Internal UUID for document structure
    pub value: Option<String>,
    pub char_index: usize,
    pub title: Option<String>,
    pub author: Option<String>,
    #[serde(default)]
    pub directives: HashMap<String, String>, // key -> value
    #[serde(default)]
    pub elements: Vec<DocumentElement>, // Document as sequence of elements
    #[serde(default)]
    pub ui_state: UIState,
    #[serde(default)]
    pub timestamp: String,
}

impl Document {
    /// Get unique notation systems detected across all staves
    pub fn get_detected_notation_systems(&self) -> Vec<super::notation::NotationSystem> {
        use std::collections::HashSet;

        let mut systems = HashSet::new();
        for element in &self.elements {
            if let DocumentElement::Stave(stave) = element {
                systems.insert(stave.notation_system);
            }
        }

        let mut result: Vec<super::notation::NotationSystem> = systems.into_iter().collect();
        result.sort_by_key(|system| match system {
            super::notation::NotationSystem::Number => 0,
            super::notation::NotationSystem::Western => 1,
            super::notation::NotationSystem::Sargam => 2,
            super::notation::NotationSystem::Bhatkhande => 3,
            super::notation::NotationSystem::Tabla => 4,
        });
        result
    }
}

// Blank lines structure (newline (whitespace* newline)+)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlankLines {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: Option<String>, // The complete blank lines content
    pub char_index: usize, // Converted from line/column for consistency
    pub line: usize,
    pub column: usize,
    pub index_in_line: usize,
    pub index_in_doc: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stave {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: Option<String>,
    pub char_index: usize, // Converted from line/column for consistency
    pub notation_system: super::notation::NotationSystem,
    pub line: usize,
    pub column: usize,
    pub index_in_line: usize,
    pub index_in_doc: usize,
    pub lines: Vec<StaveLine>,  // All lines in order
}

// Enum for different types of lines in a stave
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StaveLine {
    Text(TextLine),
    Content(Vec<crate::rhythm::types::ParsedElement>), // Keep for backward compat
    ContentLine(super::elements::ContentLine),  // New: elements parsed directly
    Lyrics(super::elements::LyricsLine),
    Whitespace(super::elements::WhitespaceLine),
    BlankLines(BlankLines),
}

impl StaveLine {
    pub fn as_content_line_mut(&mut self) -> Option<&mut crate::models::ContentLine> {
        match self {
            StaveLine::ContentLine(cl) => Some(cl),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextLine {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: Option<String>, // The text content
    pub char_index: usize,
}