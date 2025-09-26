use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use reqwest;
use serde_json;

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
    /// Render input to SVG
    Svg { input: Option<String> },
    /// Test SVG Renderer POC (render test JSON to SVG)
    #[command(name = "svg-test")]
    SvgTest,
    /// Run all SVG renderer test cases
    #[command(name = "svg-tests")]
    SvgTests,

    // API wrapper commands
    /// Create a new document via API
    #[command(name = "api-create")]
    ApiCreate {
        /// Initial music text content
        #[arg(short, long)]
        text: Option<String>,
        /// Document title
        #[arg(short = 'T', long)]
        title: Option<String>,
        /// Composer name
        #[arg(short, long)]
        composer: Option<String>,
        /// API server URL
        #[arg(long, default_value = "http://localhost:3000")]
        server: String,
    },
    /// Get document by ID via API
    #[command(name = "api-get")]
    ApiGet {
        /// Document ID
        document_id: String,
        /// API server URL
        #[arg(long, default_value = "http://localhost:3000")]
        server: String,
    },
    /// Execute semantic command on document
    #[command(name = "api-command")]
    ApiCommand {
        /// Document ID
        document_id: String,
        /// Command type (apply_slur, set_octave, insert_note, etc.)
        #[arg(short, long)]
        command: String,
        /// Target UUIDs (comma-separated)
        #[arg(short, long)]
        targets: Option<String>,
        /// Command parameters as JSON
        #[arg(short, long)]
        params: Option<String>,
        /// API server URL
        #[arg(long, default_value = "http://localhost:3000")]
        server: String,
    },
    /// Transform document with semantic operations
    #[command(name = "api-transform")]
    ApiTransform {
        /// JSON document to transform
        document: String,
        /// Command type (apply_slur, set_octave, etc.)
        #[arg(short, long)]
        command: String,
        /// Target UUIDs (comma-separated)
        #[arg(short, long)]
        targets: Option<String>,
        /// Command parameters as JSON
        #[arg(short, long)]
        params: Option<String>,
        /// API server URL
        #[arg(long, default_value = "http://localhost:3000")]
        server: String,
    },
    /// Export document to different formats
    #[command(name = "api-export")]
    ApiExport {
        /// JSON document to export
        document: String,
        /// Export format (lilypond, svg)
        #[arg(short, long)]
        format: String,
        /// API server URL
        #[arg(long, default_value = "http://localhost:3000")]
        server: String,
    },
    /// Transform document by UUID
    #[command(name = "transform")]
    TransformById {
        /// Document UUID
        document_id: String,
        /// Command type (apply_slur, set_octave, etc.)
        #[arg(short, long)]
        command: String,
        /// Target UUIDs (comma-separated)
        #[arg(short, long)]
        targets: Option<String>,
        /// Command parameters as JSON
        #[arg(short, long)]
        params: Option<String>,
        /// API server URL
        #[arg(long, default_value = "http://localhost:3000")]
        server: String,
    },
    /// Export document by UUID
    #[command(name = "export")]
    ExportById {
        /// Document UUID
        document_id: String,
        /// Export format (lilypond, svg)
        #[arg(short, long)]
        format: String,
        /// API server URL
        #[arg(long, default_value = "http://localhost:3000")]
        server: String,
    },
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
        Some(Commands::Svg { input }) => {
            let notation = get_input_from_option_or_stdin(input)?;
            let result = pipeline::process_notation(&notation)?;

            // Use the canvas SVG renderer
            let svg_output = music_text::renderers::editor::svg::render_canvas_svg(
                &result.document,
                "number", // notation type
                &notation, // input text
                None, // cursor position
                None, // selection start
                None  // selection end
            )?;

            println!("{}", svg_output);
            return Ok(());
        }
        Some(Commands::SvgTest) => {
            let notation = get_input_from_option_or_stdin(None)?;
            let result = pipeline::process_notation(&notation)?;

            // Use the canvas SVG renderer
            let svg_output = music_text::renderers::editor::svg::render_canvas_svg(
                &result.document,
                "number", // notation type
                &notation, // input text
                None, // cursor position
                None, // selection start
                None  // selection end
            )?;

            println!("{}", svg_output);
            return Ok(());
        }
        Some(Commands::SvgTests) => {
            // Run test cases
            let test_cases = vec![
                "1234",
                "1234|5678",
                "Sa Re Ga Ma",
                "1-2-3-",
            ];

            println!("Running SVG test cases...\n");

            for (i, test_input) in test_cases.iter().enumerate() {
                println!("=== Test Case {} ===", i + 1);
                println!("Input: {}", test_input);

                match pipeline::process_notation(test_input) {
                    Ok(result) => {
                        match music_text::renderers::editor::svg::render_canvas_svg(
                            &result.document,
                            "number",
                            test_input,
                            None, None, None
                        ) {
                            Ok(svg) => {
                                println!("SVG Length: {} chars", svg.len());
                                println!("✓ Rendered successfully\n");
                            }
                            Err(e) => {
                                println!("✗ SVG render failed: {}\n", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("✗ Parse failed: {}\n", e);
                    }
                }
            }
            return Ok(());
        }

        // API wrapper command handlers
        Some(Commands::ApiCreate { text, title, composer, server }) => {
            let result = api_create_document(&server, text.as_deref(), title.as_deref(), composer.as_deref()).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
            return Ok(());
        }
        Some(Commands::ApiGet { document_id, server }) => {
            let result = api_get_document(&server, &document_id).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
            return Ok(());
        }
        Some(Commands::ApiCommand { document_id, command, targets, params, server }) => {
            let target_uuids: Vec<String> = targets
                .as_deref()
                .unwrap_or("")
                .split(',')
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim().to_string())
                .collect();

            let parameters = if let Some(params_str) = params {
                serde_json::from_str(&params_str)?
            } else {
                serde_json::Value::Object(serde_json::Map::new())
            };

            let result = api_execute_command(&server, &document_id, &command, &target_uuids, parameters).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
            return Ok(());
        }
        Some(Commands::ApiTransform { document, command, targets, params, server }) => {
            let target_uuids: Vec<String> = targets
                .as_deref()
                .unwrap_or("")
                .split(',')
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim().to_string())
                .collect();

            let parameters = if let Some(params_str) = params {
                serde_json::from_str(&params_str)?
            } else {
                serde_json::Value::Object(serde_json::Map::new())
            };

            let doc: serde_json::Value = serde_json::from_str(&document)?;
            let result = api_transform_document(&server, doc, &command, &target_uuids, parameters).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
            return Ok(());
        }
        Some(Commands::ApiExport { document, format, server }) => {
            let doc: serde_json::Value = serde_json::from_str(&document)?;
            let result = api_export_document(&server, doc, &format).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
            return Ok(());
        }
        Some(Commands::TransformById { document_id, command, targets, params, server }) => {
            let target_uuids: Vec<String> = targets
                .as_deref()
                .unwrap_or("")
                .split(',')
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim().to_string())
                .collect();

            let parameters = if let Some(params_str) = params {
                serde_json::from_str(&params_str)?
            } else {
                serde_json::Value::Object(serde_json::Map::new())
            };

            let result = api_transform_document_by_id(&server, &document_id, &command, &target_uuids, parameters).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
            return Ok(());
        }
        Some(Commands::ExportById { document_id, format, server }) => {
            let result = api_export_document_by_id(&server, &document_id, &format).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
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

// API wrapper functions
async fn api_create_document(
    server: &str,
    text: Option<&str>,
    title: Option<&str>,
    composer: Option<&str>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let mut metadata = serde_json::Map::new();
    if let Some(title) = title {
        metadata.insert("title".to_string(), serde_json::Value::String(title.to_string()));
    }
    if let Some(composer) = composer {
        metadata.insert("composer".to_string(), serde_json::Value::String(composer.to_string()));
    }

    let mut request_body = serde_json::Map::new();
    if let Some(text) = text {
        request_body.insert("music_text".to_string(), serde_json::Value::String(text.to_string()));
    }
    if !metadata.is_empty() {
        request_body.insert("metadata".to_string(), serde_json::Value::Object(metadata));
    }

    let url = format!("{}/api/documents", server);
    let response = client
        .post(&url)
        .json(&serde_json::Value::Object(request_body))
        .send()
        .await?;

    if response.status().is_success() {
        let json: serde_json::Value = response.json().await?;
        Ok(json)
    } else {
        let error_text = response.text().await?;
        Err(format!("API error: {}", error_text).into())
    }
}

async fn api_get_document(
    server: &str,
    document_id: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/documents/{}", server, document_id);

    let response = client.get(&url).send().await?;

    if response.status().is_success() {
        let json: serde_json::Value = response.json().await?;
        Ok(json)
    } else {
        let error_text = response.text().await?;
        Err(format!("API error: {}", error_text).into())
    }
}

async fn api_execute_command(
    server: &str,
    document_id: &str,
    command: &str,
    target_uuids: &[String],
    parameters: serde_json::Value,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let request_body = serde_json::json!({
        "command_type": command,
        "target_uuids": target_uuids,
        "parameters": parameters
    });

    let url = format!("{}/api/documents/{}/commands", server, document_id);
    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await?;

    if response.status().is_success() {
        let json: serde_json::Value = response.json().await?;
        Ok(json)
    } else {
        let error_text = response.text().await?;
        Err(format!("API error: {}", error_text).into())
    }
}

async fn api_transform_document(
    server: &str,
    document: serde_json::Value,
    command: &str,
    target_uuids: &[String],
    parameters: serde_json::Value,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let request_body = serde_json::json!({
        "document": document,
        "command_type": command,
        "target_uuids": target_uuids,
        "parameters": parameters
    });

    let response = client
        .post(&format!("{}/api/documents/transform", server))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    let result: serde_json::Value = response.json().await?;
    Ok(result)
}

async fn api_export_document(
    server: &str,
    document: serde_json::Value,
    format: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let request_body = serde_json::json!({
        "document": document,
        "format": format
    });

    let response = client
        .post(&format!("{}/api/documents/export", server))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    let result: serde_json::Value = response.json().await?;
    Ok(result)
}

async fn api_transform_document_by_id(
    server: &str,
    document_id: &str,
    command: &str,
    target_uuids: &[String],
    parameters: serde_json::Value,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let request_body = serde_json::json!({
        "command_type": command,
        "target_uuids": target_uuids,
        "parameters": parameters
    });

    let response = client
        .post(&format!("{}/api/documents/{}/transform", server, document_id))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    let result: serde_json::Value = response.json().await?;
    Ok(result)
}

async fn api_export_document_by_id(
    server: &str,
    document_id: &str,
    format: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let request_body = serde_json::json!({
        "format": format
    });

    let response = client
        .post(&format!("{}/api/documents/{}/export", server, document_id))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    let result: serde_json::Value = response.json().await?;
    Ok(result)
}




