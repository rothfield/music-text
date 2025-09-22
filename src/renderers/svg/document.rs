use serde::{Deserialize, Serialize};

/// Main document structure for SVG renderer POC
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Document {
    pub title: Option<String>,
    pub composer: Option<String>,
    pub notation_type: String,
    pub font_size: f32,
    pub supports_utf8: bool,
    pub elements: Vec<Element>,
}

/// Element types in the document
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Element {
    #[serde(rename = "pitch")]
    Pitch {
        value: String,
        octave: i8,
        accidental: Option<String>,
        ornaments: Vec<Ornament>,
        lyrics: Vec<String>,
    },
    #[serde(rename = "dash")]
    Dash { is_rest: bool },
    #[serde(rename = "barline")]
    Barline { style: String },
}

/// Ornament types from doremi-script analysis
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Ornament {
    #[serde(rename = "before_grace_notes")]
    BeforeGraceNotes { notes: Vec<OrnamentNote> },
    #[serde(rename = "on_top_grace_notes")]
    OnTopGraceNotes { notes: Vec<OrnamentNote> },
    #[serde(rename = "after_grace_notes")]
    AfterGraceNotes { notes: Vec<OrnamentNote> },
    #[serde(rename = "symbolic_ornament")]
    SymbolicOrnament { symbol: String },
}

/// Note representation within ornaments
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrnamentNote {
    pub value: String,
    pub octave: i8,
    pub accidental: Option<String>,
}

impl Document {
    /// Create a new empty document with default settings
    pub fn new() -> Self {
        Self {
            title: None,
            composer: None,
            notation_type: "number".to_string(),
            font_size: 14.0,
            supports_utf8: true,
            elements: Vec::new(),
        }
    }

    /// Validate that the document has a supported notation type
    pub fn validate(&self) -> Result<(), String> {
        match self.notation_type.as_str() {
            "number" | "sargam" | "western" => Ok(()),
            other => Err(format!("Unsupported notation type: {}", other)),
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}