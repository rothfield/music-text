use crate::models::{Degree, PitchCode};

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
        "S##".to_string(), "s##".to_string(), "R##".to_string(), "G##".to_string(),
        "P##".to_string(), "p##".to_string(), "D##".to_string(), "N##".to_string(), "M#".to_string(),
        "Sbb".to_string(), "sbb".to_string(), "Rbb".to_string(), "Gbb".to_string(),
        "Pbb".to_string(), "pbb".to_string(), "Dbb".to_string(), "Nbb".to_string(), "mbb".to_string(),
        "S#".to_string(), "s#".to_string(), "R#".to_string(), "G#".to_string(),
        "P#".to_string(), "p#".to_string(), "D#".to_string(), "N#".to_string(),
        "Sb".to_string(), "sb".to_string(), "Pb".to_string(), "pb".to_string(), "mb".to_string(),
    ];
    // Sort by length (longest first) to ensure proper regex matching
    symbols.sort_by(|a, b| b.len().cmp(&a.len()));
    symbols
}

/// Convert a Degree back to sargam notation string
/// This is the reverse operation of lookup - takes a degree and returns the sargam syllable
pub fn degree_to_string(degree: Degree) -> Option<String> {
    let result = match degree {
        // Natural sargam (using uppercase)
        Degree::N1 => "S",
        Degree::N2 => "R",
        Degree::N3 => "G",
        Degree::N4 => "M",
        Degree::N5 => "P",
        Degree::N6 => "D",
        Degree::N7 => "N",
        // Sharps (using # with uppercase)
        Degree::N1s => "S#",
        Degree::N2s => "R#",
        Degree::N3s => "G#",
        Degree::N4s => "M#",
        Degree::N5s => "P#",
        Degree::N6s => "D#",
        Degree::N7s => "N#",
        // Flats (using komal/lowercase)
        Degree::N1b => "s",
        Degree::N2b => "r",
        Degree::N3b => "g",
        Degree::N4b => "m",
        Degree::N5b => "p",
        Degree::N6b => "d",
        Degree::N7b => "n",
        // Double sharps
        Degree::N1ss => "S##",
        Degree::N2ss => "R##",
        Degree::N3ss => "G##",
        Degree::N4ss => "M##",
        Degree::N5ss => "P##",
        Degree::N6ss => "D##",
        Degree::N7ss => "N##",
        // Double flats
        Degree::N1bb => "sbb",
        Degree::N2bb => "rbb",
        Degree::N3bb => "gbb",
        Degree::N4bb => "mbb",
        Degree::N5bb => "pbb",
        Degree::N6bb => "dbb",
        Degree::N7bb => "nbb",
    };
    Some(result.to_string())
}

/// Convert PitchCode directly to sargam notation string
/// Direct mapping without going through Degree abstraction
pub fn pitchcode_to_string(pitchcode: PitchCode) -> Option<String> {
    let result = match pitchcode {
        // Natural sargam (using uppercase/lowercase as appropriate)
        PitchCode::N1 => "S",
        PitchCode::N2 => "R",
        PitchCode::N3 => "G",
        PitchCode::N4 => "m",    // shuddha Ma
        PitchCode::N5 => "P",
        PitchCode::N6 => "D",
        PitchCode::N7 => "N",
        // Sharps (using # with uppercase)
        PitchCode::N1s => "S#",
        PitchCode::N2s => "R#",
        PitchCode::N3s => "G#",
        PitchCode::N4s => "M",   // tivra Ma (uppercase M)
        PitchCode::N5s => "P#",
        PitchCode::N6s => "D#",
        PitchCode::N7s => "N#",
        // Flats (using komal/lowercase)
        PitchCode::N1b => "s",   // Could also be "Sb" but "s" is simpler
        PitchCode::N2b => "r",   // komal Re
        PitchCode::N3b => "g",   // komal Ga
        PitchCode::N4b => "mb",  // komal Ma (rare, use explicit flat)
        PitchCode::N5b => "p",   // Could also be "Pb" but "p" is simpler
        PitchCode::N6b => "d",   // komal Dha
        PitchCode::N7b => "n",   // komal Ni
        // Double sharps
        PitchCode::N1ss => "S##",
        PitchCode::N2ss => "R##",
        PitchCode::N3ss => "G##",
        PitchCode::N4ss => "M##", // Double sharp Ma
        PitchCode::N5ss => "P##",
        PitchCode::N6ss => "D##",
        PitchCode::N7ss => "N##",
        // Double flats
        PitchCode::N1bb => "sbb",
        PitchCode::N2bb => "rbb",
        PitchCode::N3bb => "gbb",
        PitchCode::N4bb => "mbb",
        PitchCode::N5bb => "pbb",
        PitchCode::N6bb => "dbb",
        PitchCode::N7bb => "nbb",
    };
    Some(result.to_string())
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
        "S#" | "s#" => Some(Degree::N1s),
        "S##" | "s##" => Some(Degree::N1ss),
        "Sb" | "sb" => Some(Degree::N1b),
        "Sbb" | "sbb" => Some(Degree::N1bb),
        "R#" => Some(Degree::N2s),
        "R##" => Some(Degree::N2ss),
        "Rbb" => Some(Degree::N2bb),
        "G#" => Some(Degree::N3s),
        "G##" => Some(Degree::N3ss),
        "Gbb" => Some(Degree::N3bb),
        "mb" => Some(Degree::N4b),
        "mbb" => Some(Degree::N4bb),
        "M#" => Some(Degree::N4ss), // M# is 4##
        "P#" | "p#" => Some(Degree::N5s),
        "P##" | "p##" => Some(Degree::N5ss),
        "Pb" | "pb" => Some(Degree::N5b),
        "Pbb" | "pbb" => Some(Degree::N5bb),
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