pub mod document_parser;
pub mod stave_parser;
pub mod pipeline;

// Re-export main parsing functionality
pub use document_parser::{
    parse_notation, pest_pair_to_json, parse_document_structure,
    Pair, Error, Rule, Document, Stave, ContentLine, MusicalElement, Position, TextLine
};

// Re-export stave parsing functionality
pub use stave_parser::{
    parse_document_staves
};

// Re-export pipeline functionality
pub use pipeline::{
    process_notation, ProcessingResult
};