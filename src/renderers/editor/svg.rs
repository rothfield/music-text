/// Canvas WYSIWYG SVG Renderer
/// Specialized SVG renderer for the web canvas editor that provides real-time visual feedback
use crate::parse::Document;
use crate::models::{pitch_systems, Notation};
use std::fmt::Write;

/// Canvas-specific SVG configuration optimized for real-time rendering
pub struct CanvasSvgConfig {
    pub width: f32,
    pub height: f32,
    pub font_size: f32,
    pub show_cursor: bool,
    pub cursor_position: usize,
    pub show_selection: bool,
    pub selection_start: usize,
    pub selection_end: usize,
}

impl Default for CanvasSvgConfig {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 400.0,
            font_size: 20.0,
            show_cursor: false,
            cursor_position: 0,
            show_selection: false,
            selection_start: 0,
            selection_end: 0,
        }
    }
}

/// Canvas SVG renderer for WYSIWYG editing
pub struct CanvasSvgRenderer {
    config: CanvasSvgConfig,
    current_x: f32,
    current_y: f32,
    char_positions: std::collections::HashMap<usize, f32>,  // Maps character position to x coordinate
}

impl CanvasSvgRenderer {
    pub fn new(config: CanvasSvgConfig) -> Self {
        Self {
            config,
            current_x: 20.0,
            current_y: 60.0,
            char_positions: std::collections::HashMap::new(),
        }
    }

    /// Render document to SVG optimized for canvas display
    pub fn render(&mut self, document: &Document, notation_type: &str, input_text: &str) -> Result<String, String> {
        let mut svg = String::new();

        // SVG header
        writeln!(svg, r#"<?xml version="1.0" encoding="UTF-8"?>"#).unwrap();
        writeln!(svg, r#"<svg xmlns="http://www.w3.org/2000/svg""#).unwrap();
        writeln!(svg, r#"     width="{}" height="{}""#, self.config.width, self.config.height).unwrap();
        writeln!(svg, r#"     viewBox="0 0 {} {}">"#, self.config.width, self.config.height).unwrap();

        // Canvas-specific styling
        svg.push_str(&self.generate_canvas_css());

        // Background
        writeln!(svg, "  <rect width=\"{}\" height=\"{}\" fill=\"#fafafa\" stroke=\"#dddddd\" stroke-width=\"1\"/>",
                self.config.width, self.config.height).unwrap();

        // Main content group
        writeln!(svg, r#"  <g class="canvas-content" transform="translate({}, {})">"#,
                self.current_x, self.current_y).unwrap();

        // Reset position for content
        self.current_x = 0.0;
        self.current_y = 0.0;

        // Render title at the top if it exists
        if let Some(ref title) = document.title {
            if !title.trim().is_empty() {
                self.render_title_at_top(&mut svg, title)?;
                self.current_y += 40.0; // Add space after title
            }
        }

        // Render music elements or placeholder if no musical content
        let mut char_position = 0;
        let has_musical_content = !document.elements.is_empty();

        if has_musical_content {
            for element in &document.elements {
                match element {
                    crate::parse::model::DocumentElement::Stave(stave) => {
                        for (line_index, line) in stave.lines.iter().enumerate() {
                            if let crate::parse::model::StaveLine::ContentLine(content_line) = line {
                                // Move to new line for each content line (except first)
                                if line_index > 0 {
                                    self.current_y += 60.0; // Line spacing
                                    self.current_x = 0.0;   // Reset to start of line
                                }

                                for content_element in &content_line.elements {
                                    if let crate::parse::model::ContentElement::Beat(beat) = content_element {
                                        let (new_x, chars_consumed) = self.render_beat_with_cursor(
                                            &mut svg, beat, notation_type, char_position, input_text
                                        )?;
                                        self.current_x = new_x;
                                        char_position += chars_consumed;
                                    } else if let crate::parse::model::ContentElement::Barline(barline) = content_element {
                                        // Store character position for barline
                                        self.char_positions.insert(char_position, self.current_x);

                                        // Check if cursor should be shown at this position
                                        if self.config.show_cursor && char_position == self.config.cursor_position {
                                            self.render_cursor_at_position(&mut svg, self.current_x);
                                        }

                                        self.render_barline(&mut svg, barline)?;
                                        char_position += 1; // Barlines typically consume one character
                                    }
                                    // Handle other content elements (whitespace, unknown tokens, etc.)
                                    else if let crate::parse::model::ContentElement::Whitespace(ws) = content_element {
                                        let default_space = " ".to_string();
                                        let ws_value = ws.value.as_ref().unwrap_or(&default_space);
                                        // Track character positions for whitespace
                                        for (char_idx, _) in ws_value.char_indices() {
                                            let char_pos = char_position + char_idx;
                                            let char_x = self.current_x + (char_idx as f32 * self.get_char_width(" "));
                                            self.char_positions.insert(char_pos, char_x);

                                            // Check if cursor should be shown at this position
                                            if self.config.show_cursor && char_pos == self.config.cursor_position {
                                                self.render_cursor_at_position(&mut svg, char_x);
                                            }
                                        }
                                        self.current_x += self.get_char_width(ws_value);
                                        char_position += ws_value.len();
                                    }
                                    else if let crate::parse::model::ContentElement::UnknownToken(token) = content_element {
                                        // Track character positions for unknown tokens
                                        for (char_idx, _) in token.token_value.char_indices() {
                                            let char_pos = char_position + char_idx;
                                            let char_x = self.current_x + (char_idx as f32 * self.get_char_width("1"));
                                            self.char_positions.insert(char_pos, char_x);

                                            // Check if cursor should be shown at this position
                                            if self.config.show_cursor && char_pos == self.config.cursor_position {
                                                self.render_cursor_at_position(&mut svg, char_x);
                                            }
                                        }
                                        // Render unknown tokens with a different style
                                        writeln!(svg, r#"    <text x="{:.1}" y="{}" class="canvas-unknown">{}</text>"#,
                                                self.current_x, self.current_y, token.token_value).unwrap();
                                        self.current_x += self.get_char_width(&token.token_value);
                                        char_position += token.token_value.len();
                                    }
                                }
                            }
                        }
                    }
                    _ => {} // Handle other document elements as needed
                }
            }
        } else {
            // Render title or placeholder when no musical content
            self.render_title_or_placeholder(&mut svg, document, input_text)?;
        }

        // Store final position for cursor at end of content
        self.char_positions.insert(char_position, self.current_x);

        // Show selection highlighting if enabled
        if self.config.show_selection {
            self.render_selection_highlighting(&mut svg);
        }

        // Show cursor if enabled - now handles any character position
        if self.config.show_cursor {
            // Find the correct x position for the cursor based on character positions
            let cursor_x = self.char_positions.get(&self.config.cursor_position)
                .copied()
                .unwrap_or(self.current_x);
            self.render_cursor_at_position(&mut svg, cursor_x);
        }

        writeln!(svg, "  </g>").unwrap();
        writeln!(svg, "</svg>").unwrap();

        Ok(svg)
    }

    /// Generate canvas-specific CSS
    fn generate_canvas_css(&self) -> String {
        // Always use embedded CSS to ensure all styles are included
        self.generate_embedded_css()
    }

    /// Generate CSS for development (external + dynamic overrides)
    fn generate_dev_css(&self) -> String {
        let note_size = self.config.font_size;
        let octave_size = note_size * 0.6;

        format!(r#"
  <?xml-stylesheet type="text/css" href="/assets/svg-styles.css"?>
  <style>
    <![CDATA[
    /* Dynamic overrides for calculated values */
    .canvas-note {{
      font-size: {}px;
    }}

    .canvas-octave-upper {{
      font-size: {}px;
    }}

    .canvas-octave-lower {{
      font-size: {}px;
    }}

    .canvas-barline {{
      font-size: {}px;
    }}

    .canvas-placeholder {{
      font-size: {}px;
    }}
    ]]>
  </style>
"#, note_size, octave_size, octave_size, note_size, note_size * 0.9)
    }

    /// Generate embedded CSS for production (self-contained)
    fn generate_embedded_css(&self) -> String {
        let note_size = self.config.font_size;
        let octave_size = note_size * 0.6;

        format!(r#"
  <style>
    <![CDATA[
    @font-face {{
      font-family: 'Bravura';
      src: url('/fonts/Bravura.woff2') format('woff2'),
           url('/fonts/Bravura.woff') format('woff');
      font-weight: normal;
      font-style: normal;
    }}

    .canvas-content {{
      font-family: monospace, 'Courier New', monospace;
    }}

    .canvas-note {{
      font-size: {}px;
      fill: deepskyblue;
      font-weight: normal;
      font-family: monospace, 'Courier New', monospace;
    }}

    .canvas-octave-upper {{
      font-size: {}px;
      fill: red;
      font-weight: normal;
      font-family: monospace, 'Courier New', monospace;
    }}

    .canvas-octave-lower {{
      font-size: {}px;
      fill: red;
      font-weight: normal;
      font-family: monospace, 'Courier New', monospace;
    }}

    .canvas-barline {{
      font-size: {}px;
      fill: mediumorchid;
      font-weight: bold;
      font-family: monospace, 'Courier New', monospace;
    }}

    .canvas-cursor {{
      stroke: #e74c3c;
      stroke-width: 2;
      animation: blink 1s infinite;
    }}

    .canvas-beat-arc {{
      stroke: orange;
      stroke-width: 2;
      fill: none;
      opacity: 0.8;
    }}

    .canvas-selection {{
      fill: rgba(173, 216, 230, 0.3);
      stroke: none;
    }}

    .canvas-placeholder {{
      font-size: {}px;
      fill: gray;
      font-style: italic;
      font-weight: normal;
      font-family: monospace, 'Courier New', monospace;
    }}

    .canvas-unknown {{
      font-size: {}px;
      fill: gray;
      font-weight: normal;
      font-style: italic;
      font-family: monospace, 'Courier New', monospace;
    }}

    .canvas-title {{
      font-size: {}px;
      fill: black;
      font-weight: bold;
      font-family: monospace, 'Courier New', monospace;
    }}

    @keyframes blink {{
      0%, 50% {{ opacity: 1; }}
      51%, 100% {{ opacity: 0; }}
    }}

    /* Character replacement styles removed - handled in rendering logic */
    ]]>
  </style>
"#, note_size, octave_size, octave_size, note_size, note_size * 0.9, note_size * 0.8, note_size * 1.5)
    }

    /// Render a beat with cursor awareness
    fn render_beat_with_cursor(
        &mut self,
        svg: &mut String,
        beat: &crate::parse::model::Beat,
        notation_type: &str,
        start_char_position: usize,
        input_text: &str
    ) -> Result<(f32, usize), String> {
        let beat_start_x = self.current_x;
        let mut chars_consumed = 0;
        let mut element_positions = Vec::new();

        for beat_element in &beat.elements {
            match beat_element {
                crate::parse::model::BeatElement::Note(note) => {
                    // Convert pitch code to notation
                    let notation = match notation_type {
                        "sargam" => Notation::Sargam,
                        "western" => Notation::Western,
                        _ => Notation::Number,
                    };

                    let note_value = pitch_systems::pitchcode_to_string(note.pitch_code, notation)
                        .unwrap_or_else(|| "1".to_string());

                    // Store character position for each character in the note
                    for (char_idx, _) in note_value.char_indices() {
                        let char_pos = start_char_position + chars_consumed + char_idx;
                        let char_x = self.current_x + (char_idx as f32 * self.get_char_width("1"));
                        self.char_positions.insert(char_pos, char_x);

                        // Character positions are stored for cursor rendering at the end
                    }

                    element_positions.push(self.current_x);

                    // Render note
                    writeln!(svg, r#"    <text x="{:.1}" y="{}" class="canvas-note">{}</text>"#,
                            self.current_x, self.current_y, note_value).unwrap();

                    // Render octave markers
                    if note.octave > 0 {
                        let dots = "•".repeat(note.octave as usize);
                        writeln!(svg, r#"    <text x="{:.1}" y="{}" class="canvas-octave-upper">{}</text>"#,
                                self.current_x + 3.0, self.current_y - 15.0, dots).unwrap();
                    } else if note.octave < 0 {
                        let dots = "•".repeat((-note.octave) as usize);
                        writeln!(svg, r#"    <text x="{:.1}" y="{}" class="canvas-octave-lower">{}</text>"#,
                                self.current_x + 3.0, self.current_y + 20.0, dots).unwrap();
                    }

                    self.current_x += self.get_char_width(&note_value);
                    chars_consumed += note_value.len();
                }
                crate::parse::model::BeatElement::Dash(_) => {
                    // Store character position for dash
                    let char_pos = start_char_position + chars_consumed;
                    self.char_positions.insert(char_pos, self.current_x);

                    // Character position stored for cursor rendering at the end

                    element_positions.push(self.current_x);
                    writeln!(svg, r#"    <text x="{:.1}" y="{}" class="canvas-note">-</text>"#,
                            self.current_x, self.current_y).unwrap();

                    self.current_x += self.get_char_width("-");
                    chars_consumed += 1;
                }
                _ => {
                    // Handle other beat elements
                    chars_consumed += 1;
                }
            }
        }

        // Draw beat grouping arc under notes if multiple elements
        if element_positions.len() > 1 {
            self.render_beat_arc(svg, &element_positions);
        }

        // No beat spacing - elements flow naturally

        Ok((self.current_x, chars_consumed))
    }

    /// Render barline
    /// Render barline using SMuFL music font symbols
    fn render_barline(&mut self, svg: &mut String, barline: &crate::parse::model::Barline) -> Result<(), String> {
        let symbol = match barline {
            crate::parse::model::Barline::Single(_) => "|",
            crate::parse::model::Barline::Double(_) => "||",
            crate::parse::model::Barline::Final(_) => "||",
            crate::parse::model::Barline::RepeatStart(_) => "|:",
            crate::parse::model::Barline::RepeatEnd(_) => ":|",
            crate::parse::model::Barline::RepeatBoth(_) => ":|:",
        };

        writeln!(svg, r#"    <text x="{:.1}" y="{}" class="canvas-barline">{}</text>"#,
                self.current_x, self.current_y, symbol).unwrap();

        let width_adjustment = self.get_char_width(symbol);
        self.current_x += width_adjustment + 10.0;
        Ok(())
    }

    /// Render cursor at current position
    fn render_cursor(&self, svg: &mut String) {
        self.render_cursor_at_position(svg, self.current_x);
    }

    /// Render cursor at specific position
    fn render_cursor_at_position(&self, svg: &mut String, x: f32) {
        writeln!(svg, r#"    <line x1="{:.1}" y1="{}" x2="{:.1}" y2="{}" class="canvas-cursor"/>"#,
                x, self.current_y - 20.0, x, self.current_y + 5.0).unwrap();
    }

    /// Render arc connecting beat elements
    fn render_beat_arc(&self, svg: &mut String, positions: &[f32]) {
        if positions.len() < 2 {
            return;
        }

        let start_x = positions[0];
        let end_x = positions[positions.len() - 1] + self.get_char_width("1"); // Add actual character width

        // Position arc just under the notes like a hammock
        let arc_start_y = self.current_y + 8.0;  // Start point just under notes
        let arc_end_y = self.current_y + 8.0;    // End point just under notes

        // Calculate ellipse parameters
        let width = end_x - start_x;
        let rx = width / 2.0;  // X-radius (half the width)
        let ry = 8.0;          // Y-radius (made less tall)

        // Use elliptical arc command: A rx,ry x-axis-rotation large-arc-flag,sweep-flag x,y
        // sweep-flag=0 creates an upward curve (counter-clockwise for hammock under notes)
        writeln!(svg, r#"    <path d="M {:.1} {:.1} A {:.1} {:.1} 0 0 0 {:.1} {:.1}" class="canvas-beat-arc"/>"#,
                start_x, arc_start_y, rx, ry, end_x, arc_end_y).unwrap();
    }

    /// Render title at the top of the page
    fn render_title_at_top(&mut self, svg: &mut String, title: &str) -> Result<(), String> {
        let title_x = self.config.width / 2.0;
        let title_y = self.current_y + 30.0; // Position from top

        writeln!(svg, r#"    <text x="{:.1}" y="{:.1}" class="canvas-title" text-anchor="middle">{}</text>"#,
                title_x, title_y, title).unwrap();

        Ok(())
    }

    /// Render title or placeholder when no musical content exists
    fn render_title_or_placeholder(&mut self, svg: &mut String, document: &Document, input_text: &str) -> Result<(), String> {
        let display_text = if let Some(ref title) = document.title {
            if title.trim().is_empty() {
                "No musical content"
            } else {
                title.trim()
            }
        } else if input_text.trim().is_empty() {
            "Enter musical notation"
        } else {
            "No musical content detected"
        };

        // Center the text
        let text_x = self.config.width / 2.0;
        let text_y = self.config.height / 2.0;

        writeln!(svg, r#"    <text x="{:.1}" y="{:.1}" class="canvas-placeholder" text-anchor="middle">{}</text>"#,
                text_x, text_y, display_text).unwrap();

        // Show cursor if enabled and input matches cursor position
        if self.config.show_cursor {
            let cursor_x = text_x + self.get_char_width(display_text) / 2.0;
            self.render_cursor_at_position(svg, cursor_x);
        }

        Ok(())
    }

    /// Render selection highlighting for the configured range
    fn render_selection_highlighting(&self, svg: &mut String) {
        let start_pos = self.config.selection_start;
        let end_pos = self.config.selection_end;

        // Get x coordinates for start and end positions
        let start_x = self.char_positions.get(&start_pos).copied().unwrap_or(0.0);
        let end_x = self.char_positions.get(&end_pos).copied().unwrap_or_else(|| {
            // If end position is beyond tracked positions, extend to current x
            self.current_x
        });

        if end_x > start_x {
            self.render_selection_at_range(svg, start_x, end_x);
        }
    }

    /// Render selection highlight rectangle
    fn render_selection_at_range(&self, svg: &mut String, start_x: f32, end_x: f32) {
        let selection_height = self.config.font_size + 10.0;
        let selection_y = self.current_y - self.config.font_size + 5.0;
        writeln!(svg, r#"    <rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" class="canvas-selection"/>"#,
                start_x - 2.0, selection_y, end_x - start_x + 4.0, selection_height).unwrap();
    }

    /// Get approximate character width
    fn get_char_width(&self, text: &str) -> f32 {
        // Approximate monospace character width based on font size
        text.chars().count() as f32 * (self.config.font_size * 0.6)
    }
}

impl Default for CanvasSvgRenderer {
    fn default() -> Self {
        Self::new(CanvasSvgConfig::default())
    }
}

/// Convenience function to render document for canvas display
pub fn render_canvas_svg(
    document: &Document,
    notation_type: &str,
    input_text: &str,
    cursor_position: Option<usize>,
    selection_start: Option<usize>,
    selection_end: Option<usize>
) -> Result<String, String> {
    let mut config = CanvasSvgConfig::default();

    if let Some(pos) = cursor_position {
        config.show_cursor = true;
        config.cursor_position = pos;
    }

    if let (Some(start), Some(end)) = (selection_start, selection_end) {
        config.show_selection = true;
        config.selection_start = start;
        config.selection_end = end;
    }

    let mut renderer = CanvasSvgRenderer::new(config);
    renderer.render(document, notation_type, input_text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::recursive_descent::parse_document;

    #[test]
    fn test_canvas_svg_title_only() {
        let input = "11222x";
        let document = parse_document(input).expect("Should parse successfully");

        // Should have empty elements (title-only document)
        assert_eq!(document.elements.len(), 0);
        assert_eq!(document.title, Some("11222x".to_string()));

        let svg = render_canvas_svg(&document, "number", input, None).expect("Should render SVG");

        // Should contain SVG structure
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        // Should contain title as placeholder text
        assert!(svg.contains("11222x"));
    }
}