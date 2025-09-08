use crate::document::model::{Document, Source, Position};
use super::error::ParseError;
use super::stave::parse_stave_from_paragraph;

/// Hand-written recursive descent parser for music notation
pub fn parse_document(input: &str) -> Result<Document, ParseError> {
    if input.trim().is_empty() {
        return Ok(Document {
            staves: Vec::new(),
            source: Source {
                value: input.to_string(),
                position: Position { line: 1, column: 1 },
            },
        });
    }

    // Split into paragraphs by blank lines
    let paragraphs = split_into_paragraphs(input);
    let mut staves = Vec::new();

    for (para_index, paragraph) in paragraphs.iter().enumerate() {
        if !paragraph.trim().is_empty() {
            match parse_stave_from_paragraph(paragraph, para_index + 1) {
                Ok(stave) => staves.push(stave),
                Err(e) => return Err(e),
            }
        }
    }

    if staves.is_empty() {
        return Err(ParseError {
            message: "No staves found in document".to_string(),
            line: 1,
            column: 1,
        });
    }

    Ok(Document {
        staves,
        source: Source {
            value: input.to_string(),
            position: Position { line: 1, column: 1 },
        },
    })
}

/// Split input into paragraphs separated by blank lines
fn split_into_paragraphs(input: &str) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut current_paragraph = String::new();
    
    for line in input.lines() {
        if line.trim().is_empty() {
            // Blank line - end current paragraph
            if !current_paragraph.trim().is_empty() {
                paragraphs.push(current_paragraph.trim().to_string());
                current_paragraph.clear();
            }
        } else {
            // Non-blank line - add to current paragraph
            if !current_paragraph.is_empty() {
                current_paragraph.push('\n');
            }
            current_paragraph.push_str(line);
        }
    }
    
    // Don't forget the last paragraph
    if !current_paragraph.trim().is_empty() {
        paragraphs.push(current_paragraph.trim().to_string());
    }
    
    paragraphs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_paragraphs() {
        let input = "line1\nline2\n\nline3\nline4\n\n\nline5";
        let paragraphs = split_into_paragraphs(input);
        assert_eq!(paragraphs, vec!["line1\nline2", "line3\nline4", "line5"]);
    }
}