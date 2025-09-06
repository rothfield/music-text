use music_text::{parse_notation, Pair, Rule};

fn main() {
    let test_cases = vec![
        // Single pitches
        "1",
        "A",
        "1#",
        "1##",
        "Bb",
        "Bbb",
        
        // Content lines
        "1 2 3 4 5 6 7",
        "C D E F G A B",
        "1# 2b 3## | 4 5 | 6bb 7",
        
        // Staves with text
        "lyrics here\n1 2 3 4\nmore text",
        "1 2 | 3 4",
        
        // Multiple staves (should work - both have barlines)
        "|1\n\n|2",
        "1 | 2\n\n3 | 4",
        
        // Multiple staves (should fail - no barlines)
        "1\n\n2",
        
        // Test individual digit parsing
        "|1 22",
        
        // Single elements that should fail
        "2",
    ];
    
    for input in test_cases {
        println!("\nTesting: {:?}", input);
        match parse_notation(input) {
            Ok(pairs) => {
                println!("✓ Parsed successfully");
                for pair in pairs {
                    print_pair(&pair, 0);
                }
            }
            Err(e) => println!("✗ Parse error: {}", e),
        }
    }
}

fn print_pair(pair: &Pair<Rule>, indent: usize) {
    let indent_str = "  ".repeat(indent);
    println!("{}{:?}: {:?}", indent_str, pair.as_rule(), pair.as_str());
    for inner_pair in pair.clone().into_inner() {
        print_pair(&inner_pair, indent + 1);
    }
}