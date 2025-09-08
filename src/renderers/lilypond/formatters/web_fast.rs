use crate::renderers::lilypond::renderer::TemplateContext;

/// Web-fast LilyPond formatter for SVG generation
pub struct WebFastFormatter;

impl WebFastFormatter {
    pub fn new() -> Self {
        Self
    }
    
    /// Format notes content using web-optimized template
    pub fn format(&self, notes_content: &str) -> String {
        self.format_with_lyrics(notes_content, "")
    }
    
    /// Format notes and lyrics using web-optimized template with addLyrics pattern
    pub fn format_with_lyrics(&self, notes_content: &str, lyrics_content: &str) -> String {
        let lyrics_option = if lyrics_content.is_empty() {
            None
        } else {
            Some(lyrics_content.to_string())
        };
        
        let context = TemplateContext {
            version: "2.24.0".to_string(),
            staves: notes_content.to_string(),
            source_comment: None,
            title: None, 
            composer: None,
            time_signature: None,
            key_signature: None,
            lyrics: lyrics_option,
            midi: false,
            tempo: None,
        };
        
        let template_str = include_str!("../web-fast.ly.mustache");
        let template = mustache::compile_str(template_str)
            .expect("Failed to compile web-fast LilyPond template");
        
        template.render_to_string(&context)
            .expect("Failed to render web-fast LilyPond template")
    }
}

impl Default for WebFastFormatter {
    fn default() -> Self {
        Self::new()
    }
}