use crate::rhythm::types::{ParsedElement, Position};
use crate::models::pitch::Degree;
use crate::parse::recursive_descent::ParseError;
use crate::parse::model::NotationSystem;
use crate::rhythm::analyzer::{Beat, BeatElement, Event, Item};
use crate::rhythm::converters::BarlineType;
use crate::models::pitch_systems::{sargam, western, number};
use fraction::Fraction;
use std::iter::Peekable;
use std::str::Chars;

/// Parse content line according to grammar: content_line ends in newline or EOI
pub fn parse_content_line(input: &str) -> Result<Vec<ParsedElement>, ParseError> {
    // For backward compatibility, detect notation system automatically
    let notation_system = detect_notation_system_from_input(input);
    parse_content_line_with_system(input, 1, notation_system, 0)
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

/// Parse content line with correct row number (backward compatibility)
pub fn parse_content_line_with_row(input: &str, row: usize) -> Result<Vec<ParsedElement>, ParseError> {
    // For backward compatibility, detect notation system automatically
    let notation_system = detect_notation_system_from_input(input);
    parse_content_line_with_system(input, row, notation_system, 0)
}

/// Parse content line with notation system specified - main implementation
pub fn parse_content_line_with_system(input: &str, row: usize, notation_system: NotationSystem, line_start_doc_index: usize) -> Result<Vec<ParsedElement>, ParseError> {
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
                    position: Position { row: actual_row, col: position, char_index: line_start_doc_index + position },
                });
                position += 1;
                break;
            }
            '|' => {
                elements.push(ParsedElement::Barline {
                    style: "|".to_string(),
                    position: Position { row: actual_row, col: position, char_index: line_start_doc_index + position },
                    tala: None,
                });
                position += 1;
            }
            ' ' => {
                elements.push(ParsedElement::Whitespace {
                    value: " ".to_string(),
                    position: Position { row: actual_row, col: position, char_index: line_start_doc_index + position },
                });
                position += 1;
            }
            '-' => {
                elements.push(ParsedElement::Dash {
                    degree: None,
                    octave: None,
                    position: Position { row: actual_row, col: position, char_index: line_start_doc_index + position },
                    duration: None,
                });
                position += 1;
            }
            _ => {
                // Try to parse as a pitch note based on the specified notation system
                if let Some((note_value, degree, consumed_chars)) = try_parse_pitch_by_system(ch, &mut chars, notation_system) {
                    elements.push(create_note_element(
                        note_value,
                        degree,
                        Position { row: actual_row, col: position, char_index: line_start_doc_index + position }
                    ));

                    // Consume the additional characters from the iterator (first char already consumed by main loop)
                    for _ in 1..consumed_chars {
                        chars.next();
                    }
                    position += consumed_chars;
                } else {
                    // Not a pitch note - treat as unknown character
                    elements.push(ParsedElement::Unknown {
                        value: ch.to_string(),
                        position: Position { row: actual_row, col: position, char_index: line_start_doc_index + position },
                    });
                    position += 1;
                }
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

/// Factory method for creating note elements - eliminates duplication
fn create_note_element(value: String, degree: Degree, position: Position) -> ParsedElement {
    ParsedElement::Note {
        degree,
        octave: 0,
        value,
        position,
        children: Vec::new(),
        duration: None,
        slur: None,
        beat_group: None,
        in_slur: false,
        in_beat_group: false,
    }
}

/// Try to parse a number note with accidentals (1-7 with optional #, b, ##, bb, ♯, ♭, ♯♯, ♭♭)
/// This function is called when we already know the first character is 1-7
/// Returns (matched_string, degree, consumed_chars) on success, None on failure
fn try_parse_number_note(first_char: char, chars: &mut Peekable<Chars>) -> Option<(String, Degree, usize)> {
    // We already know first_char is 1-7
    if !('1'..='7').contains(&first_char) {
        return None;
    }

    // Collect remaining characters to check for accidentals
    let lookahead_chars: Vec<char> = chars.clone().collect();

    let base_digit = first_char;
    let remaining = &lookahead_chars[..];

    // Try productions in order (longest first)
    // Double accidentals first
    if remaining.len() >= 2 {
        if remaining.starts_with(&['#', '#']) {
            let token = format!("{}##", base_digit);
            let degree = match base_digit {
                '1' => Degree::N1ss, '2' => Degree::N2ss, '3' => Degree::N3ss, '4' => Degree::N4ss,
                '5' => Degree::N5ss, '6' => Degree::N6ss, '7' => Degree::N7ss,
                _ => return None,
            };
            return Some((token, degree, 3));
        }
        if remaining.starts_with(&['b', 'b']) {
            let token = format!("{}bb", base_digit);
            let degree = match base_digit {
                '1' => Degree::N1bb, '2' => Degree::N2bb, '3' => Degree::N3bb, '4' => Degree::N4bb,
                '5' => Degree::N5bb, '6' => Degree::N6bb, '7' => Degree::N7bb,
                _ => return None,
            };
            return Some((token, degree, 3));
        }
        // Unicode double accidentals
        if remaining.starts_with(&['♯', '♯']) {
            let token = format!("{}♯♯", base_digit);
            let degree = match base_digit {
                '1' => Degree::N1ss, '2' => Degree::N2ss, '3' => Degree::N3ss, '4' => Degree::N4ss,
                '5' => Degree::N5ss, '6' => Degree::N6ss, '7' => Degree::N7ss,
                _ => return None,
            };
            return Some((token, degree, 3)); // Note: Unicode chars are still 1 char each in char iterator
        }
        if remaining.starts_with(&['♭', '♭']) {
            let token = format!("{}♭♭", base_digit);
            let degree = match base_digit {
                '1' => Degree::N1bb, '2' => Degree::N2bb, '3' => Degree::N3bb, '4' => Degree::N4bb,
                '5' => Degree::N5bb, '6' => Degree::N6bb, '7' => Degree::N7bb,
                _ => return None,
            };
            return Some((token, degree, 3));
        }
    }

    // Single accidentals
    if !remaining.is_empty() {
        if remaining[0] == '#' {
            let token = format!("{}#", base_digit);
            let degree = match base_digit {
                '1' => Degree::N1s, '2' => Degree::N2s, '3' => Degree::N3s, '4' => Degree::N4s,
                '5' => Degree::N5s, '6' => Degree::N6s, '7' => Degree::N7s,
                _ => return None,
            };
            return Some((token, degree, 2));
        }
        if remaining[0] == 'b' {
            let token = format!("{}b", base_digit);
            let degree = match base_digit {
                '1' => Degree::N1b, '2' => Degree::N2b, '3' => Degree::N3b, '4' => Degree::N4b,
                '5' => Degree::N5b, '6' => Degree::N6b, '7' => Degree::N7b,
                _ => return None,
            };
            return Some((token, degree, 2));
        }
        // Unicode single accidentals
        if remaining[0] == '♯' {
            let token = format!("{}♯", base_digit);
            let degree = match base_digit {
                '1' => Degree::N1s, '2' => Degree::N2s, '3' => Degree::N3s, '4' => Degree::N4s,
                '5' => Degree::N5s, '6' => Degree::N6s, '7' => Degree::N7s,
                _ => return None,
            };
            return Some((token, degree, 2));
        }
        if remaining[0] == '♭' {
            let token = format!("{}♭", base_digit);
            let degree = match base_digit {
                '1' => Degree::N1b, '2' => Degree::N2b, '3' => Degree::N3b, '4' => Degree::N4b,
                '5' => Degree::N5b, '6' => Degree::N6b, '7' => Degree::N7b,
                _ => return None,
            };
            return Some((token, degree, 2));
        }
    }

    // Natural note (no accidentals)
    let token = base_digit.to_string();
    let degree = match base_digit {
        '1' => Degree::N1, '2' => Degree::N2, '3' => Degree::N3, '4' => Degree::N4,
        '5' => Degree::N5, '6' => Degree::N6, '7' => Degree::N7,
        _ => return None,
    };
    Some((token, degree, 1))
}

/// Generic function to try parsing any pitch system using its symbol list and lookup function
/// This leverages the existing pitch system modules (sargam, western, etc.)
fn try_parse_pitch_system<F, L>(
    first_char: char,
    chars: &mut Peekable<Chars>,
    get_symbols: F,
    lookup: L,
) -> Option<(String, Degree, usize)>
where
    F: Fn() -> Vec<String>,
    L: Fn(&str) -> Option<Degree>,
{
    let symbols = get_symbols();

    // Collect lookahead characters for matching
    let mut lookahead: Vec<char> = vec![first_char];
    lookahead.extend(chars.clone());
    let lookahead_str: String = lookahead.iter().collect();

    // Try symbols in order (they should already be sorted longest first)
    for symbol in &symbols {
        if lookahead_str.starts_with(symbol) {
            if let Some(degree) = lookup(symbol) {
                return Some((symbol.clone(), degree, symbol.chars().count()));
            }
        }
    }

    None
}

/// Detect notation system from input string (for backward compatibility)
fn detect_notation_system_from_input(input: &str) -> NotationSystem {
    if input.chars().any(|c| matches!(c, 'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N' | 's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n')) {
        NotationSystem::Sargam
    } else if input.chars().any(|c| matches!(c, '1'..='7')) {
        NotationSystem::Number
    } else {
        NotationSystem::Western
    }
}

/// Try to parse a pitch note using the specified notation system
fn try_parse_pitch_by_system(
    first_char: char,
    chars: &mut Peekable<Chars>,
    notation_system: NotationSystem,
) -> Option<(String, Degree, usize)> {
    match notation_system {
        NotationSystem::Number => {
            if ('1'..='7').contains(&first_char) {
                try_parse_number_note(first_char, chars)
            } else {
                None
            }
        }
        NotationSystem::Sargam => {
            try_parse_pitch_system(first_char, chars, sargam::get_all_symbols, sargam::lookup)
        }
        NotationSystem::Western => {
            try_parse_pitch_system(first_char, chars, western::get_all_symbols, western::lookup)
        }
        _ => None, // Other systems not implemented yet
    }
}
