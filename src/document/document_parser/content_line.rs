use crate::document::model::PitchCode;
use crate::old_models::{ParsedElement, Degree, Position as OldPosition};
use super::error::ParseError;

// Old ContentLine function removed - using new ParsedElement architecture

/// Check if a line contains musical content
pub fn is_content_line(line: &str) -> bool {
    let trimmed = line.trim();
    
    // Has barline
    if trimmed.contains('|') {
        return true;
    }
    
    // Check for musical elements without barlines (need at least 3 musical elements: pitches or dashes)
    let element_count = count_musical_elements(trimmed);
    element_count >= 3
}

/// Count musical elements in a line (pitches AND dashes)
pub fn count_musical_elements(line: &str) -> usize {
    let mut count = 0;
    let chars: Vec<char> = line.chars().collect();
    
    for i in 0..chars.len() {
        let ch = chars[i];
        
        // Dashes (note extensions)
        if ch == '-' {
            count += 1;
            continue;
        }
        
        // Number pitches: 1-7 - avoid counting digits that are clearly part of numbers
        if ch.is_ascii_digit() && ch >= '1' && ch <= '7' && !is_part_of_number(&chars, i) {
            count += 1;
            continue;
        }
        
        // Western pitches: C D E F G A B - avoid counting letters that are part of words
        if matches!(ch, 'C' | 'D' | 'E' | 'F' | 'G' | 'A' | 'B') && !is_part_of_english_word(&chars, i) {
            count += 1;
            continue;
        }
        
        // Sargam pitches: S R G M P D N (both cases) - avoid counting letters that are part of words
        if matches!(ch, 'S' | 's' | 'R' | 'r' | 'G' | 'g' | 'M' | 'm' | 'P' | 'p' | 'D' | 'd' | 'N' | 'n') && !is_part_of_english_word(&chars, i) {
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

/// Check if a digit is part of a multi-digit number (like "96" in "tempo: 96")
fn is_part_of_number(chars: &[char], i: usize) -> bool {
    let prev_char = if i > 0 { Some(chars[i - 1]) } else { None };
    let next_char = if i + 1 < chars.len() { Some(chars[i + 1]) } else { None };
    
    // If the digit has non-musical digits (0, 8, 9) adjacent to it, it's part of a number
    if prev_char.map_or(false, |c| c == '0' || c == '8' || c == '9') {
        return true;
    }
    if next_char.map_or(false, |c| c == '0' || c == '8' || c == '9') {
        return true;
    }
    
    false
}

/// Check if a letter is part of an English word (like "m" in "tempo" or "p" in "tempo")
fn is_part_of_english_word(chars: &[char], i: usize) -> bool {
    let prev_char = if i > 0 { Some(chars[i - 1]) } else { None };
    let next_char = if i + 1 < chars.len() { Some(chars[i + 1]) } else { None };
    
    // If surrounded by lowercase letters, it's likely part of an English word
    let prev_is_lowercase = prev_char.map_or(false, |c| c.is_ascii_lowercase());
    let next_is_lowercase = next_char.map_or(false, |c| c.is_ascii_lowercase());
    
    prev_is_lowercase || next_is_lowercase
}

/// Count musical pitches in a line (legacy function - kept for compatibility)
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
        // With barlines - always content lines
        assert!(is_content_line("|123"));
        
        // Without barlines - need 3+ musical elements (pitches or dashes)
        assert!(is_content_line("1 2 3"));      // 3 pitches
        assert!(is_content_line("S R G M"));     // 4 pitches
        assert!(is_content_line("1--"));         // 1 pitch + 2 dashes = 3 elements
        assert!(is_content_line("1-2"));         // 1 pitch + 1 dash + 1 pitch = 3 elements
        assert!(is_content_line("---"));         // 3 dashes
        
        // Not content lines
        assert!(!is_content_line("1 2"));        // Only 2 elements
        assert!(!is_content_line("--"));         // Only 2 dashes
        assert!(!is_content_line("____"));       // Underscores (not dashes)
        assert!(!is_content_line("text line"));  // No musical elements
    }

    #[test]
    fn test_count_musical_elements() {
        // Pure pitches
        assert_eq!(count_musical_elements("123"), 3);
        assert_eq!(count_musical_elements("1 2 3"), 3);
        assert_eq!(count_musical_elements("SRG"), 3);
        assert_eq!(count_musical_elements("C D E"), 3);
        
        // Mixed pitches and dashes
        assert_eq!(count_musical_elements("1--"), 3);      // 1 pitch + 2 dashes
        assert_eq!(count_musical_elements("1-2"), 3);      // 1 pitch + 1 dash + 1 pitch
        assert_eq!(count_musical_elements("1-2-3"), 5);    // 3 pitches + 2 dashes
        
        // Pure dashes
        assert_eq!(count_musical_elements("---"), 3);      // 3 dashes
        assert_eq!(count_musical_elements("--"), 2);       // 2 dashes
        
        // Non-musical elements
        assert_eq!(count_musical_elements("12"), 2);       // 2 pitches
        assert_eq!(count_musical_elements("____"), 0);     // Underscores don't count
        assert_eq!(count_musical_elements("text"), 0);     // No musical elements
    }

    #[test]
    fn test_count_pitches() {
        // Legacy function - only counts pitches, not dashes
        assert_eq!(count_musical_pitches("123"), 3);
        assert_eq!(count_musical_pitches("1 2 3"), 3);
        assert_eq!(count_musical_pitches("SRG"), 3);
        assert_eq!(count_musical_pitches("C D E"), 3);
        assert_eq!(count_musical_pitches("12"), 2);
        assert_eq!(count_musical_pitches("____"), 0);
        
        // Dashes are NOT counted by this legacy function
        assert_eq!(count_musical_pitches("1--"), 1);       // Only counts the pitch
        assert_eq!(count_musical_pitches("---"), 0);       // No pitches
    }
}

/// Phase 2: Parse content line text into ParsedElements (tokenization)
/// This is the new architecture: text -> ParsedElement directly
pub fn parse_content_line(line: &str, line_num: usize) -> Result<Vec<ParsedElement>, ParseError> {
    let mut elements = Vec::new();
    let mut col = 1;
    
    for ch in line.chars() {
        let element = match ch {
            '|' => ParsedElement::Barline {
                style: "|".to_string(),
                position: OldPosition {
                    row: line_num,
                    col,
                },
                tala: None,
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
                let degree = convert_pitchcode_to_degree(pitch_code);
                
                ParsedElement::Note {
                    degree,
                    octave: 0, // Default octave, will be updated by spatial processing
                    value: ch.to_string(),
                    position: OldPosition {
                        row: line_num,
                        col,
                    },
                    children: vec![], // No children at content line level
                    duration: None, // Will be set by FSM
                    slur: None, // Will be set by spatial processing
                }
            },
            // Western notation: C D E F G A B
            'C' | 'D' | 'E' | 'F' | 'G' | 'A' | 'B' => {
                let pitch_code = PitchCode::from_source(&ch.to_string());
                let degree = convert_pitchcode_to_degree(pitch_code);
                
                ParsedElement::Note {
                    degree,
                    octave: 0,
                    value: ch.to_string(),
                    position: OldPosition {
                        row: line_num,
                        col,
                    },
                    children: vec![],
                    duration: None,
                    slur: None,
                }
            },
            // Sargam notation: S R G M P D N (both cases)
            'S' | 's' | 'R' | 'r' | 'M' | 'm' | 'P' | 'p' | 'N' | 'n' => {
                let pitch_code = PitchCode::from_source(&ch.to_string());
                let degree = convert_pitchcode_to_degree(pitch_code);
                
                ParsedElement::Note {
                    degree,
                    octave: 0,
                    value: ch.to_string(),
                    position: OldPosition {
                        row: line_num,
                        col,
                    },
                    children: vec![],
                    duration: None,
                    slur: None,
                }
            },
            // Handle lowercase sargam 'g' and 'd' separately (they're komal variants)
            'g' | 'd' => {
                let pitch_code = PitchCode::from_source(&ch.to_string());
                let degree = convert_pitchcode_to_degree(pitch_code);
                
                ParsedElement::Note {
                    degree,
                    octave: 0,
                    value: ch.to_string(),
                    position: OldPosition {
                        row: line_num,
                        col,
                    },
                    children: vec![],
                    duration: None,
                    slur: None,
                }
            },
            '-' => ParsedElement::Dash {
                degree: None, // Will be inherited from previous note
                octave: None, // Will be inherited from previous note
                position: OldPosition {
                    row: line_num,
                    col,
                },
                duration: None, // Will be set by FSM
            },
            ' ' => ParsedElement::Whitespace {
                value: " ".to_string(),
                position: OldPosition {
                    row: line_num,
                    col,
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
    
    Ok(elements)
}

/// Convert new PitchCode to old Degree format (from rhythm_fsm.rs)
fn convert_pitchcode_to_degree(pitch_code: PitchCode) -> Degree {
    match pitch_code {
        PitchCode::N1bb => Degree::N1bb,
        PitchCode::N1b => Degree::N1b,
        PitchCode::N1 => Degree::N1,
        PitchCode::N1s => Degree::N1s,
        PitchCode::N1ss => Degree::N1ss,
        PitchCode::N2bb => Degree::N2bb,
        PitchCode::N2b => Degree::N2b,
        PitchCode::N2 => Degree::N2,
        PitchCode::N2s => Degree::N2s,
        PitchCode::N2ss => Degree::N2ss,
        PitchCode::N3bb => Degree::N3bb,
        PitchCode::N3b => Degree::N3b,
        PitchCode::N3 => Degree::N3,
        PitchCode::N3s => Degree::N3s,
        PitchCode::N3ss => Degree::N3ss,
        PitchCode::N4bb => Degree::N4bb,
        PitchCode::N4b => Degree::N4b,
        PitchCode::N4 => Degree::N4,
        PitchCode::N4s => Degree::N4s,
        PitchCode::N4ss => Degree::N4ss,
        PitchCode::N5bb => Degree::N5bb,
        PitchCode::N5b => Degree::N5b,
        PitchCode::N5 => Degree::N5,
        PitchCode::N5s => Degree::N5s,
        PitchCode::N5ss => Degree::N5ss,
        PitchCode::N6bb => Degree::N6bb,
        PitchCode::N6b => Degree::N6b,
        PitchCode::N6 => Degree::N6,
        PitchCode::N6s => Degree::N6s,
        PitchCode::N6ss => Degree::N6ss,
        PitchCode::N7bb => Degree::N7bb,
        PitchCode::N7b => Degree::N7b,
        PitchCode::N7 => Degree::N7,
        PitchCode::N7s => Degree::N7s,
        PitchCode::N7ss => Degree::N7ss,
    }
}