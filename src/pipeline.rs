use crate::document::{parse_document, Document, Stave};
use crate::stave_parser::parse_document_staves;
use serde::{Deserialize, Serialize};

/// The complete processing pipeline output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    pub original_input: String,
    pub parsed_document: Document,
    pub processed_staves: Vec<Stave>,
    pub minimal_lilypond: String,
    pub full_lilypond: String,
    pub vexflow_svg: String,
    pub vexflow_data: serde_json::Value,
}

/// Raw PEST parse output (not part of main pipeline but useful for debugging)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PestResult {
    pub success: bool,
    pub parse_tree: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Convert staves to minimal LilyPond notation
fn staves_to_minimal_lilypond(staves: &[Stave]) -> String {
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
fn staves_to_full_lilypond(staves: &[Stave]) -> String {
    let minimal = staves_to_minimal_lilypond(staves);
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

/// Convert staves to VexFlow SVG
fn staves_to_vexflow_svg(staves: &[Stave]) -> String {
    let mut svg = String::new();
    svg.push_str(r#"<svg width="600" height="200" xmlns="http://www.w3.org/2000/svg">"#);
    svg.push_str("\n");
    
    // Background
    svg.push_str("  <rect width=\"600\" height=\"200\" fill=\"#fafafa\" stroke=\"#333\" stroke-width=\"1\"/>");
    svg.push_str("\n");
    
    // Title
    svg.push_str("  <text x=\"20\" y=\"25\" font-family=\"serif\" font-size=\"16\" font-weight=\"bold\" fill=\"#333\">VexFlow-style Musical Notation</text>");
    svg.push_str("\n");
    
    // Staff lines
    let staff_y = 80;
    let staff_width = 500;
    let staff_x = 50;
    
    for i in 0..5 {
        let y = staff_y + i * 10;
        svg.push_str(&format!("  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#333\" stroke-width=\"1\"/>", 
            staff_x, y, staff_x + staff_width, y));
        svg.push_str("\n");
    }
    
    // Treble clef
    svg.push_str(&format!("  <text x=\"{}\" y=\"{}\" font-family=\"serif\" font-size=\"40\" fill=\"#333\">𝄞</text>", 
        staff_x + 10, staff_y + 25));
    svg.push_str("\n");
    
    // Time signature
    svg.push_str(&format!("  <text x=\"{}\" y=\"{}\" font-family=\"serif\" font-size=\"16\" fill=\"#333\">4</text>", 
        staff_x + 50, staff_y));
    svg.push_str("\n");
    svg.push_str(&format!("  <text x=\"{}\" y=\"{}\" font-family=\"serif\" font-size=\"16\" fill=\"#333\">4</text>", 
        staff_x + 50, staff_y + 20));
    svg.push_str("\n");
    
    // Notes
    let mut note_x = staff_x + 80;
    
    for stave in staves {
        for element in &stave.content_line.elements {
            match element {
                crate::document::model::MusicalElement::Note(note) => {
                    let note_y = match note.syllable.as_str() {
                        "1" => staff_y + 50,  // C below staff
                        "2" => staff_y + 45,  // D
                        "3" => staff_y + 40,  // E
                        "4" => staff_y + 35,  // F
                        "5" => staff_y + 30,  // G
                        "6" => staff_y + 25,  // A
                        "7" => staff_y + 20,  // B
                        "C" => staff_y + 50,  // C below staff
                        "D" => staff_y + 45,  // D
                        "E" => staff_y + 40,  // E
                        "F" => staff_y + 35,  // F
                        "G" => staff_y + 30,  // G
                        "A" => staff_y + 25,  // A
                        "B" => staff_y + 20,  // B
                        _ => staff_y + 20,    // Default to B
                    };
                    
                    // Note head
                    svg.push_str(&format!("  <ellipse cx=\"{}\" cy=\"{}\" rx=\"6\" ry=\"4\" fill=\"#333\"/>", 
                        note_x, note_y));
                    svg.push_str("\n");
                    
                    // Stem
                    svg.push_str(&format!("  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#333\" stroke-width=\"2\"/>", 
                        note_x + 6, note_y, note_x + 6, note_y - 25));
                    svg.push_str("\n");
                    
                    // Ledger line if needed (for C below staff)
                    if note_y >= staff_y + 45 {
                        svg.push_str(&format!("  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#333\" stroke-width=\"1\"/>", 
                            note_x - 8, staff_y + 50, note_x + 14, staff_y + 50));
                        svg.push_str("\n");
                    }
                    
                    // Slur indication
                    if note.in_slur {
                        svg.push_str(&format!("  <text x=\"{}\" y=\"{}\" font-family=\"serif\" font-size=\"10\" fill=\"#666\">slur</text>", 
                            note_x - 5, note_y - 30));
                        svg.push_str("\n");
                    }
                    
                    // Beat group indication
                    if note.in_beat_group {
                        svg.push_str(&format!("  <text x=\"{}\" y=\"{}\" font-family=\"serif\" font-size=\"10\" fill=\"#999\">beat</text>", 
                            note_x - 5, note_y + 20));
                        svg.push_str("\n");
                    }
                    
                    note_x += 60;
                }
                crate::document::model::MusicalElement::Barline { .. } => {
                    // Barline
                    svg.push_str(&format!("  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#333\" stroke-width=\"2\"/>", 
                        note_x - 10, staff_y, note_x - 10, staff_y + 40));
                    svg.push_str("\n");
                    note_x += 20;
                }
                _ => {}
            }
        }
    }
    
    // Footer
    svg.push_str("  <text x=\"20\" y=\"180\" font-family=\"serif\" font-size=\"10\" fill=\"#999\">Generated by Music Text Parser • VexFlow-style rendering</text>");
    svg.push_str("\n");
    
    svg.push_str("</svg>");
    svg
}

/// Convert staves to VexFlow JSON data
fn staves_to_vexflow_data(staves: &[Stave]) -> serde_json::Value {
    let mut notes = Vec::new();
    
    for stave in staves {
        for element in &stave.content_line.elements {
            if let crate::document::model::MusicalElement::Note(note) = element {
                let pitch = match note.syllable.as_str() {
                    "1" => "c/4",
                    "2" => "d/4",
                    "3" => "e/4", 
                    "4" => "f/4",
                    "5" => "g/4",
                    "6" => "a/4",
                    "7" => "b/4",
                    "C" => "c/4",
                    "D" => "d/4",
                    "E" => "e/4", 
                    "F" => "f/4",
                    "G" => "g/4",
                    "A" => "a/4",
                    "B" => "b/4",
                    _ => "c/4",
                };
                
                notes.push(serde_json::json!({
                    "keys": [pitch],
                    "duration": "q",
                    "syllable": note.syllable,
                    "in_slur": note.in_slur,
                    "in_beat_group": note.in_beat_group
                }));
            }
        }
    }
    
    serde_json::json!({
        "notes": notes,
        "time_signature": "4/4",
        "clef": "treble"
    })
}

/// Orchestrates the complete parsing pipeline
/// 
/// Input String → document_parser → stave_parser → converters → ProcessingResult
pub fn process_notation(input: &str) -> Result<ProcessingResult, String> {
    // Stage 1: Parse text into Document structure
    let parsed_document = parse_document(input)?;
    
    // Stage 2: Process document into staves
    let processed_staves = parse_document_staves(parsed_document.clone())?;
    
    // Stage 3: Convert to output formats
    let minimal_lilypond = staves_to_minimal_lilypond(&processed_staves);
    let full_lilypond = staves_to_full_lilypond(&processed_staves);
    let vexflow_svg = staves_to_vexflow_svg(&processed_staves);
    let vexflow_data = staves_to_vexflow_data(&processed_staves);
    
    Ok(ProcessingResult {
        original_input: input.to_string(),
        parsed_document,
        processed_staves,
        minimal_lilypond,
        full_lilypond,
        vexflow_svg,
        vexflow_data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_notation_single_stave() {
        let input = "|1 2 3";
        let result = process_notation(input).unwrap();
        
        assert_eq!(result.original_input, input);
        assert_eq!(result.parsed_document.staves.len(), 1);
        assert_eq!(result.processed_staves.len(), 1);
    }

    #[test]
    fn test_process_notation_multi_stave() {
        let input = "|1 2\n\n|3 4";
        let result = process_notation(input).unwrap();
        
        assert_eq!(result.parsed_document.staves.len(), 2);
        assert_eq!(result.processed_staves.len(), 2);
    }
}