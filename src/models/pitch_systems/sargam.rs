use crate::models::pitch::Degree;

// IMPORTANT: SARGAM IS CASE SENSITIVE
// - s is an alias for S, p is an alias for P
// - SrRgGmMPdDnN maps to C Db D Eb E F F# G Ab A Bb B
// - Lowercase letters (except s,p) represent flat variants of uppercase

/// Get all valid sargam symbols (for regex pattern generation)
/// Returns symbols sorted by length (longest first) for proper regex matching
pub fn get_all_symbols() -> Vec<String> {
    let mut symbols = vec![
        // Natural sargam
        "S".to_string(), "s".to_string(),
        "R".to_string(), "G".to_string(), "m".to_string(),
        "P".to_string(), "p".to_string(),
        "D".to_string(), "N".to_string(),
        // Komal (flattened) sargam
        "r".to_string(), "g".to_string(), "d".to_string(), "n".to_string(),
        // Tivra (sharpened) sargam
        "M".to_string(),
        // Extended sargam with explicit accidentals (sorted by length)
        "S##".to_string(), "R##".to_string(), "G##".to_string(), "P##".to_string(),
        "D##".to_string(), "N##".to_string(), "M#".to_string(),
        "Sbb".to_string(), "Rbb".to_string(), "Gbb".to_string(), "Pbb".to_string(),
        "Dbb".to_string(), "Nbb".to_string(), "mbb".to_string(),
        "S#".to_string(), "R#".to_string(), "G#".to_string(), "P#".to_string(),
        "D#".to_string(), "N#".to_string(),
        "Sb".to_string(), "S-".to_string(), "R-".to_string(), "G-".to_string(),
        "Pb".to_string(), "mb".to_string(),
    ];
    // Sort by length (longest first) to ensure proper regex matching
    symbols.sort_by(|a, b| b.len().cmp(&a.len()));
    symbols
}

/// Sargam notation pitch lookup
/// Maps Sargam notation (S R G M P D N) with case-sensitive variants to degrees
pub fn lookup(symbol: &str) -> Option<Degree> {
    match symbol {
        // Natural sargam
        "S" | "s" => Some(Degree::N1),    // Sa (both uppercase and lowercase)
        "R" => Some(Degree::N2),    // shuddha Re  
        "G" => Some(Degree::N3),    // shuddha Ga
        "m" => Some(Degree::N4),    // shuddha Ma
        "P" | "p" => Some(Degree::N5),    // Pa (both uppercase and lowercase)
        "D" => Some(Degree::N6),    // shuddha Dha
        "N" => Some(Degree::N7),    // shuddha Ni
        // Komal (flattened) sargam
        "r" => Some(Degree::N2b),   // komal Re
        "g" => Some(Degree::N3b),   // komal Ga  
        "d" => Some(Degree::N6b),   // komal Dha
        "n" => Some(Degree::N7b),   // komal Ni
        // Tivra (sharpened) sargam
        "M" => Some(Degree::N4s),   // tivra Ma
        // Extended sargam with explicit accidentals
        "S#" => Some(Degree::N1s),
        "S##" => Some(Degree::N1ss),
        "Sb" => Some(Degree::N1b),
        "S-" => Some(Degree::N1b),   // Alternative flat notation
        "Sbb" => Some(Degree::N1bb),
        "R#" => Some(Degree::N2s),
        "R##" => Some(Degree::N2ss),
        "R-" => Some(Degree::N2b),   // Alternative flat notation
        "Rbb" => Some(Degree::N2bb),
        "G#" => Some(Degree::N3s),
        "G##" => Some(Degree::N3ss),
        "G-" => Some(Degree::N3b),   // Alternative flat notation
        "Gbb" => Some(Degree::N3bb),
        "mb" => Some(Degree::N4b),
        "mbb" => Some(Degree::N4bb),
        "M#" => Some(Degree::N4ss), // M# is 4##
        "P#" => Some(Degree::N5s),
        "P##" => Some(Degree::N5ss),
        "Pb" => Some(Degree::N5b),
        "Pbb" => Some(Degree::N5bb),
        "D#" => Some(Degree::N6s),
        "D##" => Some(Degree::N6ss),
        "Dbb" => Some(Degree::N6bb),
        "N#" => Some(Degree::N7s),
        "N##" => Some(Degree::N7ss),
        "Nbb" => Some(Degree::N7bb),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sargam_natural_notes() {
        assert_eq!(lookup("S"), Some(Degree::N1));
        assert_eq!(lookup("R"), Some(Degree::N2));
        assert_eq!(lookup("G"), Some(Degree::N3));
        assert_eq!(lookup("m"), Some(Degree::N4));
        assert_eq!(lookup("P"), Some(Degree::N5));
        assert_eq!(lookup("D"), Some(Degree::N6));
        assert_eq!(lookup("N"), Some(Degree::N7));
    }

    #[test]
    fn test_sargam_komal_variants() {
        assert_eq!(lookup("r"), Some(Degree::N2b));   // komal Re
        assert_eq!(lookup("g"), Some(Degree::N3b));   // komal Ga
        assert_eq!(lookup("d"), Some(Degree::N6b));   // komal Dha
        assert_eq!(lookup("n"), Some(Degree::N7b));   // komal Ni
    }

    #[test]
    fn test_sargam_tivra_ma() {
        assert_eq!(lookup("M"), Some(Degree::N4s));   // tivra Ma
    }

    #[test]
    fn test_lowercase_sargam_pitches() {
        // Test that lowercase s and p map to the same pitch codes as uppercase S and P
        assert_eq!(lookup("s"), Some(Degree::N1)); // Sa
        assert_eq!(lookup("S"), Some(Degree::N1)); // Sa
        assert_eq!(lookup("p"), Some(Degree::N5)); // Pa
        assert_eq!(lookup("P"), Some(Degree::N5)); // Pa
        
        // Verify they are equivalent
        assert_eq!(lookup("s"), lookup("S"));
        assert_eq!(lookup("p"), lookup("P"));
    }

    #[test]
    fn test_sargam_extended_accidentals() {
        assert_eq!(lookup("S#"), Some(Degree::N1s));
        assert_eq!(lookup("R-"), Some(Degree::N2b));   // Alternative flat notation
        assert_eq!(lookup("G##"), Some(Degree::N3ss));
        assert_eq!(lookup("M#"), Some(Degree::N4ss));  // M# is 4##
    }

    #[test]
    fn test_sargam_invalid() {
        assert_eq!(lookup("X"), None);
        assert_eq!(lookup(""), None);
        assert_eq!(lookup("unknown"), None);
    }

    #[test]
    fn test_case_sensitive_mapping() {
        // Verify the SrRgGmMPdDnN -> C Db D Eb E F F# G Ab A Bb B mapping
        assert_eq!(lookup("S"), Some(Degree::N1));   // C
        assert_eq!(lookup("r"), Some(Degree::N2b));  // Db  
        assert_eq!(lookup("R"), Some(Degree::N2));   // D
        assert_eq!(lookup("g"), Some(Degree::N3b));  // Eb
        assert_eq!(lookup("G"), Some(Degree::N3));   // E
        assert_eq!(lookup("m"), Some(Degree::N4));   // F
        assert_eq!(lookup("M"), Some(Degree::N4s));  // F#
        assert_eq!(lookup("P"), Some(Degree::N5));   // G
        assert_eq!(lookup("d"), Some(Degree::N6b));  // Ab
        assert_eq!(lookup("D"), Some(Degree::N6));   // A
        assert_eq!(lookup("n"), Some(Degree::N7b));  // Bb
        assert_eq!(lookup("N"), Some(Degree::N7));   // B
    }
}