/// Canvas WYSIWYG SVG Renderer
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

/// Canvas SVG renderer for WYSIWYG editing
pub struct CanvasSvgRenderer {
    config: CanvasSvgConfig,
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

impl CanvasSvgRenderer {
    pub fn new(config: CanvasSvgConfig) -> Self {
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

        // Track actual position in input text for proper bbox tracking
        let input_chars: Vec<char> = input_text.chars().collect();

        if has_musical_content {
            let mut elements_iter = document.elements.iter().peekable();
            while let Some(element) = elements_iter.next() {
                let is_last_element = elements_iter.peek().is_none();

                match element {
                    crate::parse::model::DocumentElement::Stave(stave) => {
                        let mut lines_iter = stave.lines.iter().peekable();
                        while let Some(line) = lines_iter.next() {
                            let is_last_line_in_stave = lines_iter.peek().is_none();
                            match line {
                                crate::parse::model::StaveLine::ContentLine(content_line) => {
                                    // Store Y coordinate at start of line for proper bbox tracking
                                    let line_y = self.current_y;

                                    for content_element in &content_line.elements {
                                    if let crate::parse::model::ContentElement::Beat(beat) = content_element {
                                        let (new_x, chars_consumed) = self.render_beat_with_cursor(
                                            &mut svg, beat, notation_type, char_position
                                        )?;
                                        self.current_x = new_x;
                                        char_position += chars_consumed;
                                    } else if let crate::parse::model::ContentElement::Barline(barline) = content_element {
                                        // Store character position for barline with line Y
                                        self.char_positions.insert(char_position, (self.current_x, line_y));

                                        let barline_width = self.render_barline(&mut svg, barline, char_position)?;

                                        // Store position AFTER the barline
                                        self.char_positions.insert(char_position + 1, (self.current_x, line_y));

                                        char_position += 1; // Barlines typically consume one character
                                    }
                                    // Handle other content elements (whitespace, unknown tokens, etc.)
                                    else if let crate::parse::model::ContentElement::Whitespace(ws) = content_element {
                                        let default_space = " ".to_string();
                                        let ws_value = ws.value.as_ref().unwrap_or(&default_space);
                                        // Track character positions for whitespace with line Y
                                        for (char_idx, _) in ws_value.char_indices() {
                                            let char_pos = char_position + char_idx;
                                            let char_x = self.current_x + (char_idx as f32 * self.get_char_width(" "));
                                            self.char_positions.insert(char_pos, (char_x, line_y));
                                        }
                                        self.current_x += self.get_char_width(ws_value);
                                        char_position += ws_value.len();
                                    }
                                    else if let crate::parse::model::ContentElement::UnknownToken(token) = content_element {
                                        // Track character positions for unknown tokens with line Y
                                        for (char_idx, _) in token.token_value.char_indices() {
                                            let char_pos = char_position + char_idx;
                                            let char_x = self.current_x + (char_idx as f32 * self.get_char_width("1"));
                                            self.char_positions.insert(char_pos, (char_x, line_y));
                                        }
                                        // Render unknown tokens with a different style
                                        writeln!(svg, r#"    <text x="{:.1}" y="{}" class="canvas-unknown">{}</text>"#,
                                                self.current_x, self.current_y, token.token_value).unwrap();
                                        self.current_x += self.get_char_width(&token.token_value);
                                        char_position += token.token_value.len();
                                    }
                                }
                                // After rendering a content line, track the newline character position
                                // The newline character itself should be at the end of the current line
                                // before we move to the next line

                                // Check if there's actually a newline in the input at this position
                                if char_position < input_chars.len() && input_chars[char_position] == '\n' {
                                    // Track the newline position at the end of the current line (same Y as line)
                                    self.char_positions.insert(char_position, (self.current_x, line_y));
                                    char_position += 1; // Account for the newline character

                                    // Now move to next line
                                    let next_line_y = self.current_y + 60.0; // Calculate next line Y before moving
                                    self.emit_newline();

                                    // Track position at start of new line (after the newline)
                                    if char_position <= input_chars.len() {
                                        self.char_positions.insert(char_position, (0.0, next_line_y));
                                    }
                                } else if !(is_last_element && is_last_line_in_stave) {
                                    // No actual newline, but more lines or elements to render - move to next line for rendering
                                    // Don't emit newline if this is the last line of the last element and there's no actual newline
                                    self.emit_newline();
                                }
                            }
                                crate::parse::model::StaveLine::Lyrics(lyrics_line) => {
                                    // Render lyrics line below content
                                    self.render_lyrics_line(&mut svg, lyrics_line)?;
                                }
                                crate::parse::model::StaveLine::Upper(upper_line) => {
                                    // Render upper line token by token - octave markers as blanks
                                    self.render_upper_line(&mut svg, upper_line, &mut char_position)?;
                                }
                                crate::parse::model::StaveLine::Lower(lower_line) => {
                                    // Render lower line token by token - syllable separators as blanks
                                    self.render_lower_line(&mut svg, lower_line, &mut char_position)?;
                                }
                                crate::parse::model::StaveLine::Text(text_line) => {
                                    // Render text lines as regular text
                                    self.render_text_line(&mut svg, text_line, &mut char_position)?;
                                }
                                crate::parse::model::StaveLine::Whitespace(whitespace_line) => {
                                    // Track character positions for whitespace lines
                                    if let Some(ref ws_value) = whitespace_line.value {
                                        for (char_idx, _) in ws_value.char_indices() {
                                            let char_pos = char_position + char_idx;
                                            self.char_positions.insert(char_pos, (self.current_x, self.current_y));
                                        }
                                        char_position += ws_value.len();
                                    }
                                }
                                crate::parse::model::StaveLine::BlankLines(blank_lines) => {
                                    // Track character positions for blank lines (newlines)
                                    if let Some(ref bl_value) = blank_lines.value {
                                        // Each character in blank lines is typically a newline
                                        for (char_idx, ch) in bl_value.char_indices() {
                                            let char_pos = char_position + char_idx;
                                            if ch == '\n' {
                                                // Position at end of current line for the newline character (same Y)
                                                let current_line_y = self.current_y;
                                                self.char_positions.insert(char_pos, (self.current_x, current_line_y));

                                                // Move to next line
                                                self.emit_newline();

                                                // Position at start of new line (after the newline)
                                                if char_pos + 1 <= input_chars.len() {
                                                    self.char_positions.insert(char_pos + 1, (0.0, self.current_y));
                                                }
                                            }
                                        }
                                        char_position += bl_value.len();
                                    }
                                }
                                _ => {} // Handle other line types as needed
                            }
                        }

                    }
                                                                                crate::parse::model::DocumentElement::BlankLines(blank_lines) => {
                        // Track character positions for blank lines outside staves, if a value exists
                        if let Some(ref bl_value) = blank_lines.value {
                            for (char_idx, _) in bl_value.char_indices() {
                                let char_pos = char_position + char_idx;
                                self.char_positions.insert(char_pos, (self.current_x, self.current_y));
                            }
                            char_position += bl_value.len();
                        }

                        // Trust the parser to give us the correct number of newlines.
                        let newline_count = blank_lines.newline_count();
                        for _ in 0..newline_count {
                            self.emit_newline();
                        }
                    }
                }
            }
        } else {
            // Render title or placeholder when no musical content
            self.render_title_or_placeholder(&mut svg, document, input_text)?;
        }

        // IMPORTANT: Ensure we track the EOF position
        // char_position might be less than input_text.len() if we haven't tracked all characters
        // This can happen if the parser doesn't capture trailing whitespace or other characters

        // Always store the current position as it represents where we've tracked up to
        self.char_positions.insert(char_position, (self.current_x, self.current_y));

        // If we haven't tracked all the way to the end, fill in the gap
        if char_position < input_text.len() {
            // There are untracked characters at the end - this shouldn't normally happen
            // but we'll track them at the current position
            for pos in char_position..=input_text.len() {
                self.char_positions.insert(pos, (self.current_x, self.current_y));
            }
        }

        // Special handling for EOF position
        // The EOF position is where the cursor goes when at the very end of the input
        if !self.char_positions.contains_key(&input_text.len()) {
            if input_text.ends_with('\n') {
                // If ending with newline, EOF is at start of next line
                self.char_positions.insert(input_text.len(), (0.0, self.current_y));
            } else {
                // Otherwise, EOF is right after the last character
                self.char_positions.insert(input_text.len(), (self.current_x, self.current_y));
            }
        }

        // Debug: Log position tracking info (disabled for production)
        // eprintln!("Position tracking: tracked {} chars, input has {} chars, stored {} positions",
        //          char_position, input_text.len(), self.char_positions.len());

        // Show selection highlighting if enabled
        if self.config.show_selection {
            self.render_selection_highlighting(&mut svg);
        }

        // Show cursor if enabled - now handles any character position
        if self.config.show_cursor {
            // Find the correct position for the cursor based on character positions
            let (cursor_x, cursor_y) = self.char_positions.get(&self.config.cursor_position)
                .copied()
                .unwrap_or((self.current_x, self.current_y));
            self.render_cursor_at_position(&mut svg, cursor_x, cursor_y);
        }

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
      stroke-width: 1;
      animation: blink 1s infinite;
    }}

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
"#, note_size, octave_size, octave_size, note_size, note_size * 0.9, note_size * 0.8, note_size * 1.5, note_size * 0.9, note_size)
    }

    /// Render a beat with cursor awareness
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

    /// Render cursor at specific position
    fn render_cursor_at_position(&self, svg: &mut String, x: f32, y: f32) {
        writeln!(svg, r#"    <line x1="{:.1}" y1="{}" x2="{:.1}" y2="{}" class="canvas-cursor" id="svg-cursor"/>"#,
                x, y - 20.0, x, y + 5.0).unwrap();
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

        // Show cursor if enabled and input matches cursor position
        if self.config.show_cursor {
            let cursor_x = text_x + self.get_char_width(display_text) / 2.0;
            self.render_cursor_at_position(svg, cursor_x, text_y);
        }

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

        let svg = render_canvas_svg(&document, "number", input, None, None, None).expect("Should render SVG");

        // Should contain SVG structure
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        // Should contain title as placeholder text
        assert!(svg.contains("11222x"));
    }
}
