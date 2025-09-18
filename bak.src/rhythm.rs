// Rhythm Module - Stage 2 Analysis (Kept at Root)
// Temporal analysis and rhythm FSM for musical content
// NOTE: Contains duplicate types with models/ - prevents move to analyze/
// Future work: Type unification needed to enable further reorganization
pub mod analyzer;
pub mod types;
pub mod converters;

// Re-export main functionality
pub use analyzer::*;
pub use types::*;
pub use converters::*;