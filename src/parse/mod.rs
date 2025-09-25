// Parse Module - Stage 1 of Pipeline Architecture
// Hand-written recursive descent parser for music notation text
// RENAMED from document/ during incremental refactoring
// Clean, maintainable, and debuggable music notation parsing

use uuid::Uuid;

pub mod recursive_descent;
pub mod model;
pub mod lower_line_parser;
pub mod upper_line_parser;
pub mod content_line_parser_v3;
pub mod actions;

// Grammar rule modules
pub mod title_line;
pub mod directive_line;
pub mod text_line;
pub mod header_line;
pub mod document_header;
pub mod line_classifier;
pub mod pitch;
pub mod beat;

// Re-export key types and functions for convenience
pub use model::{Document, Directive, Stave, ContentLine, ContentElement, TextLine, PitchCode, NotationSystem, LowerLine, LowerElement, UpperLine, UpperElement, WhitespaceLine, Beat, BeatElement, Note, Dash, BreathMark};
pub use recursive_descent::{parse_document, ParseError};
pub use lower_line_parser::parse_lower_line;
pub use upper_line_parser::parse_upper_line;
pub use pitch::{parse_pitch, is_pitch_start};
pub use beat::parse_beat;

/// Trait for elements that have a unique identifier
pub trait HasId {
    fn id(&self) -> &Uuid;
    fn set_id(&mut self, id: Uuid);

    /// Generate and set a new UUID
    fn generate_id(&mut self) {
        self.set_id(Uuid::new_v4());
    }
}