use crate::models::pitch::Degree;

/// Get all valid number symbols (for regex pattern generation)
/// Returns symbols sorted by length (longest first) for proper regex matching
pub fn get_all_symbols() -> Vec<String> {
    let mut symbols = vec![
        // Double accidentals (longest first)
        "1##".to_string(), "2##".to_string(), "3##".to_string(), "4##".to_string(),
        "5##".to_string(), "6##".to_string(), "7##".to_string(),
        "1bb".to_string(), "2bb".to_string(), "3bb".to_string(), "4bb".to_string(),
        "5bb".to_string(), "6bb".to_string(), "7bb".to_string(),
        // Single accidentals
        "1#".to_string(), "2#".to_string(), "3#".to_string(), "4#".to_string(),
        "5#".to_string(), "6#".to_string(), "7#".to_string(),
        "1b".to_string(), "2b".to_string(), "3b".to_string(), "4b".to_string(),
        "5b".to_string(), "6b".to_string(), "7b".to_string(),
        // Natural numbers
        "1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(),
        "5".to_string(), "6".to_string(), "7".to_string(),
    ];
    // Already sorted by length (longest first)
    symbols
}

/// Number notation pitch lookup
/// Maps number notation (1 2 3 4 5 6 7) with optional accidentals to degrees
pub fn lookup(symbol: &str) -> Option<Degree> {
    match symbol {
        // Natural numbers
        "1" => Some(Degree::N1),
        "2" => Some(Degree::N2),
        "3" => Some(Degree::N3),
        "4" => Some(Degree::N4),
        "5" => Some(Degree::N5),
        "6" => Some(Degree::N6),
        "7" => Some(Degree::N7),
        // Sharps
        "1#" => Some(Degree::N1s),
        "2#" => Some(Degree::N2s),
        "3#" => Some(Degree::N3s),
        "4#" => Some(Degree::N4s),
        "5#" => Some(Degree::N5s),
        "6#" => Some(Degree::N6s),
        "7#" => Some(Degree::N7s),
        // Flats
        "1b" => Some(Degree::N1b),
        "2b" => Some(Degree::N2b),
        "3b" => Some(Degree::N3b),
        "4b" => Some(Degree::N4b),
        "5b" => Some(Degree::N5b),
        "6b" => Some(Degree::N6b),
        "7b" => Some(Degree::N7b),
        // Double sharps
        "1##" => Some(Degree::N1ss),
        "2##" => Some(Degree::N2ss),
        "3##" => Some(Degree::N3ss),
        "4##" => Some(Degree::N4ss),
        "5##" => Some(Degree::N5ss),
        "6##" => Some(Degree::N6ss),
        "7##" => Some(Degree::N7ss),
        // Double flats
        "1bb" => Some(Degree::N1bb),
        "2bb" => Some(Degree::N2bb),
        "3bb" => Some(Degree::N3bb),
        "4bb" => Some(Degree::N4bb),
        "5bb" => Some(Degree::N5bb),
        "6bb" => Some(Degree::N6bb),
        "7bb" => Some(Degree::N7bb),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_natural_notes() {
        assert_eq!(lookup("1"), Some(Degree::N1));
        assert_eq!(lookup("2"), Some(Degree::N2));
        assert_eq!(lookup("3"), Some(Degree::N3));
        assert_eq!(lookup("4"), Some(Degree::N4));
        assert_eq!(lookup("5"), Some(Degree::N5));
        assert_eq!(lookup("6"), Some(Degree::N6));
        assert_eq!(lookup("7"), Some(Degree::N7));
    }

    #[test]
    fn test_number_sharps() {
        assert_eq!(lookup("1#"), Some(Degree::N1s));
        assert_eq!(lookup("4#"), Some(Degree::N4s));
        assert_eq!(lookup("6#"), Some(Degree::N6s));
    }

    #[test]
    fn test_number_flats() {
        assert_eq!(lookup("7b"), Some(Degree::N7b));
        assert_eq!(lookup("3b"), Some(Degree::N3b));
        assert_eq!(lookup("6b"), Some(Degree::N6b));
    }

    #[test]
    fn test_number_double_accidentals() {
        assert_eq!(lookup("1##"), Some(Degree::N1ss));
        assert_eq!(lookup("2bb"), Some(Degree::N2bb));
    }

    #[test]
    fn test_number_invalid() {
        assert_eq!(lookup("8"), None);
        assert_eq!(lookup("0"), None);
        assert_eq!(lookup("X"), None);
        assert_eq!(lookup(""), None);
    }
}