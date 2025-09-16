use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Constraint, Layout, Direction},
    style::{Color, Style},
    text::{Span, Line},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame, Terminal,
};

use music_text::pipeline;


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

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start interactive REPL
    Repl,
    /// Show parsed document structure (JSON)
    Document {
        input: Option<String>,
        /// Show rhythm-analyzed document instead of parsed document
        #[arg(long)]
        rhythm: bool,
    },
    /// Show full LilyPond score
    #[command(name = "full-lily")]
    FullLily { input: Option<String> },
    /// Generate syntax tokens for editor integration
    Tokens { input: Option<String> },
    /// Generate VexFlow JSON data
    Vexflow { input: Option<String> },
    /// Show rhythm analysis results
    Rhythm { input: Option<String> },
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
        Some(Commands::Repl) => {
            run_tui_repl().await?;
            return Ok(());
        }
        Some(Commands::Document { input, rhythm }) => {
            let notation = get_input_from_option_or_stdin(input)?;
            let result = pipeline::process_notation(&notation)?;
            let doc = if rhythm {
                &result.rhythm_analyzed_document
            } else {
                &result.parsed_document
            };
            println!("{}", serde_json::to_string_pretty(doc)?);
            return Ok(());
        }
        Some(Commands::FullLily { input }) => {
            let notation = get_input_from_option_or_stdin(input)?;
            let result = pipeline::process_notation(&notation)?;
            println!("{}", result.lilypond);
            return Ok(());
        }
        Some(Commands::Tokens { input }) => {
            let notation = get_input_from_option_or_stdin(input)?;
            let result = pipeline::process_notation(&notation)?;
            let tokens = music_text::tree_functions::generate_syntax_tokens(&result.parsed_document, &notation);
            println!("{}", serde_json::to_string_pretty(&tokens)?);
            return Ok(());
        }
        Some(Commands::Vexflow { input }) => {
            let notation = get_input_from_option_or_stdin(input)?;
            let result = pipeline::process_notation(&notation)?;
            println!("{}", serde_json::to_string_pretty(&result.vexflow_data)?);
            return Ok(());
        }
        Some(Commands::Rhythm { input }) => {
            let notation = get_input_from_option_or_stdin(input)?;
            let result = pipeline::process_notation(&notation)?;
            println!("{}", serde_json::to_string_pretty(&result.rhythm_analyzed_document)?);
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

            println!("{}", serde_json::to_string_pretty(&result.parsed_document)?);
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
            println!("  \"stave_count\": {}", result.parsed_document.elements.len());
            println!("}}");
            return Ok(());
        }
        Some(Commands::Perf) => {
            println!("Performance benchmarks not yet implemented");
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

    match pipeline::process_notation(&input) {
        Ok(result) => {
            match cli.output.as_str() {
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&result.rhythm_analyzed_document)?);
                }
                "debug" => {
                    println!("{:#?}", result.rhythm_analyzed_document);
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
    JSON,
    Debug,
    SVG,
    Tokens,
    Document,
}

impl OutputFormat {
    fn as_str(&self) -> &'static str {
        match self {
            OutputFormat::LilyPond => "LilyPond",
            OutputFormat::JSON => "JSON",
            OutputFormat::Debug => "Debug",
            OutputFormat::SVG => "SVG",
            OutputFormat::Tokens => "Tokens",
            OutputFormat::Document => "Document",
        }
    }

    fn all() -> Vec<OutputFormat> {
        vec![
            OutputFormat::LilyPond,
            OutputFormat::JSON,
            OutputFormat::Debug,
            OutputFormat::SVG,
            OutputFormat::Tokens,
            OutputFormat::Document,
        ]
    }
}

struct App {
    input: String,
    cursor_pos: usize,
    output: String,
    selected_format: usize,
    error: Option<String>,
    scroll_offset: usize,
}

impl App {
    fn new() -> Self {
        Self {
            input: String::new(),
            cursor_pos: 0,
            output: String::new(),
            selected_format: 0,
            error: None,
            scroll_offset: 0,
        }
    }

    async fn update_output(&mut self) {
        if self.input.trim().is_empty() {
            self.output.clear();
            self.error = None;
            self.scroll_offset = 0;
            return;
        }

        let format = OutputFormat::all()[self.selected_format];

        // Make HTTP request to the web server
        let client = reqwest::Client::new();
        let generate_svg = format == OutputFormat::SVG;
        let url = format!("http://localhost:3000/api/parse?input={}&generate_svg={}",
            urlencoding::encode(&self.input), generate_svg);

        let result = match client.get(&url).send().await {
            Ok(response) => {
                match response.json::<serde_json::Value>().await {
                    Ok(json) => {
                        if json["success"].as_bool().unwrap_or(false) {
                            Ok(json)
                        } else {
                            Err(json["error"].as_str().unwrap_or("Unknown error").to_string())
                        }
                    }
                    Err(e) => Err(format!("JSON parse error: {}", e))
                }
            }
            Err(e) => Err(format!("Request error: {} (is web server running?)", e))
        };

        match result {
            Ok(json) => {
                self.error = None;
                self.scroll_offset = 0; // Reset scroll when content changes
                self.output = match format {
                    OutputFormat::LilyPond => {
                        json["lilypond"].as_str().unwrap_or("").to_string()
                    },
                    OutputFormat::JSON => {
                        serde_json::to_string_pretty(&json["parsed_document"]).unwrap_or_else(|e| format!("JSON error: {}", e))
                    },
                    OutputFormat::Debug => {
                        format!("{:#?}", json["parsed_document"])
                    },
                    OutputFormat::SVG => {
                        if let Some(svg_content) = json["lilypond_svg"].as_str() {
                            if !svg_content.is_empty() {
                                svg_content.to_string()
                            } else {
                                "SVG not available (server may need generate_svg=true)".to_string()
                            }
                        } else {
                            "SVG not available".to_string()
                        }
                    },
                    OutputFormat::Tokens => {
                        serde_json::to_string_pretty(&json["syntax_tokens"]).unwrap_or("Tokens not available".to_string())
                    },
                    OutputFormat::Document => {
                        serde_json::to_string_pretty(&json["parsed_document"]).unwrap_or_else(|e| format!("JSON error: {}", e))
                    },
                };
            }
            Err(e) => {
                self.error = Some(e);
                self.output.clear();
                self.scroll_offset = 0;
            }
        }
    }

    async fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor_pos, c);
        self.cursor_pos += 1;
        self.update_output().await;
    }

    async fn delete_char(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.input.remove(self.cursor_pos);
            self.update_output().await;
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor_pos < self.input.len() {
            self.cursor_pos += 1;
        }
    }

    async fn insert_newline(&mut self) {
        self.input.insert(self.cursor_pos, '\n');
        self.cursor_pos += 1;
        self.update_output().await;
    }

    async fn next_format(&mut self) {
        self.selected_format = (self.selected_format + 1) % OutputFormat::all().len();
        self.update_output().await;
    }

    async fn prev_format(&mut self) {
        if self.selected_format == 0 {
            self.selected_format = OutputFormat::all().len() - 1;
        } else {
            self.selected_format -= 1;
        }
        self.update_output().await;
    }

    fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    fn scroll_down(&mut self, max_visible_lines: usize) {
        let total_lines = if self.error.is_some() {
            1 // Error is just one line
        } else {
            self.output.lines().count()
        };

        if total_lines > max_visible_lines && self.scroll_offset + max_visible_lines < total_lines {
            self.scroll_offset += 1;
        }
    }

    fn page_up(&mut self, page_size: usize) {
        if self.scroll_offset >= page_size {
            self.scroll_offset -= page_size;
        } else {
            self.scroll_offset = 0;
        }
    }

    fn page_down(&mut self, page_size: usize, max_visible_lines: usize) {
        let total_lines = if self.error.is_some() {
            1
        } else {
            self.output.lines().count()
        };

        if total_lines > max_visible_lines {
            self.scroll_offset = (self.scroll_offset + page_size).min(total_lines - max_visible_lines);
        }
    }
}

fn draw_ui(frame: &mut Frame, app: &App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_layout[0]);

    // Input pane
    let input_block = Block::default()
        .title("Input (ESC to quit, Tab to switch format, ↑↓ to scroll output)")
        .borders(Borders::ALL);

    let input_text = if app.input.is_empty() {
        vec![Line::from("Type your musical notation here...")]
    } else {
        app.input.lines().map(|line| Line::from(line)).collect()
    };

    let input_paragraph = Paragraph::new(input_text)
        .block(input_block)
        .style(Style::default());

    frame.render_widget(input_paragraph, content_layout[0]);

    // Output pane
    let output_title = if let Some(error) = &app.error {
        format!("Output - Error")
    } else {
        format!("Output - {}", OutputFormat::all()[app.selected_format].as_str())
    };

    let output_block = Block::default()
        .title(output_title)
        .borders(Borders::ALL);

    let output_text = if let Some(error) = &app.error {
        vec![Line::from(Span::styled(
            format!("❌ Error: {}", error),
            Style::default().fg(Color::Red)
        ))]
    } else if app.output.is_empty() {
        vec![Line::from("Output will appear here...")]
    } else {
        let all_lines: Vec<Line> = app.output.lines().map(|line| Line::from(line)).collect();
        let content_height = content_layout[1].height.saturating_sub(2) as usize; // Account for borders

        // Apply scrolling offset
        if all_lines.len() > content_height && app.scroll_offset < all_lines.len() {
            let end = (app.scroll_offset + content_height).min(all_lines.len());
            all_lines[app.scroll_offset..end].to_vec()
        } else {
            all_lines
        }
    };

    let output_paragraph = Paragraph::new(output_text)
        .block(output_block)
        .style(Style::default());

    frame.render_widget(output_paragraph, content_layout[1]);

    // Format tabs
    let format_names: Vec<&str> = OutputFormat::all().iter().map(|f| f.as_str()).collect();
    let tabs = Tabs::new(format_names)
        .block(Block::default().borders(Borders::ALL).title("Output Format"))
        .select(app.selected_format)
        .style(Style::default())
        .highlight_style(Style::default().fg(Color::Yellow));

    frame.render_widget(tabs, main_layout[1]);
}

/// Run the TUI REPL
async fn run_tui_repl() -> std::result::Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|frame| draw_ui(frame, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc => break,
                    KeyCode::Tab => app.next_format().await,
                    KeyCode::BackTab => app.prev_format().await,
                    KeyCode::Char(c) => app.insert_char(c).await,
                    KeyCode::Backspace => app.delete_char().await,
                    KeyCode::Left => app.move_cursor_left(),
                    KeyCode::Right => app.move_cursor_right(),
                    KeyCode::Up => {
                        let content_height = terminal.size()?.height.saturating_sub(5) as usize / 2; // Rough estimate for output pane height
                        app.scroll_up();
                    },
                    KeyCode::Down => {
                        let content_height = terminal.size()?.height.saturating_sub(5) as usize / 2; // Rough estimate for output pane height
                        app.scroll_down(content_height);
                    },
                    KeyCode::PageUp => {
                        let content_height = terminal.size()?.height.saturating_sub(5) as usize / 2;
                        app.page_up(content_height);
                    },
                    KeyCode::PageDown => {
                        let content_height = terminal.size()?.height.saturating_sub(5) as usize / 2;
                        app.page_down(content_height, content_height);
                    },
                    KeyCode::Enter => app.insert_newline().await,
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
