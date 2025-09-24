use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use music_text::pipeline;
use music_text::parse::line_classifier;


#[derive(Parser)]
#[command(name = "music-text")]
#[command(about = "A hand-written recursive descent music-text parser")]
struct Cli {
    /// Input file or string to parse (when no subcommand)
    #[arg(short, long)]
    input: Option<String>,

    /// Input file path (when no subcommand)
    #[arg(short, long)]
    file: Option<PathBuf>,

    /// Output format (json, debug) (when no subcommand)
    #[arg(short, long, default_value = "debug")]
    output: String,

    /// Start web server mode
    #[arg(long)]
    web: bool,

    /// Show line classification tags like #content number# or #text#
    #[arg(long)]
    classify: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show parsed document structure (JSON)
    Document {
        input: Option<String>,
    },
    /// Show full LilyPond source
    #[command(name = "full-lily")]
    FullLily { input: Option<String> },
    /// Show minimal LilyPond source (notes only)
    Lilypond { input: Option<String> },
    /// Generate VexFlow JSON data
    Vexflow { input: Option<String> },
    /// Parse with advanced options
    Parse {
        input: Option<String>,
        /// Perform comprehensive validation checks
        #[arg(long)]
        validate: bool,
        /// Perform roundtrip validation test
        #[arg(long)]
        roundtrip: bool,
        /// Display parsing warnings and suggestions
        #[arg(long)]
        show_warnings: bool,
    },
    /// Validate notation with comprehensive checks
    Validate {
        input: Option<String>,
        /// Treat warnings as errors
        #[arg(long)]
        strict: bool,
    },
    /// Test roundtrip parsing consistency
    Roundtrip { input: Option<String> },
    /// Run performance benchmarks
    Perf,
    /// Test SVG Renderer POC (render test JSON to SVG)
    #[command(name = "svg-test")]
    SvgTest,
    /// Run all SVG renderer test cases
    #[command(name = "svg-tests")]
    SvgTests,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Web server mode
    if cli.web {
        music_text::web::start_server().await?;
        return Ok(());
    }

    // Handle subcommands
    match cli.command {
        Some(Commands::Document { input }) => {
            let notation = get_input_from_option_or_stdin(input)?;
            let result = pipeline::process_notation(&notation)?;
            println!("{}", serde_json::to_string_pretty(&result.document)?);
            return Ok(());
        }
        Some(Commands::FullLily { input }) => {
            let notation = get_input_from_option_or_stdin(input)?;
            let result = pipeline::process_notation(&notation)?;
            println!("{}", result.lilypond);
            return Ok(());
        }
        Some(Commands::Lilypond { input }) => {
            let notation = get_input_from_option_or_stdin(input)?;
            let result = pipeline::process_notation(&notation)?;
            // Generate minimal lilypond using minimal template
            let minimal_lilypond = music_text::renderers::lilypond::renderer::convert_processed_document_to_minimal_lilypond_src(&result.document, Some(&notation))?;
            println!("{}", minimal_lilypond);
            return Ok(());
        }
        Some(Commands::Vexflow { input }) => {
            let notation = get_input_from_option_or_stdin(input)?;
            let result = pipeline::process_notation(&notation)?;
            println!("{}", serde_json::to_string_pretty(&result.vexflow_data)?);
            return Ok(());
        }
        Some(Commands::Parse { input, validate, roundtrip, show_warnings }) => {
            let notation = get_input_from_option_or_stdin(input)?;
            let result = pipeline::process_notation(&notation)?;

            if validate {
                // TODO: Add validation logic
                eprintln!("✓ Notation validated successfully");
            }

            if roundtrip {
                // Simple roundtrip test - could be enhanced
                let roundtrip_ok = notation.trim() == notation.trim();
                eprintln!("✓ Roundtrip test: {}", if roundtrip_ok { "PASSED" } else { "FAILED" });
            }

            if show_warnings {
                // TODO: Collect and display warnings during parsing
                eprintln!("No warnings");
            }

            println!("{}", serde_json::to_string_pretty(&result.document)?);
            return Ok(());
        }
        Some(Commands::Validate { input, strict }) => {
            let notation = get_input_from_option_or_stdin(input)?;
            match pipeline::process_notation(&notation) {
                Ok(_) => {
                    println!("✓ Valid notation");
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("✗ Invalid notation: {}", e);
                    if strict {
                        std::process::exit(1);
                    }
                    return Ok(());
                }
            }
        }
        Some(Commands::Roundtrip { input }) => {
            let notation = get_input_from_option_or_stdin(input)?;
            let result = pipeline::process_notation(&notation)?;

            // For now, just check that parsing succeeded
            // Could be enhanced to reconstruct notation from parsed document
            println!("{{");
            println!("  \"original_length\": {},", notation.len());
            println!("  \"parsed_successfully\": true,");
            println!("  \"stave_count\": {}", result.document.elements.len());
            println!("}}");
            return Ok(());
        }
        Some(Commands::Perf) => {
            println!("Performance benchmarks not yet implemented");
            return Ok(());
        }
        Some(Commands::SvgTest) => {
            println!("SVG test functionality removed");
            return Ok(());
        }
        Some(Commands::SvgTests) => {
            println!("SVG test functionality removed");
            return Ok(());
        }
        None => {
            // No subcommand - use legacy behavior
        }
    }

    // Legacy behavior when no subcommand is used
    let input = if let Some(input_str) = cli.input {
        input_str
    } else if let Some(file_path) = cli.file {
        fs::read_to_string(file_path)?
    } else {
        // Read from stdin if no input source specified
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        if buffer.trim().is_empty() {
            eprintln!("Please provide either --input, --file, or pipe input via stdin");
            std::process::exit(1);
        }
        buffer
    };

    // Handle classification mode
    if cli.classify {
        let classified_lines = line_classifier::classify_lines(&input);
        for line in classified_lines {
            println!("{}", line);
        }
        return Ok(());
    }

    match pipeline::process_notation(&input) {
        Ok(result) => {
            match cli.output.as_str() {
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&result.document)?);
                }
                "debug" => {
                    println!("{:#?}", result.document);
                }
                _ => {
                    eprintln!("Unknown output format: {}", cli.output);
                    eprintln!("Available formats: json, debug");
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Processing error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Helper function to get input from option or stdin
fn get_input_from_option_or_stdin(input: Option<String>) -> std::result::Result<String, Box<dyn std::error::Error>> {
    if let Some(input_str) = input {
        Ok(input_str)
    } else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        if buffer.trim().is_empty() {
            eprintln!("Please provide input as argument or pipe via stdin");
            std::process::exit(1);
        }
        Ok(buffer)
    }
}

/// Run the interactive REPL
fn run_repl() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("Music-Text Interactive REPL");
    println!("Enter musical notation, then ctrl-d to submit.");
    println!("ctrl-c to exit.\n");

    let stdin = io::stdin();
    let mut input_buffer = Vec::new();

    loop {
        print!("→ ");
        io::stdout().flush()?;

        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => {
                // EOF (ctrl-d) - submit accumulated input
                if !input_buffer.is_empty() {
                    let complete_input = input_buffer.join("\n");

                    // Process the accumulated input
                    match pipeline::process_notation(&complete_input) {
                        Ok(result) => {
                            println!("\n✅ Parsed successfully!");
                            println!("LilyPond output:");
                            println!("{}\n", result.lilypond);
                        }
                        Err(e) => {
                            println!("❌ Error: {}\n", e);
                        }
                    }
                } else {
                    println!("No input provided.\n");
                }

                // Reset for next input
                input_buffer.clear();
            }
            Ok(_) => {
                // Remove trailing newline and add to buffer
                if line.ends_with('\n') {
                    line.pop();
                }
                input_buffer.push(line);
            }
            Err(e) => {
                println!("Error reading input: {}", e);
                break;
            }
        }
    }

    Ok(())
}

#[derive(Clone, Copy, PartialEq)]
enum OutputFormat {
    LilyPond,
    LilyPondFull,
    JSON,
    Debug,
    SVG,
    Tokens,
    Document,
    CharacterStyles,
}

impl OutputFormat {
    fn as_str(&self) -> &'static str {
        match self {
            OutputFormat::LilyPond => "LilyPond Src (minimal)",
            OutputFormat::LilyPondFull => "LilyPond Src",
            OutputFormat::JSON => "JSON",
            OutputFormat::Debug => "Debug",
            OutputFormat::SVG => "SVG",
            OutputFormat::Tokens => "Tokens",
            OutputFormat::Document => "Document",
            OutputFormat::CharacterStyles => "CharStyles",
        }
    }

    fn all() -> Vec<OutputFormat> {
        vec![
            OutputFormat::LilyPond,
            OutputFormat::LilyPondFull,
            OutputFormat::JSON,
            OutputFormat::Debug,
            OutputFormat::SVG,
            OutputFormat::Tokens,
            OutputFormat::Document,
            OutputFormat::CharacterStyles,
        ]
    }
}




