use crate::parser_v2::{Document, Stave, MusicalElement, Accidental};

pub struct HtmlCssConverterV2 {
    pub include_css: bool,
    pub include_wrapper: bool,
    pub notation_system: String,
}

impl Default for HtmlCssConverterV2 {
    fn default() -> Self {
        HtmlCssConverterV2 {
            include_css: true,
            include_wrapper: false, // Default to fragment for web integration
            notation_system: "number".to_string(),
        }
    }
}

impl HtmlCssConverterV2 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_system(system: &str) -> Self {
        HtmlCssConverterV2 {
            notation_system: system.to_string(),
            ..Self::default()
        }
    }

    pub fn convert_document(&self, document: &Document) -> String {
        let mut html = String::new();

        if self.include_wrapper {
            html.push_str("<!DOCTYPE html>\n");
            html.push_str("<html lang=\"en\">\n<head>\n");
            html.push_str("<meta charset=\"UTF-8\">\n");
            html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
            html.push_str("<title>Musical Notation</title>\n");
            
            if self.include_css {
                html.push_str("<link rel=\"stylesheet\" href=\"css/musical-notation.css\">\n");
            }
            
            html.push_str("</head>\n<body>\n");
        }

        let notation_class = match self.notation_system.as_str() {
            "sargam" => "sargam-notation",
            "western" => "western-notation", 
            "number" | _ => "number-notation",
        };

        html.push_str(&format!("<div class=\"musical-notation {}\">\n", notation_class));

        // Add metadata from directives
        if !document.directives.is_empty() {
            html.push_str(&self.render_metadata(&document.directives));
        }

        // Render each stave
        for stave in &document.staves {
            html.push_str(&self.render_stave(stave));
        }

        html.push_str("</div>\n");

        if self.include_wrapper {
            html.push_str("</body>\n</html>");
        }

        html
    }

    fn render_metadata(&self, directives: &std::collections::HashMap<String, String>) -> String {
        let mut html = String::new();
        
        html.push_str("<div class=\"metadata\">\n");
        
        if let Some(title) = directives.get("title") {
            html.push_str(&format!("<h2 class=\"title\">{}</h2>\n", html_escape(title)));
        }
        
        if let Some(key) = directives.get("key") {
            html.push_str(&format!("<div class=\"key-signature\">Key: {}</div>\n", html_escape(key)));
        }
        
        if let Some(time) = directives.get("time") {
            html.push_str(&format!("<div class=\"time-signature\">{}</div>\n", html_escape(time)));
        }
        
        html.push_str("</div>\n");
        html
    }

    fn render_stave(&self, stave: &Stave) -> String {
        let mut html = String::new();
        
        html.push_str("<div class=\"stave\">\n");
        
        // Render upper annotations
        for annotation in &stave.upper_annotations {
            html.push_str(&format!("<div class=\"upper-annotation\">{}</div>\n", html_escape(annotation)));
        }
        
        // Render content line (the musical notation)
        html.push_str("<div class=\"content-line\">\n");
        if let Some(line_number) = stave.content.line_number {
            html.push_str(&format!("<span class=\"line-number\">{})</span> ", line_number));
        }
        
        html.push_str("<span class=\"musical-content\">\n");
        for element in &stave.content.elements {
            html.push_str(&self.render_musical_element(element));
        }
        html.push_str("</span>\n");
        html.push_str("</div>\n");
        
        // Render lower annotations  
        for annotation in &stave.lower_annotations {
            html.push_str(&format!("<div class=\"lower-annotation\">{}</div>\n", html_escape(annotation)));
        }
        
        // Lyrics are now embedded in notes as syllables - no separate lyrics rendering needed
        
        html.push_str("</div>\n");
        html
    }

    fn render_musical_element(&self, element: &MusicalElement) -> String {
        match element {
            MusicalElement::Note { pitch, accidental, octave } => {
                self.render_note(pitch, accidental, octave)
            },
            MusicalElement::Dash => {
                "<span class=\"note-wrapper\"><span class=\"note dash\">-</span></span>\n".to_string()
            },
            MusicalElement::Rest => {
                "<span class=\"note-wrapper\"><span class=\"note rest\">-</span></span>\n".to_string()
            },
            MusicalElement::Barline(style) => {
                self.render_barline(style)
            },
        }
    }

    fn render_note(&self, pitch: &str, accidental: &Option<Accidental>, octave: &i8) -> String {
        let mut html = String::new();
        let mut classes = vec!["note-wrapper"];
        
        // Add octave class
        let octave_class = match *octave {
            1 => Some("octave-upper-1"),
            2 => Some("octave-upper-2"), 
            -1 => Some("octave-lower-1"),
            -2 => Some("octave-lower-2"),
            _ => None,
        };
        
        if let Some(octave_cls) = octave_class {
            classes.push(octave_cls);
        }
        
        html.push_str(&format!("<span class=\"{}\">\n", classes.join(" ")));
        
        // Render accidental if present
        if let Some(ref acc) = accidental {
            html.push_str(&format!("<span class=\"accidental {}\">{}</span>", 
                self.accidental_class(acc),
                self.accidental_symbol(acc)
            ));
        }
        
        // Render the note itself
        html.push_str(&format!("<span class=\"note\">{}</span>", 
            self.pitch_symbol(pitch)));
        
        html.push_str("</span>\n");
        html
    }

    fn render_barline(&self, style: &str) -> String {
        let class = match style {
            "|" => "barline-single",
            "||" => "barline-double",
            _ => "barline-single",
        };
        
        format!("<span class=\"barline {}\"></span>\n", class)
    }

    fn render_lyrics_line(&self, lyrics_line: &crate::parser_v2::LyricsLine) -> String {
        let mut html = String::new();
        
        html.push_str("<div class=\"lyrics-line\">\n");
        
        for syllable in &lyrics_line.syllables {
            html.push_str(&format!("<span class=\"lyric\">{}</span>\n", html_escape(syllable)));
        }
        
        html.push_str("</div>\n");
        html
    }

    fn pitch_symbol(&self, pitch: &str) -> String {
        match self.notation_system.as_str() {
            "sargam" => self.pitch_to_sargam(pitch),
            "western" => pitch.to_uppercase(),
            "number" | _ => self.pitch_to_number(pitch),
        }
    }

    fn pitch_to_number(&self, pitch: &str) -> String {
        match pitch.to_uppercase().as_str() {
            "1" => "1".to_string(),
            "2" => "2".to_string(), 
            "3" => "3".to_string(),
            "4" => "4".to_string(),
            "5" => "5".to_string(),
            "6" => "6".to_string(),
            "7" => "7".to_string(),
            _ => pitch.to_string(), // Fallback to original
        }
    }

    fn pitch_to_sargam(&self, pitch: &str) -> String {
        match pitch.to_uppercase().as_str() {
            "1" => "S".to_string(),
            "2" => "R".to_string(),
            "3" => "G".to_string(), 
            "4" => "M".to_string(),
            "5" => "P".to_string(),
            "6" => "D".to_string(),
            "7" => "N".to_string(),
            "S" => "S".to_string(),
            "R" => "R".to_string(),
            "G" => "G".to_string(),
            "M" => "M".to_string(),
            "P" => "P".to_string(),
            "D" => "D".to_string(),
            "N" => "N".to_string(),
            _ => pitch.to_string(), // Fallback to original
        }
    }

    fn accidental_class(&self, accidental: &Accidental) -> &str {
        match accidental {
            Accidental::Sharp => "sharp",
            Accidental::Flat => "flat",
        }
    }

    fn accidental_symbol(&self, accidental: &Accidental) -> &str {
        match accidental {
            Accidental::Sharp => "♯",
            Accidental::Flat => "♭",
        }
    }
}

// HTML escape utility function
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}