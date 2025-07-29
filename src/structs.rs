use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChunkInfo {
    pub value: String,
    pub col: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LineInfo {
    pub line_number: usize,
    pub line_text: String,
    pub chunks: Vec<ChunkInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Token {
    #[serde(rename = "type")]
    pub token_type: String,
    pub value: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Title {
    pub text: String,
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Directive {
    pub key: String,
    pub value: String,
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub title: Option<Title>,
    pub directives: Vec<Directive>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    pub metadata: Metadata,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(rename = "val")]
    pub value: String,
    pub row: usize,
    pub col: usize,
    pub divisions: usize,
    pub dash_consumed: bool,
    pub nodes: Vec<Node>,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum TokenType {
    Pitch,
    Barline,
    Symbols,
    Word,
    Unknown,
    Whitespace,
}

impl TokenType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TokenType::Pitch => "PITCH",
            TokenType::Barline => "BARLINE",
            TokenType::Symbols => "SYMBOLS",
            TokenType::Word => "WORD",
            TokenType::Unknown => "UNKNOWN",
            TokenType::Whitespace => "WHITESPACE",
        }
    }
}
