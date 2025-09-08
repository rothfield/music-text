// Document parser module - Hand-written recursive descent parser
// Clean, maintainable, and debuggable music notation parsing

pub mod manual_parser;
pub mod model;

// Re-export key types and functions for convenience
pub use model::{Document, Stave, ContentLine, MusicalElement, TextLine, Position, PitchCode, NotationSystem, Source};
pub use manual_parser::{parse_document, ParseError};