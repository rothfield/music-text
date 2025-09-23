/// Try to parse header lines from the current position
/// Returns None if the current line cannot be parsed as a header line

use std::collections::HashMap;
use crate::parse::{header_line};

#[derive(Debug, Clone)]
pub struct DocumentHeader {
    pub title: Option<String>,
    pub author: Option<String>,
    pub directives: HashMap<String, String>,
}

/// Try to parse one or more header lines
/// Returns None if the current line is not a valid header line
pub fn try_parse(
    lines: &[&str],
    line_idx: &mut usize,
) -> Option<DocumentHeader> {
    if *line_idx >= lines.len() {
        return None;
    }

    let start_idx = *line_idx;
    let mut title = None;
    let mut author = None;
    let mut directives = HashMap::new();
    let mut parsed_any = false;

    // Try to parse header lines until we can't anymore
    while *line_idx < lines.len() {
        let line = lines[*line_idx];

        // Blank line ends header block
        if line.trim().is_empty() {
            break;
        }

        // Try to parse as a header line
        match header_line::parse(line) {
            header_line::HeaderLine::Title(title_line) => {
                title = Some(title_line.title);
                author = Some(title_line.author);
                parsed_any = true;
            }
            header_line::HeaderLine::Directive(directive) => {
                directives.insert(directive.key, directive.value);
                parsed_any = true;
            }
            header_line::HeaderLine::Text(text) => {
                // Check if this looks like musical content
                let empty_string = String::new();
                let text_content = text.value.as_ref().unwrap_or(&empty_string);
                let trimmed = text_content.trim();

                // Musical content indicators:
                // - Starts with barline |
                // - Is just numbers/notes like "123" or "SRG"
                // - Contains musical symbols
                if trimmed.starts_with('|') ||
                   trimmed.starts_with(':') ||
                   trimmed.starts_with('[') ||
                   trimmed.starts_with('(') ||
                   (trimmed.len() <= 10 && trimmed.chars().all(|c|
                       matches!(c, '1'..='7' | 'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N' |
                                  's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n' |
                                  'A'..='G' | 'a'..='g' | ' ' | '-' | '.' | '\\' | '#' | 'b'))) {
                    // This looks like musical content, not a header
                    *line_idx = start_idx;
                    return None;
                }

                // Only accept text lines if we've already parsed header content
                // Otherwise, it might be musical content
                if parsed_any {
                    if title.is_none() && !text_content.trim().is_empty() {
                        title = Some(text_content.trim().to_string());
                    }
                } else {
                    // First line that's not obviously musical - could be a title
                    // But be conservative - if it doesn't look like a title, reject it
                    if trimmed.len() > 2 && !trimmed.chars().all(|c| c.is_ascii_digit()) {
                        title = Some(text_content.trim().to_string());
                        parsed_any = true;
                    } else {
                        *line_idx = start_idx;
                        return None;
                    }
                }
            }
        }

        *line_idx += 1;
    }

    // If we didn't parse anything, restore position and return None
    if !parsed_any {
        *line_idx = start_idx;
        return None;
    }

    Some(DocumentHeader {
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
        let result = try_parse(&input, &mut line_idx);
        assert!(result.is_some());

        let header = result.unwrap();
        assert_eq!(header.title, Some("Amazing Grace".to_string()));
        assert_eq!(header.author, Some("Bach".to_string()));
        assert_eq!(header.directives.get("Author"), Some(&"John Newton".to_string()));
        assert_eq!(header.directives.get("Tempo"), Some(&"120".to_string()));
        assert_eq!(line_idx, 3); // Should be positioned at the blank line
    }

    #[test]
    fn test_parse_no_header() {
        let input = vec!["|1 2 3 4|"];
        let mut line_idx = 0;
        let result = try_parse(&input, &mut line_idx);
        assert!(result.is_none()); // Should return None for musical content
        assert_eq!(line_idx, 0); // Should stay at beginning
    }
}