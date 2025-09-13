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
pub use lilypond::{render_lilypond, render_lilypond_with_directives, render_lilypond_from_document};
pub use vexflow::{render_vexflow_svg, render_vexflow_svg_with_directives, render_vexflow_svg_from_document,
                  render_vexflow_data, render_vexflow_data_with_directives, render_vexflow_data_from_document};
pub use lilypond_generator::LilyPondGenerator;