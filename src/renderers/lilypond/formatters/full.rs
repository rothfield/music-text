use crate::renderers::lilypond::renderer::TemplateContext;

/// Full LilyPond formatter using mustache template
pub struct FullFormatter;

impl FullFormatter {
    pub fn new() -> Self {
        Self
    }
    
    /// Format notes content using full LilyPond template
    pub fn format(&self, notes_content: &str) -> String {
        let context = TemplateContext {
            version: "2.24.0".to_string(),
            staves: notes_content.to_string(),
            source_comment: None,
            title: None, 
            composer: None,
            time_signature: None,
            key_signature: None,
            lyrics: None,
            midi: false,
            tempo: None,
        };
        
        let template_str = include_str!("../standard.ly.mustache");
        let template = mustache::compile_str(template_str)
            .expect("Failed to compile LilyPond template");
        
        template.render_to_string(&context)
            .expect("Failed to render LilyPond template")
    }
}

impl Default for FullFormatter {
    fn default() -> Self {
        Self::new()
    }
}