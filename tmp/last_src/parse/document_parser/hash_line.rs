/// Check if a line is a hash line (multi-stave marker)
pub fn is_hash_line(content: &str) -> bool {
    let trimmed = content.trim();
    trimmed.len() >= 3 && trimmed.chars().all(|c| c == '#')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] 
    fn test_is_hash_line() {
        assert!(is_hash_line("###"));
        assert!(is_hash_line("####"));
        assert!(is_hash_line("  ###  "));
        assert!(!is_hash_line("##"));
        assert!(!is_hash_line("abc"));
    }
}