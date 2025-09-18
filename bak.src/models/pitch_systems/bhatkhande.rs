use crate::models::pitch::Degree;

/// Get all valid bhatkhande symbols (for regex pattern generation)
/// Returns symbols sorted by length (longest first) for proper regex matching
pub fn get_all_symbols() -> Vec<String> {
    let mut symbols = vec![
        // Devanagari with accidentals (longest first)
        "रे#".to_string(), "रेb".to_string(), "नि#".to_string(), "निb".to_string(),
        "स#".to_string(), "सb".to_string(), "ग#".to_string(), "गb".to_string(),
        "म#".to_string(), "मb".to_string(), "प#".to_string(), "पb".to_string(),
        "ध#".to_string(), "धb".to_string(),
        // Roman with accidentals  
        "S#".to_string(), "R#".to_string(), "G#".to_string(), "M#".to_string(),
        "P#".to_string(), "D#".to_string(), "N#".to_string(),
        "Sb".to_string(), "Rb".to_string(), "Gb".to_string(), "Mb".to_string(),
        "Pb".to_string(), "Db".to_string(), "Nb".to_string(),
        // Basic Devanagari (2 chars)
        "रे".to_string(), "नि".to_string(),
        // Basic Devanagari (1 char) and Roman
        "स".to_string(), "ग".to_string(), "म".to_string(), "प".to_string(), "ध".to_string(),
        "S".to_string(), "R".to_string(), "G".to_string(), "M".to_string(),
        "P".to_string(), "D".to_string(), "N".to_string(),
    ];
    // Already sorted by length (longest first)
    symbols
}

/// Bhatkhande notation pitch lookup
/// Maps Bhatkhande Devanagari and Roman equivalents to degrees
pub fn lookup(symbol: &str) -> Option<Degree> {
    match symbol {
        // Basic Bhatkhande sargam notes (Devanagari)
        "स" | "S" => Some(Degree::N1),    // Sa
        "रे" | "R" => Some(Degree::N2),   // Re  
        "ग" | "G" => Some(Degree::N3),    // Ga
        "म" | "M" => Some(Degree::N4),    // Ma
        "प" | "P" => Some(Degree::N5),    // Pa
        "ध" | "D" => Some(Degree::N6),    // Dha
        "नि" | "N" => Some(Degree::N7),   // Ni
        // Sharp accidentals 
        "स#" | "S#" => Some(Degree::N1s),  // Sa sharp
        "रे#" | "R#" => Some(Degree::N2s), // Re sharp  
        "ग#" | "G#" => Some(Degree::N3s),  // Ga sharp
        "म#" | "M#" => Some(Degree::N4s),  // Ma sharp (corresponds to F#)
        "प#" | "P#" => Some(Degree::N5s),  // Pa sharp
        "ध#" | "D#" => Some(Degree::N6s),  // Dha sharp
        "नि#" | "N#" => Some(Degree::N7s), // Ni sharp
        // Flat accidentals
        "सb" | "Sb" => Some(Degree::N1b),  // Sa flat
        "रेb" | "Rb" => Some(Degree::N2b), // Re flat
        "गb" | "Gb" => Some(Degree::N3b),  // Ga flat
        "मb" | "Mb" => Some(Degree::N4b),  // Ma flat
        "पb" | "Pb" => Some(Degree::N5b),  // Pa flat
        "धb" | "Db" => Some(Degree::N6b),  // Dha flat
        "निb" | "Nb" => Some(Degree::N7b), // Ni flat
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bhatkhande_devanagari() {
        // Test basic Bhatkhande sargam notes (Devanagari)
        assert_eq!(lookup("स"), Some(Degree::N1)); // Sa
        assert_eq!(lookup("रे"), Some(Degree::N2)); // Re
        assert_eq!(lookup("ग"), Some(Degree::N3)); // Ga
        assert_eq!(lookup("म"), Some(Degree::N4)); // Ma
        assert_eq!(lookup("प"), Some(Degree::N5)); // Pa
        assert_eq!(lookup("ध"), Some(Degree::N6)); // Dha
        assert_eq!(lookup("नि"), Some(Degree::N7)); // Ni
    }

    #[test]
    fn test_bhatkhande_roman() {
        // Test basic Bhatkhande sargam notes (Roman)
        assert_eq!(lookup("S"), Some(Degree::N1)); // Sa
        assert_eq!(lookup("R"), Some(Degree::N2)); // Re
        assert_eq!(lookup("G"), Some(Degree::N3)); // Ga
        assert_eq!(lookup("M"), Some(Degree::N4)); // Ma
        assert_eq!(lookup("P"), Some(Degree::N5)); // Pa
        assert_eq!(lookup("D"), Some(Degree::N6)); // Dha
        assert_eq!(lookup("N"), Some(Degree::N7)); // Ni
    }

    #[test]
    fn test_bhatkhande_accidentals() {
        // Test accidentals with both Devanagari and Roman
        assert_eq!(lookup("S#"), Some(Degree::N1s));
        assert_eq!(lookup("स#"), Some(Degree::N1s));
        assert_eq!(lookup("M#"), Some(Degree::N4s)); // Ma sharp = F#
        assert_eq!(lookup("म#"), Some(Degree::N4s)); // Ma sharp = F#
        assert_eq!(lookup("Db"), Some(Degree::N6b));
        assert_eq!(lookup("धb"), Some(Degree::N6b));
    }

    #[test]
    fn test_bhatkhande_invalid() {
        assert_eq!(lookup("X"), None);
        assert_eq!(lookup(""), None);
        assert_eq!(lookup("invalid"), None);
    }
}