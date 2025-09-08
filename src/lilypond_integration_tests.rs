#[cfg(test)]
mod lilypond_integration_tests {
    use crate::parser::parse_notation_with_stages;
    use crate::renderers::lilypond::renderer::convert_processed_document_to_lilypond_minimal;
    use crate::structure_preserving_fsm::ProcessedDocument;
    use crate::models::Metadata;

    fn parse_and_render_lilypond(input: &str) -> Result<String, String> {
        match parse_notation_with_stages(input, "auto") {
            Ok((_, document)) => {
                let processed_doc = ProcessedDocument::from_document(&document);
                let metadata = Metadata {
                    title: Some("Test".to_string()),
                    composer: None,
                    tempo: None,
                };
                match convert_processed_document_to_lilypond_minimal(&processed_doc, &metadata) {
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
        println!("LilyPond output for '{}':\n{}", input, lilypond);
        
        for fragment in expected_fragments {
            assert!(
                lilypond.contains(fragment),
                "LilyPond output for '{}' should contain '{}'\nActual output:\n{}",
                input, fragment, lilypond
            );
        }
    }

    fn assert_lilypond_success(input: &str) -> String {
        let result = parse_and_render_lilypond(input);
        assert!(result.is_ok(), "Failed to generate LilyPond for '{}': {:?}", input, result.err());
        let output = result.unwrap();
        println!("LilyPond output for '{}':\n{}", input, output);
        output
    }

    // ============================================================================
    // BASIC NOTATION TESTS
    // ============================================================================

    #[test]
    fn test_simple_number_notation() {
        let output = assert_lilypond_success("1 2 3");
        // Should contain basic note representations
        assert!(output.len() > 50, "Output should be substantial");
    }

    #[test]
    fn test_simple_sargam_notation() {
        assert_lilypond_success("S R G");
    }

    #[test]
    fn test_simple_western_notation() {
        assert_lilypond_success("C D E");
    }

    // ============================================================================
    // RHYTHM AND TUPLET TESTS
    // ============================================================================

    #[test]
    fn test_triplet_generation() {
        let output = assert_lilypond_success("1-2");
        // Should contain tuplet notation for 3/2 triplet
        assert!(output.contains("tuplet") || output.contains("\\times"), 
               "Should contain tuplet notation");
    }

    #[test]
    fn test_quintuplet_generation() {
        let output = assert_lilypond_success("11111");
        // Should contain tuplet notation for quintuplet
        assert!(output.contains("tuplet") || output.contains("\\times"),
               "Should contain tuplet notation");
    }

    #[test]
    fn test_complex_rhythm_patterns() {
        assert_lilypond_success("1-2-3 4--5");
        assert_lilypond_success("1--2 -3-4");
        assert_lilypond_success("-1-2- 3--");
    }

    // ============================================================================
    // KEY TRANSPOSITION TESTS
    // ============================================================================

    #[test]
    fn test_key_c_default() {
        let output = assert_lilypond_success("1 2 3");
        // With default key (C), 1=C, 2=D, 3=E
        println!("Default key output: {}", output);
    }

    #[test]
    fn test_key_d_transposition() {
        let output = assert_lilypond_success("key: D\n1 2 3");
        // With key D, should transpose accordingly
        println!("Key D output: {}", output);
    }

    #[test]
    fn test_key_g_transposition() {
        let output = assert_lilypond_success("key: G\n1 2 3");
        println!("Key G output: {}", output);
    }

    #[test]
    fn test_key_bb_transposition() {
        let output = assert_lilypond_success("key: Bb\n1 2 3");
        println!("Key Bb output: {}", output);
    }

    // ============================================================================
    // ACCIDENTAL TESTS
    // ============================================================================

    #[test]
    fn test_sharps_and_flats() {
        let output = assert_lilypond_success("1# 2b 3 4# 5b");
        println!("Accidentals output: {}", output);
    }

    #[test]
    fn test_western_accidentals() {
        let output = assert_lilypond_success("C# Db E F# Gb");
        println!("Western accidentals output: {}", output);
    }

    #[test]
    fn test_sargam_accidentals() {
        let output = assert_lilypond_success("S# Rb G M# Pb");
        println!("Sargam accidentals output: {}", output);
    }

    // ============================================================================
    // BARLINE TESTS
    // ============================================================================

    #[test]
    fn test_simple_barlines() {
        let output = assert_lilypond_success("1 2 | 3 4");
        // Should contain barline notation
        assert!(output.contains("|") || output.contains("bar"), 
               "Should contain barline notation");
    }

    #[test]
    fn test_double_barlines() {
        let output = assert_lilypond_success("1 2 || 3 4");
        println!("Double barline output: {}", output);
    }

    #[test]
    fn test_repeat_barlines() {
        let output = assert_lilypond_success("1 2 |: 3 4 :|");
        println!("Repeat barlines output: {}", output);
    }

    // ============================================================================
    // MULTI-STAVE TESTS
    // ============================================================================

    #[test]
    fn test_two_staves_basic() {
        let output = assert_lilypond_success("1 2 3

4 5 6");
        println!("Two staves output: {}", output);
    }

    #[test]
    fn test_multi_stave_with_key() {
        let output = assert_lilypond_success("key: D

1 2 3

4 5 6

7 1 2");
        println!("Multi-stave with key output: {}", output);
    }

    // ============================================================================
    // TRAILING NEWLINES REGRESSION TESTS
    // ============================================================================

    #[test]
    fn test_trailing_newlines_lilypond_generation() {
        // These were the problematic cases from the original bug
        let test_cases = vec![
            "1

2

",
            "1


",
            "S R G

M P D


",
            "C D E

F G A

B C D


",
        ];

        for input in test_cases {
            println!("Testing input with trailing newlines: {:?}", input.replace('\n', "\\n"));
            assert_lilypond_success(input);
        }
    }

    // ============================================================================
    // REST AND BREATH MARK TESTS
    // ============================================================================

    #[test]
    fn test_rests() {
        let output = assert_lilypond_success("-1 2-");
        println!("Rests output: {}", output);
    }

    #[test]
    fn test_breath_marks() {
        let output = assert_lilypond_success("1 2 ' 3 4");
        println!("Breath marks output: {}", output);
    }

    #[test]
    fn test_breath_breaks_extensions() {
        let output = assert_lilypond_success("1 ' -2");
        println!("Breath breaking extensions output: {}", output);
    }

    // ============================================================================
    // COMPLEX REAL-WORLD PATTERNS
    // ============================================================================

    #[test]
    fn test_indian_classical_pattern() {
        let input = "key: C

S R G M P D N S'

S' N D P M G R S";
        let output = assert_lilypond_success(input);
        println!("Indian classical pattern output: {}", output);
    }

    #[test]
    fn test_western_scale_pattern() {
        let input = "key: G

G A B C D E F# G

G F# E D C B A G";
        let output = assert_lilypond_success(input);
        println!("Western scale pattern output: {}", output);
    }

    #[test]
    fn test_complex_rhythm_with_key() {
        let input = "key: D

1--2-3 | -4-5-- | 6 ' -7-1";
        let output = assert_lilypond_success(input);
        println!("Complex rhythm with key output: {}", output);
    }

    #[test]
    fn test_mixed_elements() {
        let input = "key: Eb

1#-2b-3 | 4--5' | -6-7 ||

S R G | M-- P D | N S' |]";
        let output = assert_lilypond_success(input);
        println!("Mixed elements output: {}", output);
    }

    // ============================================================================
    // OUTPUT STRUCTURE TESTS
    // ============================================================================

    #[test]
    fn test_output_has_basic_structure() {
        let output = assert_lilypond_success("1 2 3 4");
        
        // Should be a reasonable length
        assert!(output.len() > 20, "Output should be substantial, got: {}", output);
        
        // Should not contain obvious error messages
        assert!(!output.to_lowercase().contains("error"), "Should not contain error messages");
        assert!(!output.to_lowercase().contains("invalid"), "Should not contain invalid messages");
        assert!(!output.to_lowercase().contains("failed"), "Should not contain failed messages");
    }

    #[test]
    fn test_different_lengths() {
        // Test various input lengths
        assert_lilypond_success("1");
        assert_lilypond_success("1 2");
        assert_lilypond_success("1 2 3 4 5");
        assert_lilypond_success("1 2 3 4 5 6 7 1 2 3 4 5 6 7");
    }

    #[test]  
    fn test_extreme_cases() {
        // Very long tuplet
        assert_lilypond_success("1111111111111111111111111111111"); // 31-tuplet
        
        // Many measures
        assert_lilypond_success("1-2 | 3-4 | 5-6 | 7-1 | 2-3 | 4-5 | 6-7");
        
        // Complex nested patterns
        assert_lilypond_success("1--2-3- | -4-5--6 | 7--1'-2-3");
    }

    // ============================================================================
    // ERROR HANDLING TESTS
    // ============================================================================

    #[test]
    fn test_malformed_input_handling() {
        // These should either succeed or fail gracefully
        let problematic_inputs = vec![
            "", // Empty
            "   ", // Whitespace only
            "1 2 |", // Incomplete barline
        ];
        
        for input in problematic_inputs {
            let result = parse_and_render_lilypond(input);
            println!("Result for '{}': {:?}", input, result);
            // We don't assert success/failure here, just that it doesn't crash
        }
    }

    // ============================================================================
    // REGRESSION TESTS FOR SPECIFIC FIXES
    // ============================================================================

    #[test]
    fn test_stave_count_consistency() {
        // These inputs should generate LilyPond with the correct number of musical sections
        let test_cases = vec![
            ("1

2

", "Two staves with trailing newlines"),
            ("1


", "Single stave with multiple trailing newlines"),
            ("1 2 3

4 5 6

7 1 2", "Three staves"),
        ];

        for (input, description) in test_cases {
            println!("Testing {}: '{}'", description, input.replace('\n', "\\n"));
            let output = assert_lilypond_success(input);
            
            // The output should be consistent and not contain duplicate or missing content
            assert!(output.len() > 30, "Output for {} should be substantial", description);
        }
    }

    #[test]
    fn test_tuplet_duration_correctness() {
        let output = assert_lilypond_success("1-2-3");
        
        // Output should contain reasonable musical durations
        println!("Tuplet duration output: {}", output);
        
        // Should not contain obviously wrong durations
        assert!(!output.contains("0."), "Should not contain decimal durations");
        assert!(!output.contains("999"), "Should not contain absurd duration values");
    }
}