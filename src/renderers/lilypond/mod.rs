use crate::document::Stave;
use crate::converters::lilypond::pitch::pitchcode_to_lilypond;
use serde::Serialize;

#[derive(Serialize)]
struct TemplateContext {
    version: String,
    staves: String,
    source_comment: Option<String>,
    title: Option<String>,
    composer: Option<String>,
    time_signature: Option<String>,
    key_signature: Option<String>,
    lyrics: Option<String>,
    midi: bool,
    tempo: Option<String>,
}

/// Convert staves to minimal LilyPond notation
pub fn render_minimal_lilypond(staves: &[Stave]) -> String {
    let mut result = String::from("\\version \"2.24.0\"\n{\n  ");
    
    for (i, stave) in staves.iter().enumerate() {
        if i > 0 {
            result.push_str(" | ");
        }
        
        for element in &stave.content_line.elements {
            match element {
                crate::document::model::MusicalElement::Note(note) => {
                    // Use new converter system: PitchCode -> LilyPond with complete pitch support
                    match pitchcode_to_lilypond(note.pitch_code, note.octave, None) {
                        Ok(lily_note) => {
                            result.push_str(&format!("{}4 ", lily_note)); // Default to quarter notes for minimal
                        }
                        Err(_) => {
                            result.push_str("c4 "); // Fallback to C if conversion fails
                        }
                    }
                }
                crate::document::model::MusicalElement::Barline { .. } => {
                    result.push_str("| ");
                }
                crate::document::model::MusicalElement::Space { .. } => {
                    // Spaces are handled as timing in real implementation
                }
                _ => {} // Handle other elements
            }
        }
    }
    
    result.push_str("\n}");
    result
}

/// Convert staves to full LilyPond score using template
pub fn render_full_lilypond(staves: &[Stave]) -> String {
    let staves_content = render_staves_content(staves);
    
    let context = TemplateContext {
        version: "2.24.0".to_string(),
        staves: staves_content,
        source_comment: None,
        title: None, 
        composer: None,
        time_signature: None,
        key_signature: None,
        lyrics: None,
        midi: false,
        tempo: None,
    };
    
    let template_str = include_str!("standard.ly.mustache");
    let template = mustache::compile_str(template_str)
        .expect("Failed to compile LilyPond template");
    
    template.render_to_string(&context)
        .expect("Failed to render LilyPond template")
}

/// Convert staves to fast web-optimized LilyPond for SVG generation
pub fn render_web_fast_lilypond(staves: &[Stave]) -> String {
    let staves_content = render_staves_content(staves);
    
    let context = TemplateContext {
        version: "2.24.0".to_string(),
        staves: staves_content,
        source_comment: None,
        title: None, 
        composer: None,
        time_signature: None,
        key_signature: None,
        lyrics: None,
        midi: false,
        tempo: None,
    };
    
    let template_str = include_str!("web-fast.ly.mustache");
    let template = mustache::compile_str(template_str)
        .expect("Failed to compile web-fast LilyPond template");
    
    template.render_to_string(&context)
        .expect("Failed to render web-fast LilyPond template")
}

fn render_staves_content(staves: &[Stave]) -> String {
    let mut result = String::new();
    
    for (i, stave) in staves.iter().enumerate() {
        if i > 0 {
            result.push_str(" | ");
        }
        
        for element in &stave.content_line.elements {
            match element {
                crate::document::model::MusicalElement::Note(note) => {
                    // Use new converter system: PitchCode -> LilyPond with complete pitch support
                    match pitchcode_to_lilypond(note.pitch_code, note.octave, None) {
                        Ok(lily_note) => {
                            result.push_str(&format!("{}4 ", lily_note)); // Default to quarter notes for template
                        }
                        Err(_) => {
                            result.push_str("c4 "); // Fallback to C if conversion fails
                        }
                    }
                }
                crate::document::model::MusicalElement::Barline { .. } => {
                    result.push_str("| ");
                }
                crate::document::model::MusicalElement::Space { .. } => {
                    // Spaces handled as timing
                }
                _ => {}
            }
        }
    }
    
    result
}