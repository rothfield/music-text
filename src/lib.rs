pub mod document;
pub mod stave;
pub mod rhythm;
pub mod pipeline;
pub mod renderers;
pub mod converters;
pub mod old_models;
pub mod smoke_test;

// Re-export main parsing functionality
pub use document::{
    parse_document, ParseError,
    Document, Stave, ContentLine, ContentElement, Position, TextLine
};

// Re-export stave parsing functionality
pub use stave::{
    parse_document_staves, ProcessedStave
};

// Re-export pipeline functionality
pub use pipeline::{
    process_notation, ProcessingResult
};

// Re-export rhythm FSM functionality
pub use rhythm::{
    process_rhythm, Beat, BeatElement, Event, Item
};