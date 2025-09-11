pub mod renderer;
pub mod formatters;

use crate::stave::ProcessedStave;
use renderer::LilyPondRenderer;

// Single LilyPond rendering function
pub fn render_lilypond(staves: &[ProcessedStave]) -> String {
    let renderer = LilyPondRenderer::new();
    renderer.render(staves)
}