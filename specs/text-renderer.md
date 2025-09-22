# Music-Text Text Renderer Specification

## Executive Summary

This specification outlines a Rust-based text renderer for music-text that provides fine typographic rendering of text-based musical notation. The renderer focuses on the visual presentation of music-text's textual elements (numbers, letters, symbols) with proper spacing, alignment, and aesthetic polish, similar to beautifully crafted manuscripts.

## Historical Context and Design Philosophy

### Evolution of Music Notation Interfaces

The approach to digital music notation has evolved significantly over the decades:

1. **WYSIWYG Era (1990s)**: Early implementations focused on direct manipulation interfaces. The author implemented a WYSIWYG music editor in Java AWT in 1995, following the prevailing paradigm that visual editing should mirror the final output. This approach, while intuitive for some users, proved complex to implement and maintain, with challenges in handling the precise spatial relationships required for musical notation.

2. **Markup/Markdown Paradigm Shift (2010s)**: By 2015, influenced by the success of Markdown and similar markup languages, the author moved to a text-based approach. This shift recognized that:
   - Text is the most flexible and version-controllable format
   - Separation of content from presentation simplifies both editing and rendering
   - Plain text editors provide powerful editing capabilities without custom implementation

3. **Modern Hybrid Approach (2020s)**: Current best practice combines:
   - **Text-based input** via code editors with syntax highlighting
   - **Read-only rendered preview** for visual feedback
   - **Clear separation** between editing and display concerns

### Learning from Doremi-Script

The doremi-script project (explored in this specification's research phase) provides valuable insights:

- **Text-first input**: Used a textarea for entering doremi notation
- **React for rendering convenience**: Leveraged React/Reagent (ClojureScript) components purely for declarative rendering, not for editing
- **Component mapping**: Each grammar element (pitch, barline, measure) mapped to a React component
- **No WYSIWYG editing**: Despite using React, editing remained text-based

This validated the text-input/rendered-output separation but used React primarily for its component model and efficient DOM updates.

### Design Decision: Drawing Commands Architecture

For music-text rendering, this specification recommends a **drawing commands** architecture over direct DOM manipulation or framework-dependent approaches. This decision is based on:

1. **Historical lessons**: WYSIWYG complexity vs. markup simplicity
2. **Technology independence**: Not tied to React, Vue, or other JS frameworks
3. **Rust-centric design**: Keeping all logic in Rust, with JS as a thin rendering layer
4. **Testability**: Drawing commands are data that can be unit tested
5. **Flexibility**: Same commands can target SVG, Canvas, or other backends

### Architectural Philosophy

The architecture embodies several key principles:

- **Read-only rendering**: No interactive editing in the rendered view
- **Text as source of truth**: All musical information stored as text
- **Separation of concerns**: Layout logic (Rust) vs. rendering (JS/SVG)
- **Progressive enhancement**: Start simple, add features incrementally
- **Framework agnostic**: Avoid dependency on specific JS frameworks

This approach aligns with modern development practices where:
- Editors handle text manipulation (with full IDE features)
- Renderers focus solely on beautiful visual output
- Version control systems work naturally with text formats

## 1. Overview

### 1.1 Goals
- Render music-text notation with high typographic quality
- Preserve the text-based nature of the notation (no traditional stems, noteheads, etc.)
- Implement primarily in Rust for performance and reliability
- Support multiple output targets (Web via WASM, native applications)

### 1.2 Non-Goals
- Traditional music notation rendering (stems, beams, etc.)
- Real-time editing (this is for display/print)
- Audio playback

## 2. Architecture

### 2.1 Core Rust Modules

```rust
// Core layout engine
mod layout {
    pub struct LayoutEngine;
    pub struct LayoutTree;
    pub struct Spacing;
    pub struct LineBreaker;
}

// Rendering abstraction
mod renderer {
    pub trait Renderer {
        fn render(&mut self, layout: &LayoutTree) -> Result<(), Error>;
        fn measure_text(&self, text: &str, font: &Font) -> TextMetrics;
    }
}

// Backend implementations
mod backends {
    pub mod canvas;  // Web Canvas via WASM
    pub mod svg;      // SVG generation
    pub mod pdf;      // PDF export
}

// Typography
mod typography {
    pub struct Font;
    pub struct Glyph;
    pub struct TextShaper;
}
```

### 2.2 Data Flow

```
music-text source
    ↓
Parser (existing)
    ↓
AST/Document
    ↓
Layout Engine (Rust)
    ↓
Layout Tree
    ↓
Renderer (Rust)
    ↓
Output (Canvas/SVG/PDF)
```

## 3. Layout Engine

### 3.1 Core Types

```rust
use crate::parse::model::{Document, Note, Barline};

#[derive(Debug, Clone)]
pub struct LayoutElement {
    pub kind: ElementKind,
    pub bounds: Rectangle,
    pub content: String,
    pub style: Style,
}

#[derive(Debug, Clone)]
pub enum ElementKind {
    Pitch { value: String, accidental: Option<Accidental> },
    Dash { is_rest: bool },
    Barline { style: BarlineStyle },
    OctaveMarker { octave: i8 },
    Slur { start: Point, end: Point, curve: CubicBezier },
    Lyric { syllable: String },
    Ornament { symbol: OrnamentType },
}

#[derive(Debug, Clone)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub struct Style {
    pub font_family: String,
    pub font_size: f32,
    pub color: Color,
    pub weight: FontWeight,
}
```

### 3.2 Layout Algorithm

```rust
impl LayoutEngine {
    pub fn layout(&self, document: &Document) -> Result<LayoutTree, LayoutError> {
        let mut tree = LayoutTree::new();

        // Phase 1: Calculate element sizes
        for stave in &document.staves {
            let measured = self.measure_stave(stave)?;
            tree.add_stave(measured);
        }

        // Phase 2: Line breaking
        let systems = self.break_into_systems(&tree)?;

        // Phase 3: Spacing adjustment
        self.justify_systems(&mut systems)?;

        // Phase 4: Vertical alignment
        self.align_vertically(&mut systems)?;

        // Phase 5: Collision resolution
        self.resolve_collisions(&mut systems)?;

        Ok(LayoutTree { systems })
    }

    fn calculate_pitch_width(&self, pitch: &Note) -> f32 {
        let base_width = self.measure_text(&pitch.to_string());
        let accidental_width = pitch.accidental
            .map(|a| self.measure_text(&a.to_string()))
            .unwrap_or(0.0);

        base_width + accidental_width + PITCH_PADDING
    }

    fn calculate_beat_spacing(&self, beat: &Beat) -> f32 {
        // Proportional spacing based on duration
        let duration_factor = (beat.total_duration as f32).log2() + 1.0;
        let content_width: f32 = beat.elements
            .iter()
            .map(|e| self.calculate_element_width(e))
            .sum();

        (content_width * duration_factor).max(MIN_BEAT_WIDTH)
    }
}
```

### 3.3 Spacing Rules

```rust
pub struct SpacingRules {
    pub pitch_padding: f32,        // Space around pitches
    pub dash_width: f32,          // Width of dash character
    pub barline_padding: f32,      // Space around barlines
    pub beat_separator_width: f32, // Space between beats
    pub min_measure_width: f32,    // Minimum measure width
}

impl Default for SpacingRules {
    fn default() -> Self {
        Self {
            pitch_padding: 0.2,      // em units
            dash_width: 0.5,         // em units
            barline_padding: 0.3,    // em units
            beat_separator_width: 1.0, // em units
            min_measure_width: 4.0,  // em units
        }
    }
}
```

## 4. Drawing Commands Architecture

### 4.1 Command Types

Based on the successful pattern used in previous projects, the renderer uses a drawing commands architecture where Rust generates high-level drawing instructions that are executed by a thin JavaScript layer:

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum DrawCommand {
    Text {
        x: f32,
        y: f32,
        text: String,
        font_family: String,
        font_size: f32,
        color: String,
    },
    Symbol {
        x: f32,
        y: f32,
        symbol: MusicSymbol,
        font_size: f32,
    },
    Line {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        width: f32,
        color: String,
    },
    Curve {
        // Cubic bezier for slurs
        x1: f32, y1: f32,  // start
        cx1: f32, cy1: f32, // control point 1
        cx2: f32, cy2: f32, // control point 2
        x2: f32, y2: f32,   // end
        width: f32,
        color: String,
    },
    Dot {
        x: f32,
        y: f32,
        radius: f32,
        color: String,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct RenderCommands {
    pub width: f32,
    pub height: f32,
    pub commands: Vec<DrawCommand>,
}
```

### 4.2 Command Generation

```rust
impl LayoutEngine {
    pub fn generate_commands(&self, layout: &LayoutTree) -> RenderCommands {
        let mut commands = Vec::new();

        for system in &layout.systems {
            for element in &system.elements {
                match &element.kind {
                    ElementKind::Pitch { value, accidental } => {
                        // Main pitch
                        commands.push(DrawCommand::Text {
                            x: element.bounds.x,
                            y: element.bounds.y,
                            text: value.clone(),
                            font_family: "Iowan Old Style".to_string(),
                            font_size: element.style.font_size,
                            color: "#000000".to_string(),
                        });

                        // Accidental if present
                        if let Some(acc) = accidental {
                            commands.push(DrawCommand::Symbol {
                                x: element.bounds.x + element.bounds.width * 0.7,
                                y: element.bounds.y,
                                symbol: match acc {
                                    Accidental::Sharp => MusicSymbol::Sharp,
                                    Accidental::Flat => MusicSymbol::Flat,
                                },
                                font_size: element.style.font_size * 0.8,
                            });
                        }
                    },

                    ElementKind::Slur { start, end, curve } => {
                        commands.push(DrawCommand::Curve {
                            x1: curve.p0.x, y1: curve.p0.y,
                            cx1: curve.p1.x, cy1: curve.p1.y,
                            cx2: curve.p2.x, cy2: curve.p2.y,
                            x2: curve.p3.x, y2: curve.p3.y,
                            width: 1.0,
                            color: "#000000".to_string(),
                        });
                    },

                    ElementKind::OctaveMarker { .. } => {
                        commands.push(DrawCommand::Dot {
                            x: element.bounds.x,
                            y: element.bounds.y,
                            radius: 2.0,
                            color: "#000000".to_string(),
                        });
                    },

                    // ... other element types
                }
            }
        }

        RenderCommands {
            width: layout.width,
            height: layout.height,
            commands,
        }
    }
}
```

### 4.3 JavaScript Executor

The JavaScript side receives these commands and renders them to the chosen backend (SVG recommended):

```javascript
// Renderer that executes drawing commands
class MusicTextRenderer {
    constructor(container) {
        this.container = container;
    }

    render(commandData) {
        // Parse commands from Rust (via WASM or JSON)
        const { width, height, commands } = commandData;

        // Create SVG element
        const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
        svg.setAttribute("width", width);
        svg.setAttribute("height", height);
        svg.setAttribute("viewBox", `0 0 ${width} ${height}`);

        // Execute each command
        for (const cmd of commands) {
            switch (cmd.type) {
                case "Text":
                    this.renderText(svg, cmd);
                    break;
                case "Symbol":
                    this.renderSymbol(svg, cmd);
                    break;
                case "Line":
                    this.renderLine(svg, cmd);
                    break;
                case "Curve":
                    this.renderCurve(svg, cmd);
                    break;
                case "Dot":
                    this.renderDot(svg, cmd);
                    break;
            }
        }

        this.container.appendChild(svg);
    }

    renderText(svg, cmd) {
        const text = document.createElementNS("http://www.w3.org/2000/svg", "text");
        text.setAttribute("x", cmd.x);
        text.setAttribute("y", cmd.y);
        text.setAttribute("font-family", cmd.font_family);
        text.setAttribute("font-size", cmd.font_size);
        text.setAttribute("fill", cmd.color);
        text.textContent = cmd.text;
        svg.appendChild(text);
    }

    renderCurve(svg, cmd) {
        const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
        const d = `M ${cmd.x1} ${cmd.y1} C ${cmd.cx1} ${cmd.cy1}, ${cmd.cx2} ${cmd.cy2}, ${cmd.x2} ${cmd.y2}`;
        path.setAttribute("d", d);
        path.setAttribute("stroke", cmd.color);
        path.setAttribute("stroke-width", cmd.width);
        path.setAttribute("fill", "none");
        svg.appendChild(path);
    }

    // ... other render methods
}
```

### 4.4 Advantages of This Approach

1. **Testability**: Commands can be tested as data
   ```rust
   #[test]
   fn test_generates_text_command() {
       let commands = engine.generate_commands(&layout);
       assert!(matches!(commands[0], DrawCommand::Text { .. }));
   }
   ```

2. **Flexibility**: Same commands can target different renderers
3. **Debugging**: Can log/inspect commands as JSON
4. **Performance**: Batch all layout calculation in Rust, minimal JS overhead
5. **Portability**: Commands could be sent over network, stored, replayed

## 5. Rendering Backend (Alternative Direct Implementation)

### 5.1 Renderer Trait

```rust
pub trait Renderer {
    type Error;

    /// Initialize the renderer with given dimensions
    fn init(&mut self, width: f32, height: f32) -> Result<(), Self::Error>;

    /// Clear the rendering surface
    fn clear(&mut self);

    /// Render text at position
    fn draw_text(
        &mut self,
        text: &str,
        x: f32,
        y: f32,
        style: &Style
    ) -> Result<(), Self::Error>;

    /// Draw a bezier curve (for slurs)
    fn draw_curve(
        &mut self,
        curve: &CubicBezier,
        style: &StrokeStyle
    ) -> Result<(), Self::Error>;

    /// Draw a line (for barlines, etc.)
    fn draw_line(
        &mut self,
        start: Point,
        end: Point,
        style: &StrokeStyle
    ) -> Result<(), Self::Error>;

    /// Measure text dimensions
    fn measure_text(&self, text: &str, style: &Style) -> TextMetrics;

    /// Finalize and return output
    fn finish(self) -> Result<Vec<u8>, Self::Error>;
}
```

### 4.2 Canvas Backend (WASM)

```rust
#[cfg(target_arch = "wasm32")]
pub mod canvas {
    use wasm_bindgen::prelude::*;
    use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

    pub struct CanvasRenderer {
        context: CanvasRenderingContext2d,
        canvas: HtmlCanvasElement,
    }

    impl CanvasRenderer {
        pub fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
            let context = canvas
                .get_context("2d")?
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()?;

            Ok(Self { context, canvas })
        }
    }

    impl Renderer for CanvasRenderer {
        type Error = JsValue;

        fn draw_text(
            &mut self,
            text: &str,
            x: f32,
            y: f32,
            style: &Style
        ) -> Result<(), Self::Error> {
            self.context.set_font(&format!(
                "{}px {}",
                style.font_size,
                style.font_family
            ));
            self.context.fill_text(text, x as f64, y as f64)?;
            Ok(())
        }

        fn draw_curve(
            &mut self,
            curve: &CubicBezier,
            style: &StrokeStyle
        ) -> Result<(), Self::Error> {
            self.context.begin_path();
            self.context.move_to(curve.p0.x as f64, curve.p0.y as f64);
            self.context.bezier_curve_to(
                curve.p1.x as f64, curve.p1.y as f64,
                curve.p2.x as f64, curve.p2.y as f64,
                curve.p3.x as f64, curve.p3.y as f64,
            );
            self.context.stroke();
            Ok(())
        }

        // ... other methods ...
    }
}
```

### 4.3 SVG Backend

```rust
pub mod svg {
    use std::fmt::Write;

    pub struct SvgRenderer {
        width: f32,
        height: f32,
        elements: Vec<SvgElement>,
    }

    impl Renderer for SvgRenderer {
        type Error = std::fmt::Error;

        fn draw_text(
            &mut self,
            text: &str,
            x: f32,
            y: f32,
            style: &Style
        ) -> Result<(), Self::Error> {
            self.elements.push(SvgElement::Text {
                x, y, text: text.to_string(), style: style.clone()
            });
            Ok(())
        }

        fn finish(self) -> Result<Vec<u8>, Self::Error> {
            let mut svg = String::new();
            write!(&mut svg,
                r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#,
                self.width, self.height
            )?;

            for element in self.elements {
                element.write_to(&mut svg)?;
            }

            write!(&mut svg, "</svg>")?;
            Ok(svg.into_bytes())
        }
    }
}
```

## 5. Typography System

### 5.1 Font Management

```rust
pub struct FontManager {
    fonts: HashMap<String, Font>,
    fallback_chain: Vec<String>,
}

impl FontManager {
    pub fn new() -> Self {
        let mut manager = Self {
            fonts: HashMap::new(),
            fallback_chain: vec![
                "Iowan Old Style".to_string(),
                "Georgia".to_string(),
                "serif".to_string(),
            ],
        };

        // Load music symbol font
        manager.load_font("Bravura", include_bytes!("../fonts/Bravura.otf"));

        manager
    }

    pub fn measure_text(&self, text: &str, font_name: &str, size: f32) -> TextMetrics {
        let font = self.fonts.get(font_name)
            .or_else(|| self.get_fallback_font())
            .expect("No fonts available");

        font.measure(text, size)
    }
}
```

### 5.2 Music Symbol Mapping

```rust
pub struct SymbolMapper {
    symbols: HashMap<SymbolType, char>,
}

impl SymbolMapper {
    pub fn new() -> Self {
        let mut symbols = HashMap::new();

        // SMuFL codepoints
        symbols.insert(SymbolType::Mordent, '\u{E56C}');
        symbols.insert(SymbolType::Trill, '\u{E566}');
        symbols.insert(SymbolType::Turn, '\u{E567}');
        symbols.insert(SymbolType::SingleBarline, '\u{E030}');
        symbols.insert(SymbolType::DoubleBarline, '\u{E031}');
        symbols.insert(SymbolType::RepeatLeft, '\u{E040}');
        symbols.insert(SymbolType::RepeatRight, '\u{E041}');

        Self { symbols }
    }

    pub fn get_symbol(&self, symbol_type: SymbolType) -> Option<char> {
        self.symbols.get(&symbol_type).copied()
    }
}
```

## 6. Spatial Components

### 6.1 Octave Markers

```rust
impl LayoutEngine {
    fn layout_octave_markers(&self, note: &Note, x: f32, y: f32) -> Vec<LayoutElement> {
        let mut elements = vec![];

        match note.octave.cmp(&0) {
            std::cmp::Ordering::Greater => {
                // Upper octave - dots above
                for i in 0..note.octave {
                    elements.push(LayoutElement {
                        kind: ElementKind::OctaveMarker { octave: note.octave },
                        bounds: Rectangle {
                            x: x + (i as f32 * DOT_SPACING),
                            y: y - OCTAVE_OFFSET,
                            width: DOT_SIZE,
                            height: DOT_SIZE,
                        },
                        content: "•".to_string(),
                        style: Style::octave_marker(),
                    });
                }
            },
            std::cmp::Ordering::Less => {
                // Lower octave - dots below
                for i in 0..note.octave.abs() {
                    elements.push(LayoutElement {
                        kind: ElementKind::OctaveMarker { octave: note.octave },
                        bounds: Rectangle {
                            x: x + (i as f32 * DOT_SPACING),
                            y: y + OCTAVE_OFFSET,
                            width: DOT_SIZE,
                            height: DOT_SIZE,
                        },
                        content: "•".to_string(),
                        style: Style::octave_marker(),
                    });
                }
            },
            _ => {} // Octave 0 - no markers
        }

        elements
    }
}
```

### 6.2 Slur Calculation

```rust
impl LayoutEngine {
    fn calculate_slur_curve(
        &self,
        start: &LayoutElement,
        end: &LayoutElement
    ) -> CubicBezier {
        let start_x = start.bounds.x + start.bounds.width / 2.0;
        let end_x = end.bounds.x + end.bounds.width / 2.0;
        let y = start.bounds.y - SLUR_HEIGHT;

        let distance = end_x - start_x;
        let control_offset = distance * 0.3;

        CubicBezier {
            p0: Point { x: start_x, y },
            p1: Point { x: start_x + control_offset, y: y - SLUR_CURVE },
            p2: Point { x: end_x - control_offset, y: y - SLUR_CURVE },
            p3: Point { x: end_x, y },
        }
    }
}
```

### 6.3 Lyrics Alignment

```rust
impl LayoutEngine {
    fn align_lyrics(
        &self,
        notes: &[LayoutElement],
        syllables: &[String]
    ) -> Vec<LayoutElement> {
        syllables.iter()
            .zip(notes.iter())
            .map(|(syllable, note)| {
                let text_width = self.measure_text(syllable);
                let center_x = note.bounds.x + note.bounds.width / 2.0;

                LayoutElement {
                    kind: ElementKind::Lyric {
                        syllable: syllable.clone()
                    },
                    bounds: Rectangle {
                        x: center_x - text_width / 2.0,
                        y: note.bounds.y + LYRICS_OFFSET,
                        width: text_width,
                        height: LYRICS_HEIGHT,
                    },
                    content: syllable.clone(),
                    style: Style::lyrics(),
                }
            })
            .collect()
    }
}
```

## 7. WASM Integration

### 7.1 JavaScript Interface

```rust
#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct TextRenderer {
        engine: LayoutEngine,
        renderer: Box<dyn Renderer>,
    }

    #[wasm_bindgen]
    impl TextRenderer {
        #[wasm_bindgen(constructor)]
        pub fn new(canvas_id: &str) -> Result<TextRenderer, JsValue> {
            let canvas = get_canvas_by_id(canvas_id)?;
            let renderer = CanvasRenderer::new(canvas)?;

            Ok(TextRenderer {
                engine: LayoutEngine::new(),
                renderer: Box::new(renderer),
            })
        }

        #[wasm_bindgen]
        pub fn render_music_text(&mut self, source: &str) -> Result<(), JsValue> {
            // Parse the music-text
            let document = parse_music_text(source)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

            // Calculate layout
            let layout = self.engine.layout(&document)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

            // Render
            self.renderer.render(&layout)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

            Ok(())
        }
    }
}
```

### 7.2 Build Configuration

```toml
# Cargo.toml
[dependencies]
# Core dependencies
serde = { version = "1.0", features = ["derive"] }

# WASM dependencies
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = [
    "CanvasRenderingContext2d",
    "HtmlCanvasElement",
    "TextMetrics"
]}

# Native dependencies
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
# For SVG/PDF generation
svg = "0.10"
printpdf = "0.5"
```

## 8. Usage Example

### 8.1 Rust API

```rust
use music_text_renderer::{TextRenderer, RenderOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let music_text = r#"
        title: Example Piece
        key: C

        1-2-3 | 4--5 | 6-7-1'
        La la la  Do  re  mi
    "#;

    let options = RenderOptions {
        width: 800.0,
        height: 600.0,
        font_size: 14.0,
        notation: NotationType::Number,
    };

    // Render to SVG
    let svg = TextRenderer::render_to_svg(music_text, &options)?;
    std::fs::write("output.svg", svg)?;

    Ok(())
}
```

### 8.2 JavaScript API

```javascript
import init, { TextRenderer } from './music_text_renderer_wasm.js';

async function renderMusic() {
    await init();

    const renderer = new TextRenderer('canvas');

    const musicText = `
        1-2-3 | 4--5 | 6-7-1'
        La la la  Do  re  mi
    `;

    renderer.render_music_text(musicText);
}
```

## 9. Testing Strategy

### 9.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pitch_spacing() {
        let engine = LayoutEngine::new();
        let note = Note {
            pitch_code: PitchCode::C,
            octave: 0,
            accidental: None,
        };

        let width = engine.calculate_pitch_width(&note);
        assert!(width > 0.0);
        assert!(width < 2.0); // Reasonable bounds
    }

    #[test]
    fn test_slur_curve() {
        let engine = LayoutEngine::new();
        let start = create_test_element(0.0, 0.0);
        let end = create_test_element(100.0, 0.0);

        let curve = engine.calculate_slur_curve(&start, &end);

        // Verify curve properties
        assert_eq!(curve.p0.x, 0.0);
        assert_eq!(curve.p3.x, 100.0);
        assert!(curve.p1.y < curve.p0.y); // Curves upward
    }
}
```

### 9.2 Visual Tests

```rust
#[test]
fn test_render_simple_melody() {
    let input = "1-2-3 | 4--5";
    let expected_svg = include_str!("../tests/fixtures/simple_melody.svg");

    let rendered = TextRenderer::render_to_svg(input, &Default::default())
        .expect("Failed to render");

    assert_svg_similar(&rendered, expected_svg);
}
```

## 10. Performance Considerations

### 10.1 Optimization Strategies
- Cache text measurements
- Reuse layout calculations when possible
- Batch rendering operations
- Use dirty rectangles for partial updates

### 10.2 Memory Management
- Pool allocations for temporary objects
- Use arena allocators for layout tree
- Minimize string allocations

## 11. Future Extensions

### 11.1 Potential Features
- Animation support for educational purposes
- Interactive elements (hover, click)
- Multiple pages/pagination
- Responsive layout for different screen sizes

### 11.2 Additional Backends
- Native GPU rendering (wgpu)
- Terminal output (for CLI tools)
- Direct printing support

## Conclusion

This Rust-based text renderer for music-text provides a solid foundation for high-quality typographic rendering of text-based musical notation. The modular architecture allows for multiple output targets while maintaining the core layout logic in Rust for maximum performance and reliability.