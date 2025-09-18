/// Grammar rule: header_line = title_line | directive_line | text_line

use crate::parse::{title_line, directive_line, text_line};

#[derive(Debug, Clone)]
pub enum HeaderLine {
    Title(title_line::TitleLine),
    Directive(directive_line::DirectiveLine),
    Text(text_line::TextLine),
}

pub fn parse(line: &str) -> HeaderLine {
    // Parse in grammar order: title_line | directive_line | text_line

    // Try title_line first
    if let Some(title) = title_line::parse(line) {
        return HeaderLine::Title(title);
    }

    // Try directive_line second
    if let Some(directive) = directive_line::parse(line) {
        return HeaderLine::Directive(directive);
    }

    // Fallback to text_line (always succeeds)
    HeaderLine::Text(text_line::parse(line))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header_line() {
        // Should parse as title_line
        let result = parse("        Amazing Grace        Bach");
        match result {
            HeaderLine::Title(title) => {
                assert_eq!(title.title, "Amazing Grace");
                assert_eq!(title.author, "Bach");
            }
            _ => panic!("Expected title line"),
        }

        // Should parse as directive_line
        let result = parse("Author: John Newton");
        match result {
            HeaderLine::Directive(directive) => {
                assert_eq!(directive.key, "Author");
                assert_eq!(directive.value, "John Newton");
            }
            _ => panic!("Expected directive line"),
        }

        // Should parse as text_line (fallback)
        let result = parse("Just some title text");
        match result {
            HeaderLine::Text(text) => {
                assert_eq!(text.content, "Just some title text");
            }
            _ => panic!("Expected text line"),
        }
    }
}