use crate::parse::model::{NotationSystem, PitchCode};
use crate::parse::ParseError;
use std::iter::Peekable;
use std::str::{Chars, CharIndices};

/// Parse a pitch according to the grammar:
/// pitch = note_in_system
/// note_in_system = sargam_note | number_note | western_note | tabla_note | hindi_note
///
/// Pitches are atomic units including accidentals (e.g., "1", "1#", "1b", "C#", "S", etc.)
pub fn parse_pitch(
    chars: &mut Peekable<Chars>,
    notation_system: NotationSystem,
    line: usize,
    column: usize,
) -> Result<(String, PitchCode, usize), ParseError> {
    let first_char = chars.next().ok_or_else(|| {
        ParseError {
            message: "Unexpected end of input, expected pitch".to_string(),
            line,
            column,
        }
    })?;

    match notation_system {
        NotationSystem::Number => parse_number_pitch(first_char, chars, line, column),
        NotationSystem::Western => parse_western_pitch(first_char, chars, line, column),
        NotationSystem::Sargam => parse_sargam_pitch(first_char, chars, line, column),
        NotationSystem::Bhatkhande => parse_bhatkhande_pitch(first_char, chars, line, column),
        NotationSystem::Tabla => parse_tabla_pitch(first_char, chars, line, column),
    }
}

/// Check if a character can start a pitch in the given notation system
pub fn is_pitch_start(ch: char, notation_system: NotationSystem) -> bool {
    match notation_system {
        NotationSystem::Number => matches!(ch, '1'..='7'),
        NotationSystem::Western => matches!(ch, 'A'..='G' | 'a'..='g'),
        NotationSystem::Sargam => matches!(ch, 'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N' | 's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n'),
        NotationSystem::Bhatkhande => matches!(ch, 'स' | 'र' | 'ग' | 'म' | 'प' | 'ध' | 'न'),
        NotationSystem::Tabla => matches!(ch, 'd' | 'D' | 't' | 'T' | 'k' | 'K' | 'g' | 'G'),
    }
}

/// Parse a pitch using CharIndices and regex patterns from models
pub fn parse_pitch_with_indices(
    chars: &mut Peekable<CharIndices>,
    notation_system: NotationSystem,
    line: usize,
    input: &str,
) -> Result<(String, PitchCode), ParseError> {
    // Get the starting position
    let start_pos = chars.peek().map(|(pos, _)| *pos).unwrap_or(0);
    let remaining_input = &input[start_pos..];

    // Use regex from models to find pitch match
    let regex = crate::models::pitch_systems::get_regex_for_system(notation_system);

    if let Some(mat) = regex.find(remaining_input) {
        let pitch_str = mat.as_str();

        // Use models lookup to get the Degree
        let notation = system_to_notation(notation_system);
        let degree = crate::models::pitch_systems::lookup_pitch(pitch_str, notation)
            .ok_or_else(|| ParseError {
                message: format!("Invalid pitch '{}' for notation system {:?}", pitch_str, notation_system),
                line,
                column: column_from_pos(input, start_pos),
            })?;

        // Convert Degree to PitchCode using the bridge
        let pitch_code = crate::models::pitch_systems::degree_to_pitch_code(degree);

        // Advance CharIndices by match length
        for _ in 0..pitch_str.len() {
            chars.next();
        }

        Ok((pitch_str.to_string(), pitch_code))
    } else {
        Err(ParseError {
            message: format!("Expected pitch for notation system {:?}", notation_system),
            line,
            column: column_from_pos(input, start_pos),
        })
    }
}

/// Helper function to convert NotationSystem to Notation
fn system_to_notation(system: NotationSystem) -> crate::models::Notation {
    match system {
        NotationSystem::Western => crate::models::Notation::Western,
        NotationSystem::Number => crate::models::Notation::Number,
        NotationSystem::Sargam => crate::models::Notation::Sargam,
        NotationSystem::Bhatkhande => crate::models::Notation::Bhatkhande,
        NotationSystem::Tabla => crate::models::Notation::Tabla,
    }
}

/// Helper function to calculate column from position in input
fn column_from_pos(input: &str, pos: usize) -> usize {
    input[..pos].chars().rev().take_while(|&c| c != '\n').count() + 1
}

/// Parse number notation: 1, 2, 3, 4, 5, 6, 7 with optional accidentals
fn parse_number_pitch(
    base: char,
    chars: &mut Peekable<Chars>,
    line: usize,
    column: usize,
) -> Result<(String, PitchCode, usize), ParseError> {
    if !matches!(base, '1'..='7') {
        return Err(ParseError {
            message: format!("Expected number pitch (1-7), found '{}'", base),
            line,
            column,
        });
    }

    let mut pitch_str = String::from(base);
    let mut length = 1;

    // Check for accidentals
    if let Some(&'#') = chars.peek() {
        pitch_str.push(chars.next().unwrap());
        length += 1;
        if let Some(&'#') = chars.peek() {
            pitch_str.push(chars.next().unwrap());
            length += 1;
        }
    } else if let Some(&'b') = chars.peek() {
        pitch_str.push(chars.next().unwrap());
        length += 1;
        if let Some(&'b') = chars.peek() {
            pitch_str.push(chars.next().unwrap());
            length += 1;
        }
    }

    let pitch_code = PitchCode::from_source(&pitch_str).ok_or_else(|| ParseError {
        message: format!("Invalid number pitch: {}", pitch_str),
        line,
        column,
    })?;
    Ok((pitch_str, pitch_code, length))
}

/// Parse Western notation: C, D, E, F, G, A, B with optional accidentals
fn parse_western_pitch(
    base: char,
    chars: &mut Peekable<Chars>,
    line: usize,
    column: usize,
) -> Result<(String, PitchCode, usize), ParseError> {
    let base_upper = base.to_ascii_uppercase();
    if !matches!(base_upper, 'A'..='G') {
        return Err(ParseError {
            message: format!("Expected Western pitch (A-G), found '{}'", base),
            line,
            column,
        });
    }

    let mut pitch_str = String::from(base_upper);
    let mut length = 1;

    // Check for accidentals
    if let Some(&'#') = chars.peek() {
        pitch_str.push(chars.next().unwrap());
        length += 1;
        if let Some(&'#') = chars.peek() {
            pitch_str.push(chars.next().unwrap());
            length += 1;
        }
    } else if let Some(&'b') = chars.peek() {
        pitch_str.push(chars.next().unwrap());
        length += 1;
        if let Some(&'b') = chars.peek() {
            pitch_str.push(chars.next().unwrap());
            length += 1;
        }
    }

    let pitch_code = PitchCode::from_source(&pitch_str).ok_or_else(|| ParseError {
        message: format!("Invalid Western pitch: {}", pitch_str),
        line,
        column,
    })?;
    Ok((pitch_str, pitch_code, length))
}

/// Parse Sargam notation: S, R, G, M, P, D, N (uppercase/lowercase variants)
fn parse_sargam_pitch(
    base: char,
    chars: &mut Peekable<Chars>,
    line: usize,
    column: usize,
) -> Result<(String, PitchCode, usize), ParseError> {
    let valid_sargam = matches!(base, 'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N' | 's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n');
    if !valid_sargam {
        return Err(ParseError {
            message: format!("Expected Sargam pitch (S,R,G,M,P,D,N), found '{}'", base),
            line,
            column,
        });
    }

    let mut pitch_str = String::from(base);
    let mut length = 1;

    // Some Sargam notes support accidentals
    if matches!(base.to_ascii_uppercase(), 'S' | 'R' | 'M' | 'P' | 'N') {
        if let Some(&'#') = chars.peek() {
            pitch_str.push(chars.next().unwrap());
            length += 1;
            if let Some(&'#') = chars.peek() {
                pitch_str.push(chars.next().unwrap());
                length += 1;
            }
        } else if let Some(&'b') = chars.peek() {
            pitch_str.push(chars.next().unwrap());
            length += 1;
            if let Some(&'b') = chars.peek() {
                pitch_str.push(chars.next().unwrap());
                length += 1;
            }
        }
    }

    let pitch_code = PitchCode::from_source_with_context(&pitch_str, NotationSystem::Sargam).ok_or_else(|| ParseError {
        message: format!("Invalid Sargam pitch: {}", pitch_str),
        line,
        column,
    })?;
    Ok((pitch_str, pitch_code, length))
}

/// Parse Bhatkhande/Hindi notation: स, र, ग, म, प, ध, न
fn parse_bhatkhande_pitch(
    base: char,
    chars: &mut Peekable<Chars>,
    line: usize,
    column: usize,
) -> Result<(String, PitchCode, usize), ParseError> {
    // Check if it's a valid Bhatkhande character
    if !matches!(base, 'स' | 'र' | 'ग' | 'म' | 'प' | 'ध' | 'न') {
        // Some Bhatkhande notes are two characters
        if base == 'र' && chars.peek() == Some(&'े') {
            let mut pitch_str = String::from(base);
            pitch_str.push(chars.next().unwrap());
            let pitch_code = PitchCode::from_source(&pitch_str).ok_or_else(|| ParseError {
                message: format!("Invalid Bhatkhande pitch: {}", pitch_str),
                line,
                column,
            })?;
            return Ok((pitch_str, pitch_code, 2));
        } else if base == 'न' && chars.peek() == Some(&'ि') {
            let mut pitch_str = String::from(base);
            pitch_str.push(chars.next().unwrap());
            let pitch_code = PitchCode::from_source(&pitch_str).ok_or_else(|| ParseError {
                message: format!("Invalid Bhatkhande pitch: {}", pitch_str),
                line,
                column,
            })?;
            return Ok((pitch_str, pitch_code, 2));
        }

        return Err(ParseError {
            message: format!("Expected Bhatkhande pitch, found '{}'", base),
            line,
            column,
        });
    }

    let pitch_str = String::from(base);
    let pitch_code = PitchCode::from_source(&pitch_str).ok_or_else(|| ParseError {
        message: format!("Invalid Bhatkhande pitch: {}", pitch_str),
        line,
        column,
    })?;
    Ok((pitch_str, pitch_code, 1))
}

/// Parse Tabla notation: dha, dhin, ta, ka, taka, trkt, ge
fn parse_tabla_pitch(
    first: char,
    chars: &mut Peekable<Chars>,
    line: usize,
    column: usize,
) -> Result<(String, PitchCode, usize), ParseError> {
    let mut pitch_str = String::from(first);
    let mut length = 1;

    match first.to_ascii_lowercase() {
        'd' => {
            // Could be "dha" or "dhin"
            if let Some(&next) = chars.peek() {
                if next.to_ascii_lowercase() == 'h' {
                    pitch_str.push(chars.next().unwrap());
                    length += 1;
                    if let Some(&next2) = chars.peek() {
                        match next2.to_ascii_lowercase() {
                            'a' => {
                                pitch_str.push(chars.next().unwrap());
                                length += 1;
                            }
                            'i' => {
                                pitch_str.push(chars.next().unwrap());
                                length += 1;
                                if chars.peek() == Some(&'n') || chars.peek() == Some(&'N') {
                                    pitch_str.push(chars.next().unwrap());
                                    length += 1;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        't' => {
            // Could be "ta", "taka", or "trkt"
            if let Some(&next) = chars.peek() {
                match next.to_ascii_lowercase() {
                    'a' => {
                        pitch_str.push(chars.next().unwrap());
                        length += 1;
                        // Check for "taka"
                        if chars.peek() == Some(&'k') || chars.peek() == Some(&'K') {
                            pitch_str.push(chars.next().unwrap());
                            length += 1;
                            if chars.peek() == Some(&'a') || chars.peek() == Some(&'A') {
                                pitch_str.push(chars.next().unwrap());
                                length += 1;
                            }
                        }
                    }
                    'r' => {
                        // Check for "trkt"
                        pitch_str.push(chars.next().unwrap());
                        length += 1;
                        if chars.peek() == Some(&'k') || chars.peek() == Some(&'K') {
                            pitch_str.push(chars.next().unwrap());
                            length += 1;
                            if chars.peek() == Some(&'t') || chars.peek() == Some(&'T') {
                                pitch_str.push(chars.next().unwrap());
                                length += 1;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        'k' => {
            // Could be "ka"
            if chars.peek() == Some(&'a') || chars.peek() == Some(&'A') {
                pitch_str.push(chars.next().unwrap());
                length += 1;
            }
        }
        'g' => {
            // Could be "ge"
            if chars.peek() == Some(&'e') || chars.peek() == Some(&'E') {
                pitch_str.push(chars.next().unwrap());
                length += 1;
            }
        }
        _ => {
            return Err(ParseError {
                message: format!("Expected Tabla syllable, found '{}'", first),
                line,
                column,
            });
        }
    }

    // All tabla syllables map to N1 (tonic)
    let pitch_code = PitchCode::N1;
    Ok((pitch_str, pitch_code, length))
}