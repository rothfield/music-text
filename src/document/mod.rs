// Document parser module - clean separation of concerns
// - grammar.pest: Defines the syntax
// - pest_interface.rs: Pest parser generation and interface
// - tree_transformer.rs: Transforms parse tree into AST
// - model.rs: Domain types (Document, Stave, etc.)

pub mod pest_interface;
pub mod tree_transformer;
pub mod model;

// Re-export key types and functions for convenience
pub use model::{Document, Stave, ContentLine, MusicalElement, TextLine, Position};
pub use tree_transformer::build_document;
pub use pest_interface::{parse, Rule, Error};

// Convenience function that combines parsing and transformation
pub fn parse_document(input: &str) -> Result<Document, String> {
    build_document(input)
}