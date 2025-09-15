// Parse Module - Stage 1 of Pipeline Architecture  
// Hand-written recursive descent parser for music notation text
// RENAMED from document/ during incremental refactoring
// Clean, maintainable, and debuggable music notation parsing

pub mod recursive_descent;
pub mod model;
pub mod lower_line_parser;
pub mod upper_line_parser;
pub mod content_line_parser;

// Re-export key types and functions for convenience
pub use model::{Document, Directive, Stave, ContentLine, ContentElement, TextLine, Position, PitchCode, NotationSystem, Source, LowerLine, LowerElement, UpperLine, UpperElement, WhitespaceLine};
pub use recursive_descent::{parse_document, ParseError};
pub use lower_line_parser::parse_lower_line;
pub use upper_line_parser::parse_upper_line;
pub use content_line_parser::{parse_content_line, parse_content_line_with_row};