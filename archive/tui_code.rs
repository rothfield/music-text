// TUI-related code removed from main.rs on 2025-09-24
// This file contains all the TUI functionality that was removed

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
use music_text::parse::line_classifier;

// Commands enum had Repl variant:
#[derive(Subcommand)]
enum Commands {
    /// Start interactive REPL
    Repl,
    // ... other commands
}

// App struct and implementation
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

        // Use new parser structure (no HTTP requests)
        let result = pipeline::process_notation(&self.input);

        match result {
            Ok(processing_result) => {
                self.error = None;
                self.scroll_offset = 0; // Reset scroll when content changes
                self.output = match format {
                    OutputFormat::LilyPond => {
                        music_text::renderers::lilypond::renderer::convert_processed_document_to_minimal_lilypond_src(&processing_result.document, Some(&self.input))
                            .unwrap_or_else(|e| format!("Error generating minimal lilypond: {}", e))
                    },
                    OutputFormat::LilyPondFull => {
                        processing_result.lilypond
                    },
                    OutputFormat::JSON => {
                        serde_json::to_string_pretty(&processing_result.document)
                            .unwrap_or_else(|e| format!("JSON error: {}", e))
                    },
                    OutputFormat::Debug => {
                        format!("{:#?}", processing_result.document)
                    },
                    OutputFormat::SVG => {
                        processing_result.vexflow_svg.clone()
                    },
                    OutputFormat::Tokens => {
                        let spans = music_text::renderers::codemirror::render_codemirror_spans(&processing_result.document, &self.input);
                        serde_json::to_string_pretty(&spans).unwrap_or("Spans not available".to_string())
                    },
                    OutputFormat::Document => {
                        serde_json::to_string_pretty(&processing_result.document)
                            .unwrap_or_else(|e| format!("JSON error: {}", e))
                    },
                    OutputFormat::CharacterStyles => {
                        let (_spans, styles) = music_text::renderers::codemirror::render_codemirror(&processing_result.document, &self.input);
                        serde_json::to_string_pretty(&styles).unwrap_or("Character styles not available".to_string())
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
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_layout[0]);

    // Input pane
    let input_block = Block::default()
        .title("Input (ESC=quit, Tab=format, ↑↓=scroll)")
        .borders(Borders::ALL);

    // Simple input text rendering (cursor not visible in this simplified version)
    let input_text = if app.input.is_empty() {
        vec![Line::from("Type your musical notation here...")]
    } else {
        app.input.lines().map(|line| Line::from(line)).collect()
    };

    let input_paragraph = Paragraph::new(input_text)
        .block(input_block)
        .style(Style::default())
        .wrap(ratatui::widgets::Wrap { trim: true });

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

// In main() function, this case was handled:
// Some(Commands::Repl) => {
//     run_tui_repl().await?;
//     return Ok(());
// }