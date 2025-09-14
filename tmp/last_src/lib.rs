// Music-Text: Incremental Pipeline Architecture
// Parse → Analyze → Render pipeline for musical notation processing

// Stage 1: Parsing - Text input to structured representation
pub mod parse;                 // RENAMED from document/ - Hand-written recursive descent parser

// Stage 2: Analysis - Semantic processing and rhythm analysis  
pub mod rhythm;               // Rhythm FSM and temporal analysis (kept at root due to type conflicts)
pub mod stave;                // Stave-level processing and grouping

// Stage 3: Rendering - Output generation
pub mod renderers;            // CONSOLIDATED with converters/ - Multi-format output generation

// Foundation: Core domain models and orchestration
pub mod models;               // Core domain types and pitch systems
pub mod pipeline;             // Top-level processing orchestration
pub mod tokenizer;            // Smart tokenization and notation system classification

// Testing and utilities
pub mod smoke_test;

// Re-export main parsing functionality
pub use parse::{
    parse_document, ParseError,
    Document, Stave, ContentLine, ContentElement, Position, TextLine
};

// Re-export rhythm analysis functionality
pub use stave::{
    analyze_rhythm
};

// Re-export pipeline functionality
pub use pipeline::{
    process_notation, ProcessingResult
};

// Re-export rhythm FSM functionality
pub use rhythm::{
    process_rhythm, Beat, BeatElement, Event, Item
};