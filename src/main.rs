use music_text::{parse_document, process_notation};
use music_text::renderers::render_web_fast_lilypond;
use std::io::{self, Read};
use std::fs;
use clap::{Parser, Subcommand, CommandFactory};
use clap_complete::{generate, Generator, Shell};
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

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
    /// Launch GUI editor
    #[cfg(feature = "gui")]
    Gui,
    /// Start interactive REPL
    Repl,
    /// Run performance benchmarks
    Perf,
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
    /// Generate LilyPond SVG files (.ly and .svg) to disk
    #[command(name = "lilypond-svg")]
    LilypondSvg { 
        input: Option<String>,
        /// Output filename prefix (without extension)
        #[arg(short, long, default_value = "output")]
        output: String,
    },
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
        #[cfg(feature = "gui")]
        Some(Commands::Gui) => {
            println!("Launching GUI editor...");
            run_gui();
        },
        Some(Commands::Repl) => {
            if let Err(err) = run_repl() {
                eprintln!("REPL error: {:?}", err);
            }
        },
        Some(Commands::Perf) => {
            println!("Running performance benchmarks...");
            run_performance_tests();
        },
        Some(Commands::Document { input }) => process_stage("document", input),
        Some(Commands::Processed { input }) => process_stage("processed", input),
        Some(Commands::MinimalLily { input }) => process_stage("minimal-lily", input),
        Some(Commands::FullLily { input }) => process_stage("full-lily", input),
        Some(Commands::Vexflow { input }) => process_stage("vexflow", input),
        Some(Commands::VexflowSvg { input }) => process_stage("vexflow-svg", input),
        Some(Commands::All { input }) => process_stage("all", input),
        Some(Commands::LilypondSvg { input, output }) => generate_lilypond_svg_files(input, output),
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
        "parse" => show_parse_output(&input),
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

fn show_parse_output(input: &str) {
    match parse_document(input) {
        Ok(document) => {
            println!("{}", serde_json::to_string_pretty(&document).unwrap());
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
    
    show_parse_output(input);
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

fn generate_lilypond_svg_files(input: Option<String>, output_prefix: String) {
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
        std::process::exit(1);
    }
    
    // Process notation using the same pipeline as web UI
    let processed_staves = match process_notation(&input) {
        Ok(result) => result.processed_staves,
        Err(e) => {
            eprintln!("Processing error: {}", e);
            std::process::exit(1);
        }
    };
    
    // Generate optimized LilyPond source using web fast renderer
    let lilypond_source = render_web_fast_lilypond(&processed_staves);
    
    // Write .ly file to disk
    let ly_filename = format!("{}.ly", output_prefix);
    if let Err(e) = fs::write(&ly_filename, &lilypond_source) {
        eprintln!("Error writing LilyPond file {}: {}", ly_filename, e);
        std::process::exit(1);
    }
    println!("LilyPond source written to: {}", ly_filename);
    
    // Generate SVG using LilyPond generator
    let temp_dir = std::env::temp_dir().join("music-text-cli");
    let generator = music_text::renderers::lilypond_generator::LilyPondGenerator::new(temp_dir.to_string_lossy().to_string());
    
    // Use Tokio runtime for async SVG generation
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(generator.generate_svg(&lilypond_source));
    
    if result.success {
        if let Some(svg_content) = result.svg_content {
            // Write .svg file to disk  
            let svg_filename = format!("{}.svg", output_prefix);
            if let Err(e) = fs::write(&svg_filename, &svg_content) {
                eprintln!("Error writing SVG file {}: {}", svg_filename, e);
                std::process::exit(1);
            }
            println!("SVG file written to: {}", svg_filename);
        } else {
            eprintln!("Error: SVG generation succeeded but no content returned");
            std::process::exit(1);
        }
    } else {
        let error_msg = result.error.unwrap_or("Unknown error".to_string());
        eprintln!("SVG generation failed: {}", error_msg);
        std::process::exit(1);
    }
}

#[cfg(feature = "gui")]
fn run_gui() {
    use eframe::egui;
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Music-Text GUI"),
        ..Default::default()
    };
    
    let _ = eframe::run_native(
        "Music-Text",
        options,
        Box::new(|_cc| Ok(Box::new(MusicTextApp::default()))),
    );
}

#[cfg(feature = "gui")]
struct MusicTextApp {
    input_text: String,
    output_text: String,
}

#[cfg(feature = "gui")]
impl Default for MusicTextApp {
    fn default() -> Self {
        Self {
            input_text: "|1 2 3".to_owned(),
            output_text: String::new(),
        }
    }
}

#[cfg(feature = "gui")]
impl eframe::App for MusicTextApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Music-Text GUI");
            
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Input:");
                    ui.text_edit_multiline(&mut self.input_text);
                });
                
                ui.vertical(|ui| {
                    ui.label("Output:");
                    ui.text_edit_multiline(&mut self.output_text);
                });
            });
            
            if ui.button("Parse").clicked() {
                self.parse_input();
            }
        });
    }
}

#[cfg(feature = "gui")]
impl MusicTextApp {
    fn parse_input(&mut self) {
        match process_notation(&self.input_text) {
            Ok(result) => {
                self.output_text = result.minimal_lilypond;
            }
            Err(e) => {
                self.output_text = format!("Error: {}", e);
            }
        }
    }
}

fn run_repl() -> Result<()> {

    let mut rl = DefaultEditor::new()?;
    #[cfg(feature = "with-file-history")]
    let _ = rl.load_history("history.txt");
    
    let mut input_buffer = Vec::new();
    
    loop {
        let prompt = "";
        let readline = rl.readline(prompt);
        
        match readline {
            Ok(line) => {
                // Check for submission trigger
                if line.trim() == "$" {
                    if !input_buffer.is_empty() {
                        let complete_input = input_buffer.join("\n");
                        rl.add_history_entry(complete_input.as_str())?;

                        // Process the accumulated input
                        match process_notation(&complete_input) {
                            Ok(result) => {
                                println!("\n{}\n", result.minimal_lilypond);
                            }
                            Err(e) => {
                                println!("Error: {}\n", e);
                            }
                        }
                    }
                    // Reset for next input
                    input_buffer.clear();
                } else {
                    // Accumulate input (including blank lines)
                    input_buffer.push(line);
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    #[cfg(feature = "with-file-history")]
    rl.save_history("history.txt")?;
    Ok(())
}

fn run_performance_tests() {
    use std::time::Instant;
    
    let test_cases = vec![
        "|1 2 3",
        "|SRG MPD",
        "|1-2-3 4-5",
        "____\n|123\n\n|345\n_____\n\n|333",
    ];
    
    println!("Running performance benchmarks...\n");
    
    for (i, test_case) in test_cases.iter().enumerate() {
        println!("Test {}: {}", i + 1, test_case.replace('\n', "\\n"));
        
        let start = Instant::now();
        for _ in 0..100 {
            let _ = process_notation(test_case);
        }
        let duration = start.elapsed();
        
        println!("  100 iterations: {:?} ({:.2}ms per iteration)\n", 
                 duration, 
                 duration.as_millis() as f64 / 100.0);
    }
}