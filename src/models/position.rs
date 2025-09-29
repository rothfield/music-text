use serde::{Deserialize, Serialize};

// Position and metadata models

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SlurPosition {
    None,
    Start,
    Middle,
    End,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub index_in_line: usize,
    pub index_in_doc: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attributes {
    pub slur_position: SlurPosition,
    pub value: Option<String>,
    pub position: Position,
}

