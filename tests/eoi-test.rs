#[cfg(test)]
mod eoi_tests {
    use music_text::pipeline::process_notation;

    #[test] 
    fn test_123_vs_123_4_eoi() {
        println!("=== Testing EOI handling ===");
        
        // Test 123 alone (EOI issue)
        let input1 = "123";
        let result1 = process_notation(input1).unwrap();
        println!("\n=== Input: '{}' ===", input1);
        
        for (i, stave) in result1.processed_staves.iter().enumerate() {
            println!("Stave {}: {} rhythm items", i, stave.rhythm_items.len());
            for (j, item) in stave.rhythm_items.iter().enumerate() {
                match item {
                    music_text::rhythm::Item::Beat(beat) => {
                        println!("  Item {}: Beat - divisions: {}, elements: {}, tied_to_previous: {}, is_tuplet: {}", 
                            j, beat.divisions, beat.elements.len(), beat.tied_to_previous, beat.is_tuplet);
                    },
                    _ => {
                        println!("  Item {}: Other", j);
                    }
                }
            }
        }
        
        // Test 123 4 (should work)
        let input2 = "123 4";
        let result2 = process_notation(input2).unwrap();
        println!("\n=== Input: '{}' ===", input2);
        
        for (i, stave) in result2.processed_staves.iter().enumerate() {
            println!("Stave {}: {} rhythm items", i, stave.rhythm_items.len());
            for (j, item) in stave.rhythm_items.iter().enumerate() {
                match item {
                    music_text::rhythm::Item::Beat(beat) => {
                        println!("  Item {}: Beat - divisions: {}, elements: {}, tied_to_previous: {}, is_tuplet: {}", 
                            j, beat.divisions, beat.elements.len(), beat.tied_to_previous, beat.is_tuplet);
                    },
                    _ => {
                        println!("  Item {}: Other", j);
                    }
                }
            }
        }
        
        println!("\n=== LilyPond Comparison ===");
        println!("123 lilypond: {}", result1.lilypond);
        println!("123 4 lilypond: {}", result2.lilypond);
    }
}