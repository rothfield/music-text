// Hand-written recursive descent parser module
pub mod document;
pub mod stave;
pub mod hash_line;
pub mod content_line;
pub mod upper_line;
pub mod lower_line;
pub mod error;

// Re-export main functionality
pub use document::parse_document;
pub use error::ParseError;