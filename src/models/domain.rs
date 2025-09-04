// src/models/mod.rs
// Core data structures for the music-text parser

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::models::Degree;

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BarlineType {
    Single,      // "|"
    Double,      // "||" 
    Final,       // "|."
    RepeatStart, // "|:"
    RepeatEnd,   // ":|"
    RepeatBoth,  // ":|:"
}

impl BarlineType {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "|" => Ok(BarlineType::Single),
            "||" => Ok(BarlineType::Double),
            "|." => Ok(BarlineType::Final),
            "|:" => Ok(BarlineType::RepeatStart),
            ":|" => Ok(BarlineType::RepeatEnd),
            ":|:" => Ok(BarlineType::RepeatBoth),
            _ => Err(format!("Unknown barline type: '{}'", s)),
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            BarlineType::Single => "|",
            BarlineType::Double => "||",
            BarlineType::Final => "|.",
            BarlineType::RepeatStart => "|:",
            BarlineType::RepeatEnd => ":|",
            BarlineType::RepeatBoth => ":|:",
        }
    }
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    pub degree: Option<Degree>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub octave: Option<i8>, // 0 = middle, 1 = upper, -1 = lower, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slur_start: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slur_end: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub beat_bracket_start: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub beat_bracket_end: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syl: Option<String>,  // Syllable/lyric text associated with this note
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_fraction: Option<String>, // Duration as fraction "1/2", "1/4", etc
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
            degree: None,
            octave: None,
            slur_start: None,
            slur_end: None,
            beat_bracket_start: None,
            beat_bracket_end: None,
            syl: None,
            duration_fraction: None,
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
            degree: None,
            octave: None,
            slur_start: None,
            slur_end: None,
            beat_bracket_start: None,
            beat_bracket_end: None,
            syl: None,
            duration_fraction: None,
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
    // SlurStart,
    // SlurEnd,
    Dash,
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
            // TokenType::SlurStart => "SLUR_START",
            // TokenType::SlurEnd => "SLUR_END",
            TokenType::Dash => "DASH",
        }
    }
}