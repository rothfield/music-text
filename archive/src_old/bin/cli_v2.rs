use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::io::{self, Read};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the input file (reads from stdin if not provided)
    input_file: Option<PathBuf>,
    
    /// Generate VexFlow JavaScript output
    #[arg(long)]
    to_vexflow: bool,
    
    /// Generate HTML/CSS output
    #[arg(long)]
    to_html: bool,
    
    /// Notation system (number, sargam, western)
    #[arg(long, default_value = "number")]
    system: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Read input
    let raw_text = match args.input_file {
        Some(path) => fs::read_to_string(path)?,
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            buffer
        }
    };
    
    if args.to_vexflow {
        // Parse with FSM for VexFlow output
        match music_text_parser::parse_with_fsm(&raw_text) {
            Ok(document_with_fsm) => {
                let fsm_elements = &document_with_fsm.staves[0].fsm_output;
                
                // Convert to V1 metadata format for VexFlow converter compatibility
                let metadata = music_text_parser::Metadata {
                    title: None,
                    directives: document_with_fsm.directives.into_iter().map(|(k, v)| {
                        music_text_parser::Directive { key: k, value: v, row: 0, col: 0 }
                    }).collect(),
                    detected_system: Some(args.system),
                    attributes: std::collections::HashMap::new(),
                };
                
                let vexflow_js = music_text_parser::converters::vexflow::convert_elements_to_vexflow_js(
                    fsm_elements, &metadata
                )?;
                
                println!("{}", vexflow_js);
            }
            Err(e) => {
                eprintln!("Parse error: {}", e);
                std::process::exit(1);
            }
        }
    } else if args.to_html {
        // Generate HTML/CSS output
        let html = music_text_parser::convert_v2_to_html_css(&raw_text, Some(args.system));
        println!("{}", html);
    } else {
        // Default: just parse and show document structure
        match music_text_parser::parse(&raw_text) {
            Ok(document) => {
                println!("✅ Parse successful!");
                println!("Staves: {}", document.staves.len());
                for (i, stave) in document.staves.iter().enumerate() {
                    println!("  Stave {}: {} content elements", i + 1, stave.content.elements.len());
                }
            }
            Err(e) => {
                eprintln!("❌ Parse failed: {}", e);
                std::process::exit(1);
            }
        }
    }
    
    Ok(())
}