use clap::Parser;
use colored::*;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::io::{self, Read};
use colored::control;
use serde_yaml;

use notation_parser::{
    unified_parser, lex_text, convert_to_lilypond, generate_flattened_spatial_view, 
    generate_legend_string, generate_outline, Token, TokenType, Document, Node,
    tokenize_chunk, guess_notation, lookup_pitch, parse_css_for_ansi, colorize_string,
    format_document_to_text, tokenize_with_handwritten_lexer
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the input file (reads from stdin if not provided)
    input_file: Option<PathBuf>,
}

fn is_dash(value: &str) -> bool {
    value.chars().all(|c| c == '-')
}

fn format_header(section_type: &str, action: &str, filename: Option<&str>) -> String {
    let content = match filename {
        Some(name) => format!("{} {}: {}", section_type, action, name),
        None => format!("{} {}", section_type, action),
    };
    
    let total_length: usize = 40;
    let underscores_needed = total_length.saturating_sub(content.len());
    let left_underscores = underscores_needed / 2;
    let right_underscores = underscores_needed - left_underscores;
    
    format!("{}{}{}", 
            "_".repeat(left_underscores),
            content,
            "_".repeat(right_underscores))
}

fn _collect_node_types_from_document(document: &Document) -> HashMap<String, String> {
    
    fn collect_from_nodes(nodes: &[Node], types: &mut HashMap<String, String>) {
        for node in nodes {
            types.entry(node.node_type.clone()).or_insert_with(|| node.value.clone());
            collect_from_nodes(&node.nodes, types);
        }
    }
    
    let mut node_types = HashMap::new();
    collect_from_nodes(&document.nodes, &mut node_types);
    node_types
}

fn _find_musical_lines_by_packed_pitches(tokens: &[Token]) -> Vec<usize> {
    let mut musical_lines = Vec::new();
    let mut tokens_by_line: HashMap<usize, Vec<&Token>> = HashMap::new();
    
    // Group tokens by line
    for token in tokens {
        if token.token_type == "PITCH" {
            tokens_by_line.entry(token.line).or_default().push(token);
        }
    }
    
    // Check each line for 3+ packed pitches from same notation system
    for (line_num, line_tokens) in tokens_by_line {
        if has_packed_musical_sequence(&line_tokens) {
            musical_lines.push(line_num);
        }
    }
    
    musical_lines.sort();
    musical_lines.dedup();
    musical_lines
}

fn has_packed_musical_sequence(tokens: &[&Token]) -> bool {
    if tokens.len() < 3 {
        return false;
    }
    
    // Sort tokens by column position
    let mut sorted_tokens: Vec<&Token> = tokens.iter().cloned().collect();
    sorted_tokens.sort_by_key(|t| t.col);
    
    // Find sequences of consecutive columns
    let mut sequences = Vec::new();
    let mut current_sequence = Vec::new();
    
    for (i, token) in sorted_tokens.iter().enumerate() {
        if i == 0 || token.col == sorted_tokens[i-1].col + 1 {
            // Consecutive or first token
            current_sequence.push(*token);
        } else {
            // Gap found - save current sequence and start new one
            if current_sequence.len() >= 3 {
                sequences.push(current_sequence.clone());
            }
            current_sequence = vec![*token];
        }
    }
    
    // Don't forget the last sequence
    if current_sequence.len() >= 3 {
        sequences.push(current_sequence);
    }
    
    // Check if any sequence has 3+ pitches from same notation system
    for sequence in sequences {
        if sequence.len() >= 3 && is_same_notation_system(&sequence) {
            return true;
        }
    }
    
    false
}

fn is_same_notation_system(tokens: &[&Token]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    
    // Collect all pitch values (excluding dashes)
    let pitch_values: Vec<&str> = tokens.iter()
        .map(|t| t.value.as_str())
        .filter(|&v| !is_dash(v))
        .collect();
    
    if pitch_values.is_empty() {
        return false; // All dashes - not a valid musical sequence
    }
    
    // Guess notation from the pitch values
    let notation = guess_notation(&pitch_values);
    
    // Check if all non-dash pitches belong to the same notation system
    for &pitch_value in &pitch_values {
        if lookup_pitch(pitch_value, notation).is_none() {
            return false; // This pitch doesn't belong to the detected notation system
        }
    }
    
    true
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test formatter if TEST_FORMATTER is set
    if std::env::var("TEST_FORMATTER").is_ok() {
        let input = "| S\n ";
        println!("Testing formatter with input: {:?}", input);
        
        match unified_parser(input) {
            Ok(mut document) => {
                println!("Detected system: {:?}", document.metadata.detected_system);
                println!("Document notation_system: {:?}", document.notation_system);
                
                let formatted_original = format_document_to_text(&document);
                println!("Auto-converted: {:?}", formatted_original);
                
                // Test cross-system conversion
                document.notation_system = Some("Western".to_string());
                let formatted_western = format_document_to_text(&document);
                println!("To Western: {:?}", formatted_western);
                
                document.notation_system = Some("Number".to_string());
                let formatted_number = format_document_to_text(&document);
                println!("To Number: {:?}", formatted_number);
                
                println!("Roundtrip success: {}", input == formatted_original);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
        return Ok(());
    }

    let args = Args::parse();
    
    // Create base path for output files first
    let base_path = args.input_file.as_ref()
        .map(|p| p.clone())
        .unwrap_or_else(|| PathBuf::from("stdin"));
    
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

    let document = unified_parser(&raw_text)?;

    // --- Use handwritten lexer for tokenization ---
    let all_tokens = tokenize_with_handwritten_lexer(&raw_text);
    
    let mut used_tokens: HashMap<String, String> = HashMap::new();
    for token in &all_tokens {
        used_tokens
            .entry(token.token_type.clone())
            .or_insert_with(|| token.value.clone());
    }

    control::set_override(true);
    let styles = parse_css_for_ansi("styles.css");
    let legend_string_tokenizer = generate_legend_string(&styles, &used_tokens, None, false);

    // Build the raw tokenizer output first to calculate its width
    let mut raw_tokenizer_output = String::new();
    for token in &all_tokens {
        let (color, reverse) = styles
            .get(&token.token_type)
            .cloned()
            .unwrap_or(("white".to_string(), false));
        raw_tokenizer_output.push_str(&colorize_string(&token.value, &color, reverse));
    }

    let title_text = "--- Tokenizer Output ---";
    let tokenizer_output = format!(
        "{}

{}
{}",
        legend_string_tokenizer,
        title_text.bold(),
        raw_tokenizer_output
    );

    // .clr file
    let tokenizer_clr_path = base_path.with_extension("tokenizer.clr");
    fs::write(&tokenizer_clr_path, &tokenizer_output)?;
    eprintln!("Wrote tokenizer output to {}", tokenizer_clr_path.display());
    // .json file
    let lexer_json_path = base_path.with_extension("lexer.json");
    fs::write(&lexer_json_path, serde_json::to_string_pretty(&all_tokens)?)?;
    eprintln!("Wrote lexer JSON output to {}", lexer_json_path.display());

    // --- Flatten Spatial Relationships Artifacts ---
    // .yaml file
    let flattener_yaml_path = base_path.with_extension("flattener.yaml");
    let yaml_output = serde_yaml::to_string(&document)?;
    fs::write(&flattener_yaml_path, &yaml_output)?;
    eprintln!("Wrote flattener YAML output to {}", flattener_yaml_path.display());

    // .clr file
    let main_lines: HashSet<usize> = all_tokens.iter().filter(|t| t.token_type == "BARLINE").map(|t| t.line).collect();
    let legend_string_flattener = generate_legend_string(&styles, &used_tokens, Some(&document.metadata), true);
    let flattened_spatial_output = format!(
        "{}

{}",
        legend_string_flattener,
        generate_flattened_spatial_view(&document, &lines_info, &styles, &main_lines)
    );
    let flattened_spatial_clr_path = base_path.with_extension("flattener.clr");
    fs::write(&flattened_spatial_clr_path, &flattened_spatial_output)?;
    eprintln!("Wrote flatten spatial relationships output to {}", flattened_spatial_clr_path.display());

    // --- LilyPond Output ---
    let lilypond_output = convert_to_lilypond(&document)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let lilypond_path = base_path.with_extension("ly");
    fs::write(&lilypond_path, &lilypond_output)?;
    eprintln!("Wrote LilyPond output to {}", lilypond_path.display());

    // --- Generate Outline ---
    let outline_path = base_path.with_extension("outline");
    generate_outline(&document, outline_path.to_str().unwrap())?;
    eprintln!("Wrote outline to {}", outline_path.display());

    // --- Final Output to Stdout ---
    control::set_override(true); // Keep colors enabled for stdout output
    
    // Print tokenizer output
    let tokenizer_filename = tokenizer_clr_path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("stdin.tokenizer.clr");
    println!("\n\n{}", format_header("colorizer", "begin", Some(tokenizer_filename)));
    print!("{}", tokenizer_output);
    println!("\n\n{}\n", format_header("colorizer", "end", None));
    
    // Print flattener output
    let flattener_filename = flattened_spatial_clr_path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("stdin.flattener.clr");
    println!("\n\n{}", format_header("colorizer", "begin", Some(flattener_filename)));
    print!("{}", flattened_spatial_output);
    println!("\n\n{}\n", format_header("colorizer", "end", None));
    
    // Print outline output
    let outline_content = fs::read_to_string(&outline_path)?;
    let outline_filename = outline_path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("stdin.outline");
    println!("\n\n{}", format_header("outline", "begin", Some(outline_filename)));
    print!("{}", outline_content);
    println!("\n\n{}\n", format_header("outline", "end", None));

    Ok(())
}
