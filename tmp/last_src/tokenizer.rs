use once_cell::sync::Lazy;
use regex::Regex;
use crate::parse::model::NotationSystem;
use crate::models::pitch_systems::{tabla, sargam, number, western, bhatkhande};

/// Compiled regex patterns for each notation system
static TABLA_RE: Lazy<Regex> = Lazy::new(|| build_regex_for_system(NotationSystem::Tabla));
static SARGAM_RE: Lazy<Regex> = Lazy::new(|| build_regex_for_system(NotationSystem::Sargam));
static NUMBER_RE: Lazy<Regex> = Lazy::new(|| build_regex_for_system(NotationSystem::Number));
static WESTERN_RE: Lazy<Regex> = Lazy::new(|| build_regex_for_system(NotationSystem::Western));
static BHATKHANDE_RE: Lazy<Regex> = Lazy::new(|| build_regex_for_system(NotationSystem::Bhatkhande));

/// Build regex pattern for a specific notation system
fn build_regex_for_system(system: NotationSystem) -> Regex {
    let symbols = match system {
        NotationSystem::Tabla => tabla::get_all_symbols(),
        NotationSystem::Sargam => sargam::get_all_symbols(),
        NotationSystem::Number => number::get_all_symbols(),
        NotationSystem::Western => western::get_all_symbols(),
        NotationSystem::Bhatkhande => bhatkhande::get_all_symbols(),
    };
    
    // Escape special regex characters and join with alternation
    let escaped_symbols: Vec<String> = symbols.iter()
        .map(|s| regex::escape(s))
        .collect();
    
    let pattern = format!("(?i)({})", escaped_symbols.join("|"));
    Regex::new(&pattern).expect("Failed to compile regex pattern")
}

/// Classify input text and return best matching notation system with confidence score
fn classify_notation_system(input: &str) -> (NotationSystem, f32) {
    let systems = [
        (NotationSystem::Tabla, &*TABLA_RE),
        (NotationSystem::Sargam, &*SARGAM_RE),
        (NotationSystem::Number, &*NUMBER_RE),
        (NotationSystem::Western, &*WESTERN_RE),
        (NotationSystem::Bhatkhande, &*BHATKHANDE_RE),
    ];
    
    let mut best_system = NotationSystem::Number; // Default
    let mut best_score = 0.0;
    
    for (system, regex) in &systems {
        let matches: Vec<_> = regex.find_iter(input).collect();
        let matched_chars: usize = matches.iter().map(|m| m.as_str().len()).sum();
        let total_musical_chars = input.chars().filter(|c| !c.is_whitespace() && *c != '|').count();
        
        let score = if total_musical_chars > 0 {
            matched_chars as f32 / total_musical_chars as f32
        } else {
            0.0
        };
        
        if score > best_score {
            best_score = score;
            best_system = *system;
        }
    }
    
    (best_system, best_score)
}

/// Tokenize input using the specified notation system
fn tokenize_with_system(input: &str, system: NotationSystem) -> Vec<&str> {
    let regex = match system {
        NotationSystem::Tabla => &*TABLA_RE,
        NotationSystem::Sargam => &*SARGAM_RE,
        NotationSystem::Number => &*NUMBER_RE,
        NotationSystem::Western => &*WESTERN_RE,
        NotationSystem::Bhatkhande => &*BHATKHANDE_RE,
    };
    
    regex.find_iter(input)
        .map(|mat| mat.as_str())
        .collect()
}

/// Main entry point: classify input and tokenize using best matching system
pub fn classify_and_tokenize(input: &str) -> (NotationSystem, Vec<&str>) {
    let (system, _score) = classify_notation_system(input);
    let tokens = tokenize_with_system(input, system);
    (system, tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tabla_classification_and_tokenization() {
        let (system, tokens) = classify_and_tokenize("tatatata");
        assert_eq!(system, NotationSystem::Tabla);
        assert_eq!(tokens, vec!["ta", "ta", "ta", "ta"]);
        
        let (system, tokens) = classify_and_tokenize("dhagekata");
        assert_eq!(system, NotationSystem::Tabla);
        assert_eq!(tokens, vec!["dha", "ge", "ka", "ta"]);
        
        let (system, tokens) = classify_and_tokenize("trkaterekita");
        assert_eq!(system, NotationSystem::Tabla);
        assert_eq!(tokens, vec!["trka", "terekita"]);
    }

    #[test]
    fn test_sargam_classification_and_tokenization() {
        let (system, tokens) = classify_and_tokenize("SRG");
        assert_eq!(system, NotationSystem::Sargam);
        assert_eq!(tokens, vec!["S", "R", "G"]);
        
        let (system, tokens) = classify_and_tokenize("P# S R");
        assert_eq!(system, NotationSystem::Sargam);
        assert_eq!(tokens, vec!["P#", "S", "R"]);
    }

    #[test]
    fn test_number_classification_and_tokenization() {
        let (system, tokens) = classify_and_tokenize("123");
        assert_eq!(system, NotationSystem::Number);
        assert_eq!(tokens, vec!["1", "2", "3"]);
        
        let (system, tokens) = classify_and_tokenize("1# 2bb 3");
        assert_eq!(system, NotationSystem::Number);
        assert_eq!(tokens, vec!["1#", "2bb", "3"]);
    }

    #[test]
    fn test_western_classification_and_tokenization() {
        let (system, tokens) = classify_and_tokenize("CDE");
        assert_eq!(system, NotationSystem::Western);
        assert_eq!(tokens, vec!["C", "D", "E"]);
        
        let (system, tokens) = classify_and_tokenize("F# Bb C##");
        assert_eq!(system, NotationSystem::Western);
        assert_eq!(tokens, vec!["F#", "Bb", "C##"]);
    }

    #[test]
    fn test_mixed_content_handling() {
        // Test case like "P# hello R |" - should ignore non-musical content
        let (system, tokens) = classify_and_tokenize("P# hello R |");
        assert_eq!(system, NotationSystem::Sargam);
        assert_eq!(tokens, vec!["P#", "R"]); // Should ignore non-musical content
    }

    #[test]
    fn test_case_insensitive_matching() {
        let (system, tokens) = classify_and_tokenize("TATATATA");
        assert_eq!(system, NotationSystem::Tabla);
        assert_eq!(tokens, vec!["TA", "TA", "TA", "TA"]); // Preserves original case
        
        let (system, tokens) = classify_and_tokenize("DhaGe");
        assert_eq!(system, NotationSystem::Tabla);
        assert_eq!(tokens, vec!["Dha", "Ge"]); // Preserves original case
    }
}