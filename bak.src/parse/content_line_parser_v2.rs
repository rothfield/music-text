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

/// Single-pass parser that directly creates beat structures
/// This eliminates the need for separate rhythm analysis
pub fn parse_content_line_direct(
    input: &str,
    row: usize,
    notation_system: NotationSystem,
    line_start_doc_index: usize
) -> Result<Vec<Item>, ParseError> {
    let mut items = Vec::new();
    let mut current_beat = Vec::new();
    let mut chars = input.chars().peekable();
    let mut position = 0;
    let actual_row = row - 1;

    // State for tracking ties across beats
    let mut last_note_degree: Option<Degree> = None;
    let mut pending_tie = false;

    while let Some(ch) = chars.next() {
        match ch {
            '\n' => {
                // End of line - finish current beat if any
                finish_beat_if_needed(&mut items, &mut current_beat, pending_tie);
                break;
            }

            '|' => {
                // Barline - finish current beat and add barline
                finish_beat_if_needed(&mut items, &mut current_beat, pending_tie);

                // Parse barline type (could be ||, |:, :|, etc.)
                let mut barline_str = String::from("|");
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '|' || next_ch == ':' {
                        barline_str.push(next_ch);
                        chars.next();
                        position += 1;
                    } else {
                        break;
                    }
                }

                if let Ok(barline_type) = BarlineType::from_str(&barline_str) {
                    items.push(Item::Barline(barline_type, None));
                }
                position += 1;
            }

            ' ' => {
                // Space - beat boundary!
                // But skip multiple spaces
                position += 1;
                while chars.peek() == Some(&' ') {
                    chars.next();
                    position += 1;
                }
                finish_beat_if_needed(&mut items, &mut current_beat, pending_tie);
                pending_tie = false;
            }

            '-' => {
                // Dash - continues current beat or starts new one
                if current_beat.is_empty() {
                    // Start new beat with dash (rest)
                    if last_note_degree.is_some() {
                        // This dash creates a tie to previous beat
                        pending_tie = true;
                    }
                    current_beat.push(BeatElement {
                        event: Event::Rest,
                        subdivisions: 1,
                        duration: Fraction::new(1u64, 4u64),
                        tuplet_duration: Fraction::new(1u64, 4u64),
                        tuplet_display_duration: None,
                        value: "-".to_string(),
                        position: Position {
                            row: actual_row,
                            col: position,
                            char_index: line_start_doc_index + position
                        },
                    });
                } else {
                    // Add dash to current beat (extends it)
                    current_beat.push(BeatElement {
                        event: Event::Rest,
                        subdivisions: 1,
                        duration: Fraction::new(1u64, 4u64),
                        tuplet_duration: Fraction::new(1u64, 4u64),
                        tuplet_display_duration: None,
                        value: "-".to_string(),
                        position: Position {
                            row: actual_row,
                            col: position,
                            char_index: line_start_doc_index + position
                        },
                    });
                }
                position += 1;
            }

            '\'' => {
                // Breathmark
                finish_beat_if_needed(&mut items, &mut current_beat, pending_tie);
                items.push(Item::Breathmark);
                last_note_degree = None;
                pending_tie = false;
                position += 1;
            }

            _ => {
                // Try to parse as pitch
                if let Some((note_value, degree, consumed)) =
                    try_parse_pitch_by_system(ch, &mut chars, notation_system) {

                    // Create note element
                    let note_element = BeatElement {
                        event: Event::Note {
                            degree,
                            octave: 0,  // Will be set by upper/lower line processing
                            children: vec![],
                            slur: None,
                        },
                        subdivisions: 1,
                        duration: Fraction::new(1u64, 4u64),
                        tuplet_duration: Fraction::new(1u64, 4u64),
                        tuplet_display_duration: None,
                        value: note_value,
                        position: Position {
                            row: actual_row,
                            col: position,
                            char_index: line_start_doc_index + position
                        },
                    };

                    current_beat.push(note_element);
                    last_note_degree = Some(degree);

                    // Consume additional characters
                    for _ in 1..consumed {
                        chars.next();
                    }
                    position += consumed;
                } else {
                    // Unknown character - add as unknown event
                    current_beat.push(BeatElement {
                        event: Event::Unknown { text: ch.to_string() },
                        subdivisions: 1,
                        duration: Fraction::new(1u64, 4u64),
                        tuplet_duration: Fraction::new(1u64, 4u64),
                        tuplet_display_duration: None,
                        value: ch.to_string(),
                        position: Position {
                            row: actual_row,
                            col: position,
                            char_index: line_start_doc_index + position
                        },
                    });
                    position += 1;
                }
            }
        }
    }

    // Finish any remaining beat
    finish_beat_if_needed(&mut items, &mut current_beat, pending_tie);

    Ok(items)
}

/// Helper to finish current beat and add to items
fn finish_beat_if_needed(
    items: &mut Vec<Item>,
    current_beat: &mut Vec<BeatElement>,
    tied_to_previous: bool
) {
    if current_beat.is_empty() {
        return;
    }

    let divisions = current_beat.iter().map(|e| e.subdivisions).sum::<usize>();

    // Detect tuplets: not a power of 2 and more than one element
    let is_tuplet = current_beat.len() > 1 &&
                    divisions > 1 &&
                    (divisions & (divisions - 1)) != 0;

    let tuplet_ratio = if is_tuplet {
        let power_of_2 = find_next_lower_power_of_2(divisions);
        Some((divisions, power_of_2))
    } else {
        None
    };

    // Update durations for each element
    for element in current_beat.iter_mut() {
        if is_tuplet {
            if let Some((_, power_of_2)) = tuplet_ratio {
                let each_unit = Fraction::new(1u64, 4u64) / power_of_2;
                element.duration = Fraction::new(element.subdivisions as u64, divisions as u64);
                element.tuplet_duration = each_unit * element.subdivisions;
                element.tuplet_display_duration = Some(each_unit * element.subdivisions);
            }
        } else {
            let base_duration = Fraction::new(element.subdivisions as u64, divisions as u64)
                              * Fraction::new(1u64, 4u64);
            element.duration = base_duration;
            element.tuplet_duration = base_duration;
        }
    }

    items.push(Item::Beat(Beat {
        divisions,
        elements: current_beat.clone(),
        tied_to_previous,
        is_tuplet,
        tuplet_ratio,
    }));

    current_beat.clear();
}

fn find_next_lower_power_of_2(n: usize) -> usize {
    let mut power = 1;
    while power * 2 < n {
        power *= 2;
    }
    power.max(2)
}

/// Try to parse a pitch using the specified notation system
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
        _ => None,
    }
}

/// Parse number notation (1-7 with accidentals)
fn try_parse_number_note(first_char: char, chars: &mut Peekable<Chars>) -> Option<(String, Degree, usize)> {
    if !('1'..='7').contains(&first_char) {
        return None;
    }

    let lookahead_chars: Vec<char> = chars.clone().collect();
    let base_digit = first_char;
    let remaining = &lookahead_chars[..];

    // Check for accidentals (##, bb, #, b, ♯, ♭)
    if remaining.len() >= 2 {
        if remaining.starts_with(&['#', '#']) {
            let token = format!("{}##", base_digit);
            let degree = match base_digit {
                '1' => Degree::N1ss, '2' => Degree::N2ss, '3' => Degree::N3ss,
                '4' => Degree::N4ss, '5' => Degree::N5ss, '6' => Degree::N6ss,
                '7' => Degree::N7ss, _ => return None,
            };
            return Some((token, degree, 3));
        }
        if remaining.starts_with(&['b', 'b']) {
            let token = format!("{}bb", base_digit);
            let degree = match base_digit {
                '1' => Degree::N1bb, '2' => Degree::N2bb, '3' => Degree::N3bb,
                '4' => Degree::N4bb, '5' => Degree::N5bb, '6' => Degree::N6bb,
                '7' => Degree::N7bb, _ => return None,
            };
            return Some((token, degree, 3));
        }
    }

    if !remaining.is_empty() {
        if remaining[0] == '#' {
            let token = format!("{}#", base_digit);
            let degree = match base_digit {
                '1' => Degree::N1s, '2' => Degree::N2s, '3' => Degree::N3s,
                '4' => Degree::N4s, '5' => Degree::N5s, '6' => Degree::N6s,
                '7' => Degree::N7s, _ => return None,
            };
            return Some((token, degree, 2));
        }
        if remaining[0] == 'b' {
            let token = format!("{}b", base_digit);
            let degree = match base_digit {
                '1' => Degree::N1b, '2' => Degree::N2b, '3' => Degree::N3b,
                '4' => Degree::N4b, '5' => Degree::N5b, '6' => Degree::N6b,
                '7' => Degree::N7b, _ => return None,
            };
            return Some((token, degree, 2));
        }
    }

    // Natural note
    let token = base_digit.to_string();
    let degree = match base_digit {
        '1' => Degree::N1, '2' => Degree::N2, '3' => Degree::N3, '4' => Degree::N4,
        '5' => Degree::N5, '6' => Degree::N6, '7' => Degree::N7, _ => return None,
    };
    Some((token, degree, 1))
}

/// Generic pitch system parser
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
    let mut lookahead = vec![first_char];
    lookahead.extend(chars.clone());
    let lookahead_str: String = lookahead.iter().collect();

    for symbol in &symbols {
        if lookahead_str.starts_with(symbol) {
            if let Some(degree) = lookup(symbol) {
                return Some((symbol.clone(), degree, symbol.chars().count()));
            }
        }
    }

    None
}