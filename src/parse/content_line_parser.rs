use crate::rhythm::types::{ParsedElement, Degree, Position};
use crate::parse::recursive_descent::ParseError;

/// Parse content line according to grammar: content_line ends in newline or EOI
pub fn parse_content_line(input: &str) -> Result<Vec<ParsedElement>, ParseError> {
    parse_content_line_with_row(input, 1)
}

/// Parse content line with correct row number
pub fn parse_content_line_with_row(input: &str, row: usize) -> Result<Vec<ParsedElement>, ParseError> {
    let mut elements = Vec::new();
    let mut chars = input.chars().peekable();
    let mut position = 0;
    let actual_row = row - 1;  // Adjust for post-increment line numbering

    while let Some(ch) = chars.next() {
        match ch {
            '\n' => {
                // content_line ends in newline - include the newline as part of content_line
                elements.push(ParsedElement::Newline {
                    value: "\n".to_string(),
                    position: Position { row: actual_row, col: position },
                });
                position += 1;
                break;
            }
            '|' => {
                elements.push(ParsedElement::Barline {
                    style: "|".to_string(),
                    position: Position { row: actual_row, col: position },
                    tala: None,
                });
                position += 1;
            }
            ' ' => {
                elements.push(ParsedElement::Whitespace {
                    value: " ".to_string(),
                    position: Position { row: actual_row, col: position },
                });
                position += 1;
            }
            '-' => {
                elements.push(ParsedElement::Dash {
                    degree: None,
                    octave: None,
                    position: Position { row: actual_row, col: position },
                    duration: None,
                });
                position += 1;
            }
            '1'..='7' => {
                let degree = match ch {
                    '1' => Degree::N1,
                    '2' => Degree::N2,
                    '3' => Degree::N3,
                    '4' => Degree::N4,
                    '5' => Degree::N5,
                    '6' => Degree::N6,
                    '7' => Degree::N7,
                    _ => unreachable!(),
                };
                elements.push(ParsedElement::Note {
                    degree,
                    octave: 0,
                    value: ch.to_string(),
                    position: Position { row: actual_row, col: position },
                    children: Vec::new(),  // Will be populated by analyzer with octave markers, ornaments
                    duration: None,
                    slur: None,
                    beat_group: None,
                    in_slur: false,
                    in_beat_group: false,
                });
                position += 1;
            }
            'S' | 's' => {
                elements.push(ParsedElement::Note {
                    degree: Degree::N1,
                    octave: 0,
                    value: ch.to_string(),
                    position: Position { row: actual_row, col: position },
                    children: Vec::new(),
                    duration: None,
                    slur: None,
                    beat_group: None,
                    in_slur: false,
                    in_beat_group: false,
                });
                position += 1;
            }
            'R' | 'r' => {
                elements.push(ParsedElement::Note {
                    degree: Degree::N2,
                    octave: 0,
                    value: ch.to_string(),
                    position: Position { row: actual_row, col: position },
                    children: Vec::new(),
                    duration: None,
                    slur: None,
                    beat_group: None,
                    in_slur: false,
                    in_beat_group: false,
                });
                position += 1;
            }
            'G' | 'g' => {
                elements.push(ParsedElement::Note {
                    degree: Degree::N3,
                    octave: 0,
                    value: ch.to_string(),
                    position: Position { row: actual_row, col: position },
                    children: Vec::new(),
                    duration: None,
                    slur: None,
                    beat_group: None,
                    in_slur: false,
                    in_beat_group: false,
                });
                position += 1;
            }
            'M' | 'm' => {
                elements.push(ParsedElement::Note {
                    degree: Degree::N4,
                    octave: 0,
                    value: ch.to_string(),
                    position: Position { row: actual_row, col: position },
                    children: Vec::new(),
                    duration: None,
                    slur: None,
                    beat_group: None,
                    in_slur: false,
                    in_beat_group: false,
                });
                position += 1;
            }
            'P' | 'p' => {
                elements.push(ParsedElement::Note {
                    degree: Degree::N5,
                    octave: 0,
                    value: ch.to_string(),
                    position: Position { row: actual_row, col: position },
                    children: Vec::new(),
                    duration: None,
                    slur: None,
                    beat_group: None,
                    in_slur: false,
                    in_beat_group: false,
                });
                position += 1;
            }
            'D' | 'd' => {
                elements.push(ParsedElement::Note {
                    degree: Degree::N6,
                    octave: 0,
                    value: ch.to_string(),
                    position: Position { row: actual_row, col: position },
                    children: Vec::new(),
                    duration: None,
                    slur: None,
                    beat_group: None,
                    in_slur: false,
                    in_beat_group: false,
                });
                position += 1;
            }
            'N' | 'n' => {
                elements.push(ParsedElement::Note {
                    degree: Degree::N7,
                    octave: 0,
                    value: ch.to_string(),
                    position: Position { row: actual_row, col: position },
                    children: Vec::new(),
                    duration: None,
                    slur: None,
                    beat_group: None,
                    in_slur: false,
                    in_beat_group: false,
                });
                position += 1;
            }
            _ => {
                elements.push(ParsedElement::Unknown {
                    value: ch.to_string(),
                    position: Position { row: actual_row, col: position },
                });
                position += 1;
            }
        }
    }

    // If we reach here, we hit EOI without a newline, which is also valid for content_line
    Ok(elements)
}