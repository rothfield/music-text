#[cfg(test)]
mod pest_grammar_tests {
    use pest::Parser;
    use crate::parser::{MusicTextParser, Rule};

    fn count_staves_in_pairs<'a>(pairs: pest::iterators::Pairs<'a, Rule>) -> usize {
        let mut count = 0;
        for pair in pairs {
            count_staves_in_pair(&pair, &mut count);
        }
        count
    }

    fn count_staves_in_pair(pair: &pest::iterators::Pair<Rule>, count: &mut usize) {
        match pair.as_rule() {
            Rule::stave => {
                *count += 1;
            }
            _ => {
                for inner in pair.clone().into_inner() {
                    count_staves_in_pair(&inner, count);
                }
            }
        }
    }

    #[test]
    fn test_two_staves_no_trailing_newlines() {
        let input = "1\n2";
        let result = MusicTextParser::parse(Rule::document, input);
        assert!(result.is_ok(), "Should parse successfully: {:?}", result.err());
        let stave_count = count_staves_in_pairs(result.unwrap());
        assert_eq!(stave_count, 2, "Should have exactly 2 staves");
    }

    #[test] 
    fn test_two_staves_with_trailing_newlines() {
        let input = "1\n2\n\n";
        let result = MusicTextParser::parse(Rule::document, input);
        
        if let Ok(pairs) = result {
            let stave_count = count_staves_in_pairs(pairs);
            // This test documents the CURRENT behavior (should fail initially)
            // We expect 2 staves but currently get 3
            assert_eq!(stave_count, 2, "Should have exactly 2 staves, but currently produces {}", stave_count);
        } else {
            panic!("Parse failed: {:?}", result.err());
        }
    }

    #[test]
    fn test_single_stave_with_trailing_newlines() {
        let input = "1\n\n";
        let result = MusicTextParser::parse(Rule::document, input);
        
        if let Ok(pairs) = result {
            let stave_count = count_staves_in_pairs(pairs);
            assert_eq!(stave_count, 1, "Should have exactly 1 stave");
        } else {
            panic!("Parse failed: {:?}", result.err());
        }
    }
}