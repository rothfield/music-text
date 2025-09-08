// Hand-written recursive descent parser module
pub mod document;
pub mod stave;
pub mod underline;
pub mod content_line;
pub mod error;

// Re-export main functionality
pub use document::parse_document;
pub use error::ParseError;