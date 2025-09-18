use crate::parse::model::{Beat, BeatElement, Note, Dash, BreathMark, Source, Position, PitchString, NotationSystem};
use crate::parse::pitch::{parse_pitch, is_pitch_start};
use crate::parse::ParseError;
use std::iter::Peekable;
use std::str::Chars;

/// Parse a beat according to the grammar:
/// beat = (pitch | dash) beat-element*
/// beat-element = pitch | dash | breath-mark
///
/// Returns the parsed beat and the number of characters consumed
pub fn parse_beat(
    chars: &mut Peekable<Chars>,
    notation_system: NotationSystem,
    line_num: usize,
    start_column: usize,
    start_index_in_line: usize,
    start_index_in_doc: usize,
) -> Result<(Beat, usize), ParseError> {
    let mut elements = Vec::new();
    let mut column = start_column;
    let mut index_in_line = start_index_in_line;
    let mut total_consumed = 0;

    // First element must be pitch or dash
    match chars.peek() {
        Some(&'-') => {
            chars.next();
            elements.push(BeatElement::Dash(Dash {
                source: Source {
                    value: Some("-".to_string()),
                    position: Position {
                        line: line_num,
                        column,
                        index_in_line,
                        index_in_doc: start_index_in_doc + index_in_line,
                    },
                },
            }));
            column += 1;
            index_in_line += 1;
            total_consumed += 1;
        }
        Some(&ch) if is_pitch_start(ch, notation_system) => {
            let (pitch_str, pitch_code, consumed) = parse_pitch(chars, notation_system, line_num, column)?;

            elements.push(BeatElement::Note(Note {
                pitch_string: PitchString {
                    source: Source {
                        value: Some(pitch_str),
                        position: Position {
                            line: line_num,
                            column,
                            index_in_line,
                            index_in_doc: start_index_in_doc + index_in_line,
                        },
                    },
                },
                pitch_code,
                octave: 0, // Default octave, will be adjusted by spatial annotations
                notation_system,
            }));

            column += consumed;
            index_in_line += consumed;
            total_consumed += consumed;
        }
        Some(&ch) => {
            return Err(ParseError {
                message: format!("Expected pitch or dash to start beat, found '{}'", ch),
                line: line_num,
                column,
            });
        }
        None => {
            return Err(ParseError {
                message: "Unexpected end of input, expected pitch or dash to start beat".to_string(),
                line: line_num,
                column,
            });
        }
    }

    // Continue parsing beat-elements
    loop {
        match chars.peek() {
            // Beat terminators
            Some(&' ') | Some(&'|') | Some(&'\n') | None => break,

            // Dash
            Some(&'-') => {
                chars.next();
                elements.push(BeatElement::Dash(Dash {
                    source: Source {
                        value: Some("-".to_string()),
                        position: Position {
                            line: line_num,
                            column,
                            index_in_line,
                            index_in_doc: start_index_in_doc + index_in_line,
                        },
                    },
                }));
                column += 1;
                index_in_line += 1;
                total_consumed += 1;
            }

            // Breath mark
            Some(&',') => {
                chars.next();
                elements.push(BeatElement::BreathMark(BreathMark {
                    source: Source {
                        value: Some(",".to_string()),
                        position: Position {
                            line: line_num,
                            column,
                            index_in_line,
                            index_in_doc: start_index_in_doc + index_in_line,
                        },
                    },
                }));
                column += 1;
                index_in_line += 1;
                total_consumed += 1;
            }

            // Another pitch
            Some(&ch) if is_pitch_start(ch, notation_system) => {
                let (pitch_str, pitch_code, consumed) = parse_pitch(chars, notation_system, line_num, column)?;

                elements.push(BeatElement::Note(Note {
                    pitch_string: PitchString {
                        source: Source {
                            value: Some(pitch_str),
                            position: Position {
                                line: line_num,
                                column,
                                index_in_line,
                                index_in_doc: start_index_in_doc + index_in_line,
                            },
                        },
                    },
                    pitch_code,
                    octave: 0,
                    notation_system,
                }));

                column += consumed;
                index_in_line += consumed;
                total_consumed += consumed;
            }

            // Unknown character ends the beat
            Some(_) => break,
        }
    }

    let beat = Beat {
        elements,
        source: Source {
            value: None, // Will be filled by caller if needed
            position: Position {
                line: line_num,
                column: start_column,
                index_in_line: start_index_in_line,
                index_in_doc: start_index_in_doc,
            },
        },
    };

    Ok((beat, total_consumed))
}