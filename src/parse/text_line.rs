/// Grammar rule: text_line = text_content (newline | EOI)
/// Fallback for any header line that's not title_line or directive_line

use crate::parse::model::TextLine;

pub fn parse(line: &str) -> TextLine {
    TextLine {
        value: Some(line.to_string()),
        char_index: 0, // TODO: Should be passed in from caller
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text_line() {
        // Text line always succeeds (fallback)
        let result = parse("Just some text");
        assert_eq!(result.value.as_ref().unwrap(), "Just some text");

        let result = parse("  Indented text  ");
        assert_eq!(result.value.as_ref().unwrap(), "  Indented text  ");

        let result = parse("");
        assert_eq!(result.value.as_ref().unwrap(), "");
    }
}