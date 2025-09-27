/// Editor WYSIWYG SVG Renderer
/// Specialized SVG renderer for the web canvas editor that provides real-time visual feedback
use crate::parse::Document;
use crate::models::{pitch_systems, Notation};
use std::fmt::Write;

/// Trait for types that have an optional value field
trait HasValue {
    fn value(&self) -> Option<&String>;
    fn newline_count(&self) -> usize {
        self.value().map_or(0, |v| v.matches('\n').count())
    }
}

// Implement for all line types that have value fields
impl HasValue for crate::parse::model::ContentLine {
    fn value(&self) -> Option<&String> { self.value.as_ref() }
}
impl HasValue for crate::parse::model::LyricsLine {
    fn value(&self) -> Option<&String> { self.value.as_ref() }
}
impl HasValue for crate::parse::model::UpperLine {
    fn value(&self) -> Option<&String> { self.value.as_ref() }
}
impl HasValue for crate::parse::model::LowerLine {
    fn value(&self) -> Option<&String> { self.value.as_ref() }
}
impl HasValue for crate::parse::model::TextLine {
    fn value(&self) -> Option<&String> { self.value.as_ref() }
}
impl HasValue for crate::parse::model::WhitespaceLine {
    fn value(&self) -> Option<&String> { self.value.as_ref() }
}
impl HasValue for crate::parse::model::BlankLines {
    fn value(&self) -> Option<&String> { self.value.as_ref() }
}

// Helper for StaveLine enum
impl HasValue for crate::parse::model::StaveLine {
    fn value(&self) -> Option<&String> {
        match self {
            crate::parse::model::StaveLine::ContentLine(line) => line.value(),
            crate::parse::model::StaveLine::Lyrics(line) => line.value(),
            crate::parse::model::StaveLine::Upper(line) => line.value(),
            crate::parse::model::StaveLine::Lower(line) => line.value(),
            crate::parse::model::StaveLine::Text(line) => line.value(),
            crate::parse::model::StaveLine::Whitespace(line) => line.value(),
            crate::parse::model::StaveLine::BlankLines(line) => line.value(),
            _ => None,
        }
    }
}

/// Editor-specific SVG configuration optimized for real-time rendering
pub struct EditorSvgConfig {
    pub width: f32,
    pub height: f32,
    pub font_size: f32,
    pub show_cursor: bool,
    pub cursor_position: usize,
    pub show_selection: bool,
    pub selection_start: usize,
    pub selection_end: usize,
}

impl Default for EditorSvgConfig {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 340.0,
            font_size: 20.0,
            show_cursor: false,
            cursor_position: 0,
            show_selection: false,
            selection_start: 0,
            selection_end: 0,
        }
    }
}

/// Editor SVG renderer for WYSIWYG editing
pub struct EditorSvgRenderer {
    config: EditorSvgConfig,
    current_x: f32,
    current_y: f32,
    char_positions: std::collections::HashMap<usize, (f32, f32)>,  // Maps character position to (x, y) coordinates
    element_coordinates: Vec<ElementCoordinate>,  // Track all element coordinates for JS access
    element_id_counter: usize,  // Simple counter for unique element IDs
}

/// Coordinate information for an element
#[derive(Debug, Clone)]
pub struct ElementCoordinate {
    pub char_start: usize,
    pub char_end: usize,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub element_type: String,
}

impl EditorSvgRenderer {
    pub fn new(config: EditorSvgConfig) -> Self {
        Self {
            config,
            current_x: 20.0,
            current_y: 20.0,  // Reduced top padding from 60.0 to 20.0
            char_positions: std::collections::HashMap::new(),
            element_coordinates: Vec::new(),
            element_id_counter: 0,
        }
    }

    fn emit_newline(&mut self) {
        self.current_y += 60.0;
        self.current_x = 0.0;
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

        // IMPORTANT: Store the initial cursor position (position 0 - before any content)
        // Cursor position 0 means the cursor is BEFORE the first character
        // This is always at the start of the content area
        self.char_positions.insert(0, (0.0, 0.0));

        // Title is now handled as part of normal document structure - no special rendering

        // Just render all characters from input text directly
        let mut char_position = 0;
        let input_chars: Vec<char> = input_text.chars().collect();

        for (char_idx, ch) in input_text.char_indices() {
            let char_width = self.get_char_width(&ch.to_string());

            // Each character gets its own unique UUID
            let char_uuid = uuid::Uuid::new_v4().to_string();

            // Store character position
            self.char_positions.insert(char_idx, (self.current_x, self.current_y));

            // Handle newlines
            if ch == '\n' {
                // Move to next line
                self.current_y += 60.0;
                self.current_x = 0.0;
                char_position = char_idx + 1;
                continue;
            }

            // Render character
            writeln!(svg, r#"    <text id="char-{}" x="{:.1}" y="{}" class="char" data-source-uuid="{}" data-char-index="{}" data-width="{:.1}">{}</text>"#,
                self.element_id_counter,
                self.current_x, self.current_y,
                char_uuid,
                char_idx,
                char_width,
                ch
            ).unwrap();

            self.element_id_counter += 1;
            self.current_x += char_width;
        }

        // Store EOF position
        self.char_positions.insert(input_text.len(), (self.current_x, self.current_y));

        // Show selection highlighting if enabled
        if self.config.show_selection {
            self.render_selection_highlighting(&mut svg);
        }

        // Cursor rendering removed - handled by client-side JavaScript

        writeln!(svg, "  </g>").unwrap();

        // Add metadata group with coordinate information for JavaScript access
        writeln!(svg, r#"  <metadata id="coordinate-data">"#).unwrap();
        writeln!(svg, r#"    <![CDATA["#).unwrap();
        writeln!(svg, r#"    {{"#).unwrap();
        writeln!(svg, r#"      "elements": ["#).unwrap();

        for (i, coord) in self.element_coordinates.iter().enumerate() {
            let comma = if i < self.element_coordinates.len() - 1 { "," } else { "" };
            writeln!(svg, r#"        {{"#).unwrap();
            writeln!(svg, r#"          "charStart": {},"#, coord.char_start).unwrap();
            writeln!(svg, r#"          "charEnd": {},"#, coord.char_end).unwrap();
            writeln!(svg, r#"          "x": {:.1},"#, coord.x).unwrap();
            writeln!(svg, r#"          "y": {:.1},"#, coord.y).unwrap();
            writeln!(svg, r#"          "width": {:.1},"#, coord.width).unwrap();
            writeln!(svg, r#"          "height": {:.1},"#, coord.height).unwrap();
            writeln!(svg, r#"          "type": "{}""#, coord.element_type).unwrap();
            writeln!(svg, r#"        }}{}"#, comma).unwrap();
        }

        writeln!(svg, r#"      ],"#).unwrap();
        writeln!(svg, r#"      "characterPositions": {{"#).unwrap();

        // Add character position mappings
        let mut char_positions: Vec<_> = self.char_positions.iter().collect();
        char_positions.sort_by_key(|(k, _)| *k);

        for (i, (pos, (x, y))) in char_positions.iter().enumerate() {
            let comma = if i < char_positions.len() - 1 { "," } else { "" };
            writeln!(svg, r#"        "{}": {{"x": {:.1}, "y": {:.1}}}{}"#, pos, x, y, comma).unwrap();
        }

        writeln!(svg, r#"      }}"#).unwrap();
        writeln!(svg, r#"    }}"#).unwrap();
        writeln!(svg, r#"    ]]>"#).unwrap();
        writeln!(svg, r#"  </metadata>"#).unwrap();

        writeln!(svg, "</svg>").unwrap();

        Ok(svg)
    }

    /// Generate canvas-specific CSS
    fn generate_canvas_css(&self) -> String {
        // Always use embedded CSS to ensure all styles are included
        self.generate_embedded_css()
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

    .char {{
      font-size: {}px;
      fill: deepskyblue;
      font-weight: normal;
      font-family: monospace, 'Courier New', monospace;
      cursor: pointer;
    }}

    .char.cursor {{
      fill: white;
      stroke: black;
      stroke-width: 2px;
      opacity: 0.9;
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

    /* Cursor CSS removed - handled by client-side JavaScript */

    .canvas-beat-arc {{
      stroke: orange;
      stroke-width: 2;
      fill: none;
      opacity: 0.8;
    }}

    .canvas-selection {{
      fill: rgba(65, 105, 225, 0.4);
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

    .canvas-lyrics {{
      font-size: {}px;
      fill: #666;
      font-style: italic;
      font-family: monospace, 'Courier New', monospace;
    }}

    .canvas-text {{
      font-size: {}px;
      fill: #333;
      font-weight: normal;
      font-family: monospace, 'Courier New', monospace;
    }}

    @keyframes blink {{
      0%, 50% {{ opacity: 1; }}
      51%, 100% {{ opacity: 0; }}
    }}

    /* Character replacement styles removed - handled in rendering logic */
    ]]>
  </style>
"#, note_size, note_size, octave_size, octave_size, note_size, note_size * 0.9, note_size * 0.8, note_size * 1.5, note_size * 0.9, note_size)
    }

    /// Render content line as individual characters using document element UUIDs
    fn render_content_line_chars(
        &mut self,
        svg: &mut String,
        content_line: &crate::parse::model::ContentLine,
        line_text: &str,
        start_char_position: usize
    ) -> Result<(), String> {
        // Extract character-to-UUID mapping from content line elements
        let mut char_uuid_map = std::collections::HashMap::new();
        let mut current_char_index = 0;

        for content_element in &content_line.elements {
            match content_element {
                crate::parse::model::ContentElement::Beat(beat) => {
                    for beat_element in &beat.elements {
                        match beat_element {
                            crate::parse::model::BeatElement::Note(note) => {
                                if let Some(ref value) = note.value {
                                    for _ in value.chars() {
                                        char_uuid_map.insert(current_char_index, note.id.to_string());
                                        current_char_index += 1;
                                    }
                                }
                            }
                            crate::parse::model::BeatElement::Dash(dash) => {
                                if let Some(ref value) = dash.value {
                                    for _ in value.chars() {
                                        char_uuid_map.insert(current_char_index, dash.id.to_string());
                                        current_char_index += 1;
                                    }
                                }
                            }
                            _ => {
                                // Handle other beat elements
                                current_char_index += 1;
                            }
                        }
                    }
                }
                crate::parse::model::ContentElement::Whitespace(ws) => {
                    if let Some(ref value) = ws.value {
                        for _ in value.chars() {
                            // Use a fallback UUID for whitespace
                            char_uuid_map.insert(current_char_index, uuid::Uuid::new_v4().to_string());
                            current_char_index += 1;
                        }
                    }
                }
                crate::parse::model::ContentElement::UnknownToken(token) => {
                    for _ in token.token_value.chars() {
                        // Use a fallback UUID for unknown tokens
                        char_uuid_map.insert(current_char_index, uuid::Uuid::new_v4().to_string());
                        current_char_index += 1;
                    }
                }
                _ => {
                    // Handle other content elements
                    current_char_index += 1;
                }
            }
        }

        // Now render each character with its corresponding UUID
        for (char_idx, ch) in line_text.char_indices() {
            let char_pos = start_char_position + char_idx;
            let char_width = self.get_char_width(&ch.to_string());

            // Get the UUID for this character, or generate fallback
            let char_uuid = char_uuid_map.get(&char_idx)
                .cloned()
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

            // Store character position
            self.char_positions.insert(char_pos, (self.current_x, self.current_y));

            // Render character with document UUID
            writeln!(svg, r#"    <text id="char-{}" x="{:.1}" y="{}" class="char" data-source-uuid="{}" data-char-index="{}" data-width="{:.1}">{}</text>"#,
                self.element_id_counter,
                self.current_x, self.current_y,
                char_uuid,
                char_idx,
                char_width,
                ch
            ).unwrap();

            self.element_id_counter += 1;
            self.current_x += char_width;
        }

        Ok(())
    }

    /// Render a beat with cursor awareness (DEPRECATED - use render_content_line_chars)
    fn render_beat_with_cursor(
        &mut self,
        svg: &mut String,
        beat: &crate::parse::model::Beat,
        notation_type: &str,
        start_char_position: usize
    ) -> Result<(f32, usize), String> {
        let mut chars_consumed = 0;
        let mut element_positions = Vec::new(); // Will store (x_pos, width)

        // Store Y coordinate at the start of this beat - important for bbox tracking
        let beat_y = self.current_y;

        // Store initial x position to calculate beat width later
        let beat_start_x = self.current_x;

        // Start a group for the beat with its UUID
        writeln!(svg, r#"    <g id="beat-{}" data-beat-id="{}" data-element-type="beat">"#,
                self.element_id_counter, beat.id).unwrap();
        self.element_id_counter += 1;

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

                    let note_width = self.get_char_width(&note_value);
                    element_positions.push((self.current_x, note_width));

                    // For notes, we need to store cursor positions correctly:
                    // - Cursor position N is BEFORE character N
                    // - Cursor position N+1 is AFTER character N

                    // The cursor position before this note (if not already stored)
                    let note_start_pos = start_char_position + chars_consumed;
                    if !self.char_positions.contains_key(&note_start_pos) {
                        self.char_positions.insert(note_start_pos, (self.current_x, beat_y));
                    }

                    // Store cursor position after this note
                    let pos_after_note = note_start_pos + note_value.len();
                    self.char_positions.insert(pos_after_note, (self.current_x + note_width, beat_y));

                    // Track element coordinates
                    self.element_coordinates.push(ElementCoordinate {
                        char_start: start_char_position + chars_consumed,
                        char_end: start_char_position + chars_consumed + note_value.len(),
                        x: self.current_x,
                        y: self.current_y,
                        width: note_width,
                        height: self.config.font_size,
                        element_type: "note".to_string(),
                    });

                    // Render note with unique ID and data attributes for coordinate tracking
                    writeln!(svg, r#"    <text id="el-{}" x="{:.1}" y="{}" class="canvas-note" data-char-start="{}" data-char-end="{}" data-element-type="note" data-width="{:.1}" data-height="{:.1}" data-note-id="{}">{}</text>"#,
                            self.element_id_counter,  // Use counter for unique ID
                            self.current_x, self.current_y,
                            start_char_position + chars_consumed,
                            start_char_position + chars_consumed + note_value.len(),
                            note_width, self.config.font_size,
                            note.id, // Include the note's UUID
                            note_value).unwrap();
                    self.element_id_counter += 1;

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

                    self.current_x += note_width;
                    chars_consumed += note_value.len();
                }
                crate::parse::model::BeatElement::Dash(_) => {
                    let dash_width = self.get_char_width("-");
                    element_positions.push((self.current_x, dash_width));

                    // Store cursor position before the dash (if not already stored)
                    let dash_pos = start_char_position + chars_consumed;
                    if !self.char_positions.contains_key(&dash_pos) {
                        self.char_positions.insert(dash_pos, (self.current_x, beat_y));
                    }

                    // Store cursor position after the dash
                    self.char_positions.insert(dash_pos + 1, (self.current_x + dash_width, beat_y));

                    // Track element coordinates
                    self.element_coordinates.push(ElementCoordinate {
                        char_start: dash_pos,
                        char_end: dash_pos + 1,
                        x: self.current_x,
                        y: self.current_y,
                        width: dash_width,
                        height: self.config.font_size,
                        element_type: "dash".to_string(),
                    });

                    writeln!(svg, r#"    <text id="el-{}" x="{:.1}" y="{}" class="canvas-note" data-char-start="{}" data-char-end="{}" data-element-type="dash" data-width="{:.1}" data-height="{:.1}">-</text>"#,
                            self.element_id_counter,
                            self.current_x, self.current_y,
                            dash_pos, dash_pos + 1,
                            dash_width, self.config.font_size).unwrap();
                    self.element_id_counter += 1;

                    self.current_x += dash_width;
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

        // Add an invisible clickable rectangle covering the entire beat
        let beat_width = self.current_x - beat_start_x;
        if beat_width > 0.0 {
            writeln!(svg, r#"      <rect x="{:.1}" y="{}" width="{:.1}" height="{:.1}" fill="transparent" data-char-start="{}" data-char-end="{}" data-beat-id="{}" data-element-type="beat"/>"#,
                    beat_start_x, beat_y - self.config.font_size,
                    beat_width, self.config.font_size + 10.0,
                    start_char_position,
                    start_char_position + chars_consumed,
                    beat.id).unwrap();
        }

        // Close the beat group
        writeln!(svg, "    </g>").unwrap();

        // No beat spacing - elements flow naturally

        Ok((self.current_x, chars_consumed))
    }

    /// Render barline
    /// Render barline using SMuFL music font symbols
    fn render_barline(&mut self, svg: &mut String, barline: &crate::parse::model::Barline, char_position: usize) -> Result<f32, String> {
        let symbol = match barline {
            crate::parse::model::Barline::Single(_) => "|",
            crate::parse::model::Barline::Double(_) => "||",
            crate::parse::model::Barline::Final(_) => "||",
            crate::parse::model::Barline::RepeatStart(_) => "|:",
            crate::parse::model::Barline::RepeatEnd(_) => ":|",
            crate::parse::model::Barline::RepeatBoth(_) => ":|:",
        };

        let width_adjustment = self.get_char_width(symbol);

        // Track element coordinates
        self.element_coordinates.push(ElementCoordinate {
            char_start: char_position,
            char_end: char_position + symbol.len(),
            x: self.current_x,
            y: self.current_y,
            width: width_adjustment,
            height: self.config.font_size,
            element_type: "barline".to_string(),
        });

        writeln!(svg, r#"    <text id="el-{}" x="{:.1}" y="{}" class="canvas-barline" data-char-start="{}" data-char-end="{}" data-element-type="barline" data-width="{:.1}" data-height="{:.1}">{}</text>"#,
                self.element_id_counter,
                self.current_x, self.current_y,
                char_position, char_position + symbol.len(),
                width_adjustment, self.config.font_size,
                symbol).unwrap();
        self.element_id_counter += 1;

        self.current_x += width_adjustment + 10.0;
        Ok(width_adjustment)
    }

    /// Render cursor at specific position (DISABLED)
    fn render_cursor_at_position(&self, svg: &mut String, x: f32, y: f32) {
        // Line cursor disabled - no cursor rendering
        // writeln!(svg, r#"    <line x1="{:.1}" y1="{}" x2="{:.1}" y2="{}" class="canvas-cursor" id="svg-cursor"/>"#,
        //         x, y - 20.0, x, y + 5.0).unwrap();
    }

    /// Render arc connecting beat elements
    fn render_beat_arc(&self, svg: &mut String, positions: &[(f32, f32)]) {
        if positions.len() < 2 {
            return;
        }

        let start_x = positions[0].0;
        let last_element = positions[positions.len() - 1];
        let end_x = last_element.0 + last_element.1;

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

        // Cursor rendering removed - handled by client-side JavaScript

        Ok(())
    }

    /// Render selection highlighting for the configured range
    fn render_selection_highlighting(&self, svg: &mut String) {
        let start_pos = self.config.selection_start;
        let end_pos = self.config.selection_end;

        // Get coordinates for start and end positions
        let (start_x, start_y) = self.char_positions.get(&start_pos).copied().unwrap_or((0.0, 0.0));
        let (end_x, end_y) = self.char_positions.get(&end_pos).copied().unwrap_or((self.current_x, self.current_y));

        if end_x > start_x && start_y == end_y {
            // Simple case: selection on same line
            self.render_selection_at_range(svg, start_x, end_x, start_y);
        } else if end_y != start_y {
            // TODO: Multi-line selection would require more complex logic
            // For now, just render on the start line
            self.render_selection_at_range(svg, start_x, self.config.width, start_y);
        }
    }

    /// Render selection highlight rectangle
    fn render_selection_at_range(&self, svg: &mut String, start_x: f32, end_x: f32, y: f32) {
        let selection_height = self.config.font_size + 10.0;
        let selection_y = y - self.config.font_size + 5.0;
        writeln!(svg, r#"    <rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" class="canvas-selection"/>"#,
                start_x - 2.0, selection_y, end_x - start_x + 4.0, selection_height).unwrap();
    }

    /// Get approximate character width
    fn get_char_width(&self, text: &str) -> f32 {
        // Approximate monospace character width based on font size
        text.chars().count() as f32 * (self.config.font_size * 0.6)
    }

    /// Render lyrics line below the current content line
    fn render_lyrics_line(&mut self, svg: &mut String, lyrics_line: &crate::parse::model::LyricsLine) -> Result<(), String> {
        // Position lyrics below the current content line
        let lyrics_y = self.current_y + 35.0;
        let lyrics_x = 0.0; // Start from the beginning of the line

        // Just use the raw text value from the line
        if let Some(ref text) = lyrics_line.value {
            writeln!(svg, r#"    <text x="{:.1}" y="{}" class="canvas-lyrics">{}</text>"#,
                    lyrics_x, lyrics_y, text).unwrap();
        }

        Ok(())
    }

    /// Render lower line token by token - syllable separators as blank spaces
    fn render_lower_line(&mut self, svg: &mut String, lower_line: &crate::parse::model::LowerLine, char_position: &mut usize) -> Result<(), String> {
        let lower_y = self.current_y + 35.0; // Position below content line

        // Process each element in the lower line
        for element in &lower_line.elements {
            match element {
                crate::parse::model::LowerElement::Syllable { value, .. } => {
                    // Render syllables
                    if let Some(ref val) = value {
                        for (idx, _) in val.char_indices() {
                            let char_pos = *char_position + idx;
                            let char_x = self.current_x + (idx as f32 * self.get_char_width(" "));
                            self.char_positions.insert(char_pos, (char_x, self.current_y));
                        }

                        // Render the syllable text
                        writeln!(svg, r#"    <text x="{:.1}" y="{}" class="canvas-lyrics">{}</text>"#,
                                self.current_x, lower_y, val).unwrap();

                        self.current_x += self.get_char_width(val);
                        *char_position += val.len();
                    }
                }
                crate::parse::model::LowerElement::LowerOctaveMarker { value, .. } => {
                    // Render octave markers as blank spaces (they're already shown with notes)
                    if let Some(ref val) = value {
                        for (idx, _) in val.char_indices() {
                            let char_pos = *char_position + idx;
                            let char_x = self.current_x + (idx as f32 * self.get_char_width(" "));
                            self.char_positions.insert(char_pos, (char_x, self.current_y));
                        }
                        // Just advance position, don't render anything (blank space)
                        self.current_x += self.get_char_width(val);
                        *char_position += val.len();
                    }
                }
                crate::parse::model::LowerElement::BeatGroupIndicator { value, .. } => {
                    // Render beat group indicators as blank spaces (they're visual indicators)
                    if let Some(ref val) = value {
                        for (idx, _) in val.char_indices() {
                            let char_pos = *char_position + idx;
                            let char_x = self.current_x + (idx as f32 * self.get_char_width(" "));
                            self.char_positions.insert(char_pos, (char_x, self.current_y));
                        }
                        // Just advance position, don't render anything (blank space)
                        self.current_x += self.get_char_width(val);
                        *char_position += val.len();
                    }
                }
                crate::parse::model::LowerElement::Space { value, .. } => {
                    // Render spaces
                    if let Some(ref val) = value {
                        for (idx, _) in val.char_indices() {
                            let char_pos = *char_position + idx;
                            let char_x = self.current_x + (idx as f32 * self.get_char_width(" "));
                            self.char_positions.insert(char_pos, (char_x, self.current_y));
                        }
                        self.current_x += self.get_char_width(val);
                        *char_position += val.len();
                    }
                }
                _ => {
                    // For other lower elements, render them
                    if let Some(val) = self.get_lower_element_value(element) {
                        for (idx, _) in val.char_indices() {
                            let char_pos = *char_position + idx;
                            let char_x = self.current_x + (idx as f32 * self.get_char_width(" "));
                            self.char_positions.insert(char_pos, (char_x, self.current_y));
                        }

                        // Render the actual element
                        writeln!(svg, r#"    <text x="{:.1}" y="{}" class="canvas-lyrics">{}</text>"#,
                                self.current_x, lower_y, val).unwrap();

                        self.current_x += self.get_char_width(&val);
                        *char_position += val.len();
                    }
                }
            }
        }

        Ok(())
    }

    /// Helper to get value from lower element
    fn get_lower_element_value(&self, element: &crate::parse::model::LowerElement) -> Option<String> {
        match element {
            crate::parse::model::LowerElement::Syllable { value, .. } |
            crate::parse::model::LowerElement::LowerOctaveMarker { value, .. } |
            crate::parse::model::LowerElement::BeatGroupIndicator { value, .. } |
            crate::parse::model::LowerElement::Space { value, .. } |
            crate::parse::model::LowerElement::Unknown { value, .. } |
            crate::parse::model::LowerElement::Newline { value, .. } |
            crate::parse::model::LowerElement::EndOfInput { value, .. } => value.clone(),
        }
    }

    /// Render text line as regular text
    fn render_text_line(&mut self, svg: &mut String, text_line: &crate::parse::model::TextLine, char_position: &mut usize) -> Result<(), String> {
        // Track character positions for the text line
        if let Some(ref text_value) = text_line.value {
            for (char_idx, _) in text_value.char_indices() {
                let char_pos = *char_position + char_idx;
                let char_x = self.current_x + (char_idx as f32 * self.get_char_width(" "));
                self.char_positions.insert(char_pos, (char_x, self.current_y));
            }

            // Render the text
            writeln!(svg, r#"    <text x="{:.1}" y="{}" class="canvas-text">{}</text>"#,
                    self.current_x, self.current_y, text_value).unwrap();

            *char_position += text_value.len();
        }

        Ok(())
    }

    /// Render upper line token by token - octave markers as blank spaces
    fn render_upper_line(&mut self, svg: &mut String, upper_line: &crate::parse::model::UpperLine, char_position: &mut usize) -> Result<(), String> {
        let upper_y = self.current_y - 15.0; // Position above content line

        // Process each element in the upper line
        for element in &upper_line.elements {
            match element {
                crate::parse::model::UpperElement::UpperOctaveMarker { value, .. } => {
                    // Render octave markers as blank spaces (they're already shown with notes)
                    if let Some(ref val) = value {
                        for (idx, _) in val.char_indices() {
                            let char_pos = *char_position + idx;
                            let char_x = self.current_x + (idx as f32 * self.get_char_width(" "));
                            self.char_positions.insert(char_pos, (char_x, self.current_y));
                        }
                        // Just advance position, don't render anything (blank space)
                        self.current_x += self.get_char_width(val);
                        *char_position += val.len();
                    }
                }
                crate::parse::model::UpperElement::Space { value, .. } => {
                    // Render spaces
                    if let Some(ref val) = value {
                        for (idx, _) in val.char_indices() {
                            let char_pos = *char_position + idx;
                            let char_x = self.current_x + (idx as f32 * self.get_char_width(" "));
                            self.char_positions.insert(char_pos, (char_x, self.current_y));
                        }
                        self.current_x += self.get_char_width(val);
                        *char_position += val.len();
                    }
                }
                _ => {
                    // For other upper elements (slur indicators, ornaments, etc.), render them
                    if let Some(val) = self.get_upper_element_value(element) {
                        for (idx, _) in val.char_indices() {
                            let char_pos = *char_position + idx;
                            let char_x = self.current_x + (idx as f32 * self.get_char_width(" "));
                            self.char_positions.insert(char_pos, (char_x, self.current_y));
                        }

                        // Render the actual element
                        writeln!(svg, r#"    <text x="{:.1}" y="{}" class="canvas-octave-upper">{}</text>"#,
                                self.current_x, upper_y, val).unwrap();

                        self.current_x += self.get_char_width(&val);
                        *char_position += val.len();
                    }
                }
            }
        }

        Ok(())
    }


    /// Helper to get value from upper element
    fn get_upper_element_value(&self, element: &crate::parse::model::UpperElement) -> Option<String> {
        match element {
            crate::parse::model::UpperElement::UpperOctaveMarker { value, .. } |
            crate::parse::model::UpperElement::SlurIndicator { value, .. } |
            crate::parse::model::UpperElement::UpperHashes { value, .. } |
            crate::parse::model::UpperElement::Ornament { value, .. } |
            crate::parse::model::UpperElement::Chord { value, .. } |
            crate::parse::model::UpperElement::Mordent { value, .. } |
            crate::parse::model::UpperElement::Space { value, .. } |
            crate::parse::model::UpperElement::Unknown { value, .. } |
            crate::parse::model::UpperElement::Newline { value, .. } => value.clone(),
        }
    }
}

impl Default for EditorSvgRenderer {
    fn default() -> Self {
        Self::new(EditorSvgConfig::default())
    }
}

/// Convenience function to render document for canvas display
pub fn render_editor_svg(
    document: &Document,
    cursor_position: Option<usize>,
    selection_start: Option<usize>,
    selection_end: Option<usize>
) -> Result<String, String> {
    let mut config = EditorSvgConfig::default();

    // Cursor rendering is handled by client-side JavaScript

    if let (Some(start), Some(end)) = (selection_start, selection_end) {
        config.show_selection = true;
        config.selection_start = start;
        config.selection_end = end;
    }

    let mut renderer = EditorSvgRenderer::new(config);
    // Use the document's value field as the input text
    let empty = String::new();
    let input_text = document.value.as_ref().unwrap_or(&empty);
    renderer.render(document, "", input_text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::recursive_descent::parse_document;

    #[test]
    fn test_editor_svg_title_only() {
        let input = "11222x";
        let document = parse_document(input).expect("Should parse successfully");

        // Should have empty elements (title-only document)
        assert_eq!(document.elements.len(), 0);
        assert_eq!(document.title, Some("11222x".to_string()));

        let svg = render_editor_svg(&document, "number", input, None, None, None).expect("Should render SVG");

        // Should contain SVG structure
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        // Should contain title as placeholder text
        assert!(svg.contains("11222x"));
    }
}
