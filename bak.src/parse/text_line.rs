/// Grammar rule: text_line = text_content (newline | EOI)
/// Fallback for any header line that's not title_line or directive_line

#[derive(Debug, Clone)]
pub struct TextLine {
    pub content: String,
}

pub fn parse(line: &str) -> TextLine {
    TextLine {
        content: line.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text_line() {
        // Text line always succeeds (fallback)
        let result = parse("Just some text");
        assert_eq!(result.content, "Just some text");

        let result = parse("  Indented text  ");
        assert_eq!(result.content, "  Indented text  ");

        let result = parse("");
        assert_eq!(result.content, "");
    }
}