/// Grammar rule: document_header = header_content blank_lines+

use std::collections::HashMap;
use crate::parse::{header_line, ParseError};

#[derive(Debug, Clone)]
pub struct DocumentHeader {
    pub title: Option<String>,
    pub author: Option<String>,
    pub directives: HashMap<String, String>,
}

pub fn parse(
    lines: &[&str],
    line_idx: &mut usize,
) -> Result<DocumentHeader, ParseError> {
    let mut title = None;
    let mut author = None;
    let mut directives = HashMap::new();

    // Parse header_content = header_line (newline header_line)*
    while *line_idx < lines.len() {
        let line = lines[*line_idx];

        // Check for blank line (ends header)
        if line.trim().is_empty() {
            break;
        }

        // Check for musical content line (starts header boundary)
        if is_musical_content_line(line) {
            break;
        }

        match header_line::parse(line) {
            header_line::HeaderLine::Title(title_line) => {
                title = Some(title_line.title);
                author = Some(title_line.author);
            }
            header_line::HeaderLine::Directive(directive) => {
                directives.insert(directive.key, directive.value);
            }
            header_line::HeaderLine::Text(text) => {
                // Text lines in header - could be continuation or title without author
                if title.is_none() && !text.content.trim().is_empty() {
                    title = Some(text.content.trim().to_string());
                }
            }
        }

        *line_idx += 1;
    }

    // Skip blank lines after header (blank_lines*)
    // Note: blank lines are optional - we just skip them if present
    while *line_idx < lines.len() && lines[*line_idx].trim().is_empty() {
        *line_idx += 1;
    }

    // No requirement for blank lines after header - the grammar allows
    // headers to be followed immediately by content

    Ok(DocumentHeader {
        title,
        author,
        directives,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_document_header() {
        let input = vec![
            "        Amazing Grace        Bach",
            "Author: John Newton",
            "Tempo: 120",
            "",
            "|1 2 3 4|",
        ];

        let mut line_idx = 0;
        let result = parse(&input, &mut line_idx);
        assert!(result.is_ok());

        let header = result.unwrap();
        assert_eq!(header.title, Some("Amazing Grace".to_string()));
        assert_eq!(header.author, Some("Bach".to_string()));
        assert_eq!(header.directives.get("Author"), Some(&"John Newton".to_string()));
        assert_eq!(header.directives.get("Tempo"), Some(&"120".to_string()));
        assert_eq!(line_idx, 4); // Should be positioned at the content line
    }

    #[test]
    fn test_parse_no_header() {
        let input = vec!["|1 2 3 4|"];
        let mut line_idx = 0;
        let result = parse(&input, &mut line_idx);
        assert!(result.is_ok());

        let header = result.unwrap();
        assert_eq!(header.title, None);
        assert_eq!(header.author, None);
        assert!(header.directives.is_empty());
        assert_eq!(line_idx, 0); // Should stay at beginning
    }
}

/// Check if a line looks like musical content (not header content)
fn is_musical_content_line(line: &str) -> bool {
    let trimmed = line.trim();

    // Musical content lines typically start with:
    // - Barlines: |, ||, |:, :|, |]
    // - Numbers: 1, 2, 3, etc. (at start of line for line numbers)
    // - Musical notes without proper title formatting

    if trimmed.starts_with('|') {
        return true;
    }

    // Check for musical notes
    if let Some(first_char) = trimmed.chars().next() {
        // Single musical notes or sequences
        if matches!(first_char, '1'..='7' | 'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N' |
                                's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n' |
                                'A'..='G' | 'a'..='g') {
            // It's a musical note - check if it's a simple title or actual music
            // Simple single letters/numbers without proper title spacing are likely music
            if trimmed.chars().count() <= 5 || // Single note like "1", "S", "C#", "1234♯"
               trimmed.contains(' ') || // Sequence like "1 2 3"
               trimmed.contains('-') || // Extended notes like "1--"
               trimmed.contains('|') || // Contains barlines
               trimmed.contains('\\') || // Contains backslash (escape sequences)
               trimmed.contains('.') && trimmed.len() <= 10 { // Musical notation with octave markers
                return true;
            }
        }

        // Check for line numbers (digits followed by .)
        if first_char.is_ascii_digit() {
            if trimmed.contains('.') && trimmed.len() <= 4 { // Like "1." or "12."
                return true; // Line number
            }
        }
    }

    // Check for lines that contain typical musical notation patterns
    if trimmed.contains('\\') || // Backslash escape sequences
       (trimmed.contains('.') && trimmed.chars().any(|c| matches!(c, '1'..='7' | 'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N' | 's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n' | 'A'..='G' | 'a'..='g'))) {
        return true;
    }

    false
}