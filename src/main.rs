use clap::Parser;
use std::fs;
use std::path::PathBuf;

use music_text::parse_notation_full;

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
    
    /// Output format (json, debug, ast)
    #[arg(short, long, default_value = "debug")]
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
        music_text::web_server::start_server().await?;
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
    
    println!("Parsing input: {}", input);
    println!("System: {}", cli.system);
    println!("Output format: {}", cli.output);
    
    match cli.output.as_str() {
        "json" | "debug" | "ast" => {
            let result = parse_notation_full(&input, Some(&cli.system));
            if result.success {
                if let Some(document) = result.document {
                    match cli.output.as_str() {
                        "json" => {
                            println!("{}", serde_json::to_string_pretty(&document)?);
                        }
                        "debug" => {
                            println!("{:#?}", document);
                        }
                        "ast" => {
                            println!("AST structure:");
                            println!("{:#?}", document);
                        }
                        _ => unreachable!()
                    }
                } else {
                    eprintln!("Parse error: No document generated");
                    std::process::exit(1);
                }
            } else {
                eprintln!("Parse error: {}", result.error_message.unwrap_or("Unknown error".to_string()));
                std::process::exit(1);
            }
        }
        "full" => {
            let result = parse_notation_full(&input, Some(&cli.system));
            if result.success {
                println!("=== FULL PARSE RESULT ===");
                println!("AST: {:#?}", result.document);
                if let Some(ast) = result.ast {
                    println!("\n=== AST JSON ===");
                    println!("{}", ast);
                }
                if let Some(spatial) = result.spatial {
                    println!("\n=== SPATIAL JSON ===");
                    println!("{}", spatial);
                }
                if let Some(fsm) = result.fsm {
                    println!("\n=== FSM JSON ===");
                    println!("{}", fsm);
                }
                if let Some(yaml) = result.yaml {
                    println!("\n=== YAML ===");
                    println!("{}", yaml);
                }
                if let Some(vexflow) = result.vexflow {
                    println!("\n=== VEXFLOW ===");
                    println!("{:#?}", vexflow);
                }
                if let Some(lilypond) = result.lilypond {
                    println!("\n=== LILYPOND ===");
                    println!("{}", lilypond);
                }
            } else {
                eprintln!("Parse error: {}", result.error_message.unwrap_or("Unknown error".to_string()));
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("Unknown output format: {}", cli.output);
            eprintln!("Available formats: json, debug, ast, full");
            std::process::exit(1);
        }
    }
    
    Ok(())
}
