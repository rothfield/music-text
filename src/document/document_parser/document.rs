use crate::document::model::{Document, Directive, Stave, Source, Position};
use super::error::ParseError;
use super::stave::parse_stave_from_paragraph;

/// Result of parsing a paragraph
#[derive(Debug)]
enum ParagraphContent {
    Directives(Vec<Directive>),
    Stave(Stave),
}

/// Hand-written recursive descent parser for music notation
pub fn parse_document(input: &str) -> Result<Document, ParseError> {
    if input.trim().is_empty() {
        return Ok(Document {
            directives: Vec::new(),
            staves: Vec::new(),
            source: Source {
                value: input.to_string(),
                position: Position { line: 1, column: 1 },
            },
        });
    }

    // Split into paragraphs by blank lines
    let paragraphs = split_into_paragraphs(input);
    let mut directives = Vec::new();
    let mut staves = Vec::new();

    for (para_index, paragraph) in paragraphs.iter().enumerate() {
        if !paragraph.trim().is_empty() {
            match parse_paragraph(paragraph, para_index + 1) {
                Ok(ParagraphContent::Directives(mut paragraph_directives)) => {
                    directives.append(&mut paragraph_directives);
                }
                Ok(ParagraphContent::Stave(stave)) => {
                    staves.push(stave);
                }
                Err(e) => return Err(e),
            }
        }
    }

    // Allow documents with only directives (no staves required)
    Ok(Document {
        directives,
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

/// Parse a paragraph using functional parser chain
fn parse_paragraph(paragraph: &str, line_number: usize) -> Result<ParagraphContent, ParseError> {
    try_parse_directives(paragraph, line_number)
        .or_else(|_| try_parse_stave(paragraph, line_number))
}

/// Try to parse paragraph as directives (single or multi-line)
fn try_parse_directives(paragraph: &str, line_number: usize) -> Result<ParagraphContent, ParseError> {
    let lines: Vec<&str> = paragraph.lines().collect();
    let mut directives = Vec::new();
    
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            match parse_single_directive(trimmed, line_number + i) {
                Ok(directive) => directives.push(directive),
                Err(_) => {
                    // If any line fails as directive, whole paragraph fails as directives
                    return Err(ParseError {
                        message: "Not a directive paragraph".to_string(),
                        line: line_number + i,
                        column: 1,
                    });
                }
            }
        }
    }
    
    if directives.is_empty() {
        Err(ParseError {
            message: "No directives found".to_string(),
            line: line_number,
            column: 1,
        })
    } else {
        Ok(ParagraphContent::Directives(directives))
    }
}

/// Try to parse paragraph as a stave
fn try_parse_stave(paragraph: &str, line_number: usize) -> Result<ParagraphContent, ParseError> {
    parse_stave_from_paragraph(paragraph, line_number)
        .map(ParagraphContent::Stave)
}

/// Parse a single directive line in the format "key:value" or "key: value"
fn parse_single_directive(line: &str, line_number: usize) -> Result<Directive, ParseError> {
    // Look for key:value pattern
    if let Some(colon_pos) = line.find(':') {
        let key = line[..colon_pos].trim().to_string();
        let value = line[colon_pos + 1..].trim().to_string();
        
        // Validate key is not empty and doesn't contain musical content
        if key.is_empty() {
            return Err(ParseError {
                message: "Directive key cannot be empty".to_string(),
                line: line_number,
                column: 1,
            });
        }
        
        // Reject if key contains barlines or looks like musical content
        if key.contains('|') || is_likely_musical_content(&key) {
            return Err(ParseError {
                message: "Not a valid directive".to_string(),
                line: line_number,
                column: 1,
            });
        }
        
        Ok(Directive {
            key,
            value,
            source: Source {
                value: line.to_string(),
                position: Position { line: line_number, column: 1 },
            },
        })
    } else {
        Err(ParseError {
            message: "Directive must contain colon (:)".to_string(),
            line: line_number,
            column: 1,
        })
    }
}

/// Check if a string looks like musical content
fn is_likely_musical_content(s: &str) -> bool {
    // Check for obvious musical patterns
    if s.contains('|') {  // Barlines are strong musical indicators
        return true;
    }
    
    // Count distinctive musical patterns
    let musical_patterns: usize = s.split_whitespace()
        .filter(|word| {
            // Look for sequences of musical notes
            word.len() >= 2 && 
            word.chars().all(|c| matches!(c, 
                '1'..='7' |                                    // Numbers
                'C' | 'D' | 'E' | 'F' | 'G' | 'A' | 'B' |     // Western
                'S' | 'R' | 'M' | 'P' | 'N' |                 // Sargam uppercase
                's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n' |     // Sargam lowercase
                '-' | '#' | 'b'                                // Musical symbols
            ))
        })
        .count();
    
    // If multiple musical pattern words, likely musical content
    musical_patterns >= 2
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