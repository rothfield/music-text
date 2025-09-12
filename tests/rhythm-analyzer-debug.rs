#[cfg(test)]
mod rhythm_analyzer_debug {
    use music_text::pipeline::process_notation;

    #[test]
    fn test_123_rhythm_analysis() {
        let input = "123";
        
        let result = process_notation(input).unwrap();
        println!("=== Input: '{}' ===", input);
        
        // Check processed staves
        println!("\n=== Processed Staves ===");
        for (i, stave) in result.processed_staves.iter().enumerate() {
            println!("Stave {}: {} rhythm items", i, stave.rhythm_items.len());
            
            for (j, item) in stave.rhythm_items.iter().enumerate() {
                match item {
                    music_text::rhythm::Item::Beat(beat) => {
                        println!("  Item {}: Beat - divisions: {}, elements: {}, tied_to_previous: {}, is_tuplet: {}", 
                            j, beat.divisions, beat.elements.len(), beat.tied_to_previous, beat.is_tuplet);
                        
                        for (k, element) in beat.elements.iter().enumerate() {
                            match &element.event {
                                music_text::rhythm::Event::Note { degree, octave, children, slur } => {
                                    println!("    Element {}: Note - degree: {:?}, octave: {}, subdivisions: {}", 
                                        k, degree, octave, element.subdivisions);
                                },
                                music_text::rhythm::Event::Rest => {
                                    println!("    Element {}: Rest - subdivisions: {}", k, element.subdivisions);
                                }
                            }
                        }
                    },
                    music_text::rhythm::Item::Barline(barline_type, tala) => {
                        println!("  Item {}: Barline - type: {:?}, tala: {:?}", j, barline_type, tala);
                    },
                    music_text::rhythm::Item::Breathmark => {
                        println!("  Item {}: Breathmark", j);
                    },
                    music_text::rhythm::Item::Tonic(degree) => {
                        println!("  Item {}: Tonic - degree: {:?}", j, degree);
                    }
                }
            }
        }
        
        // Check VexFlow output
        println!("\n=== VexFlow Output ===");
        println!("{}", serde_json::to_string_pretty(&result.vexflow_data).unwrap());
        
        // Check LilyPond outputs
        println!("\n=== Minimal LilyPond ===");
        println!("{}", result.lilypond);
        
        println!("\n=== Full LilyPond ===");
        println!("{}", result.lilypond);
    }

    #[test]
    fn test_1_dash_dash_2_rhythm_analysis() {
        let input = "1- -2";
        
        let result = process_notation(input).unwrap();
        println!("=== Input: '{}' ===", input);
        
        // Check processed staves
        println!("\n=== Processed Staves ===");
        for (i, stave) in result.processed_staves.iter().enumerate() {
            println!("Stave {}: {} rhythm items", i, stave.rhythm_items.len());
            
            for (j, item) in stave.rhythm_items.iter().enumerate() {
                match item {
                    music_text::rhythm::Item::Beat(beat) => {
                        println!("  Item {}: Beat - divisions: {}, elements: {}, tied_to_previous: {}, is_tuplet: {}", 
                            j, beat.divisions, beat.elements.len(), beat.tied_to_previous, beat.is_tuplet);
                        
                        for (k, element) in beat.elements.iter().enumerate() {
                            match &element.event {
                                music_text::rhythm::Event::Note { degree, octave, children, slur } => {
                                    println!("    Element {}: Note - degree: {:?}, octave: {}, subdivisions: {}", 
                                        k, degree, octave, element.subdivisions);
                                },
                                music_text::rhythm::Event::Rest => {
                                    println!("    Element {}: Rest - subdivisions: {}", k, element.subdivisions);
                                }
                            }
                        }
                    },
                    music_text::rhythm::Item::Barline(barline_type, tala) => {
                        println!("  Item {}: Barline - type: {:?}, tala: {:?}", j, barline_type, tala);
                    },
                    music_text::rhythm::Item::Breathmark => {
                        println!("  Item {}: Breathmark", j);
                    },
                    music_text::rhythm::Item::Tonic(degree) => {
                        println!("  Item {}: Tonic - degree: {:?}", j, degree);
                    }
                }
            }
        }
        
        // Check VexFlow output
        println!("\n=== VexFlow Output ===");
        println!("{}", serde_json::to_string_pretty(&result.vexflow_data).unwrap());
    }
}