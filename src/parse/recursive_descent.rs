use crate::parse::model::{Document, Stave, TextLine, WhitespaceLine, Source, NotationSystem, Position as ModelPosition};
use crate::rhythm::types::{ParsedElement, Degree, Position};

/// Parse error for recursive descent parser
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Parse error at line {}, column {}: {}", self.line, self.column, self.message)
    }
}

impl std::error::Error for ParseError {}

impl From<ParseError> for String {
    fn from(error: ParseError) -> Self {
        error.to_string()
    }
}

/// Parse document according to formal grammar:
/// document = blank_lines* (stave (blank_lines stave)*)? blank_lines?
pub fn parse_document(input: &str) -> Result<Document, ParseError> {
    let mut chars = input.chars().peekable();
    let mut line = 1;
    let mut column = 1;
    let mut elements = Vec::new();

    // Parse optional leading blank_lines*
    while is_blank_lines_start(&mut chars.clone()) {
        let blank_lines = parse_blank_lines_element(&mut chars, &mut line, &mut column)?;
        elements.push(crate::parse::model::DocumentElement::BlankLines(blank_lines));
    }

    // Parse optional (stave (blank_lines stave)*)?
    if chars.peek().is_some() {
        // We have content that might be a stave
        let first_stave = parse_stave_from_chars(&mut chars, &mut line, &mut column)?;
        elements.push(crate::parse::model::DocumentElement::Stave(first_stave));

        // Parse (blank_lines stave)*
        while chars.peek().is_some() {
            // Check if we have blank_lines
            if is_blank_lines_start(&mut chars.clone()) {
                let blank_lines = parse_blank_lines_element(&mut chars, &mut line, &mut column)?;
                elements.push(crate::parse::model::DocumentElement::BlankLines(blank_lines));

                // After blank_lines, there might be another stave
                if chars.peek().is_some() {
                    let stave = parse_stave_from_chars(&mut chars, &mut line, &mut column)?;
                    elements.push(crate::parse::model::DocumentElement::Stave(stave));
                }
            } else {
                // No blank_lines, so this should be continuation of current stave
                // This shouldn't happen with proper stave parsing, but let's handle it
                break;
            }
        }
    }

    Ok(Document {
        directives: Vec::new(),
        elements,
        source: Source {
            value: Some(input.to_string()),
            position: ModelPosition { line: 1, column: 1 },
        },
    })
}

/// Check if current position starts blank_lines (newline (whitespace* newline)+)
fn is_blank_lines_start(chars: &mut std::iter::Peekable<std::str::Chars>) -> bool {
    matches!(chars.peek(), Some(&'\n'))
}

/// Parse blank_lines (newline (whitespace* newline)+) and return BlankLines element
fn parse_blank_lines_element(chars: &mut std::iter::Peekable<std::str::Chars>, line: &mut usize, column: &mut usize) -> Result<crate::parse::model::BlankLines, ParseError> {
    let mut content = String::new();
    let start_line = *line;
    let start_column = *column;

    // First newline
    if let Some(&'\n') = chars.peek() {
        chars.next();
        content.push('\n');
        *line += 1;
        *column = 1;
    } else {
        return Err(ParseError {
            message: "Expected newline for blank_lines".to_string(),
            line: *line,
            column: *column,
        });
    }

    // (whitespace* newline)+
    while chars.peek().is_some() {
        // Collect whitespace*
        while let Some(&ch) = chars.peek() {
            if ch == ' ' {
                chars.next();
                content.push(ch);
                *column += 1;
            } else {
                break;
            }
        }

        // Must have newline
        if let Some(&'\n') = chars.peek() {
            chars.next();
            content.push('\n');
            *line += 1;
            *column = 1;
        } else {
            break; // End of blank_lines
        }
    }

    let source_value = content.clone();
    Ok(crate::parse::model::BlankLines {
        content,
        source: Source {
            value: Some(source_value),
            position: ModelPosition { line: start_line, column: start_column },
        },
    })
}

/// Parse stave from character stream following grammar:
/// stave = upper_line* content_line (lower_line | lyrics_line)* (blank_lines | (whitespace* newline)* EOI)
fn parse_stave_from_chars(chars: &mut std::iter::Peekable<std::str::Chars>, line: &mut usize, column: &mut usize) -> Result<Stave, ParseError> {
    let start_line = *line;
    let mut lines = Vec::new();
    let mut stave_content = String::new();

    // Parse upper_line*
    while let Some(&ch) = chars.peek() {
        // Stop if we hit blank_lines (stave boundary)
        if is_blank_lines_start(&mut chars.clone()) {
            break;
        }

        // Collect the current line
        let mut current_line = String::new();
        while let Some(&ch) = chars.peek() {
            if ch == '\n' {
                chars.next(); // consume newline
                current_line.push(ch);
                stave_content.push(ch);
                *line += 1;
                *column = 1;
                break;
            } else {
                let consumed_char = chars.next().unwrap();
                current_line.push(consumed_char);
                stave_content.push(consumed_char);
                *column += 1;
            }
        }

        // Check precedence: upper lines should be checked before content lines
        // because lines like ".123" could match both but should be treated as upper lines
        if is_upper_line(&current_line.trim_end_matches('\n')) {
            // Parse as upper line using our upper_line parser
            if let Ok(parsed_upper) = crate::parse::upper_line_parser::parse_upper_line(&current_line, *line - 1) {
                lines.push(crate::parse::model::StaveLine::Upper(parsed_upper));
            } else {
                // Fall back to text line
                let text_line = TextLine {
                    content: current_line.trim_end_matches('\n').to_string(),
                    source: Source {
                        value: Some(current_line.trim_end_matches('\n').to_string()),
                        position: ModelPosition { line: *line - 1, column: 1 },
                    },
                };
                lines.push(crate::parse::model::StaveLine::Text(text_line));
            }
        } else if is_content_line(&current_line.trim_end_matches('\n')) {
            // Detect notation system from the content line
            let notation_system = detect_notation_system(&current_line);
            // Parse the content line using dedicated parser with notation system
            let content_line_elements = crate::parse::content_line_parser::parse_content_line_with_system(&current_line, *line, notation_system)?;
            lines.push(crate::parse::model::StaveLine::Content(content_line_elements));
            break; // Exit after finding content_line
        } else {
            // Treat as text line
            let text_line = TextLine {
                content: current_line.trim_end_matches('\n').to_string(),
                source: Source {
                    value: Some(current_line.trim_end_matches('\n').to_string()),
                    position: ModelPosition { line: *line - 1, column: 1 },
                },
            };
            lines.push(crate::parse::model::StaveLine::Text(text_line));
        }
    }

    // Parse (lower_line | lyrics_line)* after content_line
    while let Some(&ch) = chars.peek() {
        // Stop if we hit blank_lines or EOI - but we'll consume blank_lines as stave termination
        if is_blank_lines_start(&mut chars.clone()) {
            break;
        }

        // Collect the current line
        let mut current_line = String::new();
        while let Some(&ch) = chars.peek() {
            if ch == '\n' {
                chars.next(); // consume newline
                current_line.push(ch);
                stave_content.push(ch);
                *line += 1;
                *column = 1;
                break;
            } else {
                let consumed_char = chars.next().unwrap();
                current_line.push(consumed_char);
                stave_content.push(consumed_char);
                *column += 1;
            }
        }

        if current_line.trim().is_empty() {
            // Handle whitespace* newline pattern - if followed by another newline, it's blank_lines
            if is_blank_lines_start(&mut chars.clone()) {
                break;
            }
            // Otherwise, it's a whitespace line that should be captured as part of the stave
            // Create ParsedElements like Content lines do (consistent with Pattern #1)
            let line_without_newline = current_line.trim_end_matches('\n');
            let mut elements = Vec::new();
            let mut col_position = 0;

            // Parse whitespace characters into ParsedElements
            for ch in line_without_newline.chars() {
                if ch.is_whitespace() {
                    elements.push(ParsedElement::Whitespace {
                        value: ch.to_string(),
                        position: Position { row: *line, col: col_position },
                    });
                } else {
                    // Non-whitespace in what we thought was a whitespace line - treat as unknown
                    elements.push(ParsedElement::Unknown {
                        value: ch.to_string(),
                        position: Position { row: *line, col: col_position },
                    });
                }
                col_position += 1;
            }

            // Add newline element if the original line had one
            if current_line.ends_with('\n') {
                elements.push(ParsedElement::Newline {
                    value: "\n".to_string(),
                    position: Position { row: *line, col: col_position },
                });
            }

            let whitespace_line = WhitespaceLine {
                elements,
                source: Source {
                    value: Some(current_line.clone()),
                    position: ModelPosition { line: *line, column: 1 },
                },
            };
            lines.push(crate::parse::model::StaveLine::Whitespace(whitespace_line));
        } else if is_lower_line(&current_line.trim_end_matches('\n')) {
            // Parse as lower line using our lower_line parser (include newline)
            if let Ok(parsed_lower) = crate::parse::lower_line_parser::parse_lower_line(&current_line, *line - 1) {
                lines.push(crate::parse::model::StaveLine::Lower(parsed_lower));
            } else {
                // Fall back to text line
                let text_line = TextLine {
                    content: current_line.trim_end_matches('\n').to_string(),
                    source: Source {
                        value: Some(current_line.trim_end_matches('\n').to_string()),
                        position: ModelPosition { line: *line - 1, column: 1 },
                    },
                };
                lines.push(crate::parse::model::StaveLine::Text(text_line));
            }
        } else {
            // Treat as lyrics/text line
            let text_line = TextLine {
                content: current_line.trim_end_matches('\n').to_string(),
                source: Source {
                    value: Some(current_line.trim_end_matches('\n').to_string()),
                    position: ModelPosition { line: *line - 1, column: 1 },
                },
            };
            lines.push(crate::parse::model::StaveLine::Text(text_line));
        }
    }


    if lines.is_empty() {
        return Err(ParseError {
            message: "Empty stave".to_string(),
            line: *line,
            column: *column,
        });
    }

    Ok(Stave {
        lines,
        rhythm_items: None,
        notation_system: detect_notation_system(&stave_content),
        source: Source {
            value: Some(stave_content),
            position: ModelPosition { line: start_line, column: 1 },
        },
    })
}

// Helper functions for line classification (used by parse_stave_from_chars)

/// Check if a line is a content line (has barline or musical elements)
fn is_content_line(line: &str) -> bool {
    line.contains('|') || line.chars().any(|c| matches!(c, '1'..='7' | 'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N' | 's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n'))
}

/// Check if a line is an upper line (has upper line elements)
fn is_upper_line(line: &str) -> bool {
    // Contains octave markers, slurs, ornaments, mordents, etc.
    line.contains('.') || line.contains(':') || line.contains('*') || line.contains('_') || line.contains('~')
}

/// Check if a line is a lower line (has lower line elements)
fn is_lower_line(line: &str) -> bool {
    // Contains lower octave markers, beat groups, or syllables
    line.contains('.') || line.contains(':') || line.contains("__") ||
    line.split_whitespace().any(|word| word.chars().all(|c| c.is_alphabetic() || c == '-' || c == '\''))
}


fn detect_notation_system(input: &str) -> NotationSystem {
    if input.chars().any(|c| matches!(c, 'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N' | 's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n')) {
        NotationSystem::Sargam
    } else if input.chars().any(|c| matches!(c, '1'..='7')) {
        NotationSystem::Number
    } else {
        NotationSystem::Western
    }
}