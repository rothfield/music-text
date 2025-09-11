#[cfg(test)]
mod vexflow_tie_tests {
    use music_text::pipeline::process_notation;

    #[test]
    fn test_vexflow_tie_baseline_working_case() {
        // Test the working tie case: 1- -2
        let input = "1- -2";
        
        let result = process_notation(input).unwrap();
        println!("=== VexFlow Output for '1- -2' (Working Case) ===");
        
        let vexflow_json = &result.vexflow_data;
        println!("{}", serde_json::to_string_pretty(vexflow_json).unwrap());
        
        // Extract the staves and notes for inspection
        if let Some(staves) = vexflow_json.get("staves").and_then(|s| s.as_array()) {
            if let Some(first_stave) = staves.get(0) {
                if let Some(notes) = first_stave.get("notes").and_then(|n| n.as_array()) {
                    println!("\n=== Note Analysis ===");
                    for (i, note) in notes.iter().enumerate() {
                        if note.get("type").and_then(|t| t.as_str()) == Some("Note") {
                            let keys = note.get("keys").and_then(|k| k.as_array())
                                .map(|arr| arr.iter().map(|v| v.as_str().unwrap_or("")).collect::<Vec<_>>())
                                .unwrap_or_default();
                            let duration = note.get("duration").and_then(|d| d.as_str()).unwrap_or("");
                            let tied = note.get("tied").and_then(|t| t.as_bool()).unwrap_or(false);
                            
                            println!("Note {}: keys={:?}, duration={}, tied={}", i, keys, duration, tied);
                        }
                    }
                }
            }
        }
        
        // Verify the expected tie pattern
        assert!(vexflow_json.get("staves").is_some(), "Should have staves");
    }

    #[test]
    fn test_vexflow_tie_baseline_s_pattern() {
        // Test the S- -S pattern (compare with working case)
        let input = "S- -S";
        
        let result = process_notation(input).unwrap();
        println!("=== VexFlow Output for 'S- -S' (Pattern to Fix) ===");
        
        let vexflow_json = &result.vexflow_data;
        println!("{}", serde_json::to_string_pretty(vexflow_json).unwrap());
        
        // Extract the staves and notes for inspection
        if let Some(staves) = vexflow_json.get("staves").and_then(|s| s.as_array()) {
            if let Some(first_stave) = staves.get(0) {
                if let Some(notes) = first_stave.get("notes").and_then(|n| n.as_array()) {
                    println!("\n=== Note Analysis ===");
                    for (i, note) in notes.iter().enumerate() {
                        if note.get("type").and_then(|t| t.as_str()) == Some("Note") {
                            let keys = note.get("keys").and_then(|k| k.as_array())
                                .map(|arr| arr.iter().map(|v| v.as_str().unwrap_or("")).collect::<Vec<_>>())
                                .unwrap_or_default();
                            let duration = note.get("duration").and_then(|d| d.as_str()).unwrap_or("");
                            let tied = note.get("tied").and_then(|t| t.as_bool()).unwrap_or(false);
                            
                            println!("Note {}: keys={:?}, duration={}, tied={}", i, keys, duration, tied);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_vexflow_tie_expected_pattern() {
        // Test to verify the expected tie pattern for any working tie sequence
        let input = "1- -2";
        
        let result = process_notation(input).unwrap();
        
        let vexflow_json = &result.vexflow_data;
        if let Some(staves) = vexflow_json.get("staves").and_then(|s| s.as_array()) {
            if let Some(first_stave) = staves.get(0) {
                if let Some(notes) = first_stave.get("notes").and_then(|n| n.as_array()) {
                    let note_elements: Vec<_> = notes.iter()
                        .filter(|note| note.get("type").and_then(|t| t.as_str()) == Some("Note"))
                        .collect();
                    
                    // For 1- -2, we expect:
                    // Note 0 (1): tied=false (start of tie)
                    // Note 1 (tied continuation): tied=true (receives tie from previous)
                    // Note 2 (2): tied=false (end note)
                    
                    assert_eq!(note_elements.len(), 3, "Should have exactly 3 notes for '1- -2'");
                    
                    let first_tied = note_elements[0].get("tied").and_then(|t| t.as_bool()).unwrap_or(false);
                    let second_tied = note_elements[1].get("tied").and_then(|t| t.as_bool()).unwrap_or(false);
                    let third_tied = note_elements[2].get("tied").and_then(|t| t.as_bool()).unwrap_or(false);
                    
                    println!("Tie pattern: [{}, {}, {}]", first_tied, second_tied, third_tied);
                    
                    // The working pattern should be: [false, true, false]
                    // where only the middle note (continuation) has tied=true
                    assert!(!first_tied, "First note should not be tied");
                    assert!(second_tied, "Second note should be tied (receives tie)");
                    assert!(!third_tied, "Third note should not be tied");
                }
            }
        }
    }
}