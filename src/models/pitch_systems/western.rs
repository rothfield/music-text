use crate::models::pitch::Degree;
use crate::parse::model::PitchCode;

/// Get all valid western symbols (for regex pattern generation)
/// Returns symbols sorted by length (longest first) for proper regex matching
pub fn get_all_symbols() -> Vec<String> {
    let mut symbols = vec![
        // Double accidentals (longest first)
        "C##".to_string(), "D##".to_string(), "E##".to_string(), "F##".to_string(),
        "G##".to_string(), "A##".to_string(), "B##".to_string(),
        "Cbb".to_string(), "Dbb".to_string(), "Ebb".to_string(), "Fbb".to_string(),
        "Gbb".to_string(), "Abb".to_string(), "Bbb".to_string(),
        // Single accidentals
        "C#".to_string(), "D#".to_string(), "E#".to_string(), "F#".to_string(),
        "G#".to_string(), "A#".to_string(), "B#".to_string(),
        "Cb".to_string(), "Db".to_string(), "Eb".to_string(), "Fb".to_string(),
        "Gb".to_string(), "Ab".to_string(), "Bb".to_string(),
        // Natural notes
        "C".to_string(), "D".to_string(), "E".to_string(), "F".to_string(),
        "G".to_string(), "A".to_string(), "B".to_string(),
    ];
    // Already sorted by length (longest first)
    symbols
}

/// Convert a Degree back to Western notation string
/// This is the reverse operation of lookup - takes a degree and returns the Western note name
pub fn degree_to_string(degree: Degree) -> Option<String> {
    let result = match degree {
        // Natural notes
        Degree::N1 => "C",
        Degree::N2 => "D",
        Degree::N3 => "E",
        Degree::N4 => "F",
        Degree::N5 => "G",
        Degree::N6 => "A",
        Degree::N7 => "B",
        // Sharps
        Degree::N1s => "C#",
        Degree::N2s => "D#",
        Degree::N3s => "E#",
        Degree::N4s => "F#",
        Degree::N5s => "G#",
        Degree::N6s => "A#",
        Degree::N7s => "B#",
        // Flats
        Degree::N1b => "Cb",
        Degree::N2b => "Db",
        Degree::N3b => "Eb",
        Degree::N4b => "Fb",
        Degree::N5b => "Gb",
        Degree::N6b => "Ab",
        Degree::N7b => "Bb",
        // Double sharps
        Degree::N1ss => "C##",
        Degree::N2ss => "D##",
        Degree::N3ss => "E##",
        Degree::N4ss => "F##",
        Degree::N5ss => "G##",
        Degree::N6ss => "A##",
        Degree::N7ss => "B##",
        // Double flats
        Degree::N1bb => "Cbb",
        Degree::N2bb => "Dbb",
        Degree::N3bb => "Ebb",
        Degree::N4bb => "Fbb",
        Degree::N5bb => "Gbb",
        Degree::N6bb => "Abb",
        Degree::N7bb => "Bbb",
    };
    Some(result.to_string())
}

/// Convert PitchCode directly to western notation string
/// Direct mapping without going through Degree abstraction
pub fn pitchcode_to_string(pitchcode: PitchCode) -> Option<String> {
    let result = match pitchcode {
        // Natural notes
        PitchCode::N1 => "C",
        PitchCode::N2 => "D",
        PitchCode::N3 => "E",
        PitchCode::N4 => "F",
        PitchCode::N5 => "G",
        PitchCode::N6 => "A",
        PitchCode::N7 => "B",
        // Sharps
        PitchCode::N1s => "C#",
        PitchCode::N2s => "D#",
        PitchCode::N3s => "E#",
        PitchCode::N4s => "F#",
        PitchCode::N5s => "G#",
        PitchCode::N6s => "A#",
        PitchCode::N7s => "B#",
        // Flats
        PitchCode::N1b => "Cb",
        PitchCode::N2b => "Db",
        PitchCode::N3b => "Eb",
        PitchCode::N4b => "Fb",
        PitchCode::N5b => "Gb",
        PitchCode::N6b => "Ab",
        PitchCode::N7b => "Bb",
        // Double sharps
        PitchCode::N1ss => "C##",
        PitchCode::N2ss => "D##",
        PitchCode::N3ss => "E##",
        PitchCode::N4ss => "F##",
        PitchCode::N5ss => "G##",
        PitchCode::N6ss => "A##",
        PitchCode::N7ss => "B##",
        // Double flats
        PitchCode::N1bb => "Cbb",
        PitchCode::N2bb => "Dbb",
        PitchCode::N3bb => "Ebb",
        PitchCode::N4bb => "Fbb",
        PitchCode::N5bb => "Gbb",
        PitchCode::N6bb => "Abb",
        PitchCode::N7bb => "Bbb",
    };
    Some(result.to_string())
}

/// Western notation pitch lookup
/// Maps Western note names (C D E F G A B) with optional accidentals to degrees
pub fn lookup(symbol: &str) -> Option<Degree> {
    match symbol {
        // Natural notes
        "C" => Some(Degree::N1),
        "D" => Some(Degree::N2),
        "E" => Some(Degree::N3),
        "F" => Some(Degree::N4),
        "G" => Some(Degree::N5),
        "A" => Some(Degree::N6),
        "B" => Some(Degree::N7),
        // Sharps
        "C#" => Some(Degree::N1s),
        "D#" => Some(Degree::N2s),
        "E#" => Some(Degree::N3s),
        "F#" => Some(Degree::N4s),
        "G#" => Some(Degree::N5s),
        "A#" => Some(Degree::N6s),
        "B#" => Some(Degree::N7s),
        // Flats
        "Cb" => Some(Degree::N1b),
        "Db" => Some(Degree::N2b),
        "Eb" => Some(Degree::N3b),
        "Fb" => Some(Degree::N4b),
        "Gb" => Some(Degree::N5b),
        "Ab" => Some(Degree::N6b),
        "Bb" => Some(Degree::N7b),
        // Double sharps
        "C##" => Some(Degree::N1ss),
        "D##" => Some(Degree::N2ss),
        "E##" => Some(Degree::N3ss),
        "F##" => Some(Degree::N4ss),
        "G##" => Some(Degree::N5ss),
        "A##" => Some(Degree::N6ss),
        "B##" => Some(Degree::N7ss),
        // Double flats
        "Cbb" => Some(Degree::N1bb),
        "Dbb" => Some(Degree::N2bb),
        "Ebb" => Some(Degree::N3bb),
        "Fbb" => Some(Degree::N4bb),
        "Gbb" => Some(Degree::N5bb),
        "Abb" => Some(Degree::N6bb),
        "Bbb" => Some(Degree::N7bb),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_western_natural_notes() {
        assert_eq!(lookup("C"), Some(Degree::N1));
        assert_eq!(lookup("D"), Some(Degree::N2));
        assert_eq!(lookup("E"), Some(Degree::N3));
        assert_eq!(lookup("F"), Some(Degree::N4));
        assert_eq!(lookup("G"), Some(Degree::N5));
        assert_eq!(lookup("A"), Some(Degree::N6));
        assert_eq!(lookup("B"), Some(Degree::N7));
    }

    #[test]
    fn test_western_sharps() {
        assert_eq!(lookup("C#"), Some(Degree::N1s));
        assert_eq!(lookup("F#"), Some(Degree::N4s));
        assert_eq!(lookup("A#"), Some(Degree::N6s));
    }

    #[test]
    fn test_western_flats() {
        assert_eq!(lookup("Bb"), Some(Degree::N7b));
        assert_eq!(lookup("Eb"), Some(Degree::N3b));
        assert_eq!(lookup("Ab"), Some(Degree::N6b));
    }

    #[test]
    fn test_western_double_accidentals() {
        assert_eq!(lookup("C##"), Some(Degree::N1ss));
        assert_eq!(lookup("Dbb"), Some(Degree::N2bb));
    }

    #[test]
    fn test_western_invalid() {
        assert_eq!(lookup("X"), None);
        assert_eq!(lookup("H"), None);
        assert_eq!(lookup(""), None);
    }
}