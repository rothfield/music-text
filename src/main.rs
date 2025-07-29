use clap::Parser;
use colored::*;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use colored::control;
use serde_yaml;

mod models;
mod lilypond_converter;
mod pitch;
mod lexer;
mod parser;
mod display;

use models::{ChunkInfo, LineInfo, Token, Title, Directive, Metadata, Document, Node, TokenType};
use lilypond_converter::convert_to_lilypond;
use pitch::{lookup_pitch, guess_notation, Notation};
use lexer::{lex_text, tokenize_chunk, parse_metadata};
use parser::{flatten_spatial_relationships, group_nodes_into_lines_and_beats};
use display::{generate_flattened_spatial_view, parse_css_for_ansi, colorize_string, generate_legend_string};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the input file
    input_file: PathBuf,
}

fn is_dash(value: &str) -> bool {
    value.chars().all(|c| c == '-')
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let raw_text = fs::read_to_string(&args.input_file)?;
    let lines_info = lex_text(&raw_text);

    // --- Stage 1: Lexer ---
    let mut all_tokens: Vec<Token> = Vec::new();
    for line in &lines_info {
        let mut current_pos = 0;
        for chunk in &line.chunks {
            if chunk.col > current_pos {
                all_tokens.push(Token {
                    token_type: TokenType::Whitespace.as_str().to_string(),
                    value: " ".repeat(chunk.col - current_pos),
                    line: line.line_number,
                    col: current_pos,
                });
            }
            all_tokens.extend(tokenize_chunk(&chunk.value, line.line_number, chunk.col));
            current_pos = chunk.col + chunk.value.len();
        }
        if current_pos < line.line_text.len() {
            all_tokens.push(Token {
                token_type: TokenType::Whitespace.as_str().to_string(),
                value: " ".repeat(line.line_text.len() - current_pos),
                line: line.line_number,
                col: current_pos,
            });
        }
        all_tokens.push(Token {
            token_type: "NEWLINE".to_string(),
            value: "\n".to_string(),
            line: line.line_number,
            col: line.line_text.len(),
        });
    }

    // --- Used Tokens ---
    let mut used_tokens: HashMap<String, String> = HashMap::new();
    for token in &all_tokens {
        used_tokens
            .entry(token.token_type.clone())
            .or_insert_with(|| token.value.clone());
    }

    // --- Lexer Artifacts ---
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
        "{}\n\n{}\n{}",
        legend_string_tokenizer,
        title_text.bold(),
        raw_tokenizer_output
    );

    // .clr file
    let tokenizer_clr_path = args.input_file.with_extension("tokenizer.clr");
    fs::write(&tokenizer_clr_path, &tokenizer_output)?;
    eprintln!("Wrote tokenizer output to {}", tokenizer_clr_path.display());
    // .json file
    let lexer_json_path = args.input_file.with_extension("lexer.json");
    fs::write(&lexer_json_path, serde_json::to_string_pretty(&all_tokens)?)?;
    eprintln!("Wrote lexer JSON output to {}", lexer_json_path.display());


    // --- Stage 1.5: Metadata Parser ---
    let (metadata, musical_tokens) = parse_metadata(&all_tokens);

    // --- Stage 2: Flatten Spatial Relationships ---
    let nodes = flatten_spatial_relationships(&musical_tokens, &lines_info);

    // --- Stage 3: Beat Grouping ---
    // Identify lines of music: lines with barlines OR lines with pitch tokens if no barlines
    let mut lines_of_music: Vec<usize> = all_tokens.iter().filter(|t| t.token_type == "BARLINE").map(|t| t.line).collect();
    
    // If no barlines found, include lines with pitch tokens
    if lines_of_music.is_empty() {
        lines_of_music = all_tokens.iter()
            .filter(|t| t.token_type == "PITCH" && !is_dash(&t.value))
            .map(|t| t.line)
            .collect();
        lines_of_music.sort();
        lines_of_music.dedup(); // Remove duplicates
    }
    let structured_nodes = group_nodes_into_lines_and_beats(&nodes, &lines_of_music);

    // --- Final Document Assembly ---
    let document = Document {
        metadata,
        nodes: structured_nodes,
    };

    // --- Flatten Spatial Relationships Artifacts ---
    // .yaml file
    let flattener_yaml_path = args.input_file.with_extension("flattener.yaml");
    let yaml_output = serde_yaml::to_string(&document)?;
    fs::write(&flattener_yaml_path, &yaml_output)?;
    eprintln!("Wrote flattener YAML output to {}", flattener_yaml_path.display());

    // .clr file
    let main_lines: HashSet<usize> = all_tokens.iter().filter(|t| t.token_type == "BARLINE").map(|t| t.line).collect();
    let legend_string_flattener = generate_legend_string(&styles, &used_tokens, Some(&document.metadata), true);
    let flattened_spatial_output = format!(
        "{}\n\n{}",
        legend_string_flattener,
        generate_flattened_spatial_view(&document, &lines_info, &styles, &main_lines)
    );
    let flattened_spatial_clr_path = args.input_file.with_extension("flattener.clr");
    fs::write(&flattened_spatial_clr_path, &flattened_spatial_output)?;
    eprintln!("Wrote flatten spatial relationships output to {}", flattened_spatial_clr_path.display());

    // --- LilyPond Output ---
    let lilypond_output = convert_to_lilypond(&document)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let lilypond_path = args.input_file.with_extension("ly");
    fs::write(&lilypond_path, &lilypond_output)?;
    eprintln!("Wrote LilyPond output to {}", lilypond_path.display());

    // --- Final Output to Stdout ---
    control::set_override(false); // Use terminal detection for stdout
    print!("{}", flattened_spatial_output);

    Ok(())
}