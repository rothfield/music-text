use crate::parse::model::{UpperLine, UpperElement, Attributes, Position};
use crate::parse::ParseError;

/// Parse an upper line following the grammar specification
pub fn parse_upper_line(input: &str, line_num: usize, line_start_doc_index: usize) -> Result<UpperLine, ParseError> {
    let mut elements = Vec::new();
    let mut chars = input.chars().peekable();
    let mut column = 1;
    let mut index_in_line: usize = 0;

    while let Some(ch) = chars.next() {
        let element = match ch {
            // Newline: upper_line ends in newline or EOI - include newline as part of upper_line
            '\n' => {
                elements.push(UpperElement::Newline {
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
                break; // upper_line ends at newline
            },
            // Upper octave markers: . : and *
            '.' | ':' | '*' => {
                UpperElement::UpperOctaveMarker {
                    marker: ch.to_string(),
                    source: Attributes {
                            slur_start: false,
                            slur_char_length: None,
                        value: Some(ch.to_string()),
                        position: Position { line: line_num, column, index_in_line, index_in_doc: line_start_doc_index + index_in_line },
                    },
                }
            },

            // Slurs: consecutive underscores
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

                UpperElement::SlurIndicator {
                    value: value.clone(),
                    source: Attributes {
                            slur_start: false,
                            slur_char_length: None,
                        value: Some(value),
                        position: Position { line: line_num, column: start_col, index_in_line: start_index_in_line, index_in_doc: line_start_doc_index + start_index_in_line },
                    },
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

                UpperElement::Space {
                    count,
                    source: Attributes {
                            slur_start: false,
                            slur_char_length: None,
                        value: Some(" ".repeat(count)),
                        position: Position { line: line_num, column: start_col, index_in_line: start_index_in_line, index_in_doc: line_start_doc_index + start_index_in_line },
                    },
                }
            },

            // Ornaments: <...> brackets
            '<' => {
                let start_col = column;
                let start_index_in_line = index_in_line;
                let mut content = String::new();
                content.push(ch);

                // Collect until closing >
                while let Some(&next_ch) = chars.peek() {
                    let consumed = chars.next().unwrap();
                    content.push(consumed);
                    column += 1;
                    index_in_line += 1;
                    if consumed == '>' {
                        break;
                    }
                }

                UpperElement::Ornament {
                    pitches: vec![content.clone()],
                    source: Attributes {
                            slur_start: false,
                            slur_char_length: None,
                        value: Some(content),
                        position: Position { line: line_num, column: start_col, index_in_line: start_index_in_line, index_in_doc: line_start_doc_index + start_index_in_line },
                    },
                }
            },

            // Chords: [...] brackets
            '[' => {
                let start_col = column;
                let start_index_in_line = index_in_line;
                let mut content = String::new();
                content.push(ch);

                // Collect until closing ]
                while let Some(&next_ch) = chars.peek() {
                    let consumed = chars.next().unwrap();
                    content.push(consumed);
                    column += 1;
                    index_in_line += 1;
                    if consumed == ']' {
                        break;
                    }
                }

                UpperElement::Chord {
                    chord: content.clone(),
                    source: Attributes {
                            slur_start: false,
                            slur_char_length: None,
                        value: Some(content),
                        position: Position { line: line_num, column: start_col, index_in_line: start_index_in_line, index_in_doc: line_start_doc_index + start_index_in_line },
                    },
                }
            },

            // Mordent: ~ character
            '~' => {
                UpperElement::Mordent {
                    source: Attributes {
                            slur_start: false,
                            slur_char_length: None,
                        value: Some("~".to_string()),
                        position: Position { line: line_num, column, index_in_line, index_in_doc: line_start_doc_index + index_in_line },
                    },
                }
            },

            // Handle # for upper hashes
            '#' => {
                let start_col = column;
                let start_index_in_line = index_in_line;
                let mut value = String::new();
                value.push(ch);

                // Collect consecutive hashes
                while let Some(&'#') = chars.peek() {
                    value.push(chars.next().unwrap());
                    column += 1;
                    index_in_line += 1;
                }

                UpperElement::UpperHashes {
                    value: value.clone(),
                    source: Attributes {
                            slur_start: false,
                            slur_char_length: None,
                        value: Some(value),
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
                    if next_ch == '.' || next_ch == ':' || next_ch == '*' || next_ch == '_'
                        || next_ch == ' ' || next_ch == '<' || next_ch == '['
                        || next_ch == '~' || next_ch == '#' {
                        break;
                    }
                    value.push(chars.next().unwrap());
                    column += 1;
                    index_in_line += 1;
                }

                UpperElement::Unknown {
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

    Ok(UpperLine {
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
    fn test_upper_octave_markers() {
        // Test input at EOI (no newline needed)
        let result = parse_upper_line(".  : *", 1, 0).unwrap();
        assert_eq!(result.elements.len(), 5); // dot, spaces, colon, space, asterisk

        match &result.elements[0] {
            UpperElement::UpperOctaveMarker { marker, .. } => assert_eq!(marker, "."),
            _ => panic!("Expected UpperOctaveMarker"),
        }

        match &result.elements[2] {
            UpperElement::UpperOctaveMarker { marker, .. } => assert_eq!(marker, ":"),
            _ => panic!("Expected UpperOctaveMarker"),
        }

        match &result.elements[4] {
            UpperElement::UpperOctaveMarker { marker, .. } => assert_eq!(marker, "*"),
            _ => panic!("Expected UpperOctaveMarker"),
        }
    }

    #[test]
    fn test_slurs() {
        // Test input at EOI (no newline needed)
        let result = parse_upper_line("___", 1, 0).unwrap();
        assert_eq!(result.elements.len(), 1);

        match &result.elements[0] {
            UpperElement::SlurIndicator { value, .. } => assert_eq!(value, "___"),
            _ => panic!("Expected SlurIndicator"),
        }
    }

    #[test]
    fn test_ornaments() {
        // Test input at EOI (no newline needed)
        let result = parse_upper_line("<123>", 1, 0).unwrap();
        assert_eq!(result.elements.len(), 1);

        match &result.elements[0] {
            UpperElement::Ornament { pitches, .. } => assert_eq!(pitches[0], "<123>"),
            _ => panic!("Expected Ornament"),
        }
    }

    #[test]
    fn test_chords() {
        // Test input at EOI (no newline needed)
        let result = parse_upper_line("[C]", 1, 0).unwrap();
        assert_eq!(result.elements.len(), 1);

        match &result.elements[0] {
            UpperElement::Chord { chord, .. } => assert_eq!(chord, "[C]"),
            _ => panic!("Expected Chord"),
        }
    }

    #[test]
    fn test_mordent() {
        // Test input at EOI (no newline needed)
        let result = parse_upper_line("~", 1, 0).unwrap();
        assert_eq!(result.elements.len(), 1);

        match &result.elements[0] {
            UpperElement::Mordent { .. } => {}, // Success
            _ => panic!("Expected Mordent"),
        }
    }

    #[test]
    fn test_mixed_elements() {
        // Test input at EOI (no newline needed)
        let result = parse_upper_line(".   ___  <S> ~", 1, 0).unwrap();
        assert_eq!(result.elements.len(), 7); // dot, spaces, underscores, spaces, ornament, space, mordent

        match (&result.elements[0], &result.elements[2], &result.elements[4], &result.elements[6]) {
            (UpperElement::UpperOctaveMarker { marker, .. },
             UpperElement::SlurIndicator { value, .. },
             UpperElement::Ornament { pitches, .. },
             UpperElement::Mordent { .. }) => {
                assert_eq!(marker, ".");
                assert_eq!(value, "___");
                assert_eq!(pitches[0], "<S>");
            },
            _ => panic!("Unexpected element types"),
        }
    }
}
