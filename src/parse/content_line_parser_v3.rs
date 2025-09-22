use crate::parse::model::{ContentLine, ContentElement, Attributes, Position, NotationSystem};
use crate::parse::beat::parse_beat;
use crate::parse::pitch::is_pitch_start;
use crate::parse::ParseError;
use crate::rhythm::converters::BarlineType;
use std::str::{FromStr, CharIndices};
use std::iter::Peekable;

/// Helper function to calculate column from position in input
fn column_from_pos(input: &str, pos: usize) -> usize {
    input[..pos].chars().rev().take_while(|&c| c != '\n').count() + 1
}

/// Helper function to calculate index in line from position
fn index_in_line_from_pos(input: &str, pos: usize, _line_num: usize) -> usize {
    input[..pos].chars().rev().take_while(|&c| c != '\n').count()
}

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
    let mut chars = input.char_indices().peekable();

    // Skip line number if present (e.g., "1. ")
    // Only treat as line number if digits are followed by a dot
    let mut temp_chars = chars.clone();
    let mut has_dot_after_digits = false;

    // Check if we have digits followed by a dot
    if temp_chars.peek().map_or(false, |(_, c)| c.is_ascii_digit()) {
        // Skip digits in temp iterator
        while temp_chars.peek().map_or(false, |(_, c)| c.is_ascii_digit()) {
            temp_chars.next();
        }
        // Check if next character is a dot
        if temp_chars.peek().map_or(false, |(_, c)| *c == '.') {
            has_dot_after_digits = true;
        }
    }

    if has_dot_after_digits {
        // Skip digits
        while chars.peek().map_or(false, |(_, c)| c.is_ascii_digit()) {
            chars.next();
        }
        // Skip the dot
        if chars.peek().map_or(false, |(_, c)| *c == '.') {
            chars.next();
            // Skip spaces after line number
            while chars.peek().map_or(false, |(_, c)| *c == ' ') {
                chars.next();
            }
        }
    }

    // Parse content line elements
    while let Some(&(pos, ch)) = chars.peek() {
        match ch {
            '\n' => {
                // End of content line
                break;
            }

            '|' | ':' => {
                // Parse barline using tokenization
                let barline = parse_barline(
                    &mut chars,
                    ch,
                    pos,
                    line_num,
                    input,
                    line_start_doc_index,
                )?;
                elements.push(ContentElement::Barline(barline));
            }

            ' ' => {
                // Parse whitespace
                let start_pos = pos;
                let mut space_count = 0;

                while chars.peek().map_or(false, |(_, c)| *c == ' ') {
                    chars.next();
                    space_count += 1;
                }

                let whitespace_content = " ".repeat(space_count);
                elements.push(ContentElement::Whitespace(crate::parse::model::Whitespace {
                    content: whitespace_content.clone(),
                    source: Attributes {
                            slur_start: false,
                            slur_char_length: None,
                        value: Some(whitespace_content),
                        position: Position {
                            line: line_num,
                            column: column_from_pos(input, start_pos),
                            index_in_line: index_in_line_from_pos(input, start_pos, line_num),
                            index_in_doc: line_start_doc_index + start_pos,
                        },
                    },
                }));
            }

            '-' => {
                // Parse beat starting with dash
                let beat = parse_beat(
                    &mut chars,
                    notation_system,
                    line_num,
                    input,
                    line_start_doc_index,
                )?;

                elements.push(ContentElement::Beat(beat));
            }

            ch if is_pitch_start(ch, notation_system) => {
                // Parse beat
                let beat = parse_beat(
                    &mut chars,
                    notation_system,
                    line_num,
                    input,
                    line_start_doc_index,
                )?;

                elements.push(ContentElement::Beat(beat));
            }

            _ => {
                // Unknown character - skip it
                chars.next();
            }
        }
    }

    Ok(ContentLine {
        elements,
        source: Attributes {
                            slur_start: false,
                            slur_char_length: None,
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

/// Parse barline using recursive descent tokenization
/// Grammar: barline = '|' ( '|' | ':' | '.' | ':|' )? | ':' '|' ( ':' )?
fn parse_barline(
    chars: &mut Peekable<CharIndices>,
    first_char: char,
    start_pos: usize,
    line_num: usize,
    input: &str,
    line_start_doc_index: usize,
) -> Result<crate::parse::model::Barline, ParseError> {
    let mut barline_str = String::new();

    // Handle first character
    if first_char == '|' {
        chars.next(); // consume '|'
        barline_str.push('|');

        // Look for second character
        if let Some(&(_, ch)) = chars.peek() {
            match ch {
                '|' => {
                    // || - double barline
                    barline_str.push('|');
                    chars.next();
                }
                ':' => {
                    // |: or |:|
                    barline_str.push(':');
                    chars.next();
                    // Check for |:|
                    if let Some(&(_, '|')) = chars.peek() {
                        barline_str.push('|');
                        chars.next();
                    }
                }
                '.' => {
                    // |. - final barline
                    barline_str.push('.');
                    chars.next();
                }
                _ => {
                    // Just single |
                }
            }
        }
    } else if first_char == ':' {
        chars.next(); // consume ':'
        barline_str.push(':');

        // Must be followed by |
        if let Some(&(_, '|')) = chars.peek() {
            barline_str.push('|');
            chars.next();

            // Check for :|:
            if let Some(&(_, ':')) = chars.peek() {
                barline_str.push(':');
                chars.next();
            }
        } else {
            return Err(ParseError {
                message: "Expected '|' after ':' in barline".to_string(),
                line: line_num,
                column: column_from_pos(input, start_pos + 1),
            });
        }
    }

    // Convert to BarlineType
    let barline_type = BarlineType::from_str(&barline_str)
        .map_err(|_| ParseError {
            message: format!("Invalid barline: {}", barline_str),
            line: line_num,
            column: column_from_pos(input, start_pos),
        })?;

    Ok(crate::parse::model::Barline {
        barline_type,
        source: Attributes {
                            slur_start: false,
                            slur_char_length: None,
            value: Some(barline_str),
            position: Position {
                line: line_num,
                column: column_from_pos(input, start_pos),
                index_in_line: index_in_line_from_pos(input, start_pos, line_num),
                index_in_doc: line_start_doc_index + start_pos,
            },
        },
    })
}