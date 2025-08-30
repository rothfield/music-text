use std::env;
use std::fs;
use music_text_parser::{parse_notation, convert_elements_to_staff_notation};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        std::process::exit(1);
    }
    
    let filename = &args[1];
    let input = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file {}: {}", filename, e);
            std::process::exit(1);
        }
    };
    
    // Parse the notation to get document
    let result = parse_notation(&input);
    if !result.success() {
        eprintln!("{{\"error\": \"Failed to parse notation\"}}");
        std::process::exit(1);
    }
    
    // Get FSM output and convert using V2 converter
    let elements = music_text_parser::get_last_elements();
    let document = result.get_document().expect("Document should exist");
    
    match convert_elements_to_staff_notation(&elements, &document.metadata) {
        Ok(staves) => {
            match serde_json::to_string(&staves) {
                Ok(json) => println!("{}", json),
                Err(e) => eprintln!("{{\"error\": \"JSON serialization failed: {}\"}}", e)
            }
        }
        Err(e) => eprintln!("{{\"error\": \"VexFlow conversion failed: {}\"}}", e)
    }
}