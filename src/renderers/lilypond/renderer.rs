use crate::stave::ProcessedStave;
use super::formatters::FullFormatter;

/// Main LilyPond rendering orchestrator
pub struct LilyPondRenderer {
    formatter: FullFormatter,
}

impl LilyPondRenderer {
    pub fn new() -> Self {
        Self {
            formatter: FullFormatter::new(),
        }
    }
    
    /// Convert staves to LilyPond notation
    pub fn render(&self, staves: &[ProcessedStave]) -> String {
        self.formatter.format_staves(staves)
    }
}

impl Default for LilyPondRenderer {
    fn default() -> Self {
        Self::new()
    }
}