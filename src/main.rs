use music_text::{parse_notation, pest_pair_to_json, process_notation};
use std::env;
use std::io::{self, Read};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} [stage] [input]", args[0]);
        eprintln!("       {} [stage] < input_file", args[0]);
        eprintln!();
        eprintln!("Stages:");
        eprintln!("  pest        - Show raw PEST parse tree");
        eprintln!("  document    - Show parsed document structure (JSON)");
        eprintln!("  processed   - Show processed staves (JSON)");
        eprintln!("  minimal-lily - Show minimal LilyPond notation");
        eprintln!("  full-lily   - Show full LilyPond score");
        eprintln!("  vexflow     - Show VexFlow data structure (JSON)");
        eprintln!("  vexflow-svg - Show VexFlow SVG rendering");
        eprintln!("  all         - Show all stages");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} pest \"|1 2 3\"", args[0]);
        eprintln!("  echo \"|1 2 3\" | {} document", args[0]);
        return;
    }
    
    let stage = &args[1];
    
    // Get input from command line argument or stdin
    let input = if args.len() >= 3 {
        args[2].clone()
    } else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).unwrap_or_else(|_| {
            eprintln!("Error reading from stdin");
            std::process::exit(1);
        });
        buffer.trim().to_string()
    };
    
    if input.is_empty() {
        eprintln!("Error: No input provided");
        return;
    }
    
    match stage.as_str() {
        "pest" => show_pest_output(&input),
        "document" => show_document(&input),
        "processed" => show_processed(&input),
        "minimal-lily" => show_minimal_lilypond(&input),
        "full-lily" => show_full_lilypond(&input),
        "vexflow" => show_vexflow(&input),
        "vexflow-svg" => show_vexflow_svg(&input),
        "all" => show_all_stages(&input),
        _ => {
            eprintln!("Error: Unknown stage '{}'", stage);
            eprintln!("Valid stages: pest, document, processed, minimal-lily, full-lily, vexflow, vexflow-svg, all");
        }
    }
}

fn show_pest_output(input: &str) {
    match parse_notation(input) {
        Ok(pairs) => {
            let result: Vec<serde_json::Value> = pairs
                .map(|pair| pest_pair_to_json(&pair))
                .collect();
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    }
}

fn show_document(input: &str) {
    match process_notation(input) {
        Ok(result) => {
            println!("{}", serde_json::to_string_pretty(&result.parsed_document).unwrap());
        }
        Err(e) => {
            eprintln!("Processing error: {}", e);
            std::process::exit(1);
        }
    }
}

fn show_processed(input: &str) {
    match process_notation(input) {
        Ok(result) => {
            println!("{}", serde_json::to_string_pretty(&result.processed_staves).unwrap());
        }
        Err(e) => {
            eprintln!("Processing error: {}", e);
            std::process::exit(1);
        }
    }
}

fn show_minimal_lilypond(input: &str) {
    match process_notation(input) {
        Ok(result) => {
            println!("{}", result.minimal_lilypond);
        }
        Err(e) => {
            eprintln!("Processing error: {}", e);
            std::process::exit(1);
        }
    }
}

fn show_full_lilypond(input: &str) {
    match process_notation(input) {
        Ok(result) => {
            println!("{}", result.full_lilypond);
        }
        Err(e) => {
            eprintln!("Processing error: {}", e);
            std::process::exit(1);
        }
    }
}

fn show_vexflow(input: &str) {
    match process_notation(input) {
        Ok(result) => {
            println!("{}", serde_json::to_string_pretty(&result.vexflow_data).unwrap());
        }
        Err(e) => {
            eprintln!("Processing error: {}", e);
            std::process::exit(1);
        }
    }
}

fn show_vexflow_svg(input: &str) {
    match process_notation(input) {
        Ok(result) => {
            println!("{}", result.vexflow_svg);
        }
        Err(e) => {
            eprintln!("Processing error: {}", e);
            std::process::exit(1);
        }
    }
}

fn show_all_stages(input: &str) {
    println!("=== INPUT ===");
    println!("{}\n", input);
    
    println!("=== PEST PARSE TREE ===");
    show_pest_output(input);
    println!();
    
    match process_notation(input) {
        Ok(result) => {
            println!("=== PARSED DOCUMENT ===");
            println!("{}\n", serde_json::to_string_pretty(&result.parsed_document).unwrap());
            
            println!("=== PROCESSED STAVES ===");
            println!("{}\n", serde_json::to_string_pretty(&result.processed_staves).unwrap());
            
            println!("=== MINIMAL LILYPOND ===");
            println!("{}\n", result.minimal_lilypond);
            
            println!("=== FULL LILYPOND ===");
            println!("{}\n", result.full_lilypond);
            
            println!("=== VEXFLOW DATA ===");
            println!("{}\n", serde_json::to_string_pretty(&result.vexflow_data).unwrap());
            
            println!("=== VEXFLOW SVG ===");
            println!("{}", result.vexflow_svg);
        }
        Err(e) => {
            eprintln!("Processing error: {}", e);
            std::process::exit(1);
        }
    }
}