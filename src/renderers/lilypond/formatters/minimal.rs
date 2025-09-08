/// Minimal LilyPond formatter with word wrapping
pub struct MinimalFormatter {
    wrap_width: usize,
}

impl MinimalFormatter {
    pub fn new() -> Self {
        Self {
            wrap_width: 70,
        }
    }
    
    pub fn with_wrap_width(wrap_width: usize) -> Self {
        Self {
            wrap_width,
        }
    }
    
    /// Format notes content as minimal LilyPond single line
    pub fn format(&self, notes_content: &str) -> String {
        format!("\\version \"2.24.0\" {{ {} }}", notes_content.trim())
    }
    
    /// Format notes and lyrics using addLyrics pattern (like bansuri example)
    pub fn format_with_lyrics(&self, notes_content: &str, lyrics_content: &str) -> String {
        if lyrics_content.is_empty() {
            // No lyrics, use simple format
            self.format(notes_content)
        } else {
            // Use addLyrics pattern based on bansuri example
            format!(
                "\\version \"2.24.0\"\n\
                melody = {{\n\
                  \\clef treble\n\
                  {}\n\
                }}\n\
                \n\
                text = \\lyricmode {{\n\
                  {}\n\
                }}\n\
                \n\
                \\score {{\n\
                  <<\n\
                    \\new Voice = \"one\" {{\n\
                      \\melody\n\
                    }}\n\
                    \\new Lyrics \\lyricsto \"one\" \\text\n\
                  >>\n\
                }}\n",
                notes_content.trim(),
                lyrics_content.trim()
            )
        }
    }
    
    /// Word wrap text at specified column, breaking at spaces
    fn word_wrap(&self, text: &str) -> String {
        let mut result = String::new();
        let mut current_line = String::new();
        
        for word in text.split(' ') {
            if current_line.len() + word.len() + 1 > self.wrap_width && !current_line.is_empty() {
                result.push_str(&current_line);
                result.push('\n');
                current_line = String::new();
            }
            
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }
        
        if !current_line.is_empty() {
            result.push_str(&current_line);
        }
        
        result
    }
}

impl Default for MinimalFormatter {
    fn default() -> Self {
        Self::new()
    }
}