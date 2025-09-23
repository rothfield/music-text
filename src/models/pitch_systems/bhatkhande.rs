use crate::models::{Degree, PitchCode};

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

/// Convert a Degree back to bhatkhande notation string
/// This is the reverse operation of lookup - takes a degree and returns the bhatkhande symbol
pub fn degree_to_string(degree: Degree) -> Option<String> {
    // Use uppercase romanized form for output
    let result = match degree {
        Degree::N1 => "S",
        Degree::N2 => "R",
        Degree::N3 => "G",
        Degree::N4 => "M",
        Degree::N5 => "P",
        Degree::N6 => "D",
        Degree::N7 => "N",
        // For now, use simple ASCII accidentals
        Degree::N1s => "S#",
        Degree::N2s => "R#",
        Degree::N3s => "G#",
        Degree::N4s => "M#",
        Degree::N5s => "P#",
        Degree::N6s => "D#",
        Degree::N7s => "N#",
        Degree::N1b => "Sb",
        Degree::N2b => "Rb",
        Degree::N3b => "Gb",
        Degree::N4b => "Mb",
        Degree::N5b => "Pb",
        Degree::N6b => "Db",
        Degree::N7b => "Nb",
        // Double accidentals
        Degree::N1ss => "S##",
        Degree::N2ss => "R##",
        Degree::N3ss => "G##",
        Degree::N4ss => "M##",
        Degree::N5ss => "P##",
        Degree::N6ss => "D##",
        Degree::N7ss => "N##",
        Degree::N1bb => "Sbb",
        Degree::N2bb => "Rbb",
        Degree::N3bb => "Gbb",
        Degree::N4bb => "Mbb",
        Degree::N5bb => "Pbb",
        Degree::N6bb => "Dbb",
        Degree::N7bb => "Nbb",
    };
    Some(result.to_string())
}

/// Convert PitchCode directly to bhatkhande notation string
/// Direct mapping without going through Degree abstraction
/// Uses Roman equivalents for simplicity
pub fn pitchcode_to_string(pitchcode: PitchCode) -> Option<String> {
    let result = match pitchcode {
        // Natural notes (using Roman equivalents)
        PitchCode::N1 => "S",
        PitchCode::N2 => "R",
        PitchCode::N3 => "G",
        PitchCode::N4 => "M",
        PitchCode::N5 => "P",
        PitchCode::N6 => "D",
        PitchCode::N7 => "N",
        // Sharps
        PitchCode::N1s => "S#",
        PitchCode::N2s => "R#",
        PitchCode::N3s => "G#",
        PitchCode::N4s => "M#",
        PitchCode::N5s => "P#",
        PitchCode::N6s => "D#",
        PitchCode::N7s => "N#",
        // Flats
        PitchCode::N1b => "Sb",
        PitchCode::N2b => "Rb",
        PitchCode::N3b => "Gb",
        PitchCode::N4b => "Mb",
        PitchCode::N5b => "Pb",
        PitchCode::N6b => "Db",
        PitchCode::N7b => "Nb",
        // Double sharps
        PitchCode::N1ss => "S##",
        PitchCode::N2ss => "R##",
        PitchCode::N3ss => "G##",
        PitchCode::N4ss => "M##",
        PitchCode::N5ss => "P##",
        PitchCode::N6ss => "D##",
        PitchCode::N7ss => "N##",
        // Double flats
        PitchCode::N1bb => "Sbb",
        PitchCode::N2bb => "Rbb",
        PitchCode::N3bb => "Gbb",
        PitchCode::N4bb => "Mbb",
        PitchCode::N5bb => "Pbb",
        PitchCode::N6bb => "Dbb",
        PitchCode::N7bb => "Nbb",
    };
    Some(result.to_string())
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