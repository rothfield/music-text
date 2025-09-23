// Domain model modules
pub mod core;
pub mod elements;
pub mod notation;
pub mod position;
pub mod barlines;
pub mod pitch_systems;
pub mod rhythm;

// Re-export everything for convenience
pub use core::*;
pub use elements::*;
pub use notation::*;
pub use position::*;
pub use barlines::*;
pub use rhythm::*;

// For backwards compatibility, also export the old pitch module concepts
// These are now unified in notation.rs
pub use notation::PitchCode as Degree; // Legacy alias
pub use notation::{Notation, NotationSystem, lookup_pitch};