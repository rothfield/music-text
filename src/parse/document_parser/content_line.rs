use crate::parse::model::{PitchCode, NotationSystem};
use crate::rhythm::types::{ParsedElement, Degree, Position};
use super::error::ParseError;
use crate::tokenizer::classify_and_tokenize;
use crate::models::pitch_systems::tabla;

// Old ContentLine function removed - using new ParsedElement architecture

/// Check if a character is valid for the given notation system
fn is_valid_pitch_for_system(ch: char, system: NotationSystem) -> bool {
    match system {
        NotationSystem::Number => matches!(ch, '1'..='7'),
        NotationSystem::Western => matches!(ch, 'C' | 'D' | 'E' | 'F' | 'G' | 'A' | 'B'),
        NotationSystem::Sargam => matches!(ch, 'S' | 's' | 'R' | 'r' | 'G' | 'g' | 'M' | 'm' | 'P' | 'p' | 'D' | 'd' | 'N' | 'n'),
        NotationSystem::Bhatkhande => matches!(ch, 'स' | 'र' | 'ग' | 'म' | 'प' | 'ध' | 'द' | 'न'),
        NotationSystem::Tabla => false, // Tabla uses multi-character tokens, handled separately
    }
}

/// Detect the notation system for a single line using the same logic as the model
pub fn detect_line_notation_system(line: &str) -> NotationSystem {
    // Count occurrences of each notation system's unique characters
    let mut votes = [0; 5]; // [Number, Western, Sargam, Bhatkhande, Tabla]
    
    // Check for tabla bols (multi-character patterns)
    let tabla_bols = ["dha", "ge", "na", "ka", "ta", "trka", "terekita", "dhin"];
    for bol in &tabla_bols {
        if line.contains(bol) {
            votes[4] += bol.len(); // Weight by length for longer bols
        }
    }
    
    for ch in line.chars() {
        match ch {
            '1'..='7' => votes[0] += 1, // Number
            'C' | 'E' | 'F' | 'A' | 'B' => votes[1] += 1, // Clearly Western
            's' | 'r' | 'g' | 'm' | 'n' | 'd' | 'p' => votes[2] += 1, // Clearly Sargam
            'स' | 'र' | 'ग' | 'म' | 'प' | 'ध' | 'द' | 'न' => votes[3] += 1, // Bhatkhande
            _ => {}
        }
    }
    
    // Handle ambiguous characters by context
    for ch in line.chars() {
        match ch {
            'G' | 'D' | 'R' | 'M' | 'P' | 'N' => {
                // If we already have strong Sargam indicators, count as Sargam
                if votes[2] > 0 {
                    votes[2] += 1;
                } else {
                    votes[1] += 1; // Default to Western
                }
            }
            'S' => votes[2] += 1, // S is always Sargam (no Western equivalent)
            _ => {}
        }
    }
    
    // Return the system with the most votes
    let max_idx = votes.iter().position(|&x| x == *votes.iter().max().unwrap()).unwrap();
    match max_idx {
        0 => NotationSystem::Number,
        1 => NotationSystem::Western,
        2 => NotationSystem::Sargam,
        3 => NotationSystem::Bhatkhande,
        4 => NotationSystem::Tabla,
        _ => NotationSystem::Number, // Default
    }
}

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
    let mut i = 0;
    
    while i < chars.len() {
        let ch = chars[i];
        
        // Dashes (note extensions)
        if ch == '-' {
            count += 1;
            i += 1;
            continue;
        }
        
        // Number pitches: 1-7 - avoid counting digits that are clearly part of numbers
        if ch.is_ascii_digit() && ch >= '1' && ch <= '7' && !is_part_of_number(&chars, i) {
            count += 1;
            i += 1;
            continue;
        }
        
        // Western pitches: C D E F G A B - avoid counting letters that are part of words
        if matches!(ch, 'C' | 'D' | 'E' | 'F' | 'G' | 'A' | 'B') && !is_part_of_english_word(&chars, i) {
            count += 1;
            i += 1;
            continue;
        }
        
        // Sargam pitches: S R G M P D N (both cases) - avoid counting letters that are part of words
        if matches!(ch, 'S' | 's' | 'R' | 'r' | 'G' | 'g' | 'M' | 'm' | 'P' | 'p' | 'D' | 'd' | 'N' | 'n') && !is_part_of_english_word(&chars, i) {
            count += 1;
            i += 1;
            continue;
        }
        
        // Tabla syllables - check for all valid tabla bols
        let remaining_chars = &chars[i..];
        let remaining_str: String = remaining_chars.iter().collect();
        
        // Check for tabla syllables using the same logic as the tabla module
        let tabla_bols = ["terekita", "trka", "dhin", "dha", "ge", "na", "ka", "ta"];
        let mut found_bol = false;
        for bol in &tabla_bols {
            if remaining_str.to_lowercase().starts_with(&bol.to_lowercase()) {
                count += 1;
                i += bol.len(); // Move past this tabla syllable
                found_bol = true;
                break;
            }
        }
        
        if !found_bol {
            i += 1; // Move to next character if no tabla syllable found
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
    
    // If surrounded by lowercase letters that are NOT musical notes, it's likely part of an English word
    let prev_is_non_musical_lowercase = prev_char.map_or(false, |c| 
        c.is_ascii_lowercase() && !matches!(c, 's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n')
    );
    let next_is_non_musical_lowercase = next_char.map_or(false, |c| 
        c.is_ascii_lowercase() && !matches!(c, 's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n')
    );
    
    prev_is_non_musical_lowercase || next_is_non_musical_lowercase
}

/// Count musical pitches in a line (legacy function - kept for compatibility)
pub fn count_musical_pitches(line: &str) -> usize {
    let mut count = 0;
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        let ch = chars[i];
        
        // Number pitches: 1-7
        if ch.is_ascii_digit() && ch >= '1' && ch <= '7' {
            count += 1;
            i += 1;
            continue;
        }
        
        // Western pitches: C D E F G A B
        if matches!(ch, 'C' | 'D' | 'E' | 'F' | 'G' | 'A' | 'B') {
            count += 1;
            i += 1;
            continue;
        }
        
        // Sargam pitches: S R G M P D N (both cases)
        if matches!(ch, 'S' | 's' | 'R' | 'r' | 'G' | 'g' | 'M' | 'm' | 'P' | 'p' | 'D' | 'd' | 'N' | 'n') {
            count += 1;
            i += 1;
            continue;
        }
        
        // Tabla syllables - check for all valid tabla bols
        let remaining_chars = &chars[i..];
        let remaining_str: String = remaining_chars.iter().collect();
        
        // Check for tabla syllables using the same logic as the tabla module
        let tabla_bols = ["terekita", "trka", "dhin", "dha", "ge", "na", "ka", "ta"];
        let mut found_bol = false;
        for bol in &tabla_bols {
            if remaining_str.to_lowercase().starts_with(&bol.to_lowercase()) {
                count += 1;
                i += bol.len(); // Move past this tabla syllable
                found_bol = true;
                break;
            }
        }
        
        if !found_bol {
            i += 1; // Move to next character if no tabla syllable found
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
        
        // Tabla content lines - regression test for "takata" bug
        assert!(is_content_line("ta ka ta"));    // Should recognize as content line
        assert!(is_content_line("takata"));      // Compact notation should also work
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
        
        // Tabla syllables - regression test for "takata" bug
        assert_eq!(count_musical_elements("ta ka ta"), 3); // Should count all 3 tabla syllables
        assert_eq!(count_musical_elements("takata"), 3);   // Compact notation should also work
        assert_eq!(count_musical_elements("dha ge na"), 3); // Different tabla syllables
        assert_eq!(count_musical_elements("ka"), 1);       // Single tabla syllable should count
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
        
        // Tabla syllables
        assert_eq!(count_musical_pitches("ta ka ta"), 3);  // Should count all 3 tabla syllables
        assert_eq!(count_musical_pitches("takata"), 3);    // Compact notation should also work
        assert_eq!(count_musical_pitches("dha ge na"), 3); // Different tabla syllables
    }
}

/// Phase 2: Parse content line text into ParsedElements (tokenization)
/// This is the new architecture: text -> ParsedElement directly
pub fn parse_content_line(line: &str, line_num: usize, notation_system: NotationSystem) -> Result<Vec<ParsedElement>, ParseError> {
    let mut elements = Vec::new();
    
    // Get tokens for the specific notation system only (longest first)
    let tokens = match notation_system {
        NotationSystem::Tabla => crate::models::pitch_systems::tabla::get_all_symbols(),
        NotationSystem::Number => crate::models::pitch_systems::number::get_all_symbols(),
        NotationSystem::Western => crate::models::pitch_systems::western::get_all_symbols(),
        NotationSystem::Sargam => crate::models::pitch_systems::sargam::get_all_symbols(),
        NotationSystem::Bhatkhande => crate::models::pitch_systems::bhatkhande::get_all_symbols(),
    };
    
    let mut col = 1;
    let mut i = 0;
    let chars: Vec<char> = line.chars().collect();
    
    while i < chars.len() {
        let ch = chars[i];
        
        // Handle barlines and other special characters first
        if ch == '|' {
            elements.push(ParsedElement::Barline {
                style: "|".to_string(),
                position: Position { row: line_num, col },
                tala: None,
            });
            i += 1;
            col += 1;
        } else if ch == '-' {
            elements.push(ParsedElement::Dash {
                degree: None,
                octave: None,
                position: Position { row: line_num, col },
                duration: None,
            });
            i += 1;
            col += 1;
        } else if ch == ' ' {
            elements.push(ParsedElement::Whitespace {
                value: " ".to_string(),
                position: Position { row: line_num, col },
            });
            i += 1;
            col += 1;
        } else {
            // Try to match the longest token for this specific notation system at this position
            let mut matched = false;
            let remaining = &line[i..];
            
            // Check tokens in order (longest first since get_all_symbols() returns them sorted)
            for token in &tokens {
                if remaining.to_lowercase().starts_with(&token.to_lowercase()) {
                    // Found a token - try to parse it for this notation system
                    let degree_opt = match notation_system {
                        NotationSystem::Tabla => {
                            tabla::lookup(token).map(convert_models_degree_to_rhythm_degree)
                        },
                        NotationSystem::Number => {
                            crate::models::pitch_systems::number::lookup(token)
                                .map(convert_models_degree_to_rhythm_degree)
                        },
                        NotationSystem::Western => {
                            crate::models::pitch_systems::western::lookup(token)
                                .map(convert_models_degree_to_rhythm_degree)
                        },
                        NotationSystem::Sargam => {
                            crate::models::pitch_systems::sargam::lookup(token)
                                .map(convert_models_degree_to_rhythm_degree)
                        },
                        NotationSystem::Bhatkhande => {
                            crate::models::pitch_systems::bhatkhande::lookup(token)
                                .map(convert_models_degree_to_rhythm_degree)
                        },
                    };
                    
                    if let Some(degree) = degree_opt {
                        elements.push(ParsedElement::new_note(
                            degree,
                            0,
                            token.to_string(),
                            Position { row: line_num, col },
                        ));
                        i += token.len();
                        col += token.len();
                        matched = true;
                        break;
                    }
                }
            }
            
            if !matched {
                // Not a recognized token for this system, treat as single character unknown
                i += 1;
                col += 1;
            }
        }
    }
    
    Ok(elements)
}

/// Convert models::pitch::Degree to rhythm::types::Degree
fn convert_models_degree_to_rhythm_degree(models_degree: crate::models::pitch::Degree) -> Degree {
    match models_degree {
        crate::models::pitch::Degree::N1bb => Degree::N1bb,
        crate::models::pitch::Degree::N1b => Degree::N1b,
        crate::models::pitch::Degree::N1 => Degree::N1,
        crate::models::pitch::Degree::N1s => Degree::N1s,
        crate::models::pitch::Degree::N1ss => Degree::N1ss,
        crate::models::pitch::Degree::N2bb => Degree::N2bb,
        crate::models::pitch::Degree::N2b => Degree::N2b,
        crate::models::pitch::Degree::N2 => Degree::N2,
        crate::models::pitch::Degree::N2s => Degree::N2s,
        crate::models::pitch::Degree::N2ss => Degree::N2ss,
        crate::models::pitch::Degree::N3bb => Degree::N3bb,
        crate::models::pitch::Degree::N3b => Degree::N3b,
        crate::models::pitch::Degree::N3 => Degree::N3,
        crate::models::pitch::Degree::N3s => Degree::N3s,
        crate::models::pitch::Degree::N3ss => Degree::N3ss,
        crate::models::pitch::Degree::N4bb => Degree::N4bb,
        crate::models::pitch::Degree::N4b => Degree::N4b,
        crate::models::pitch::Degree::N4 => Degree::N4,
        crate::models::pitch::Degree::N4s => Degree::N4s,
        crate::models::pitch::Degree::N4ss => Degree::N4ss,
        crate::models::pitch::Degree::N5bb => Degree::N5bb,
        crate::models::pitch::Degree::N5b => Degree::N5b,
        crate::models::pitch::Degree::N5 => Degree::N5,
        crate::models::pitch::Degree::N5s => Degree::N5s,
        crate::models::pitch::Degree::N5ss => Degree::N5ss,
        crate::models::pitch::Degree::N6bb => Degree::N6bb,
        crate::models::pitch::Degree::N6b => Degree::N6b,
        crate::models::pitch::Degree::N6 => Degree::N6,
        crate::models::pitch::Degree::N6s => Degree::N6s,
        crate::models::pitch::Degree::N6ss => Degree::N6ss,
        crate::models::pitch::Degree::N7bb => Degree::N7bb,
        crate::models::pitch::Degree::N7b => Degree::N7b,
        crate::models::pitch::Degree::N7 => Degree::N7,
        crate::models::pitch::Degree::N7s => Degree::N7s,
        crate::models::pitch::Degree::N7ss => Degree::N7ss,
    }
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