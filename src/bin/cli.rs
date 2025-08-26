use clap::Parser;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::io::{self, Read};
use serde_yaml;

use notation_parser::{
    unified_parser, lex_text, generate_flattened_spatial_view, 
    generate_outline,
    tokenize_with_handwritten_lexer,
    convert_to_lilypond, LilyPondNoteNames
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the input file (reads from stdin if not provided)
    input_file: Option<PathBuf>,
}





fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args = Args::parse();
    
    // Create base path for output files first
    let base_path = args.input_file.as_ref()
        .map(|p| p.clone())
        .unwrap_or_else(|| PathBuf::from("stdin"));
    
    // Determine output directory based on environment or default to test_output
    let output_dir_string = if args.input_file.is_some() || std::env::var("NOTATION_OUTPUT_DIR").is_ok() {
        std::env::var("NOTATION_OUTPUT_DIR")
            .unwrap_or_else(|_| "test_output".to_string())
    } else {
        // For stdin, keep outputs in current directory
        ".".to_string()
    };
    let output_dir = Path::new(&output_dir_string);
    
    // Create output directory if it doesn't exist
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }
    
    // Get just the filename from base_path
    let base_filename = base_path.file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("stdin"));
    
    // Read input from file or stdin
    let raw_text = match args.input_file {
        Some(file_path) => fs::read_to_string(&file_path)?,
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            buffer
        }
    };
    
    let lines_info = lex_text(&raw_text);

    let (document_v2, _spatial_analysis_yaml) = notation_parser::unified_parser_v2(&raw_text)?;
    let document: notation_parser::Document = document_v2.clone().into();

    // --- Use handwritten lexer for tokenization ---
    let all_tokens = tokenize_with_handwritten_lexer(&raw_text);
    
    // Build the raw tokenizer output
    let mut raw_tokenizer_output = String::new();
    for token in &all_tokens {
        raw_tokenizer_output.push_str(&token.value);
    }

    let tokenizer_output = format!("--- Tokenizer Output ---\n{}", raw_tokenizer_output);
    // .json file
    let lexer_json_path = output_dir.join(base_filename).with_extension("lexer.json");
    fs::write(&lexer_json_path, serde_json::to_string_pretty(&all_tokens)?)?;
    eprintln!("Wrote lexer JSON output to {}", lexer_json_path.display());

    // --- Flatten Spatial Relationships Artifacts ---
    // .yaml file
    let flattener_yaml_path = output_dir.join(base_filename).with_extension("flattener.yaml");
    let yaml_output = serde_yaml::to_string(&document)?;
    fs::write(&flattener_yaml_path, &yaml_output)?;
    eprintln!("Wrote flattener YAML output to {}", flattener_yaml_path.display());

    // Generate flattened view (no colorization)
    let main_lines: HashSet<usize> = all_tokens.iter().filter(|t| t.token_type == "BARLINE").map(|t| t.line).collect();
    let empty_styles = HashMap::new();
    let flattened_spatial_output = generate_flattened_spatial_view(&document, &lines_info, &empty_styles, &main_lines);
    let flattened_spatial_path = output_dir.join(base_filename).with_extension("flattener.clr");
    fs::write(&flattened_spatial_path, &flattened_spatial_output)?;
    eprintln!("Wrote flatten spatial relationships output to {}", flattened_spatial_path.display());

    // --- LilyPond Output using V2 converter ---
    let lilypond_output = notation_parser::lilypond_converter_v2::convert_document_v2_to_lilypond(&document_v2, LilyPondNoteNames::English, Some(&raw_text))
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let lilypond_path = output_dir.join(base_filename).with_extension("ly");
    fs::write(&lilypond_path, &lilypond_output)?;
    eprintln!("Wrote LilyPond output to {}", lilypond_path.display());

    // --- Generate Outline ---
    let outline_path = output_dir.join(base_filename).with_extension("outline");
    generate_outline(&document, outline_path.to_str().unwrap())?;
    eprintln!("Wrote outline to {}", outline_path.display());

    // --- Final Output to Stdout ---
    
    // Print output
    print!("{}", tokenizer_output);
    println!("\n\n");
    print!("{}", flattened_spatial_output);
    println!("\n\n");
    
    // Print outline output
    let outline_content = fs::read_to_string(&outline_path)?;
    println!("\n{}outline begin: {}\n{}\n{}outline end{}", 
        "_".repeat(5), 
        outline_path.file_stem().unwrap_or_default().to_str().unwrap_or_default(),
        outline_content.trim_end(),
        "_".repeat(15),
        "_".repeat(15));
    
    // Print VexFlow JSON output for testing
    match notation_parser::convert_fsm_to_vexflow(&document) {
        Ok(staves) => {
            match serde_json::to_string(&staves) {
                Ok(vexflow_json) => {
                    println!("\n--- VexFlow JSON Output ---");
                    println!("{}", vexflow_json);
                }
                Err(e) => {
                    println!("\n--- VexFlow JSON Output Error ---");
                    println!("{{\"error\": \"JSON serialization failed: {}\"}}", e);
                }
            }
        }
        Err(e) => {
            println!("\n--- VexFlow JSON Output Error ---");
            println!("{{\"error\": \"VexFlow conversion failed: {}\"}}", e);
        }
    }

    Ok(())
}
