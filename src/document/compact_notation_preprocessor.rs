/// Preprocessor for compact notation (e.g., "SRG" → "S R G")
/// Detects sequences of adjacent musical characters and adds spaces

pub fn preprocess_compact_notation(input: &str) -> String {
    // Only for single-line input
    if input.contains('\n') {
        return input.to_string();
    }
    
    // If input already has barlines or spaces, don't preprocess
    if input.contains('|') || input.contains(' ') {
        return input.to_string();
    }
    
    // Only preprocess truly consecutive musical characters (3+ chars, all musical)
    if input.len() >= 3 && input.chars().all(is_musical_char) {
        // Insert spaces between consecutive musical characters
        let mut result = String::new();
        for (i, ch) in input.chars().enumerate() {
            if i > 0 {
                result.push(' ');
            }
            result.push(ch);
        }
        result
    } else {
        input.to_string()
    }
}

fn is_musical_char(ch: char) -> bool {
    matches!(ch, 
        '1'..='7' |                                    // Numbers
        'C' | 'D' | 'E' | 'F' | 'G' | 'A' | 'B' |     // Western
        'S' | 'R' | 'M' | 'P' | 'N' |                 // Sargam uppercase (excluding ambiguous G, D)
        's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n'       // Sargam lowercase
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compact_notation() {
        // Numbers
        assert_eq!(preprocess_compact_notation("123"), "1 2 3");
        assert_eq!(preprocess_compact_notation("1234567"), "1 2 3 4 5 6 7");
        
        // Sargam
        assert_eq!(preprocess_compact_notation("SRG"), "S R G");
        assert_eq!(preprocess_compact_notation("SRGMPDN"), "S R G M P D N");
        assert_eq!(preprocess_compact_notation("srgm"), "s r g m");
        
        // Western
        assert_eq!(preprocess_compact_notation("CDE"), "C D E");
        assert_eq!(preprocess_compact_notation("CDEFGAB"), "C D E F G A B");
        
        // Mixed with spaces (don't preprocess - return as-is)
        assert_eq!(preprocess_compact_notation("SR G"), "SR G");
        
        // Non-musical text unchanged
        assert_eq!(preprocess_compact_notation("Hello world"), "Hello world");
        
        // With barlines - don't preprocess
        assert_eq!(preprocess_compact_notation("|SRG"), "|SRG");
    }
}