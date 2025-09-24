use eframe::egui;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::path::Path;
use std::fs;
use crate::pipeline;
// SVG renderer module removed
use crate::models::Notation;
use crate::gui::SkiaCanvasRenderer;


// Shared state for GUI testing API
pub static GUI_STATE: std::sync::OnceLock<Arc<Mutex<GuiTestState>>> = std::sync::OnceLock::new();

#[derive(Debug, Clone)]
pub struct GuiTestState {
    pub content_line: String,
    pub last_svg: String,
    pub last_image_data: Vec<u8>,
    pub input_queue: Vec<GuiTestCommand>,
}

#[derive(Debug, Clone)]
pub enum GuiTestCommand {
    TypeText(String),
    KeyPress(char),
    Backspace,
    Clear,
}

impl Default for GuiTestState {
    fn default() -> Self {
        Self {
            content_line: String::new(),
            last_svg: String::new(),
            last_image_data: Vec::new(),
            input_queue: Vec::new(),
        }
    }
}


struct MusicTextApp {
    content_line: String,
    cursor_pos: usize,
    parsed_output: String,
    lilypond_output: String,
    document_output: String,
    error_message: Option<String>,
    canvas_content: String,
    canvas_texture: Option<egui::TextureHandle>,
    // File watching
    watch_file: String,
    last_modified: Option<std::time::SystemTime>,
}

impl Default for MusicTextApp {
    fn default() -> Self {
        // Initialize shared GUI state for API testing
        let _ = GUI_STATE.set(Arc::new(Mutex::new(GuiTestState::default())));

        // Generate test SVG immediately to verify it works
        let test_svg = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><svg xmlns=\"http://www.w3.org/2000/svg\" width=\"400\" height=\"200\"><text x=\"20\" y=\"50\">Test SVG</text></svg>".to_string();

        let watch_file = "/tmp/gui_input.txt".to_string();

        // Try to load initial content from watch file
        let initial_content = if Path::new(&watch_file).exists() {
            fs::read_to_string(&watch_file).map(|content| content.trim().to_string()).unwrap_or_default()
        } else {
            // Create the watch file with a default example
            let default_content = "123";
            let _ = fs::write(&watch_file, default_content);
            default_content.to_string()
        };

        Self {
            content_line: initial_content,
            cursor_pos: 0,
            parsed_output: String::new(),
            lilypond_output: String::new(),
            document_output: String::new(),
            error_message: None,
            canvas_content: "Enter music notation here...".to_string(),
            canvas_texture: None,
            watch_file,
            last_modified: None,
        }
    }
}

impl MusicTextApp {
    fn create_texture_from_rgba(&self, ctx: &egui::Context, rgba_data: Vec<u8>, width: u32, height: u32) -> Option<egui::TextureHandle> {
        let expected_size = (width * height * 4) as usize;
        if rgba_data.len() != expected_size {
            eprintln!("RGBA data size mismatch: got {}, expected {} ({}x{})", rgba_data.len(), expected_size, width, height);
            return None;
        }

        let image = egui::ColorImage::from_rgba_unmultiplied(
            [width as usize, height as usize],
            &rgba_data,
        );
        eprintln!("Creating texture from Skia: {}x{}", width, height);

        Some(ctx.load_texture("skia_canvas", image, Default::default()))
    }


    fn svg_to_texture(&mut self, ctx: &egui::Context, svg_content: &str) -> Option<egui::TextureHandle> {

        // Try to render SVG to image using resvg
        match self.render_svg_to_image(svg_content) {
            Ok(image_data) => {
                eprintln!("SVG rendering successful, got {} bytes", image_data.len());

                // Parse SVG to get actual dimensions
                let (width, height) = self.get_svg_dimensions(svg_content).unwrap_or((800, 600));
                eprintln!("SVG dimensions: {}x{}", width, height);

                // Validate image data size
                let expected_size = (width * height * 4) as usize; // RGBA = 4 bytes per pixel
                if image_data.len() != expected_size {
                    eprintln!("Image data size mismatch: got {}, expected {} ({}x{})",
                        image_data.len(), expected_size, width, height);
                    return None;
                }

                // Create texture from image data
                let image = egui::ColorImage::from_rgba_unmultiplied(
                    [width as usize, height as usize],
                    &image_data,
                );
                eprintln!("Creating texture {}x{}", width, height);

                // Save texture as raw RGBA for debugging (skip PNG for now)
                if let Err(e) = std::fs::write("/tmp/debug_texture.rgba", &image_data) {
                    eprintln!("Failed to save debug texture: {}", e);
                } else {
                    eprintln!("Saved debug texture to /tmp/debug_texture.rgba ({}x{} RGBA)", width, height);
                }

                Some(ctx.load_texture("svg_content", image, Default::default()))
            }
            Err(e) => {
                eprintln!("SVG to texture failed: {}", e);
                None
            }
        }
    }

    fn get_svg_dimensions(&self, svg_content: &str) -> Option<(u32, u32)> {
        // Quick and dirty SVG dimension parsing
        if let Some(width_start) = svg_content.find("width=\"") {
            if let Some(width_end) = svg_content[width_start + 7..].find("\"") {
                if let Ok(width) = svg_content[width_start + 7..width_start + 7 + width_end].parse::<u32>() {
                    if let Some(height_start) = svg_content.find("height=\"") {
                        if let Some(height_end) = svg_content[height_start + 8..].find("\"") {
                            if let Ok(height) = svg_content[height_start + 8..height_start + 8 + height_end].parse::<u32>() {
                                return Some((width, height));
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn render_svg_to_image(&self, svg_content: &str) -> Result<Vec<u8>, String> {
        // Skip Cairo for now and use resvg to actually render the SVG content


        // Configure resvg with better font handling
        let mut opt = resvg::usvg::Options::default();

        // Set up font database with system fonts
        opt.fontdb_mut().load_system_fonts();

        // Skip font debugging for now
        eprintln!("Loaded {} fonts", opt.fontdb.len());

        // Set up font fallbacks - use common system fonts
        opt.fontdb_mut().set_serif_family("serif");
        opt.fontdb_mut().set_sans_serif_family("DejaVu Sans");
        opt.fontdb_mut().set_monospace_family("DejaVu Sans Mono");

        let rtree = resvg::usvg::Tree::from_str(svg_content, &opt)
            .map_err(|e| format!("SVG parsing error: {}", e))?;

        let pixmap_size = rtree.size();

        let mut pixmap = resvg::tiny_skia::Pixmap::new(
            pixmap_size.width() as u32,
            pixmap_size.height() as u32
        ).ok_or("Failed to create pixmap")?;

        // Clear with light gray background to see if anything renders
        pixmap.fill(resvg::tiny_skia::Color::from_rgba8(240, 240, 240, 255));


        resvg::render(&rtree, resvg::usvg::Transform::default(), &mut pixmap.as_mut());

        let data = pixmap.data();

        // Check for non-background pixels
        let mut has_content = false;
        for chunk in data.chunks(4) {
            if chunk.len() == 4 && !(chunk[0] == 240 && chunk[1] == 240 && chunk[2] == 240) {
                has_content = true;
                break;
            }
        }

        Ok(data.to_vec())
    }


    fn check_file_changes(&mut self, ctx: &egui::Context) -> bool {
        if !Path::new(&self.watch_file).exists() {
            return false;
        }

        if let Ok(metadata) = fs::metadata(&self.watch_file) {
            if let Ok(modified) = metadata.modified() {
                if self.last_modified.is_none() || self.last_modified.unwrap() < modified {
                    self.last_modified = Some(modified);

                    // File has changed, reload content
                    if let Ok(new_content) = fs::read_to_string(&self.watch_file) {
                        let trimmed = new_content.trim().to_string();
                        if trimmed != self.content_line {
                            self.content_line = trimmed;
                            self.cursor_pos = self.content_line.len();
                            self.update_parsing(ctx);
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn process_test_commands(&mut self, ctx: &egui::Context) {
        if let Some(state_mutex) = GUI_STATE.get() {
            if let Ok(mut state) = state_mutex.lock() {
                let commands: Vec<GuiTestCommand> = state.input_queue.drain(..).collect();
                for command in commands {
                    match command {
                        GuiTestCommand::TypeText(text) => {
                            for ch in text.chars() {
                                self.content_line.insert(self.cursor_pos, ch);
                                self.cursor_pos += 1;
                            }
                            self.update_parsing(ctx);
                        }
                        GuiTestCommand::KeyPress(ch) => {
                            self.content_line.insert(self.cursor_pos, ch);
                            self.cursor_pos += 1;
                            self.update_parsing(ctx);
                        }
                        GuiTestCommand::Backspace => {
                            if self.cursor_pos > 0 {
                                self.cursor_pos -= 1;
                                self.content_line.remove(self.cursor_pos);
                            }
                            self.update_parsing(ctx);
                        }
                        GuiTestCommand::Clear => {
                            self.content_line.clear();
                            self.cursor_pos = 0;
                            self.update_parsing(ctx);
                        }
                    }
                }

                // Only render Cairo when content actually changes
                let content_changed = state.content_line != self.content_line;

                // Update shared state with current GUI state
                state.content_line = self.content_line.clone();
                state.last_svg = String::new(); // No longer using SVG

                if content_changed && !self.content_line.is_empty() {
                    // No longer using Cairo
                }
            }
        }
    }

    fn save_to_watch_file(&self) {
        if let Err(e) = fs::write(&self.watch_file, &self.content_line) {
        }
    }

    fn setup_styling(&self, ctx: &egui::Context) {
        use egui::{Color32, Stroke, Rounding, Shadow, Margin};

        // Modern color scheme inspired by the CSS
        let mut style = (*ctx.style()).clone();

        // Primary colors
        let primary_blue = Color32::from_rgb(37, 99, 235);     // #2563eb
        let surface_white = Color32::from_rgb(255, 255, 255);   // #ffffff
        let background_gray = Color32::from_rgb(248, 250, 252); // #f8fafc
        let text_dark = Color32::from_rgb(30, 41, 59);          // #1e293b
        let border_gray = Color32::from_rgb(226, 232, 240);     // #e2e8f0
        let secondary_gray = Color32::from_rgb(100, 116, 139);  // #64748b

        // Apply modern styling
        style.visuals.widgets.noninteractive.bg_fill = surface_white;
        style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, border_gray);
        style.visuals.widgets.noninteractive.rounding = Rounding::same(8.0);

        style.visuals.widgets.inactive.bg_fill = surface_white;
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, border_gray);
        style.visuals.widgets.inactive.rounding = Rounding::same(8.0);

        style.visuals.widgets.hovered.bg_fill = background_gray;
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, primary_blue);
        style.visuals.widgets.hovered.rounding = Rounding::same(8.0);

        style.visuals.widgets.active.bg_fill = primary_blue;
        style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, surface_white);
        style.visuals.widgets.active.rounding = Rounding::same(8.0);

        // Panel styling
        style.visuals.window_fill = surface_white;
        style.visuals.window_stroke = Stroke::new(1.0, border_gray);
        style.visuals.window_rounding = Rounding::same(12.0);
        style.visuals.window_shadow = Shadow {
            blur: 16.0,
            spread: 4.0,
            offset: egui::vec2(0.0, 4.0),
            color: Color32::from_black_alpha(60),
        };

        style.visuals.panel_fill = background_gray;

        // Text styling
        style.visuals.override_text_color = Some(text_dark);

        // Button styling
        style.spacing.button_padding = egui::vec2(16.0, 8.0);
        style.visuals.widgets.noninteractive.rounding = Rounding::same(6.0);

        // Apply the styling
        ctx.set_style(style);
    }

    fn update_parsing(&mut self, ctx: &egui::Context) {
        if self.content_line.trim().is_empty() {
            self.parsed_output.clear();
            self.lilypond_output.clear();
            self.document_output.clear();
            self.canvas_content = "Enter music notation here...".to_string();
            self.canvas_texture = None;
            self.error_message = None;
            return;
        }

        // Process the musical notation and render to SVG
        match pipeline::process_notation(&self.content_line) {
            Ok(result) => {
                self.error_message = None;

                // Use Skia canvas renderer
                eprintln!("DEBUG: About to render with Skia");
                let mut renderer = SkiaCanvasRenderer::new(800, 200);
                let texture = if let Some(mut renderer) = renderer {
                    if let Some(rgba_data) = renderer.render_document(&result.document) {
                        eprintln!("DEBUG: Skia rendered {} bytes", rgba_data.len());
                        self.create_texture_from_rgba(ctx, rgba_data, 800, 200)
                    } else {
                        eprintln!("DEBUG: Skia render failed");
                        None
                    }
                } else {
                    eprintln!("DEBUG: Failed to create Skia renderer");
                    None
                };

                // Store results
                self.canvas_texture = texture;

                // Update other outputs
                self.lilypond_output = result.lilypond.clone();
                self.canvas_content = format!("Musical notation: {}", self.content_line);

                match serde_json::to_string_pretty(&result.document) {
                    Ok(json) => self.document_output = json,
                    Err(e) => self.error_message = Some(format!("JSON serialization error: {}", e)),
                }
                self.parsed_output = format!("{:#?}", result.document);
            }
            Err(e) => {
                self.error_message = Some(format!("Parse error: {}", e));
                self.canvas_content = format!("Parse Error: {}", e);
                self.canvas_texture = None;
            }
        }
    }

    fn handle_text_input(&mut self, text: &str, ctx: &egui::Context) {
        for ch in text.chars() {
            if ch.is_control() && ch != '\n' {
                continue;
            }
            self.content_line.insert(self.cursor_pos, ch);
            self.cursor_pos += 1;
        }
        self.save_to_watch_file();
        self.update_parsing(ctx);
    }

    fn handle_backspace(&mut self, ctx: &egui::Context) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.content_line.remove(self.cursor_pos);
            self.save_to_watch_file();
            self.update_parsing(ctx);
        }
    }

    fn handle_key(&mut self, key: egui::Key, modifiers: egui::Modifiers, ctx: &egui::Context) {
        match key {
            egui::Key::Backspace => self.handle_backspace(ctx),
            egui::Key::ArrowLeft => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
            }
            egui::Key::ArrowRight => {
                if self.cursor_pos < self.content_line.len() {
                    self.cursor_pos += 1;
                }
            }
            egui::Key::Home => self.cursor_pos = 0,
            egui::Key::End => self.cursor_pos = self.content_line.len(),
            _ => {}
        }
    }
}

impl eframe::App for MusicTextApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply modern styling
        self.setup_styling(ctx);
        // Check for file changes first
        self.check_file_changes(ctx);

        // Process GUI test commands from API
        self.process_test_commands(ctx);

        // Handle keyboard input
        let events: Vec<egui::Event> = ctx.input(|i| i.events.clone());
        if !events.is_empty() {
        }
        for event in events {
            match event {
                egui::Event::Text(text) => {
                    self.handle_text_input(&text, ctx);
                }
                egui::Event::Key { key, pressed: true, modifiers, .. } => {
                    self.handle_key(key, modifiers, ctx);
                }
                _ => {}
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // Content line input area
            ui.horizontal(|ui| {

                // Text input with copy/paste support
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.content_line)
                        .desired_width(600.0)
                        .hint_text("Enter music notation here...")
                );


                if response.changed() {
                    self.update_parsing(ctx);
                }

                if response.has_focus() {
                    // Keep focus on the text input
                    response.request_focus();
                }
            });

            ui.separator();

            // Error display
            if let Some(error) = &self.error_message {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
                ui.separator();
            }

            // Canvas area - try alternative approach with allocate_response
            let available_rect = ui.available_rect_before_wrap();
            let (rect, response) = ui.allocate_exact_size(available_rect.size(), egui::Sense::click());

            // Draw background
            ui.painter().rect_filled(rect, 0.0, egui::Color32::WHITE);
            ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::BLACK));

            if let Some(texture) = &self.canvas_texture {
                eprintln!("DEBUG: Drawing texture in rect: {:?}", rect);
                // Draw the texture to fill the allocated rectangle
                ui.painter().image(texture.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
            } else {
                ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "Enter music notation above", egui::FontId::default(), egui::Color32::GRAY);
            }


        });

        // Request repaint for real-time updates
        ctx.request_repaint();
    }
}

pub fn run_gui() -> Result<(), Box<dyn Error>> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_title("Music Text Canvas Editor"),
        ..Default::default()
    };

    eframe::run_native(
        "Music Text Canvas Editor",
        options,
        Box::new(|_cc| Ok(Box::new(MusicTextApp::default()))),
    ).map_err(|e| format!("Failed to run GUI: {}", e))?;

    Ok(())
}