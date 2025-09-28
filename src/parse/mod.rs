// Parse Module - Stage 1 of Pipeline Architecture
// Hand-written recursive descent parser for music notation text
// RENAMED from document/ during incremental refactoring
// Clean, maintainable, and debuggable music notation parsing

use uuid::Uuid;

pub mod model;
pub mod content_line_parser_v3;
pub mod actions;

// Grammar rule modules
pub mod line_classifier;
pub mod pitch;
pub mod beat;
pub mod html;

// Re-export key types and functions for convenience
pub use model::{Document, Directive, Stave, ContentLine, ContentElement, TextLine, PitchCode, NotationSystem, WhitespaceLine, Beat, BeatElement, Note, Dash, BreathMark};
pub use pitch::{parse_pitch, is_pitch_start};
pub use beat::parse_beat;

// ParseError is defined below
#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Parse error at line {}, column {}: {}", self.line, self.column, self.message)
    }
}

impl std::error::Error for ParseError {}

impl From<ParseError> for String {
    fn from(error: ParseError) -> Self {
        error.to_string()
    }
}

/// Trait for elements that have a unique identifier
pub trait HasId {
    fn id(&self) -> &Uuid;
    fn set_id(&mut self, id: Uuid);

    /// Generate and set a new UUID
    fn generate_id(&mut self) {
        self.set_id(Uuid::new_v4());
    }
}