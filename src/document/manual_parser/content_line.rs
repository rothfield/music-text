use crate::document::model::{ContentLine, MusicalElement, NotationSystem, Source, Position, PitchCode, Note};
use super::error::ParseError;

/// Parse a content line into musical elements
pub fn parse_content_line(line: &str, line_num: usize) -> Result<ContentLine, ParseError> {
    let mut elements = Vec::new();
    let mut col = 1;
    
    for ch in line.chars() {
        let element = match ch {
            '|' => MusicalElement::Barline {
                source: Source {
                    value: ch.to_string(),
                    position: Position { line: line_num, column: col },
                },
                in_slur: false,
                in_beat_group: false,
            },
            '1'..='7' => {
                let pitch_code = match ch {
                    '1' => PitchCode::N1,
                    '2' => PitchCode::N2, 
                    '3' => PitchCode::N3,
                    '4' => PitchCode::N4,
                    '5' => PitchCode::N5,
                    '6' => PitchCode::N6,
                    '7' => PitchCode::N7,
                    _ => unreachable!(),
                };
                MusicalElement::Note(Note {
                    syllable: ch.to_string(),
                    octave: 0,
                    pitch_code,
                    notation_system: NotationSystem::Number,
                    source: Source {
                        value: ch.to_string(),
                        position: Position { line: line_num, column: col },
                    },
                    in_slur: false,
                    in_beat_group: false,
                })
            },
            // Western notation: C D E F G A B
            'C' | 'D' | 'E' | 'F' | 'G' | 'A' | 'B' => {
                let pitch_code = PitchCode::from_source(&ch.to_string());
                MusicalElement::Note(Note {
                    syllable: ch.to_string(),
                    octave: 0,
                    pitch_code,
                    notation_system: NotationSystem::Western,
                    source: Source {
                        value: ch.to_string(),
                        position: Position { line: line_num, column: col },
                    },
                    in_slur: false,
                    in_beat_group: false,
                })
            },
            // Sargam notation: S R G M P D N (both cases)
            'S' | 's' | 'R' | 'r' | 'M' | 'm' | 'P' | 'p' | 'N' | 'n' => {
                let pitch_code = PitchCode::from_source(&ch.to_string());
                MusicalElement::Note(Note {
                    syllable: ch.to_string(),
                    octave: 0,
                    pitch_code,
                    notation_system: NotationSystem::Sargam,
                    source: Source {
                        value: ch.to_string(),
                        position: Position { line: line_num, column: col },
                    },
                    in_slur: false,
                    in_beat_group: false,
                })
            },
            // Handle lowercase sargam 'g' and 'd' separately (they're komal variants)
            'g' | 'd' => {
                let pitch_code = PitchCode::from_source(&ch.to_string());
                MusicalElement::Note(Note {
                    syllable: ch.to_string(),
                    octave: 0,
                    pitch_code,
                    notation_system: NotationSystem::Sargam,
                    source: Source {
                        value: ch.to_string(),
                        position: Position { line: line_num, column: col },
                    },
                    in_slur: false,
                    in_beat_group: false,
                })
            },
            '-' => MusicalElement::Dash {
                source: Source {
                    value: ch.to_string(),
                    position: Position { line: line_num, column: col },
                },
                in_slur: false,
                in_beat_group: false,
            },
            ' ' => MusicalElement::Space {
                count: 1,
                in_slur: false,
                in_beat_group: false,
                source: Source {
                    value: ch.to_string(),
                    position: Position { line: line_num, column: col },
                },
            },
            _ => {
                // Skip unrecognized characters for now
                col += 1;
                continue;
            }
        };
        
        elements.push(element);
        col += 1;
    }
    
    Ok(ContentLine {
        elements,
        source: Source {
            value: line.to_string(),
            position: Position { line: line_num, column: 1 },
        },
    })
}

/// Check if a line contains musical content
pub fn is_content_line(line: &str) -> bool {
    let trimmed = line.trim();
    
    // Has barline
    if trimmed.contains('|') {
        return true;
    }
    
    // Check for musical pitches without barlines (need at least 3 pitches)
    let pitch_count = count_musical_pitches(trimmed);
    pitch_count >= 3
}

/// Count musical pitches in a line
pub fn count_musical_pitches(line: &str) -> usize {
    let mut count = 0;
    let chars: Vec<char> = line.chars().collect();
    
    for i in 0..chars.len() {
        let ch = chars[i];
        
        // Number pitches: 1-7
        if ch.is_ascii_digit() && ch >= '1' && ch <= '7' {
            count += 1;
            continue;
        }
        
        // Western pitches: C D E F G A B
        if matches!(ch, 'C' | 'D' | 'E' | 'F' | 'G' | 'A' | 'B') {
            count += 1;
            continue;
        }
        
        // Sargam pitches: S R G M P D N (both cases)
        if matches!(ch, 'S' | 's' | 'R' | 'r' | 'G' | 'g' | 'M' | 'm' | 'P' | 'p' | 'D' | 'd' | 'N' | 'n') {
            count += 1;
            continue;
        }
        
        // Tabla syllables (simplified check - just look for common starts)
        if i + 2 < chars.len() {
            let three_char: String = chars[i..i+3].iter().collect();
            if matches!(three_char.to_lowercase().as_str(), "dhi" | "dha" | "trk" | "tak") {
                count += 1;
                continue;
            }
        }
    }
    
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_content_line() {
        assert!(is_content_line("|123"));
        assert!(is_content_line("1 2 3"));
        assert!(is_content_line("S R G M"));
        assert!(!is_content_line("1 2")); // Only 2 pitches
        assert!(!is_content_line("____"));
        assert!(!is_content_line("text line"));
    }

    #[test]
    fn test_count_pitches() {
        assert_eq!(count_musical_pitches("123"), 3);
        assert_eq!(count_musical_pitches("1 2 3"), 3);
        assert_eq!(count_musical_pitches("SRG"), 3);
        assert_eq!(count_musical_pitches("C D E"), 3);
        assert_eq!(count_musical_pitches("12"), 2);
        assert_eq!(count_musical_pitches("____"), 0);
    }
}