use crate::parse::model::{ContentLine, ContentElement, Source, Position, NotationSystem};
use crate::parse::beat::parse_beat;
use crate::parse::pitch::is_pitch_start;
use crate::parse::ParseError;
use crate::rhythm::converters::BarlineType;
use std::str::FromStr;

/// Parse content line according to grammar:
/// content_line = line_number? non-beat-element* beat (non-beat-element | beat)* newline
/// non-beat-element = barline | whitespace
pub fn parse_content_line(
    input: &str,
    line_num: usize,
    notation_system: NotationSystem,
    line_start_doc_index: usize,
) -> Result<ContentLine, ParseError> {
    let mut elements = Vec::new();
    let mut chars = input.chars().peekable();
    let mut column = 1;
    let mut index_in_line = 0;

    // Skip line number if present (e.g., "1. ")
    // Only treat as line number if digits are followed by a dot
    let mut temp_chars = chars.clone();
    let mut has_dot_after_digits = false;

    // Check if we have digits followed by a dot
    if temp_chars.peek().map_or(false, |c| c.is_ascii_digit()) {
        // Skip digits in temp iterator
        while temp_chars.peek().map_or(false, |c| c.is_ascii_digit()) {
            temp_chars.next();
        }
        // Check if next character is a dot
        if temp_chars.peek() == Some(&'.') {
            has_dot_after_digits = true;
        }
    }

    if has_dot_after_digits {
        // Skip digits
        while chars.peek().map_or(false, |c| c.is_ascii_digit()) {
            chars.next();
            column += 1;
            index_in_line += 1;
        }
        // Skip the dot
        if chars.peek() == Some(&'.') {
            chars.next();
            column += 1;
            index_in_line += 1;
            // Skip spaces after line number
            while chars.peek() == Some(&' ') {
                chars.next();
                column += 1;
                index_in_line += 1;
            }
        }
    }

    // Parse content line elements
    while let Some(&ch) = chars.peek() {
        match ch {
            '\n' => {
                // End of content line
                break;
            }

            '|' => {
                // Parse barline
                chars.next();
                let mut barline_str = String::from("|");
                column += 1;
                index_in_line += 1;

                // Check for multi-character barlines (||, |:, :|, |])
                while let Some(&next_ch) = chars.peek() {
                    if matches!(next_ch, '|' | ':' | ']') {
                        barline_str.push(next_ch);
                        chars.next();
                        column += 1;
                        index_in_line += 1;
                    } else if barline_str == ":" && next_ch == '|' {
                        // Handle :| barline
                        barline_str.push(next_ch);
                        chars.next();
                        column += 1;
                        index_in_line += 1;
                        break;
                    } else {
                        break;
                    }
                }

                // Convert string to BarlineType
                let barline_type = BarlineType::from_str(&barline_str)
                    .map_err(|_| ParseError {
                        message: format!("Invalid barline: {}", barline_str),
                        line: line_num,
                        column: column - barline_str.len(),
                    })?;

                elements.push(ContentElement::Barline(barline_type));
            }

            ' ' => {
                // Parse whitespace
                let start_col = column;
                let mut space_count = 0;

                while chars.peek() == Some(&' ') {
                    chars.next();
                    column += 1;
                    index_in_line += 1;
                    space_count += 1;
                }

                elements.push(ContentElement::Whitespace(" ".repeat(space_count)));
            }

            '-' => {
                // Parse beat starting with dash
                let (beat, consumed) = parse_beat(
                    &mut chars,
                    notation_system,
                    line_num,
                    column,
                    index_in_line,
                    line_start_doc_index + index_in_line,
                )?;

                column += consumed;
                index_in_line += consumed;
                elements.push(ContentElement::Beat(beat));
            }

            ch if is_pitch_start(ch, notation_system) => {
                // Parse beat
                let (beat, consumed) = parse_beat(
                    &mut chars,
                    notation_system,
                    line_num,
                    column,
                    index_in_line,
                    line_start_doc_index + index_in_line,
                )?;

                column += consumed;
                index_in_line += consumed;
                elements.push(ContentElement::Beat(beat));
            }

            _ => {
                // Unknown character - skip it
                chars.next();
                column += 1;
                index_in_line += 1;
            }
        }
    }

    Ok(ContentLine {
        elements,
        source: Source {
            value: Some(input.to_string()),
            position: Position {
                line: line_num,
                column: 1,
                index_in_line: 0,
                index_in_doc: line_start_doc_index,
            },
        },
    })
}