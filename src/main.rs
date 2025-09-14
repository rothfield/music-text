use clap::Parser;
use std::fs;
use std::path::PathBuf;

use music_text::parse;

#[derive(Parser)]
#[command(name = "music-txt")]
#[command(about = "A hand-written recursive descent music-text parser")]
struct Cli {
    /// Input file or string to parse
    #[arg(short, long)]
    input: Option<String>,
    
    /// Input file path
    #[arg(short, long)]
    file: Option<PathBuf>,
    
    /// Output format (json, debug)
    #[arg(short, long, default_value = "debug")]
    output: String,
    
    /// Start web server mode
    #[arg(long)]
    web: bool,
    
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Web server mode
    if cli.web {
        music_text::web::start_server().await?;
        return Ok(());
    }
    
    
    let input = if let Some(input_str) = cli.input {
        input_str
    } else if let Some(file_path) = cli.file {
        fs::read_to_string(file_path)?
    } else {
        eprintln!("Please provide either --input or --file");
        std::process::exit(1);
    };
    
    match music_text::parse::parse_document(&input) {
        Ok(document) => {
            match cli.output.as_str() {
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&document)?);
                }
                "debug" => {
                    println!("{:#?}", document);
                }
                _ => {
                    eprintln!("Unknown output format: {}", cli.output);
                    eprintln!("Available formats: json, debug");
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}  
