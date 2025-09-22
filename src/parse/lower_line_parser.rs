use crate::parse::model::{LowerLine, LowerElement, Attributes, Position};
use crate::parse::ParseError;

/// Parse a lower line following the grammar specification
pub fn parse_lower_line(input: &str, line_num: usize, line_start_doc_index: usize) -> Result<LowerLine, ParseError> {
    let mut elements = Vec::new();
    let mut chars = input.chars().peekable();
    let mut column = 1;
    let mut index_in_line: usize = 0;

    while let Some(ch) = chars.next() {
        let element = match ch {
            // Newline: lower_line ends in newline or EOI - include newline as part of lower_line
            '\n' => {
                elements.push(LowerElement::Newline {
                    value: "\n".to_string(),
                    source: Attributes {
                            slur_start: false,
                            slur_char_length: None,
                        value: Some("\n".to_string()),
                        position: Position { line: line_num, column, index_in_line, index_in_doc: line_start_doc_index + index_in_line },
                    },
                });
                column += 1;
                index_in_line += 1;
                break; // lower_line ends at newline
            },
            // Lower octave markers: . and :
            '.' | ':' => {
                LowerElement::LowerOctaveMarker {
                    marker: ch.to_string(),
                    source: Attributes {
                            slur_start: false,
                            slur_char_length: None,
                        value: Some(ch.to_string()),
                        position: Position { line: line_num, column, index_in_line, index_in_doc: line_start_doc_index + index_in_line },
                    },
                }
            },

            // Lower line underscores: 2+ consecutive underscores
            '_' => {
                let start_col = column;
                let start_index_in_line = index_in_line;
                let mut value = String::new();
                value.push(ch);

                // Collect consecutive underscores
                while let Some(&'_') = chars.peek() {
                    value.push(chars.next().unwrap());
                    column += 1;
                    index_in_line += 1;
                }

                if value.len() >= 2 {
                    LowerElement::BeatGroupIndicator {
                        value: value.clone(),
                        source: Attributes {
                            slur_start: false,
                            slur_char_length: None,
                            value: Some(value),
                            position: Position { line: line_num, column: start_col, index_in_line: start_index_in_line, index_in_doc: line_start_doc_index + start_index_in_line },
                        },
                    }
                } else {
                    // Single underscore becomes Unknown
                    LowerElement::Unknown {
                        value: value.clone(),
                        source: Attributes {
                            slur_start: false,
                            slur_char_length: None,
                            value: Some(value),
                            position: Position { line: line_num, column: start_col, index_in_line: start_index_in_line, index_in_doc: line_start_doc_index + start_index_in_line },
                        },
                    }
                }
            },

            // Space: consecutive spaces for alignment
            ' ' => {
                let start_col = column;
                let start_index_in_line = index_in_line;
                let mut count = 1;

                // Collect consecutive spaces
                while let Some(&' ') = chars.peek() {
                    chars.next();
                    count += 1;
                    column += 1;
                    index_in_line += 1;
                }

                LowerElement::Space {
                    count,
                    source: Attributes {
                            slur_start: false,
                            slur_char_length: None,
                        value: Some(" ".repeat(count)),
                        position: Position { line: line_num, column: start_col, index_in_line: start_index_in_line, index_in_doc: line_start_doc_index + start_index_in_line },
                    },
                }
            },

            // Syllables: letter+ with optional apostrophes and hyphens
            ch if ch.is_alphabetic() => {
                let start_col = column;
                let start_index_in_line = index_in_line;
                let mut content = String::new();
                content.push(ch);

                // Collect syllable characters: letters, digits, apostrophes, hyphens
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_alphanumeric() || next_ch == '\'' || next_ch == '-' {
                        content.push(chars.next().unwrap());
                        column += 1;
                        index_in_line += 1;
                    } else {
                        break;
                    }
                }

                LowerElement::Syllable {
                    content: content.clone(),
                    source: Attributes {
                            slur_start: false,
                            slur_char_length: None,
                        value: Some(content),
                        position: Position { line: line_num, column: start_col, index_in_line: start_index_in_line, index_in_doc: line_start_doc_index + start_index_in_line },
                    },
                }
            },

            // Unknown: anything else
            _ => {
                let start_col = column;
                let start_index_in_line = index_in_line;
                let mut value = String::new();
                value.push(ch);

                // Collect consecutive unknown characters (stop at known tokens)
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '.' || next_ch == ':' || next_ch == '_' || next_ch == ' '
                        || next_ch.is_alphabetic() {
                        break;
                    }
                    value.push(chars.next().unwrap());
                    column += 1;
                    index_in_line += 1;
                }

                LowerElement::Unknown {
                    value: value.clone(),
                    source: Attributes {
                            slur_start: false,
                            slur_char_length: None,
                        value: Some(value),
                        position: Position { line: line_num, column: start_col, index_in_line: start_index_in_line, index_in_doc: line_start_doc_index + start_index_in_line },
                    },
                }
            }
        };

        elements.push(element);
        column += 1;
        index_in_line += 1;
    }


    Ok(LowerLine {
        elements,
        source: Attributes {
                            slur_start: false,
                            slur_char_length: None,
            value: Some(input.to_string()),
            position: Position { line: line_num, column: 1, index_in_line: 0, index_in_doc: line_start_doc_index },
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lower_octave_markers() {
        // Test input at EOI (no newline needed)
        let result = parse_lower_line(".  :", 1, 0).unwrap();
        assert_eq!(result.elements.len(), 3); // dot, spaces, colon

        match &result.elements[0] {
            LowerElement::LowerOctaveMarker { marker, .. } => assert_eq!(marker, "."),
            _ => panic!("Expected LowerOctaveMarker"),
        }

        match &result.elements[2] {
            LowerElement::LowerOctaveMarker { marker, .. } => assert_eq!(marker, ":"),
            _ => panic!("Expected LowerOctaveMarker"),
        }
    }

    #[test]
    fn test_lower_underscores() {
        // Test input at EOI (no newline needed)
        let result = parse_lower_line("___", 1, 0).unwrap();
        assert_eq!(result.elements.len(), 1);

        match &result.elements[0] {
            LowerElement::BeatGroupIndicator { value, .. } => assert_eq!(value, "___"),
            _ => panic!("Expected BeatGroupIndicator"),
        }
    }

    #[test]
    fn test_syllables() {
        // Test input at EOI (no newline needed)
        let result = parse_lower_line("dha ge-na", 1, 0).unwrap();
        assert_eq!(result.elements.len(), 3); // "dha", space, "ge-na"

        match &result.elements[0] {
            LowerElement::Syllable { content, .. } => assert_eq!(content, "dha"),
            _ => panic!("Expected Syllable"),
        }

        match &result.elements[2] {
            LowerElement::Syllable { content, .. } => assert_eq!(content, "ge-na"),
            _ => panic!("Expected Syllable"),
        }
    }

    #[test]
    fn test_mixed_elements() {
        // Test input at EOI (no newline needed)
        let result = parse_lower_line(".   ___  dha", 1, 0).unwrap();
        assert_eq!(result.elements.len(), 5); // dot, spaces, underscores, spaces, syllable

        match (&result.elements[0], &result.elements[2]) {
            (LowerElement::LowerOctaveMarker { marker, .. },
             LowerElement::BeatGroupIndicator { value, .. }) => {
                assert_eq!(marker, ".");
                assert_eq!(value, "___");
            },
            _ => panic!("Unexpected element types"),
        }
    }
}
