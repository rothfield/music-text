use crate::renderers::svg::{Document, Element, FontStrategy, Ornament, SMuFLMapper, SymbolType};
use crate::models::pitch_systems;
use crate::models::pitch::Notation;
use std::fmt::Write;

/// SVG renderer configuration
pub struct SvgRendererConfig {
    pub width: f32,
    pub height: f32,
    pub font_strategy: FontStrategy,
}

impl Default for SvgRendererConfig {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 600.0,
            font_strategy: FontStrategy::default(),
        }
    }
}

/// Main SVG renderer implementing doremi-script typography
pub struct SvgRenderer {
    config: SvgRendererConfig,
    font_mapper: SMuFLMapper,
    current_x: f32,
    current_y: f32,
}

impl SvgRenderer {
    pub fn new(config: SvgRendererConfig) -> Self {
        Self {
            config,
            font_mapper: SMuFLMapper::new(),
            current_x: 20.0,  // Starting margin
            current_y: 80.0,  // Y position for music content
        }
    }

    /// Render complete document to SVG
    pub fn render(&mut self, doc: &Document) -> Result<String, String> {
        doc.validate()?;

        let mut svg = String::new();

        // SVG header
        writeln!(svg, r#"<?xml version="1.0" encoding="UTF-8"?>"#).unwrap();
        writeln!(svg, r#"<svg xmlns="http://www.w3.org/2000/svg""#).unwrap();
        writeln!(svg, r#"     width="{}" height="{}""#, self.config.width, self.config.height).unwrap();
        writeln!(svg, r#"     viewBox="0 0 {} {}">"#, self.config.width, self.config.height).unwrap();

        // Embedded CSS
        svg.push_str(&self.generate_css(doc.font_size));

        // Title if present
        if let Some(title) = &doc.title {
            writeln!(svg, r#"  <text x="{}" y="30" class="title" text-anchor="middle">{}</text>"#,
                    self.config.width / 2.0, title).unwrap();
        }

        // Composer if present
        if let Some(composer) = &doc.composer {
            writeln!(svg, r#"  <text x="{}" y="50" class="composer" text-anchor="middle">{}</text>"#,
                    self.config.width / 2.0, composer).unwrap();
        }

        // Music content
        writeln!(svg, r#"  <g class="composition" transform="translate({}, {})">"#,
                self.current_x, self.current_y).unwrap();

        // Reset position for content
        self.current_x = 0.0;
        self.current_y = 0.0;

        // Render elements
        for element in &doc.elements {
            svg.push_str(&self.render_element(element, &doc.notation_type)?);
        }

        writeln!(svg, "  </g>").unwrap();
        writeln!(svg, "</svg>").unwrap();

        Ok(svg)
    }

    /// Generate CSS by loading from external file (with fallback)
    fn generate_css(&self, base_font_size: f32) -> String {
        let mut css = String::new();

        writeln!(css, "  <!-- CSS loaded from external file for hot reloading -->").unwrap();
        writeln!(css, "  <style>").unwrap();
        writeln!(css, "  <![CDATA[").unwrap();

        // SMuFL Font Loading
        css.push_str(&self.config.font_strategy.generate_font_face_css());

        // Load CSS from external file for development
        let external_css = std::fs::read_to_string("assets/svg-styles.css")
            .unwrap_or_else(|_| {
                eprintln!("Warning: Could not load assets/svg-styles.css, using fallback CSS");
                self.generate_fallback_css(base_font_size)
            });

        css.push_str(&external_css);

        // Add dynamic font sizes based on base_font_size
        let note_size = base_font_size * 1.6;  // 22.4px at 14px base
        let ornament_size = note_size * 0.9;   // 0.9 * base size
        let octave_size = note_size * 0.6;     // 0.6 * base size
        let lyric_size = note_size * 1.01;     // 1.01 * base size

        // Override font sizes from external CSS with calculated values
        write!(css, r#"
    /* Dynamic font sizes based on base font size */
    .note {{ font-size: {}px; }}
    .smufl-symbol {{ font-family: {}; font-size: {}px; }}
    .unicode-symbol {{ font-size: {}px; }}
    .upper-octave {{ font-size: {}px; }}
    .lower-octave {{ font-size: {}px; }}
    .lyric {{ font-size: {}px; }}
    .ornament {{ font-family: {}; font-size: {}px; }}
    .grace-note {{ font-size: {}px; }}
    .barline {{ letter-spacing: {}px; }}
    .accidental {{ font-family: {}; font-size: {}px; }}
"#,
            note_size,
            self.config.font_strategy.get_music_font_family(),
            note_size,
            note_size,
            octave_size,
            octave_size,
            lyric_size,
            self.config.font_strategy.get_music_font_family(),
            ornament_size,
            ornament_size * 0.7, // Grace notes smaller
            -base_font_size * 0.2, // -0.2em letter spacing
            self.config.font_strategy.get_music_font_family(),
            note_size
        ).unwrap();

        writeln!(css, "  ]]>").unwrap();
        writeln!(css, "  </style>").unwrap();

        css
    }

    /// Generate fallback CSS when external file is not available
    fn generate_fallback_css(&self, base_font_size: f32) -> String {
        let note_size = base_font_size * 1.6;
        let octave_size = note_size * 0.6;
        let lyric_size = note_size * 1.01;
        let ornament_size = note_size * 0.9;

        format!(r#"
/* Fallback CSS when external file is not available */
.note {{ font-family: sans-serif; font-size: {}px; fill: black; }}
.upper-octave {{ font-family: sans-serif; font-size: {}px; fill: black; }}
.lower-octave {{ font-family: sans-serif; font-size: {}px; fill: black; transform: translateY(8px) !important; }}
.lyric {{ font-family: serif; font-size: {}px; fill: black; }}
.ornament {{ font-size: {}px; fill: black; }}
.title {{ font-family: sans-serif; font-size: 24px; font-weight: bold; fill: black; }}
.composer {{ font-family: sans-serif; font-size: 16px; fill: black; }}
.barline {{ letter-spacing: {}px; }}
"#, note_size, octave_size, octave_size, lyric_size, ornament_size, -base_font_size * 0.2)
    }

    /// Render individual element
    fn render_element(&mut self, element: &Element, notation_type: &str) -> Result<String, String> {
        match element {
            Element::Pitch { value, octave, accidental, ornaments, lyrics } => {
                self.render_pitch(value, *octave, accidental.as_deref(), ornaments, lyrics, notation_type)
            }
            Element::Dash { is_rest } => {
                self.render_dash(*is_rest)
            }
            Element::Barline { style } => {
                self.render_barline(style)
            }
        }
    }

    /// Render pitch element with all decorations
    fn render_pitch(&mut self, value: &str, octave: i8, accidental: Option<&str>,
                   ornaments: &[Ornament], lyrics: &[String], _notation_type: &str) -> Result<String, String> {
        let mut svg = String::new();
        let base_y = self.current_y;

        // Main note with semantic class for descenders
        let note_class = if value == "g" || value == "p" || value == "q" || value == "y" || value == "j" {
            "note has_descender"
        } else {
            "note"
        };
        writeln!(svg, r#"    <text x="{}" y="{}" class="{}">{}</text>"#,
                self.current_x, base_y, note_class, value).unwrap();

        // Octave markers (positioned via CSS with data attributes)
        if octave > 0 {
            let octave_marker = "•".repeat(octave as usize);
            writeln!(svg, r#"    <text x="{}" y="{}" class="upper-octave" data-note="{}">{}</text>"#,
                    self.current_x + 3.2, base_y - 6.4, value, octave_marker).unwrap();
        } else if octave < 0 {
            let octave_marker = "•".repeat((-octave) as usize);
            writeln!(svg, r#"    <text x="{}" y="{}" class="lower-octave" data-note="{}">{}</text>"#,
                    self.current_x + 3.2, base_y + 28.8, value, octave_marker).unwrap();
        }

        // Accidentals
        if let Some(acc) = accidental {
            let symbol_type = match acc {
                "sharp" => SymbolType::Sharp,
                "flat" => SymbolType::Flat,
                _ => return Err(format!("Unknown accidental: {}", acc)),
            };

            let symbol = self.font_mapper.get_symbol(symbol_type, self.config.font_strategy.use_smufl);
            let class = if self.config.font_strategy.use_smufl { "smufl-symbol accidental" } else { "unicode-symbol accidental" };

            writeln!(svg, r#"    <text x="{}" y="{}" class="{}">{}</text>"#,
                    self.current_x + 16.0, base_y + 11.2, class, symbol).unwrap();
        }

        // Ornaments
        let mut ornament_offset_x = 0.0;
        for ornament in ornaments {
            svg.push_str(&self.render_ornament(ornament, self.current_x + ornament_offset_x, base_y)?);
            ornament_offset_x += 10.0; // Space between multiple ornaments
        }

        // Lyrics (doremi-script positioning)
        for (i, lyric) in lyrics.iter().enumerate() {
            writeln!(svg, r#"    <text x="{}" y="{}" class="lyric">{}</text>"#,
                    self.current_x + 0.48 + (i as f32 * 15.0), base_y + 16.0, lyric).unwrap();
        }

        // Advance position (doremi-script beat spacing)
        self.current_x += 30.0; // Approximate note width with spacing

        Ok(svg)
    }

    /// Render ornament
    fn render_ornament(&self, ornament: &Ornament, x: f32, base_y: f32) -> Result<String, String> {
        let mut svg = String::new();

        match ornament {
            Ornament::SymbolicOrnament { symbol } => {
                let symbol_char = match symbol.as_str() {
                    "mordent" => {
                        let symbol_type = SymbolType::Mordent;
                        self.font_mapper.get_symbol(symbol_type, self.config.font_strategy.use_smufl)
                    },
                    "trill" => {
                        let symbol_type = SymbolType::Trill;
                        self.font_mapper.get_symbol(symbol_type, self.config.font_strategy.use_smufl)
                    },
                    _ => "?".to_string(),
                };

                let class = if self.config.font_strategy.use_smufl { "smufl-symbol ornament" } else { "unicode-symbol ornament" };
                writeln!(svg, r#"    <text x="{}" y="{}" class="{}">{}</text>"#,
                        x, base_y - 15.68, class, symbol_char).unwrap();
            },
            Ornament::BeforeGraceNotes { notes } => {
                let mut grace_x = x - 15.0; // Position before main note
                for note in notes {
                    writeln!(svg, r#"    <text x="{}" y="{}" class="grace-note">{}</text>"#,
                            grace_x, base_y - 5.0, note.value).unwrap();
                    grace_x += 8.0;
                }
            },
            Ornament::OnTopGraceNotes { notes } => {
                let mut grace_x = x;
                for note in notes {
                    writeln!(svg, r#"    <text x="{}" y="{}" class="grace-note">{}</text>"#,
                            grace_x, base_y - 20.0, note.value).unwrap();
                    grace_x += 6.0;
                }
            },
            Ornament::AfterGraceNotes { notes } => {
                let mut grace_x = x + 20.8; // doremi-script placement-after margin
                for note in notes {
                    writeln!(svg, r#"    <text x="{}" y="{}" class="grace-note">{}</text>"#,
                            grace_x, base_y - 5.0, note.value).unwrap();
                    grace_x += 8.0;
                }
            },
        }

        Ok(svg)
    }

    /// Render dash element
    fn render_dash(&mut self, is_rest: bool) -> Result<String, String> {
        let symbol = if is_rest { "⌐" } else { "–" };
        let result = format!(r#"    <text x="{}" y="{}" class="note">{}</text>"#,
                           self.current_x, self.current_y, symbol);

        // Advance position (doremi-script dash spacing)
        self.current_x += 15.0;

        Ok(result + "\n")
    }

    /// Render barline element
    fn render_barline(&mut self, style: &str) -> Result<String, String> {
        let symbol_type = match style {
            "single" => SymbolType::BarlineSingle,
            "double" => SymbolType::BarlineDouble,
            "repeat_start" => SymbolType::RepeatStart,
            "repeat_end" => SymbolType::RepeatEnd,
            _ => return Err(format!("Unknown barline style: {}", style)),
        };

        let symbol = self.font_mapper.get_symbol(symbol_type, self.config.font_strategy.use_smufl);
        let class = format!("{} barline", self.font_mapper.get_symbol_class(self.config.font_strategy.use_smufl));

        let result = format!(r#"    <text x="{}" y="{}" class="{}">{}</text>"#,
                           self.current_x, self.current_y, class, symbol);

        // Advance position (doremi-script barline spacing)
        self.current_x += 20.0;

        Ok(result + "\n")
    }
}

impl Default for SvgRenderer {
    fn default() -> Self {
        Self::new(SvgRendererConfig::default())
    }
}

/// Render music-text Document directly to SVG (POC implementation)
/// Walk the document tree directly and render SVG - no converters, no adapters
pub fn render_document_tree_to_svg(document: &crate::parse::Document, notation_type: &str) -> String {
    use crate::models::pitch_systems;
    use crate::models::pitch::Notation;

    let mut svg = String::new();

    // Format title and author like "Fugue Bach"
    let title_text = match (&document.title, &document.author) {
        (Some(title), Some(author)) => format!("{} {}", title, author),
        (Some(title), None) => {
            // If title contains multiple spaces, it might be "Title    Author" format
            // Try to split on 4+ consecutive spaces
            if let Some(gap_pos) = find_large_gap(title, 4) {
                let title_part = title[..gap_pos].trim();
                let author_part = title[gap_pos..].trim();
                if !title_part.is_empty() && !author_part.is_empty() {
                    format!("{} {}", title_part, author_part)
                } else {
                    title.clone()
                }
            } else {
                title.clone()
            }
        },
        (None, Some(author)) => author.clone(),
        (None, None) => "Music Text".to_string(),
    };

    // Load external CSS for hot reloading
    let external_css = std::fs::read_to_string("assets/svg-styles.css")
        .unwrap_or_else(|e| {
            eprintln!("Warning: Could not load assets/svg-styles.css: {}", e);
            // Fallback CSS
            r#"
.lower-octave { transform: translateY(8px) !important; }
.note { font-family: sans-serif; fill: black; }
.dash { font-family: sans-serif; fill: black; }
.barline { font-family: sans-serif; fill: black; }
"#.to_string()
        });

    // SVG header with loaded CSS
    svg.push_str(&format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="800" height="600" viewBox="0 0 800 600">
  <style>
    <![CDATA[
    {}

    /* Doremi-script CSS constants and positioning rules */
    :root {{
      --base-font-size: 22.4px;           /* 1.6em at 14px base */
      --octave-font-size: 13.44px;        /* 0.6 * base size */
      --ornament-font-size: 20.16px;      /* 0.9 * base size */
      --lyric-font-size: 22.624px;        /* 1.01 * base size */
      --grace-note-font-size: 14.11px;    /* 0.63 * base size */

      --ch-width: 13.44px;                /* 0.6 * base for monospace */
      --note-gap: 3.36px;                 /* 0.25 * ch-width */
      --note-spacing: 16.8px;             /* ch-width + note-gap */

      --octave-x-offset: 3.2px;           /* 0.2em from doremi.css - nudged left */
      --upper-octave-y-offset: -6.4px;    /* top: 0.4em from doremi.css - sits in bbox */
      --lower-octave-y-offset: 8.0px;     /* Above lower loops, below baseline */

      --barline-letter-spacing: -2.8px;   /* -0.2em */
      --barline-margin: 3.2px;            /* 0.2em - from doremi.css */

      --beat-margin-right: 16.0px;        /* 1em at 16px - doremi-script beat spacing */
      --dash-margin: 1.44px;              /* 0.09em each side - from doremi.css */
    }}

    .note {{
      font-family: sans-serif;
      font-size: var(--base-font-size);
      fill: black;
    }}

    .upper-octave {{
      font-family: sans-serif;
      font-size: var(--octave-font-size);
      fill: black;
    }}

    .lower-octave {{
      font-family: sans-serif;
      font-size: var(--octave-font-size);
      fill: black;
    }}

    .barline {{
      font-family: sans-serif;
      font-size: var(--base-font-size);
      fill: black;
      letter-spacing: var(--barline-letter-spacing);
    }}

    .dash {{
      font-family: sans-serif;
      font-size: var(--base-font-size);
      fill: black;
      /* Dashes get same spacing as notes in doremi-script */
    }}

    .lower-arc {{
      stroke: black;
      stroke-width: 1.5;
      fill: none;
    }}

    .title {{
      font-family: sans-serif;
      font-size: 24px;
      font-weight: bold;
      fill: black;
    }}

    .ornament {{
      font-family: sans-serif;
      font-size: var(--ornament-font-size);
      fill: black;
    }}

    .lyric {{
      font-family: serif;
      font-size: var(--lyric-font-size);
      fill: black;
    }}

    .grace-note {{
      font-family: sans-serif;
      font-size: var(--grace-note-font-size);
      fill: black;
    }}
    ]]>
  </style>
  <text x="400" y="30" class="title" text-anchor="middle">{}</text>
  <g class="composition" transform="translate(20, 80)">
"#, external_css, title_text));

    let mut x_pos = 0.0;
    let y_pos = 0.0;

    // Walk document elements directly
    for element in &document.elements {
        match element {
            crate::parse::model::DocumentElement::Stave(stave) => {
                // Walk stave lines
                for line in &stave.lines {
                    match line {
                        crate::parse::model::StaveLine::ContentLine(content_line) => {
                            // Walk content elements
                            for content_element in &content_line.elements {
                                match content_element {
                                    crate::parse::model::ContentElement::Beat(beat) => {
                                        x_pos = render_beat_with_arcs(&mut svg, beat, notation_type, x_pos, y_pos);
                                    }
                                    crate::parse::model::ContentElement::Barline(barline) => {
                                        let barline_symbol = match barline.barline_type {
                                            crate::rhythm::converters::BarlineType::Single => "𝄀", // Unicode single barline
                                            crate::rhythm::converters::BarlineType::Double => "‖", // Unicode double barline
                                            crate::rhythm::converters::BarlineType::Final => "𝄁", // Unicode final barline
                                            crate::rhythm::converters::BarlineType::RepeatStart => "𝄆", // Unicode repeat start
                                            crate::rhythm::converters::BarlineType::RepeatEnd => "𝄇", // Unicode repeat end
                                            crate::rhythm::converters::BarlineType::RepeatBoth => "𝄆𝄇", // Unicode repeat both
                                        };
                                        svg.push_str(&format!(r#"    <text x="{}" y="{}" class="barline">{}</text>"#, x_pos, y_pos, barline_symbol));
                                        svg.push_str("\n");
                                        x_pos += 3.2; // BARLINE_MARGIN from doremi.css
                                    }
                                    _ => {} // Skip whitespace etc
                                }
                            }
                        }
                        crate::parse::model::StaveLine::Upper(upper_line) => {
                            // Render upper line elements (octave markers, ornaments, etc.)
                            for upper_element in &upper_line.elements {
                                match upper_element {
                                    crate::parse::model::UpperElement::UpperOctaveMarker { marker, source: _ } => {
                                        svg.push_str(&format!(r#"    <text x="{}" y="{}" class="octave-marker" font-size="14">{}</text>"#, x_pos, y_pos - 15.0, marker));
                                        svg.push_str("\n");
                                        x_pos += 10.0;
                                    }
                                    _ => {} // Skip other upper elements for now
                                }
                            }
                        }
                        crate::parse::model::StaveLine::Lower(lower_line) => {
                            // Render lower line elements (lower octave markers, syllables, etc.)
                            for lower_element in &lower_line.elements {
                                match lower_element {
                                    crate::parse::model::LowerElement::LowerOctaveMarker { marker, source: _ } => {
                                        svg.push_str(&format!(r#"    <text x="{}" y="{}" class="octave-marker" font-size="14">{}</text>"#, x_pos, y_pos + 25.0, marker));
                                        svg.push_str("\n");
                                        x_pos += 10.0;
                                    }
                                    _ => {} // Skip other lower elements for now
                                }
                            }
                        }
                        _ => {} // Skip other line types
                    }
                }
            }
            _ => {} // Skip other elements
        }
    }

    svg.push_str("  </g>\n</svg>");
    svg
}

/// Render a beat with proper doremi-script spacing and lower arcs
fn render_beat_with_arcs(
    svg: &mut String,
    beat: &crate::parse::model::Beat,
    notation_type: &str,
    mut x_pos: f32,
    y_pos: f32
) -> f32 {
    let beat_start_x = x_pos;

    // Doremi-script CSS constants (must match CSS :root values)
    const BASE_FONT_SIZE: f32 = 22.4;
    const CH_WIDTH: f32 = 13.44;
    const OCTAVE_X_OFFSET: f32 = 3.2;        // 0.2em from doremi.css - nudged left
    const UPPER_OCTAVE_Y_OFFSET: f32 = -6.4;    // top: 0.4em from doremi.css - sits in bbox
    const LOWER_OCTAVE_Y_OFFSET: f32 = 8.0;     // Above lower loops, below baseline
    const BEAT_MARGIN_RIGHT: f32 = 16.0;     // 1em at 16px base

    // Count renderable elements for lower arc calculation
    let renderable_count = beat.elements.iter().filter(|element| {
        matches!(element, crate::parse::model::BeatElement::Note(_) | crate::parse::model::BeatElement::Dash(_))
    }).count();

    // Render all elements in this beat
    for beat_element in &beat.elements {
        match beat_element {
            crate::parse::model::BeatElement::Note(note) => {
                // Convert pitch code to notation using existing pitch systems
                let degree = pitch_systems::pitch_code_to_degree(note.pitch_code);
                let notation = match notation_type {
                    "sargam" => Notation::Sargam,
                    "western" => Notation::Western,
                    _ => Notation::Number,
                };
                let note_value = pitch_systems::degree_to_string(degree, notation)
                    .unwrap_or_else(|| "1".to_string());

                svg.push_str(&format!(r#"    <text x="{}" y="{}" class="note">{}</text>"#, x_pos, y_pos, note_value));
                svg.push_str("\n");

                // Render octave markers based on note.octave value (using CSS constants)
                if note.octave > 0 {
                    // Upper octave markers (dots above the note)
                    for i in 0..note.octave {
                        let marker_x = x_pos + OCTAVE_X_OFFSET;
                        let marker_y = y_pos + UPPER_OCTAVE_Y_OFFSET - (i as f32 * 5.0); // Stack vertically
                        svg.push_str(&format!(r#"    <text x="{}" y="{}" class="upper-octave">•</text>"#, marker_x, marker_y));
                        svg.push_str("\n");
                    }
                } else if note.octave < 0 {
                    // Lower octave markers (dots below the note)
                    for i in 0..(-note.octave) {
                        let marker_x = x_pos + OCTAVE_X_OFFSET;
                        let marker_y = y_pos + LOWER_OCTAVE_Y_OFFSET + (i as f32 * 5.0); // Stack vertically
                        svg.push_str(&format!(r#"    <text x="{}" y="{}" class="lower-octave">•</text>"#, marker_x, marker_y));
                        svg.push_str("\n");
                    }
                }

                x_pos += CH_WIDTH; // Notes within beat are just 1ch apart, no spacing
            }
            crate::parse::model::BeatElement::Dash(_) => {
                svg.push_str(&format!(r#"    <text x="{}" y="{}" class="dash">-</text>"#, x_pos, y_pos));
                svg.push_str("\n");
                x_pos += CH_WIDTH; // Same spacing as notes - dashes represent rests/silence
            }
            _ => {} // Skip other elements
        }
    }

    // Add lower arc if beat has multiple renderable elements (DOREMI-SCRIPT LOWER LOOPS!)
    if renderable_count > 1 {
        let n_chars = renderable_count as f32;

        // Doremi-script measurements:
        // bottom: -0.42em below text
        // width: N characters (no gaps in doremi-script) = N * 1ch
        // height: 0.75em for arc depth (padding-bottom from doremi.css)
        let arc_bottom_offset = BASE_FONT_SIZE * 0.42; // 0.42em below baseline
        let arc_height = BASE_FONT_SIZE * 0.75; // 0.75em arc depth (from doremi.css)
        let arc_width = n_chars * CH_WIDTH; // Just character widths, no gaps

        let arc_start_x = beat_start_x;
        let arc_y = y_pos + arc_bottom_offset;
        let arc_center_x = arc_start_x + arc_width / 2.0;
        let arc_control_y = arc_y + arc_height;

        // Create elliptical arc matching doremi-script border-radius: 0 0 9999px 9999px
        svg.push_str(&format!(
            r#"    <path d="M {} {} Q {} {} {} {}" class="lower-arc"/>"#,
            arc_start_x, arc_y,
            arc_center_x, arc_control_y,
            arc_start_x + arc_width, arc_y
        ));
        svg.push_str("\n");
    }

    x_pos += BEAT_MARGIN_RIGHT; // Doremi-script beat spacing (1em = 16px)
    x_pos
}

/// Find position of min_spaces or more consecutive spaces (same logic as title_line.rs)
fn find_large_gap(s: &str, min_spaces: usize) -> Option<usize> {
    let chars: Vec<char> = s.chars().collect();
    let mut space_count = 0;
    let mut gap_start = 0;

    for (i, &ch) in chars.iter().enumerate() {
        if ch == ' ' {
            if space_count == 0 {
                gap_start = i;
            }
            space_count += 1;
        } else {
            if space_count >= min_spaces {
                return Some(gap_start);
            }
            space_count = 0;
        }
    }

    // Check if we ended with a large gap
    if space_count >= min_spaces {
        Some(gap_start)
    } else {
        None
    }
}