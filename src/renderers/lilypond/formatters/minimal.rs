/// Minimal LilyPond formatter
pub struct MinimalFormatter;

impl MinimalFormatter {
    pub fn new() -> Self {
        Self
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
    
}

impl Default for MinimalFormatter {
    fn default() -> Self {
        Self::new()
    }
}