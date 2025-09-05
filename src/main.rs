use clap::Parser;
use std::fs;
use std::path::PathBuf;

use music_text::parse;

#[derive(Parser)]
#[command(name = "music-txt")]
#[command(about = "A pest grammar-based music-text parser")]
struct Cli {
    /// Input file or string to parse
    #[arg(short, long)]
    input: Option<String>,
    
    /// Input file path
    #[arg(short, long)]
    file: Option<PathBuf>,
    
    /// Output format (web, json, debug)  
    #[arg(short, long, default_value = "web")]
    output: String,
    
    /// Notation system (auto, sargam, number, western, abc, doremi)
    #[arg(short, long, default_value = "auto")]
    system: String,
    
    /// Start web server mode
    #[arg(long)]
    web: bool,
    
    /// Test lower octave FSM integration
    #[arg(long)]
    test_lower_octave: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Web server mode
    if cli.web {
        music_text::web::start_server().await?;
        return Ok(());
    }
    
    // Test lower octave integration
    if cli.test_lower_octave {
        println!("Testing lower octave FSM integration:");
        music_text::parser::test_lower_octave_integration();
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
    
    // Don't remove empty lines - they're semantically important for separating staves
    
    let result = parse(&input, Some(&cli.system));
    
    if !result.success {
        eprintln!("Parse error: {}", result.error_message.unwrap_or("Unknown error".to_string()));
        std::process::exit(1);
    }
    
    match cli.output.as_str() {
        "web" => {
            // Output same format as web API
            let response = serde_json::json!({
                "success": result.success,
                "error": result.error_message,
                "ast": result.ast,
                "spatial": result.spatial,
                "fsm": result.fsm,
                "vexflow": result.vexflow,
                "lilypond": result.lilypond,
                "yaml": result.yaml
            });
            println!("{}", serde_json::to_string_pretty(&response)?);
        }
        "json" => {
            if let Some(document) = result.document {
                println!("{}", serde_json::to_string_pretty(&document)?);
            } else {
                eprintln!("No document generated");
                std::process::exit(1);
            }
        }
        "debug" => {
            if let Some(document) = result.document {
                println!("{:#?}", document);
            } else {
                eprintln!("No document generated");
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("Unknown output format: {}", cli.output);
            eprintln!("Available formats: web, json, debug");
            std::process::exit(1);
        }
    }
    
    Ok(())
}  
