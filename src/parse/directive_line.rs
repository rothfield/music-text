/// Grammar rule: directive_line = directive whitespace* (newline | EOI)
/// Grammar rule: directive = key ":" value

#[derive(Debug, Clone)]
pub struct DirectiveLine {
    pub key: String,
    pub value: String,
}

pub fn parse(line: &str) -> Option<DirectiveLine> {
    let trimmed = line.trim();

    // Check for directive pattern: key ":" value
    if let Some(colon_pos) = trimmed.find(':') {
        let key = trimmed[..colon_pos].trim().to_string();
        let value = trimmed[colon_pos + 1..].trim().to_string();

        if !key.is_empty() {
            return Some(DirectiveLine { key, value });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_directive_line() {
        // Valid directive lines
        let result = parse("Author: John Newton");
        assert!(result.is_some());
        let directive = result.unwrap();
        assert_eq!(directive.key, "Author");
        assert_eq!(directive.value, "John Newton");

        let result = parse("Tempo: 120");
        assert!(result.is_some());
        let directive = result.unwrap();
        assert_eq!(directive.key, "Tempo");
        assert_eq!(directive.value, "120");

        let result = parse("Key: G major");
        assert!(result.is_some());
        let directive = result.unwrap();
        assert_eq!(directive.key, "Key");
        assert_eq!(directive.value, "G major");

        // With extra whitespace
        let result = parse("  Author  :   John Newton  ");
        assert!(result.is_some());
        let directive = result.unwrap();
        assert_eq!(directive.key, "Author");
        assert_eq!(directive.value, "John Newton");

        // Invalid - no colon
        let result = parse("Amazing Grace");
        assert!(result.is_none());

        // Invalid - empty key
        let result = parse(": John Newton");
        assert!(result.is_none());

        // Valid - empty value
        let result = parse("Author:");
        assert!(result.is_some());
        let directive = result.unwrap();
        assert_eq!(directive.key, "Author");
        assert_eq!(directive.value, "");
    }
}