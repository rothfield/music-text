/// Converters module - transforms FSM output to various notation formats
/// 
/// Shared utilities for all converters:
/// - Transposition logic (movable-do system)
/// - Rhythm calculations
/// - Musical transformations
///
/// Format-specific converters:
/// - LilyPond (text-based notation)
/// - VexFlow (JSON for web rendering)

pub mod transposition;
pub mod lilypond;
pub mod vexflow;
pub mod html_css_converter_v2; // HTML/CSS converter for V2 system

// Re-export main conversion functions
pub use lilypond::convert_elements_to_lilypond_src;
pub use vexflow::convert_elements_to_staff_notation;
pub use html_css_converter_v2::HtmlCssConverterV2;

// Re-export shared utilities
pub use transposition::{transpose_degree_with_octave, transpose_degree};