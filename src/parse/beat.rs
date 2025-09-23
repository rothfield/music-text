use crate::parse::model::{Beat, BeatElement, Note, Dash, BreathMark, NotationSystem, Attributes, Position};
use crate::parse::pitch::{parse_pitch_with_indices, is_pitch_start};
use crate::parse::ParseError;
use std::str::CharIndices;
use std::iter::Peekable;

/// Helper function to calculate column from position in input
fn column_from_pos(input: &str, pos: usize) -> usize {
    input[..pos].chars().rev().take_while(|&c| c != '\n').count() + 1
}

/// Helper function to calculate index in line from position
fn index_in_line_from_pos(input: &str, pos: usize, _line_num: usize) -> usize {
    input[..pos].chars().rev().take_while(|&c| c != '\n').count()
}

/// Parse a beat according to the grammar:
/// beat = (pitch | dash) beat-element*
/// beat-element = pitch | dash | breath-mark
///
/// Returns the parsed beat
pub fn parse_beat(
    chars: &mut Peekable<CharIndices>,
    notation_system: NotationSystem,
    line_num: usize,
    input: &str,
    line_start_doc_index: usize,
) -> Result<Beat, ParseError> {
    let mut elements = Vec::new();
    let beat_start_pos = chars.peek().map(|(pos, _)| *pos).unwrap_or(0);

    // First element must be pitch or dash
    match chars.peek() {
        Some(&(pos, '-')) => {
            chars.next();
            elements.push(BeatElement::Dash(Dash {
                value: Some("-".to_string()),
                char_index: line_start_doc_index + pos,
                consumed_elements: Vec::new(),
            }));
        }
        Some(&(pos, ch)) if is_pitch_start(ch, notation_system) => {
            let (pitch_str, pitch_code) = parse_pitch_with_indices(chars, notation_system, line_num, input)?;

            elements.push(BeatElement::Note(Note {
                value: Some(pitch_str),
                char_index: line_start_doc_index + pos,
                pitch_code,
                octave: 0, // Default octave, will be adjusted by spatial annotations
                notation_system,
                consumed_elements: Vec::new(), // Will be populated during spatial analysis
                numerator: None, // Will be populated by rhythm analysis
                denominator: None, // Will be populated by rhythm analysis
            }));
        }
        Some(&(pos, ch)) => {
            return Err(ParseError {
                message: format!("Expected pitch or dash to start beat, found '{}'", ch),
                line: line_num,
                column: column_from_pos(input, pos),
            });
        }
        None => {
            return Err(ParseError {
                message: "Unexpected end of input, expected pitch or dash to start beat".to_string(),
                line: line_num,
                column: 1,
            });
        }
    }

    // Continue parsing beat-elements
    loop {
        match chars.peek() {
            // Beat terminators
            Some(&(_, ' ')) | Some(&(_, '|')) | Some(&(_, '\n')) | None => break,

            // Dash
            Some(&(pos, '-')) => {
                chars.next();
                elements.push(BeatElement::Dash(Dash {
                    value: Some("-".to_string()),
                    char_index: line_start_doc_index + pos,
                    consumed_elements: Vec::new(),
                }));
            }

            // Breath mark
            Some(&(pos, '\'')) => {
                chars.next();
                elements.push(BeatElement::BreathMark(BreathMark {
                    value: Some("'".to_string()),
                    char_index: line_start_doc_index + pos,
                    consumed_elements: Vec::new(),
                }));
            }

            // Another pitch
            Some(&(pos, ch)) if is_pitch_start(ch, notation_system) => {
                let (pitch_str, pitch_code) = parse_pitch_with_indices(chars, notation_system, line_num, input)?;

                elements.push(BeatElement::Note(Note {
                    value: Some(pitch_str),
                    char_index: line_start_doc_index + pos,
                    pitch_code,
                    octave: 0,
                    notation_system,
                        consumed_elements: Vec::new(), // Will be populated during spatial analysis
                    numerator: None, // Will be populated by rhythm analysis
                    denominator: None // Will be populated by rhythm analysis
                }));
            }

            // Unknown character ends the beat
            Some(_) => break,
        }
    }

    let beat = Beat {
        elements,
        value: None, // Will be filled by caller if needed
        char_index: line_start_doc_index + beat_start_pos,
        consumed_elements: Vec::new(),
        divisions: None,        // Will be populated by rhythm analysis
        total_duration: None,   // Will be populated by rhythm analysis
        is_tuplet: None,        // Will be populated by rhythm analysis
        tuplet_ratio: None,     // Will be populated by rhythm analysis
        tied_to_previous: None, // Will be populated by rhythm analysis
    };

    Ok(beat)
}