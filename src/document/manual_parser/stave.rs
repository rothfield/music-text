use crate::document::model::{Stave, TextLine, NotationSystem, Source, Position};
use super::error::ParseError;
use super::content_line::{parse_content_line, is_content_line};
use super::underline::is_underscore_line;

/// Parse a single paragraph into a Stave
/// Pattern: aaaXaaa, aaaX, Xaaa, X where a=text line, X=content line
pub fn parse_stave_from_paragraph(paragraph: &str, start_line: usize) -> Result<Stave, ParseError> {
    let lines: Vec<&str> = paragraph.lines().collect();
    if lines.is_empty() {
        return Err(ParseError {
            message: "Empty paragraph".to_string(),
            line: start_line,
            column: 1,
        });
    }

    // Find the content line (musical notation)
    let mut content_line_index = None;
    for (i, line) in lines.iter().enumerate() {
        if is_content_line(line) {
            if content_line_index.is_some() {
                return Err(ParseError {
                    message: "Multiple content lines found in stave - only one allowed".to_string(),
                    line: start_line + i,
                    column: 1,
                });
            }
            content_line_index = Some(i);
        }
    }

    let content_index = match content_line_index {
        Some(i) => i,
        None => {
            return Err(ParseError {
                message: "No musical content line found in stave".to_string(),
                line: start_line,
                column: 1,
            });
        }
    };

    // Split into before/content/after
    let text_lines_before: Vec<TextLine> = lines[..content_index]
        .iter()
        .enumerate()
        .map(|(i, line)| TextLine {
            content: line.to_string(),
            source: Source {
                value: line.to_string(),
                position: Position {
                    line: start_line + i,
                    column: 1,
                },
            },
        })
        .collect();

    let text_lines_after: Vec<TextLine> = lines[content_index + 1..]
        .iter()
        .enumerate()
        .map(|(i, line)| TextLine {
            content: line.to_string(),
            source: Source {
                value: line.to_string(),
                position: Position {
                    line: start_line + content_index + 1 + i,
                    column: 1,
                },
            },
        })
        .collect();

    let content_line_text = lines[content_index];
    let content_line = parse_content_line(content_line_text, start_line + content_index)?;

    // Detect multi-stave markers
    let begin_multi_stave = text_lines_before
        .first()
        .map(|line| is_underscore_line(&line.content))
        .unwrap_or(false);

    // Check if ANY line after the content line is an underscore line
    let end_multi_stave = text_lines_after
        .iter()
        .any(|line| is_underscore_line(&line.content));

    Ok(Stave {
        text_lines_before,
        content_line,
        text_lines_after,
        notation_system: NotationSystem::Number, // Default for now
        source: Source {
            value: paragraph.to_string(),
            position: Position {
                line: start_line,
                column: 1,
            },
        },
        begin_multi_stave,
        end_multi_stave,
    })
}