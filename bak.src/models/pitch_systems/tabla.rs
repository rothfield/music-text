use crate::models::pitch::Degree;

/// Get all valid tabla symbols (for regex pattern generation)
/// Returns symbols sorted by length (longest first) for proper regex matching
pub fn get_all_symbols() -> Vec<String> {
    let mut symbols = vec![
        "dha".to_string(),
        "ge".to_string(),
        "na".to_string(),
        "ka".to_string(),
        "ta".to_string(),
        "trka".to_string(),
        "terekita".to_string(),
        "dhin".to_string(),
    ];
    // Sort by length (longest first) to ensure proper regex matching
    symbols.sort_by(|a, b| b.len().cmp(&a.len()));
    symbols
}

/// Tabla notation pitch lookup
/// Maps Tabla bols to degrees (all map to degree 1 since tabla is percussion)
pub fn lookup(symbol: &str) -> Option<Degree> {
    match symbol {
        // Tabla bols - all map to degree 1 since tabla is percussion (pitch doesn't matter)
        "dha" => Some(Degree::N1),
        "ge" => Some(Degree::N1),
        "na" => Some(Degree::N1),
        "ka" => Some(Degree::N1),
        "ta" => Some(Degree::N1),
        "trka" => Some(Degree::N1),
        "terekita" => Some(Degree::N1),
        "dhin" => Some(Degree::N1),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tabla_bols() {
        // Test that all tabla bols map to degree N1 (since tabla is percussion)
        assert_eq!(lookup("dha"), Some(Degree::N1));
        assert_eq!(lookup("ge"), Some(Degree::N1));
        assert_eq!(lookup("na"), Some(Degree::N1));
        assert_eq!(lookup("ka"), Some(Degree::N1));
        assert_eq!(lookup("ta"), Some(Degree::N1));
        assert_eq!(lookup("trka"), Some(Degree::N1));
        assert_eq!(lookup("terekita"), Some(Degree::N1));
        assert_eq!(lookup("dhin"), Some(Degree::N1));
    }

    #[test]
    fn test_tabla_invalid() {
        assert_eq!(lookup("unknown"), None);
        assert_eq!(lookup(""), None);
        assert_eq!(lookup("X"), None);
    }
}