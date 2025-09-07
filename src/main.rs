use music_text::{parse_notation, pest_pair_to_json, process_notation};
use std::io::{self, Read};
use clap::{Parser, Subcommand, CommandFactory};
use clap_complete::{generate, Generator, Shell};

mod web_server;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Start web server on port 3000
    #[arg(long)]
    web: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Show raw PEST parse tree
    Pest { input: Option<String> },
    /// Show parsed document structure (JSON)
    Document { input: Option<String> },
    /// Show processed staves (JSON)
    Processed { input: Option<String> },
    /// Show minimal LilyPond notation
    #[command(name = "minimal-lily")]
    MinimalLily { input: Option<String> },
    /// Show full LilyPond score
    #[command(name = "full-lily")]
    FullLily { input: Option<String> },
    /// Show VexFlow data structure (JSON)
    Vexflow { input: Option<String> },
    /// Show VexFlow SVG rendering
    #[command(name = "vexflow-svg")]
    VexflowSvg { input: Option<String> },
    /// Show all stages
    All { input: Option<String> },
    /// Generate shell completions
    Completions {
        /// Shell type
        #[arg(value_enum)]
        shell: Shell,
    },
}

fn main() {
    let cli = Cli::parse();
    
    // Check for --web flag
    if cli.web {
        web_server::start();
        return;
    }
    
    match cli.command {
        Some(Commands::Pest { input }) => process_stage("pest", input),
        Some(Commands::Document { input }) => process_stage("document", input),
        Some(Commands::Processed { input }) => process_stage("processed", input),
        Some(Commands::MinimalLily { input }) => process_stage("minimal-lily", input),
        Some(Commands::FullLily { input }) => process_stage("full-lily", input),
        Some(Commands::Vexflow { input }) => process_stage("vexflow", input),
        Some(Commands::VexflowSvg { input }) => process_stage("vexflow-svg", input),
        Some(Commands::All { input }) => process_stage("all", input),
        Some(Commands::Completions { shell }) => {
            let mut cmd = Cli::command();
            print_completions(shell, &mut cmd);
        },
        None => {
            eprintln!("Error: No command provided");
            eprintln!("Use --help to see available commands");
            std::process::exit(1);
        }
    }
}

fn process_stage(stage: &str, input: Option<String>) {
    let input = match input {
        Some(input_str) => input_str,
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer).unwrap_or_else(|_| {
                eprintln!("Error reading from stdin");
                std::process::exit(1);
            });
            buffer.trim().to_string()
        }
    };
    
    if input.is_empty() {
        eprintln!("Error: No input provided");
        return;
    }
    
    match stage {
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
            std::process::exit(1);
        }
    }
}

fn print_completions<G: Generator>(gen: G, cmd: &mut clap::Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
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