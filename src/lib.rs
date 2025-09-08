pub mod document;
pub mod stave_parser;
pub mod pipeline;
pub mod renderers;
pub mod converters;
pub mod rhythm_fsm;
pub mod old_models;

// Re-export main parsing functionality
pub use document::{
    parse as parse_notation, 
    parse_document,
    Rule, Error,
    Document, Stave, ContentLine, MusicalElement, Position, TextLine
};
pub use document::model::{DocumentElement, StaffGroup, StaffGroupType, NamedStave};
pub use document::tree_transformer::{pest_pair_to_json, build_document as parse_document_structure};

// Re-export stave parsing functionality
pub use stave_parser::{
    parse_document_staves, ProcessedStave, StaffGroupInfo
};

// Re-export pipeline functionality
pub use pipeline::{
    process_notation, ProcessingResult
};

// Re-export rhythm FSM functionality
pub use rhythm_fsm::{
    process_rhythm, Beat, BeatElement, Event, Item
};