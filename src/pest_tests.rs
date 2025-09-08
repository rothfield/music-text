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

    fn count_beats_in_pairs<'a>(pairs: pest::iterators::Pairs<'a, Rule>) -> usize {
        let mut count = 0;
        for pair in pairs {
            count_beats_in_pair(&pair, &mut count);
        }
        count
    }

    fn count_beats_in_pair(pair: &pest::iterators::Pair<Rule>, count: &mut usize) {
        match pair.as_rule() {
            Rule::beat | Rule::simple_beat => {
                *count += 1;
            }
            _ => {
                for inner in pair.clone().into_inner() {
                    count_beats_in_pair(&inner, count);
                }
            }
        }
    }

    fn parse_successfully(input: &str) -> pest::iterators::Pairs<Rule> {
        let result = MusicTextParser::parse(Rule::document, input);
        assert!(result.is_ok(), "Failed to parse '{}': {:?}", input, result.err());
        result.unwrap()
    }

    // ============================================================================
    // STAVE COUNT TESTS (Regression tests for trailing newline fix)
    // ============================================================================

    #[test]
    fn test_single_stave_no_newlines() {
        let pairs = parse_successfully("1");
        assert_eq!(count_staves_in_pairs(pairs), 1, "Single note should be 1 stave");
    }

    #[test]
    fn test_single_stave_with_trailing_newlines() {
        let pairs = parse_successfully("1\n\n");
        assert_eq!(count_staves_in_pairs(pairs), 1, "Single note + trailing newlines should be 1 stave");
    }

    #[test]
    fn test_two_staves_no_trailing_newlines() {
        let pairs = parse_successfully("1\n2");
        assert_eq!(count_staves_in_pairs(pairs), 2, "Two notes separated by newline should be 2 staves");
    }

    #[test] 
    fn test_two_staves_with_trailing_newlines() {
        let pairs = parse_successfully("1\n2\n\n");
        assert_eq!(count_staves_in_pairs(pairs), 2, "Two staves + trailing newlines should be 2 staves");
    }

    #[test]
    fn test_three_staves_with_mixed_spacing() {
        let pairs = parse_successfully("1\n\n2\n\n\n3\n\n");
        assert_eq!(count_staves_in_pairs(pairs), 3, "Three staves with varied spacing should be 3 staves");
    }

    #[test]
    fn test_staves_with_multiple_beats() {
        let pairs = parse_successfully("1-2 3\n\n4 5-6\n\n");
        assert_eq!(count_staves_in_pairs(pairs), 2, "Multi-beat staves should count correctly");
    }

    // ============================================================================
    // NOTATION SYSTEM TESTS
    // ============================================================================

    #[test]
    fn test_number_notation_basic() {
        let pairs = parse_successfully("1 2 3 4 5 6 7");
        assert_eq!(count_staves_in_pairs(pairs.clone()), 1);
        assert!(count_beats_in_pairs(pairs) >= 7, "Should have at least 7 beats");
    }

    #[test]
    fn test_sargam_notation_basic() {
        let pairs = parse_successfully("S R G M P D N");
        assert_eq!(count_staves_in_pairs(pairs.clone()), 1);
        assert!(count_beats_in_pairs(pairs) >= 7, "Should have at least 7 beats");
    }

    #[test]
    fn test_western_notation_basic() {
        let pairs = parse_successfully("C D E F G A B");
        assert_eq!(count_staves_in_pairs(pairs.clone()), 1);
        assert!(count_beats_in_pairs(pairs) >= 7, "Should have at least 7 beats");
    }

    #[test]
    fn test_mixed_notation_systems() {
        // Each line uses different notation system
        let pairs = parse_successfully("1 2 3\nS R G\nC D E");
        assert_eq!(count_staves_in_pairs(pairs), 3, "Mixed notation systems should create separate staves");
    }

    // ============================================================================
    // RHYTHM AND EXTENSION TESTS  
    // ============================================================================

    #[test]
    fn test_simple_extensions() {
        parse_successfully("1-");
        parse_successfully("1--");
        parse_successfully("1---");
    }

    #[test]
    fn test_extension_patterns() {
        parse_successfully("1-2-3");
        parse_successfully("1--2-3");
        parse_successfully("1-2--3");
    }

    #[test]
    fn test_rest_patterns() {
        parse_successfully("-1");
        parse_successfully("--1");
        parse_successfully("1 -2");
    }

    #[test]
    fn test_complex_rhythm_patterns() {
        parse_successfully("1-2-3 4--5");
        parse_successfully("1--2 -3-4");
        parse_successfully("-1-2- 3--");
    }

    // ============================================================================
    // BARLINE TESTS
    // ============================================================================

    #[test]
    fn test_simple_barlines() {
        parse_successfully("1 2 | 3 4");
        parse_successfully("1-2 | 3-4");
    }

    #[test]
    fn test_barline_types() {
        parse_successfully("1 2 || 3 4");      // Double barline
        parse_successfully("1 2 |] 3 4");      // Final barline
        parse_successfully("1 2 [| 3 4");      // Reverse final barline
        parse_successfully("1 2 |: 3 4 :|");   // Repeat barlines
    }

    #[test]
    fn test_barlines_with_newlines() {
        parse_successfully("1 2 |\n3 4");
        parse_successfully("1 2 ||\n\n3 4");
    }

    // ============================================================================
    // ACCIDENTAL TESTS
    // ============================================================================

    #[test]
    fn test_sharps_and_flats() {
        parse_successfully("1# 2b 3# 4");
        parse_successfully("C# Db E# Fb");
        parse_successfully("S# Rb G# Mb");
    }

    #[test]
    fn test_accidentals_with_extensions() {
        parse_successfully("1#- 2b-3#");
        parse_successfully("C#-- Db-E#");
    }

    // ============================================================================
    // BREATH MARK TESTS
    // ============================================================================

    #[test]
    fn test_breath_marks() {
        parse_successfully("1 2 ' 3 4");
        parse_successfully("1-2 ' 3-4");
        parse_successfully("1 ' 2 ' 3");
    }

    #[test]
    fn test_breath_marks_break_extensions() {
        // After breath mark, dashes should create rests, not extensions
        parse_successfully("1 ' -2");
        parse_successfully("1-2 ' -3-4");
    }

    // ============================================================================
    // ATTRIBUTES AND KEY TESTS
    // ============================================================================

    #[test]
    fn test_key_attributes() {
        parse_successfully("key: C\n1 2 3");
        parse_successfully("key: D\n1 2 3");
        parse_successfully("key: Bb\n1 2 3");
    }

    #[test]
    fn test_multiple_attributes() {
        parse_successfully("key: D\ntime: 4/4\nauthor: Test\n\n1 2 3");
    }

    // ============================================================================
    // SLUR TESTS (using underscores) - TODO: Implement if needed
    // ============================================================================

    // #[test]
    // fn test_slur_notation() {
    //     parse_successfully("__\n1 2 3");  // Slur above
    //     parse_successfully("___\n1 2 3"); // Longer slur
    // }

    // #[test]
    // fn test_multiple_slurs() {
    //     parse_successfully("__ __\n1 2 3 4");
    //     parse_successfully("___  __\n1 2 3 4 5");
    // }

    // ============================================================================
    // EDGE CASES AND ERROR CONDITIONS
    // ============================================================================

    #[test]
    fn test_empty_measures() {
        parse_successfully("1 2 ||  | 3 4");  // Empty measure between barlines
    }

    #[test]
    fn test_leading_trailing_whitespace() {
        parse_successfully("  1 2 3  ");
        parse_successfully("\n\n1 2 3\n\n");
        parse_successfully("\t1\t2\t3\t");
    }

    // Whitespace-only documents are actually valid in this grammar
    // They just don't produce any musical content

    #[test]
    fn test_complex_real_world_patterns() {
        // Real-world examples that should parse correctly
        parse_successfully("key: D\n\n1-2-3 | 4--5 6 | 7-1'\n\n2-3 4-- | 5 6 7");
        
        parse_successfully("S R G M | P- D N S'\nR G M P | D- N S R");
        
        parse_successfully("C D E F | G-- A B | C'-B A G | F E D C");
        
        // Complex rhythm with mixed extensions and rests
        parse_successfully("1--2-3 | -4-5-- | 6 ' -7-1");
    }

    #[test]
    fn test_multi_stave_complex_patterns() {
        let input = "key: G\n\n1-2-3 4\n\nS R G M P\n\nC D E F G";
        let pairs = parse_successfully(input);
        assert_eq!(count_staves_in_pairs(pairs), 3, "Complex multi-stave pattern should parse correctly");
    }

    // ============================================================================
    // REGRESSION TESTS FOR SPECIFIC BUG FIXES
    // ============================================================================

    #[test]
    fn test_trailing_newlines_regression() {
        // These are the specific cases that were broken before the fix
        let test_cases = vec![
            ("1\n2\n\n", 2),
            ("1\n\n", 1),
            ("1\n2\n3\n\n\n", 3),
            ("S\nR\n\n", 2),
            ("C\nD\nE\n\n\n\n", 3),
        ];

        for (input, expected_staves) in test_cases {
            let pairs = parse_successfully(input);
            let actual_staves = count_staves_in_pairs(pairs);
            assert_eq!(
                actual_staves, expected_staves,
                "Input '{}' should produce {} staves, got {}",
                input.replace('\n', "\\n"), expected_staves, actual_staves
            );
        }
    }
}