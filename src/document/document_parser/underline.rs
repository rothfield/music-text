/// Check if a line is an underscore line (multi-stave marker)
pub fn is_underscore_line(content: &str) -> bool {
    let trimmed = content.trim();
    trimmed.len() >= 3 && trimmed.chars().all(|c| c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] 
    fn test_is_underscore_line() {
        assert!(is_underscore_line("___"));
        assert!(is_underscore_line("____"));
        assert!(is_underscore_line("  ___  "));
        assert!(!is_underscore_line("__"));
        assert!(!is_underscore_line("abc"));
    }
}