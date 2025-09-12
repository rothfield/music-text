use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct PestParser;

/// Parses a string using the Pest parser and the `document` rule.
///
/// On success, it returns a `pest::iterators::Pairs` object representing the
/// Concrete Syntax Tree (CST).
///
/// On failure, it returns a `pest::error::Error`.
pub fn parse_with_pest(input: &str) -> Result<pest::iterators::Pairs<'_, Rule>, pest::error::Error<Rule>> {
    PestParser::parse(Rule::document, input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_notes_line() {
        let input = "| S R G M |\n";
        let result = parse_with_pest(input);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_directives_and_stave() {
        let input = r#"\nkey: C\ntime: 4/4\n\n| S R G M |\ndo re mi fa\n"#;
        let result = parse_with_pest(input);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_multiple_staves() {
        let input = r#"\n| S R |\ndo re\n\n| G M |\nmi fa\n"#;
        let result = parse_with_pest(input);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());
    }

    #[test]
    fn test_invalid_input() {
        let input = "this is not valid notation";
        let result = parse_with_pest(input);
        assert!(result.is_err());
    }
}
