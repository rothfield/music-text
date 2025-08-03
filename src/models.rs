// src/models/mod.rs
// Core data structures for the notation parser

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::pitch::PitchCode;

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Title {
    pub text: String,
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Directive {
    pub key: String,
    pub value: String,
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Metadata {
    pub title: Option<Title>,
    pub directives: Vec<Directive>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detected_system: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, String>, // Generic key-value attributes (Key, Transpose, TimeSignature, etc.)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    pub metadata: Metadata,
    pub nodes: Vec<Node>,
    pub notation_system: Option<String>, // "Sargam", "Western", "Number" - controls output rendering
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pitch_code: Option<PitchCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub octave: Option<i8>, // 0 = middle, 1 = upper, -1 = lower, etc.
}

impl Node {
    pub fn new(node_type: String, value: String, row: usize, col: usize) -> Self {
        Self {
            node_type,
            value,
            row,
            col,
            divisions: 0,
            dash_consumed: false,
            nodes: Vec::new(),
            pitch_code: None,
            octave: None,
        }
    }
    
    pub fn with_children(node_type: String, value: String, row: usize, col: usize, nodes: Vec<Node>) -> Self {
        Self {
            node_type,
            value,
            row,
            col,
            divisions: 0,
            dash_consumed: false,
            nodes,
            pitch_code: None,
            octave: None,
        }
    }
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