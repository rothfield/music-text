use crate::rhythm::types::{ParsedElement, Degree, Position};
use crate::parse::recursive_descent::ParseError;
use crate::rhythm::analyzer::{Beat, BeatElement, Event, Item};
use crate::rhythm::converters::BarlineType;
use fraction::Fraction;

/// Parse content line according to grammar: content_line ends in newline or EOI
pub fn parse_content_line(input: &str) -> Result<Vec<ParsedElement>, ParseError> {
    parse_content_line_with_row(input, 1)
}

/// Parse content line with beats directly created
pub fn parse_content_line_with_beats(input: &str) -> Result<Vec<Item>, ParseError> {
    parse_content_line_with_beats_and_row(input, 1)
}

/// Parse content line with beats and correct row number
pub fn parse_content_line_with_beats_and_row(input: &str, row: usize) -> Result<Vec<Item>, ParseError> {
    let elements = parse_content_line_with_row(input, row)?;
    Ok(create_beats_from_elements(&elements))
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

/// Create beats from parsed elements using simplified FSM logic
fn create_beats_from_elements(elements: &[ParsedElement]) -> Vec<Item> {
    let mut items = Vec::new();
    let mut current_beat_elements = Vec::new();
    let mut last_note_degree: Option<Degree> = None;
    let mut pending_tie = false;

    for element in elements {
        match element {
            ParsedElement::Note { .. } => {
                // Notes start new beats or extend current ones
                if current_beat_elements.is_empty() {
                    // Start new beat
                    let tied_to_previous = pending_tie && last_note_degree.is_some();
                    let beat_element = BeatElement::from(element.clone()).with_subdivisions(1);
                    current_beat_elements.push(beat_element);

                    // Update last note for tie tracking
                    if let ParsedElement::Note { degree, .. } = element {
                        last_note_degree = Some(*degree);
                    }
                    pending_tie = false;
                } else {
                    // Add to current beat
                    let beat_element = BeatElement::from(element.clone()).with_subdivisions(1);
                    current_beat_elements.push(beat_element);

                    // Update last note for tie tracking
                    if let ParsedElement::Note { degree, .. } = element {
                        last_note_degree = Some(*degree);
                    }
                }
            }
            ParsedElement::Dash { .. } => {
                if current_beat_elements.is_empty() {
                    // Dash at start - could be tie or rest
                    if last_note_degree.is_some() {
                        pending_tie = true;
                        // Create tied note element
                        let rest_element = BeatElement {
                            event: Event::Rest,
                            subdivisions: 1,
                            duration: Fraction::new(1u64, 4u64),
                            tuplet_duration: Fraction::new(1u64, 4u64),
                            tuplet_display_duration: None,
                            value: "-".to_string(),
                            position: element.position().clone(),
                        };
                        current_beat_elements.push(rest_element);
                    } else {
                        // Start with rest
                        let beat_element = BeatElement::from(element.clone()).with_subdivisions(1);
                        current_beat_elements.push(beat_element);
                    }
                } else {
                    // Extend last element
                    if let Some(last) = current_beat_elements.last_mut() {
                        last.extend_subdivision();
                    }
                }
            }
            ParsedElement::Whitespace { .. } => {
                // Finish current beat
                if !current_beat_elements.is_empty() {
                    finish_beat(&mut items, &mut current_beat_elements);
                }
            }
            ParsedElement::Barline { style, tala, .. } => {
                // Finish current beat first
                if !current_beat_elements.is_empty() {
                    finish_beat(&mut items, &mut current_beat_elements);
                }
                // Add barline
                if let Ok(barline_type) = BarlineType::from_str(style) {
                    items.push(Item::Barline(barline_type, *tala));
                }
            }
            ParsedElement::Symbol { value, .. } if value == "'" => {
                // Breathmark
                if !current_beat_elements.is_empty() {
                    finish_beat(&mut items, &mut current_beat_elements);
                }
                items.push(Item::Breathmark);
                last_note_degree = None;
                pending_tie = false;
            }
            _ => {
                // Other elements (newline, unknown, etc.) don't affect beat structure
            }
        }
    }

    // Finish any remaining beat
    if !current_beat_elements.is_empty() {
        finish_beat(&mut items, &mut current_beat_elements);
    }

    items
}

fn finish_beat(items: &mut Vec<Item>, current_beat_elements: &mut Vec<BeatElement>) {
    if current_beat_elements.is_empty() {
        return;
    }

    let divisions = current_beat_elements.iter().map(|e| e.subdivisions).sum::<usize>();

    // Tuplet detection: not a power of 2 AND more than one element
    let is_tuplet = current_beat_elements.len() > 1 &&
                   divisions > 1 &&
                   (divisions & (divisions - 1)) != 0;

    let tuplet_ratio = if is_tuplet {
        let power_of_2 = find_next_lower_power_of_2(divisions);
        Some((divisions, power_of_2))
    } else {
        None
    };

    // Calculate durations
    for beat_element in current_beat_elements.iter_mut() {
        if is_tuplet {
            if let Some((_, power_of_2)) = tuplet_ratio {
                let each_unit = Fraction::new(1u64, 4u64) / power_of_2;
                beat_element.duration = Fraction::new(beat_element.subdivisions as u64, divisions as u64);
                beat_element.tuplet_duration = each_unit * beat_element.subdivisions;
                beat_element.tuplet_display_duration = Some(each_unit * beat_element.subdivisions);
            }
        } else {
            let base_duration = Fraction::new(beat_element.subdivisions as u64, divisions as u64)
                              * Fraction::new(1u64, 4u64);
            beat_element.duration = base_duration;
            beat_element.tuplet_duration = base_duration;
        }
    }

    let beat = Beat {
        divisions,
        elements: current_beat_elements.clone(),
        tied_to_previous: false, // Simplified for now
        is_tuplet,
        tuplet_ratio,
    };

    items.push(Item::Beat(beat));
    current_beat_elements.clear();
}

fn find_next_lower_power_of_2(n: usize) -> usize {
    let mut power = 1;
    while power * 2 < n {
        power *= 2;
    }
    power.max(2)
}