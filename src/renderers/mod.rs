// Renderers Module - Stage 3 of Pipeline Architecture
// Multi-format output generation for music notation
// CONSOLIDATED with converters/ during incremental refactoring
// Generates LilyPond, VexFlow, and other output formats

pub mod transposition;
pub mod lilypond;
pub mod vexflow;
pub mod lilypond_generator;
pub mod converters_lilypond;

// Re-export main rendering functions
pub use lilypond::{render_minimal_lilypond, render_full_lilypond, render_web_fast_lilypond};
pub use vexflow::{render_vexflow_svg, render_vexflow_data};
pub use lilypond_generator::LilyPondGenerator;