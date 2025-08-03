use std::env;
use std::fs;
use notation_parser::{parse_notation, get_fsm_vexflow_output};

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
    
    // Parse the notation
    let success = parse_notation(&input);
    if !success {
        eprintln!("{{\"error\": \"Failed to parse notation\"}}");
        std::process::exit(1);
    }
    
    // Get VexFlow FSM output and print to stdout
    let vexflow_output = get_fsm_vexflow_output();
    println!("{}", vexflow_output);
}