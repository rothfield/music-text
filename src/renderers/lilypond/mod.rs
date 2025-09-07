use crate::document::Stave;

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
                    // Simple note mapping (1->c, 2->d, etc.)
                    let pitch = match note.syllable.as_str() {
                        "1" => "c",
                        "2" => "d", 
                        "3" => "e",
                        "4" => "f",
                        "5" => "g",
                        "6" => "a",
                        "7" => "b",
                        _ => "c", // fallback
                    };
                    result.push_str(&format!("{}4 ", pitch));
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

/// Convert staves to full LilyPond score
pub fn render_full_lilypond(staves: &[Stave]) -> String {
    let minimal = render_minimal_lilypond(staves);
    format!(
        "\\version \"2.24.0\"
\\paper {{
  #(set-paper-size \"a4\")
}}
\\score {{
  \\new Staff {{
    \\clef treble
{}
  }}
  \\layout {{ }}
  \\midi {{ }}
}}", 
        minimal.lines().skip(1).collect::<Vec<_>>().join("\n")
    )
}