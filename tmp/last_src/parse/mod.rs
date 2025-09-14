// Parse Module - Stage 1 of Pipeline Architecture  
// Hand-written recursive descent parser for music notation text
// RENAMED from document/ during incremental refactoring
// Clean, maintainable, and debuggable music notation parsing

pub mod document_parser;
pub mod model;

// Re-export key types and functions for convenience
pub use model::{Document, Directive, Stave, ContentLine, ContentElement, TextLine, Position, PitchCode, NotationSystem, Source};
pub use document_parser::{parse_document, ParseError};