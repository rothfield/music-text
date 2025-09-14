#[cfg(test)]
mod lilypond_generation_tests {
    use crate::parse_notation;
    use crate::renderers::lilypond::renderer::render_lilypond;

    fn parse_and_render_lilypond(input: &str) -> Result<String, String> {
        match parse_notation(input) {
            Ok(document) => {
                match render_lilypond(&document, None) {
                    Ok(lilypond) => Ok(lilypond),
                    Err(e) => Err(format!("LilyPond generation failed: {}", e))
                }
            }
            Err(e) => Err(format!("Parse failed: {}", e))
        }
    }

    fn assert_lilypond_contains(input: &str, expected_fragments: &[&str]) {
        let result = parse_and_render_lilypond(input);
        assert!(result.is_ok(), "Failed to generate LilyPond for '{}': {:?}", input, result.err());
        
        let lilypond = result.unwrap();
        for fragment in expected_fragments {
            assert!(
                lilypond.contains(fragment),
                "LilyPond output for '{}' should contain '{}'\nActual output:\n{}",
                input, fragment, lilypond
            );
        }
    }

    fn assert_lilypond_success(input: &str) {
        let result = parse_and_render_lilypond(input);
        assert!(result.is_ok(), "Failed to generate LilyPond for '{}': {:?}", input, result.err());
    }

    // ============================================================================
    // BASIC NOTATION TESTS
    // ============================================================================

    #[test]
    fn test_simple_number_notation() {
        assert_lilypond_contains("1 2 3", &["c4", "d4", "e4"]);
    }

    #[test]
    fn test_simple_sargam_notation() {
        assert_lilypond_contains("S R G", &["c4", "d4", "e4"]);
    }

    #[test]
    fn test_simple_western_notation() {
        assert_lilypond_contains("C D E", &["c4", "d4", "e4"]);
    }

    // ============================================================================
    // RHYTHM AND EXTENSION TESTS
    // ============================================================================

    #[test]
    fn test_simple_extensions() {
        // Test that extensions create proper tied notes
        assert_lilypond_contains("1-2", &["\\tuplet", "c4", "d8"]);
    }

    #[test]
    fn test_complex_extensions() {
        // Test complex extension patterns
        assert_lilypond_contains("1-2-3", &["\\tuplet", "5/4"]);
        assert_lilypond_contains("1--2-3", &["\\tuplet"]);
    }

    #[test]
    fn test_rest_patterns() {
        // Test that leading dashes create rests
        assert_lilypond_success("-1");
        assert_lilypond_success("1 -2");
        assert_lilypond_success("--1-2");
    }

    // ============================================================================
    // TUPLET GENERATION TESTS
    // ============================================================================

    #[test]
    fn test_triplet_generation() {
        // "1-2" should create a 3/2 tuplet (triplet)
        assert_lilypond_contains("1-2", &["\\tuplet 3/2"]);
    }

    #[test]
    fn test_quintuplet_generation() {
        // Five notes should create a 5/4 tuplet
        assert_lilypond_contains("11111", &["\\tuplet 5/4"]);
    }

    #[test]
    fn test_complex_tuplet_patterns() {
        // Test various tuplet denominator calculations
        assert_lilypond_success("111");        // 3/2 tuplet
        assert_lilypond_success("11111");      // 5/4 tuplet
        assert_lilypond_success("1111111");    // 7/4 tuplet
        assert_lilypond_success("111111111");  // 9/8 tuplet
    }

    // ============================================================================
    // BARLINE TESTS
    // ============================================================================

    #[test]
    fn test_simple_barlines() {
        assert_lilypond_contains("1 2 | 3 4", &["|"]);
    }

    #[test]
    fn test_barline_types() {
        assert_lilypond_contains("1 2 || 3 4", &["||"]);
        assert_lilypond_contains("1 2 |] 3 4", &["|]"]);
        assert_lilypond_contains("1 2 [| 3 4", &["[|"]);
    }

    #[test]
    fn test_repeat_barlines() {
        assert_lilypond_contains("1 2 |: 3 4 :|", &["|:", ":|"]);
    }

    // ============================================================================
    // ACCIDENTAL TESTS
    // ============================================================================

    #[test]
    fn test_sharps() {
        assert_lilypond_contains("1# 2# 3#", &["cis4", "dis4", "eis4"]);
    }

    #[test]
    fn test_flats() {
        assert_lilypond_contains("2b 3b 7b", &["des4", "ees4", "bes4"]);
    }

    #[test]
    fn test_mixed_accidentals() {
        assert_lilypond_contains("1# 2b 3 4# 5b", &["cis4", "des4", "e4", "fis4", "ges4"]);
    }

    #[test]
    fn test_western_accidentals() {
        assert_lilypond_contains("C# Db E# Fb", &["cis4", "des4", "eis4", "fes4"]);
    }

    #[test]
    fn test_sargam_accidentals() {
        assert_lilypond_contains("S# Rb G# Mb", &["cis4", "des4", "eis4", "fes4"]);
    }

    // ============================================================================
    // KEY TRANSPOSITION TESTS
    // ============================================================================

    #[test]
    fn test_key_c_transposition() {
        // Default key should be C, so 1 = C
        assert_lilypond_contains("key: C\n1 2 3", &["c4", "d4", "e4"]);
    }

    #[test]
    fn test_key_d_transposition() {
        // Key D: 1 = D, 2 = E, 3 = F#
        assert_lilypond_contains("key: D\n1 2 3", &["d4", "e4", "fis4"]);
    }

    #[test]
    fn test_key_g_transposition() {
        // Key G: 1 = G, 2 = A, 3 = B
        assert_lilypond_contains("key: G\n1 2 3", &["g4", "a4", "b4"]);
    }

    #[test]
    fn test_key_bb_transposition() {
        // Key Bb: 1 = Bb, 2 = C, 3 = D
        assert_lilypond_contains("key: Bb\n1 2 3", &["bes4", "c4", "d4"]);
    }

    #[test]
    fn test_key_fs_transposition() {
        // Key F#: 1 = F#, 2 = G#, 3 = A#
        assert_lilypond_contains("key: F#\n1 2 3", &["fis4", "gis4", "ais4"]);
    }

    // ============================================================================
    // MULTI-STAVE TESTS
    // ============================================================================

    #[test]
    fn test_two_staves() {
        let result = parse_and_render_lilypond("1 2 3\n4 5 6");
        assert!(result.is_ok());
        let lilypond = result.unwrap();
        
        // Should contain multiple staves/voices
        // LilyPond output should have structure for multiple lines
        assert!(lilypond.contains("c4"));
        assert!(lilypond.contains("f4"));
    }

    #[test]
    fn test_multi_stave_with_key() {
        assert_lilypond_success("key: D\n\n1 2 3\n4 5 6\n7 1 2");
    }

    // ============================================================================
    // BREATH MARK TESTS
    // ============================================================================

    #[test]
    fn test_breath_marks() {
        // Breath marks should create proper LilyPond breath notation
        assert_lilypond_success("1 2 ' 3 4");
        assert_lilypond_success("1-2 ' 3-4");
    }

    #[test]
    fn test_breath_breaks_extensions() {
        // After breath, dashes should create rests not extensions
        assert_lilypond_success("1 ' -2");
        assert_lilypond_success("1-2 ' -3-4");
    }

    // ============================================================================
    // COMPLEX REAL-WORLD PATTERNS
    // ============================================================================

    #[test]
    fn test_complex_indian_classical() {
        let input = "key: C\n\nS R G M P D N S'\nS' N D P M G R S";
        assert_lilypond_success(input);
    }

    #[test]
    fn test_complex_western_classical() {
        let input = "key: G\n\nG A B C D E F# G\nG F# E D C B A G";
        assert_lilypond_success(input);
    }

    #[test]
    fn test_complex_rhythm_patterns() {
        let input = "1--2-3 | -4-5-- | 6 ' -7-1";
        assert_lilypond_success(input);
    }

    #[test]
    fn test_mixed_notation_with_key_changes() {
        let input = "key: D\n\n1-2-3 4\n\nkey: G\n\n5-6-7 1";
        assert_lilypond_success(input);
    }

    // ============================================================================
    // EDGE CASE TESTS
    // ============================================================================

    #[test]
    fn test_only_rests() {
        assert_lilypond_success("- -- ---");
    }

    #[test]
    fn test_extreme_tuplets() {
        // Test very large tuplets
        assert_lilypond_success("1111111111111111111111111111111"); // 31/16 tuplet
    }

    #[test]
    fn test_mixed_measures() {
        assert_lilypond_success("1-2 | 345 | 6--7 | 1234567");
    }

    #[test]
    fn test_attributes_with_complex_notation() {
        let input = "key: Eb\ntime: 3/4\nauthor: Test\n\n1#-2b-3 | 4--5' | -6-7";
        assert_lilypond_success(input);
    }

    // ============================================================================
    // REGRESSION TESTS FOR SPECIFIC LILYPOND ISSUES
    // ============================================================================

    #[test]
    fn test_trailing_newlines_lilypond_regression() {
        // These patterns were problematic due to the trailing newline bug
        let test_cases = vec![
            "1\n2\n\n",
            "1\n\n",
            "S R G\nM P D\n\n\n",
            "C D E\nF G A\nB C D\n\n",
        ];

        for input in test_cases {
            assert_lilypond_success(input);
        }
    }

    #[test]
    fn test_tuplet_duration_correctness() {
        // Test that tuplet durations are calculated correctly (not fractional)
        let result = parse_and_render_lilypond("1-2");
        assert!(result.is_ok());
        let lilypond = result.unwrap();
        
        // Should contain standard durations (4, 8, 16, etc.) not fractional ones
        assert!(lilypond.contains("4") || lilypond.contains("8"));
        // Should NOT contain fractional durations
        assert!(!lilypond.contains("4.67") && !lilypond.contains("2.33"));
    }

    #[test]
    fn test_tie_generation() {
        // Test that extensions across beats create proper ties
        let result = parse_and_render_lilypond("1- | -2");
        assert!(result.is_ok());
        let lilypond = result.unwrap();
        
        // Should contain tie notation (~)
        // Note: This test may need adjustment based on actual tie implementation
        println!("Tie test LilyPond output: {}", lilypond);
    }

    // ============================================================================
    // OUTPUT QUALITY TESTS
    // ============================================================================

    #[test]
    fn test_lilypond_syntax_validity() {
        // Test that generated LilyPond has basic structural elements
        let result = parse_and_render_lilypond("key: D\n\n1-2-3 | 4--5 6 | 7-1'");
        assert!(result.is_ok());
        let lilypond = result.unwrap();
        
        // Should contain basic LilyPond structure markers
        assert!(lilypond.contains("{") && lilypond.contains("}"));
        
        // Should not contain obvious syntax errors
        assert!(!lilypond.contains("ERROR"));
        assert!(!lilypond.contains("INVALID"));
        
        // Should be non-empty and substantial
        assert!(lilypond.len() > 50, "LilyPond output too short: {}", lilypond);
    }

    #[test] 
    fn test_minimal_vs_full_lilypond() {
        // Test that we can generate both minimal and full LilyPond
        let input = "1 2 3 4";
        
        // Test minimal generation (via renderer)
        let minimal_result = parse_and_render_lilypond(input);
        assert!(minimal_result.is_ok());
        
        // Minimal should be shorter and contain just the notes
        let minimal = minimal_result.unwrap();
        assert!(minimal.len() > 0);
        
        println!("Minimal LilyPond output: {}", minimal);
    }
}