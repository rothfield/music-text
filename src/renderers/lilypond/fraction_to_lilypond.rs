/// Sophisticated fraction to LilyPond duration conversion
/// Based on Go implementation from fraction.go - handles complex fractions with ties,
/// loop detection, and dotted note support.

use std::collections::{HashMap, HashSet};
use fraction::Fraction;

/// Calculate Greatest Common Divisor (Euclidean algorithm)
fn gcd(mut a: i32, mut b: i32) -> i32 {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}

/// Convert a fraction to LilyPond duration strings with ties.
/// Based on the sophisticated Go implementation with dotted note support,
/// loop detection, and tie fallback.
pub fn fraction_to_lilypond(numerator: i32, denominator: i32) -> Vec<String> {
    if denominator == 0 {
        return vec!["Invalid denominator".to_string()];
    }

    // Direct mapping for common durations including dotted notes
    let mut lilypond_map = HashMap::new();
    
    // Basic durations
    lilypond_map.insert("1/1".to_string(), "1".to_string());
    lilypond_map.insert("1/2".to_string(), "2".to_string());
    lilypond_map.insert("1/4".to_string(), "4".to_string());
    lilypond_map.insert("1/8".to_string(), "8".to_string());
    lilypond_map.insert("1/16".to_string(), "16".to_string());
    lilypond_map.insert("1/32".to_string(), "32".to_string());
    lilypond_map.insert("1/64".to_string(), "64".to_string());
    lilypond_map.insert("1/128".to_string(), "128".to_string());
    
    // Single-dotted durations (3/2 of basic duration)
    lilypond_map.insert("3/2".to_string(), "1.".to_string());
    lilypond_map.insert("3/4".to_string(), "2.".to_string());
    lilypond_map.insert("3/8".to_string(), "4.".to_string());
    lilypond_map.insert("3/16".to_string(), "8.".to_string());
    lilypond_map.insert("3/32".to_string(), "16.".to_string());
    lilypond_map.insert("3/64".to_string(), "32.".to_string());
    lilypond_map.insert("3/128".to_string(), "64.".to_string());
    
    // Double-dotted durations (7/4 of basic duration)
    lilypond_map.insert("7/4".to_string(), "1..".to_string());   // double-dotted whole note
    lilypond_map.insert("7/8".to_string(), "2..".to_string());   // double-dotted half note  
    lilypond_map.insert("7/16".to_string(), "4..".to_string());  // double-dotted quarter note
    lilypond_map.insert("7/32".to_string(), "8..".to_string());  // double-dotted eighth note
    lilypond_map.insert("7/64".to_string(), "16..".to_string()); // double-dotted sixteenth note
    lilypond_map.insert("7/128".to_string(), "32..".to_string()); // double-dotted thirty-second note

    let fraction_str = format!("{}/{}", numerator, denominator);
    if let Some(lilypond_duration) = lilypond_map.get(&fraction_str) {
        return vec![lilypond_duration.clone()];
    }

    let mut result = Vec::new();
    let mut remaining_numerator = numerator;
    let mut remaining_denominator = denominator;

    let common_denominators = vec![1, 2, 4, 8, 16, 32, 64, 128];

    // Loop detection using seen fractions
    let mut seen_fractions = HashSet::new();

    while remaining_numerator > 0 {
        let current_fraction = format!("{}/{}", remaining_numerator, remaining_denominator);
        if seen_fractions.contains(&current_fraction) {
            // Loop detected! Fallback to ties
            return tie_fallback(numerator, denominator);
        }
        seen_fractions.insert(current_fraction);

        // Find the best denominator that fits
        let mut best_denominator = -1i32;
        for &denom in common_denominators.iter().rev() {
            if remaining_numerator * denom <= remaining_denominator {
                best_denominator = denom;
                break;
            }
        }

        if best_denominator == -1 {
            return vec![format!("Complex: {}/{}", numerator, denominator)];
        }

        let best_num = remaining_numerator * best_denominator / remaining_denominator;
        result.push(format!("{}", remaining_denominator / best_denominator));
        
        remaining_numerator = remaining_numerator * best_denominator - best_num * remaining_denominator;
        remaining_denominator = remaining_denominator * best_denominator;

        // Simplify the remaining fraction
        let common = gcd(remaining_numerator, remaining_denominator);
        remaining_numerator /= common;
        remaining_denominator /= common;
    }

    // Add ties between the durations
    let mut tied_result = Vec::new();
    for (i, note) in result.iter().enumerate() {
        tied_result.push(note.clone());
        if i < result.len() - 1 {
            tied_result.push("~".to_string());
        }
    }

    tied_result
}

/// Tie fallback: decomposes the fraction into the smallest possible notes and ties them.
fn tie_fallback(numerator: i32, denominator: i32) -> Vec<String> {
    let mut result = Vec::new();
    let mut remaining = numerator;
    
    while remaining > 0 {
        result.push(format!("{}", denominator));
        remaining -= 1;
    }

    let mut tied_result = Vec::new();
    for (i, note) in result.iter().enumerate() {
        tied_result.push(note.clone());
        if i < result.len() - 1 {
            tied_result.push("~".to_string());
        }
    }
    
    tied_result
}


#[cfg(test)]
mod tests {
    use super::*;
    use fraction::Fraction;

    #[test]
    fn test_basic_durations() {
        assert_eq!(fraction_to_lilypond(1, 4), vec!["4"]);
        assert_eq!(fraction_to_lilypond(1, 8), vec!["8"]);
        assert_eq!(fraction_to_lilypond(1, 2), vec!["2"]);
    }

    #[test]
    fn test_dotted_durations() {
        assert_eq!(fraction_to_lilypond(3, 8), vec!["4."]);
        assert_eq!(fraction_to_lilypond(3, 4), vec!["2."]);
        assert_eq!(fraction_to_lilypond(3, 16), vec!["8."]);
    }

    #[test]
    fn test_double_dotted() {
        assert_eq!(fraction_to_lilypond(7, 16), vec!["4.."]);
        assert_eq!(fraction_to_lilypond(7, 32), vec!["8.."]);
    }
    
    #[test]
    fn test_problem_case() {
        // Debug the 1------7 case
        println!("Testing 7/32: {:?}", fraction_to_lilypond(7, 32));
        println!("Testing 1/32: {:?}", fraction_to_lilypond(1, 32));
        
        // This should be "8.." (double-dotted eighth note)
        assert_eq!(fraction_to_lilypond(7, 32), vec!["8.."]);
        // This should be "32" (thirty-second note)  
        assert_eq!(fraction_to_lilypond(1, 32), vec!["32"]);
    }

    #[test]
    fn test_complex_fractions() {
        // Test cases from Go implementation
        let result = fraction_to_lilypond(5, 32);
        assert!(result.len() > 1);
        assert!(result.contains(&"~".to_string()));
        
        let result = fraction_to_lilypond(7, 12);
        assert!(result.len() > 1);
        
        let result = fraction_to_lilypond(11, 16);
        assert!(result.len() > 1);
        
        let result = fraction_to_lilypond(2, 3);
        assert!(result.len() > 1);
    }

}