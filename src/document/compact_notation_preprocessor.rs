/// Preprocessor for compact notation (e.g., "SRG" → "S R G")
/// Detects sequences of adjacent musical characters and adds spaces,
/// only if the sequence belongs to a single, unambiguous notation system.

pub fn preprocess_compact_notation(input: &str) -> String {
    // Only for single-line input
    if input.contains('\n') {
        return input.to_string();
    }
    
    // If input already has barlines or spaces, don't preprocess
    if input.contains('|') || input.contains(' ') {
        return input.to_string();
    }
    
    // Only preprocess truly consecutive musical characters (3+ chars)
    if input.len() < 3 {
        return input.to_string();
    }

    let chars: Vec<char> = input.chars().collect();
    
    // Determine possible systems from the first character
    let first_char = chars[0];
    let mut is_number = matches!(first_char, '1'..='7');
    let mut is_western = matches!(first_char, 'C'..='G' | 'A' | 'B');
    let mut is_sargam = matches!(first_char, 'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N' | 's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n');

    // Filter out systems that don't match all characters
    for &ch in &chars[1..] {
        if is_number && !matches!(ch, '1'..='7') {
            is_number = false;
        }
        if is_western && !matches!(ch, 'C'..='G' | 'A' | 'B') {
            is_western = false;
        }
        if is_sargam && !matches!(ch, 'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N' | 's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n') {
            is_sargam = false;
        }
    }

    let mut valid_systems = 0;
    if is_number { valid_systems += 1; }
    if is_western { valid_systems += 1; }
    if is_sargam { valid_systems += 1; }

    // If the string of characters can be unambiguously parsed as belonging to ONE system, then preprocess it.
    if valid_systems == 1 {
        return input.chars().map(|c| c.to_string()).collect::<Vec<_>>().join(" ");
    }

    // Otherwise, it's mixed, ambiguous, or not musical. Return as is.
    input.to_string()
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_unambiguous_compact_notation() {
        // Numbers
        assert_eq!(preprocess_compact_notation("123"), "1 2 3");
        assert_eq!(preprocess_compact_notation("1234567"), "1 2 3 4 5 6 7");
        
        // Sargam
        assert_eq!(preprocess_compact_notation("SRM"), "S R M"); // No ambiguous G/D
        assert_eq!(preprocess_compact_notation("srgmpdn"), "s r g m p d n");
        
        // Western
        assert_eq!(preprocess_compact_notation("CDE"), "C D E");
        assert_eq!(preprocess_compact_notation("CEFAB"), "C E F A B"); // No ambiguous G/D
    }

    #[test]
    fn test_ambiguous_but_valid_compact_notation() {
        // These contain G or D but are valid within one system
        assert_eq!(preprocess_compact_notation("SRG"), "S R G");
        assert_eq!(preprocess_compact_notation("CDG"), "C D G");
        assert_eq!(preprocess_compact_notation("GDA"), "G D A"); // Western
    }

    #[test]
    fn test_mixed_notation_is_not_preprocessed() {
        assert_eq!(preprocess_compact_notation("12S"), "12S");
        assert_eq!(preprocess_compact_notation("SRG1"), "SRG1");
        assert_eq!(preprocess_compact_notation("CDEs"), "CDEs");
    }
    
    #[test]
    fn test_strings_that_should_be_ignored() {
        // Too short
        assert_eq!(preprocess_compact_notation("12"), "12");
        assert_eq!(preprocess_compact_notation("SR"), "SR");

        // Contains separators
        assert_eq!(preprocess_compact_notation("SR G"), "SR G");
        assert_eq!(preprocess_compact_notation("|SRG"), "|SRG");
        assert_eq!(preprocess_compact_notation("1\n2"), "1\n2");
        
        // Non-musical text
        assert_eq!(preprocess_compact_notation("Hello"), "Hello");
    }
}
