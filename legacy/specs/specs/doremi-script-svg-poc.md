# Doremi-Script SVG Renderer - Proof of Concept Specification

## Executive Summary

This specification defines a standalone Rust program that ports the proven typography and spacing from the doremi-script project to generate SVG output. The program accepts a simple JSON document and produces SVG with embedded CSS styling. This is designed as a proof of concept to validate whether doremi-script's visual quality can be replicated in a modern Rust implementation.

**Key Constraint**: The JSON document structure should remain stable throughout development. NO adapters or transformations - the renderer works directly with the input format.

## 1. Project Goals

### Primary Goal
Prove that doremi-script's typography quality can be replicated in Rust SVG generation.

### Success Criteria
- SVG output visually matches doremi-script rendering
- Typography spacing and positioning preserved
- Clean CSS-based styling (no hardcoded coordinates)
- HTTP API for easy testing and integration

### Non-Goals
- Integration with existing music-text parser
- Real-time editing capabilities
- Audio playback
- Complex music notation (stems, beams, etc.)

## 2. Reference Implementation

### 2.1 Original Doremi-Script Source

The complete original doremi-script implementation (Clojure/ClojureScript) is available at `@archive/doremi-script/` in this codebase. This contains:

- **Typography Rules**: CSS implementations in the original doremi-script for exact spacing, font sizes, and positioning
- **Ornament Rendering**: Complete ornament implementation showing how sargam ornaments, grace notes, and symbolic ornaments are handled
- **Multi-notation Support**: Examples of sargam, number, western, and Devanagari notation rendering
- **Grammar Reference**: The complete EBNF grammar showing all supported ornament types and syntax

**Key Reference Files**:
- `archive/doremi-script/src/views.cljs` - HTML/CSS rendering logic with exact typography measurements
- `archive/doremi-script/grammar/doremiscript.ebnf` - Complete grammar including ornament definitions
- `archive/doremi-script/src/to_lilypond.cljc` - Grace note and ornament conversion logic

This reference implementation should be consulted for:
1. **Typography Fidelity**: Ensuring SVG output matches the proven doremi-script visual quality
2. **Ornament Behavior**: Understanding how the 4 ornament types (before/on-top/after grace notes + symbolic ornaments) should render
3. **Cross-notation Support**: Future expansion to support all notation systems (sargam, number, western, Devanagari)

The POC SVG renderer should achieve visual parity with the original doremi-script HTML/CSS output while using modern SMuFL fonts for enhanced symbol quality.

### 2.2 JSON Format Design Reference

The existing music-text domain models at `@src/models/` provide the foundation for understanding how the simplified JSON format should eventually integrate with the full parser. Key reference files:

- **`src/models/domain.rs`** - Core `Document`, `Node`, `BarlineType` structures that the JSON format should eventually map to
- **`src/models/pitch.rs`** - `Degree` enum (N1, N1s, N1b, etc.) showing how accidentals are represented internally
- **`src/models/parsed.rs`** - Parse tree structures that show the full complexity the simplified JSON abstracts away
- **`src/parse/model.rs`** - Newer document model with `Beat`, `BeatElement`, spatial assignments

**JSON-to-Models Mapping**: The spec's simplified JSON format should be designed as a clean subset of the existing models:

```rust
// JSON "pitch" element maps to existing structures:
{
  "type": "pitch",
  "value": "1",           // -> pitch lookup -> Degree::N1
  "octave": 0,            // -> directly maps to octave field
  "accidental": "sharp",  // -> influences Degree (N1 -> N1s)
  "ornaments": [...],     // -> NEW: not in current models yet
  "lyrics": [...]         // -> maps to syllable fields
}

// JSON "barline" element maps to:
{
  "type": "barline",
  "style": "single"       // -> BarlineType::Single
}
```

**Design Principle**: The JSON format should be **forward-compatible** - when ornaments are eventually added to the main parser, the JSON structure should map cleanly to the extended domain models without breaking changes.

This ensures the POC SVG renderer can serve as both a proof-of-concept and a bridge to future full pipeline integration.

## 3. JSON Document Format

The renderer accepts a simple, stable JSON format. **This structure should NOT change during development.**

### 2.1 Complete Document Schema

```json
{
  "title": "Optional piece title",
  "composer": "Optional composer name",
  "notation_type": "number",
  "font_size": 14.0,
  "supports_utf8": true,
  "elements": [
    {
      "type": "pitch",
      "value": "1",
      "octave": 0,
      "accidental": null,
      "ornaments": [],
      "lyrics": []
    },
    {
      "type": "dash",
      "is_rest": false
    },
    {
      "type": "barline",
      "style": "single"
    }
  ]
}
```

### 2.2 Element Types

#### Pitch Element
```json
{
  "type": "pitch",
  "value": "1",           // The pitch (1-7, C-B, S/R/G/M/P/D/N)
  "octave": 0,            // -2, -1, 0, 1, 2 (0 = no markers)
  "accidental": "sharp",  // null, "sharp", "flat"
  "ornaments": [          // List of ornaments (see Section 2.4 for complete structure)
    {
      "type": "symbolic_ornament",
      "symbol": "mordent"
    }
  ],
  "lyrics": ["La", "la"]  // Syllables for this pitch
}
```

#### Dash Element
```json
{
  "type": "dash",
  "is_rest": false  // true if this dash represents a rest
}
```

#### Barline Element
```json
{
  "type": "barline",
  "style": "single"  // "single", "double", "repeat_start", "repeat_end"
}
```

### 2.3 Notation Types
- `"number"`: 1, 2, 3, 4, 5, 6, 7
- `"sargam"`: S, R, G, M, P, D, N
- `"western"`: C, D, E, F, G, A, B

### 2.4 Ornament Types

Based on the doremi-script analysis, there are **four distinct ornament types** that will eventually be ported to music-text:

#### Grace Note Ornaments (rendered as note sequences)
- `"before_grace_notes"`: Small notes played before the main note (e.g., `GM`)
- `"on_top_grace_notes"`: Note sequences rendered above the main note (e.g., `NRSNS`)
- `"after_grace_notes"`: Small notes played after the main note (e.g., `PD`)

#### Symbolic Ornaments (western tradition)
- `"symbolic_ornament"`: Traditional western symbols (`mordent` → `~`, `trill` → `tr`, etc.)

**POC Scope**: For the proof of concept, we will implement **sargam ornaments only** with the following test examples:
- before_grace_notes: `GM`
- on_top_grace_notes: `NRSNS`
- after_grace_notes: `PD`
- symbolic_ornament: `mordent` (renders as `~`)

**Complete JSON Ornament Structure**:
```json
{
  "ornaments": [
    {
      "type": "before_grace_notes",
      "notes": [
        {"value": "G", "octave": 0, "accidental": null},
        {"value": "M", "octave": 0, "accidental": null}
      ]
    },
    {
      "type": "on_top_grace_notes",
      "notes": [
        {"value": "N", "octave": 0, "accidental": null},
        {"value": "R", "octave": 0, "accidental": null},
        {"value": "S", "octave": 0, "accidental": null},
        {"value": "N", "octave": 0, "accidental": null},
        {"value": "S", "octave": 0, "accidental": null}
      ]
    },
    {
      "type": "after_grace_notes",
      "notes": [
        {"value": "P", "octave": 0, "accidental": null},
        {"value": "D", "octave": 0, "accidental": null}
      ]
    },
    {
      "type": "symbolic_ornament",
      "symbol": "mordent"
    }
  ]
}

## 3. Doremi-Script Typography Rules

These rules are extracted from the doremi-script CSS and must be preserved exactly.

### 3.1 Base Typography

```css
/* Base settings from doremi.css lines 304-310 */
.note {
  font-family: sans-serif;
  font-size: 22.4px;    /* 1.6em at 14px base */
  position: relative;
  display: inline-block;
  margin-right: 0.0em;
  margin-left: 0.0em;
}
```

### 3.2 Element Spacing

```css
/* Spacing rules from doremi.css */
.beat {
  margin-right: 16px;    /* 1em at 16px */
  margin-left: 0px;
}

.dash {
  margin-left: 1.44px;   /* 0.09em */
  margin-right: 1.44px;
}

.barline {
  margin-right: 6.4px;   /* 0.4em */
  margin-left: 6.4px;
  padding: 3.2px;        /* 0.2em */
  letter-spacing: -3.2px; /* -0.2em */
}
```

### 3.3 Octave Marker Positioning

```css
/* Octave markers from doremi.css lines 237-251 */
.upper-octave {
  position: absolute;
  top: 6.4px;      /* 0.4em above note */
  left: 3.2px;     /* 0.2em from note left */
  font-size: 13.44px; /* 0.6 * base size */
}

.lower-octave {
  position: absolute;
  bottom: 28.8px;  /* 1.8em below note */
  left: 3.2px;     /* 0.2em from note left */
  font-size: 13.44px; /* 0.6 * base size */
}
```

### 3.4 Ornament Positioning

```css
/* Ornaments from doremi.css lines 110-114 */
.ornament {
  position: absolute;
  bottom: 15.68px; /* 0.98em */
  font-size: 20.16px; /* 0.9 * base size */
}

.ornament.placement-after {
  margin-left: 20.8px; /* 1.3em */
}
```

### 3.5 Lyrics Positioning

```css
/* Lyrics from doremi.css lines 349-355 */
.lyric {
  position: absolute;
  bottom: 16px;    /* 1.0em */
  margin-left: 0.48px; /* 0.03em */
  font-size: 22.624px; /* 1.01 * base size */
  font-family: serif;
}
```

### 3.6 Accidental Positioning

```css
/* Accidentals from doremi.css lines 313-316 */
.accidental {
  position: absolute;
  bottom: 11.2px;  /* 0.7em */
  left: 16px;      /* 1em from note */
  font-size: 22.4px; /* same as base */
}
```

## 4. Music Font Strategy

### 4.1 SMuFL Font Integration

The renderer uses **SMuFL (Standard Music Font Layout)** compliant fonts for professional-quality music symbols, moving beyond doremi-script's Unicode limitations.

```css
/* Font loading in SVG */
@font-face {
  font-family: 'Bravura';
  src: url('bravura.woff2') format('woff2'),
       url('bravura.woff') format('woff');
  font-display: swap;
  unicode-range: U+E000-E8FF; /* SMuFL Private Use Area */
}

@font-face {
  font-family: 'Leland';
  src: url('leland.woff2') format('woff2');
  font-display: swap;
  unicode-range: U+E000-E8FF;
}
```

### 4.2 SMuFL Symbol Mapping

```rust
pub struct SMuFLMapper {
    symbols: HashMap<SymbolType, char>,
    fallbacks: HashMap<SymbolType, &'static str>,
}

impl SMuFLMapper {
    pub fn new() -> Self {
        let mut symbols = HashMap::new();
        let mut fallbacks = HashMap::new();

        // Barlines (SMuFL E030-E04F)
        symbols.insert(SymbolType::BarlineSingle, '\u{E030}');
        fallbacks.insert(SymbolType::BarlineSingle, "|");

        symbols.insert(SymbolType::BarlineDouble, '\u{E031}');
        fallbacks.insert(SymbolType::BarlineDouble, "‖");

        symbols.insert(SymbolType::RepeatStart, '\u{E040}');
        fallbacks.insert(SymbolType::RepeatStart, "|:");

        symbols.insert(SymbolType::RepeatEnd, '\u{E041}');
        fallbacks.insert(SymbolType::RepeatEnd, ":|");

        // Accidentals (SMuFL E260-E2AF)
        symbols.insert(SymbolType::Sharp, '\u{E262}');
        fallbacks.insert(SymbolType::Sharp, "♯");

        symbols.insert(SymbolType::Flat, '\u{E260}');
        fallbacks.insert(SymbolType::Flat, "♭");

        // Ornaments (SMuFL E560-E5FF)
        symbols.insert(SymbolType::Mordent, '\u{E56C}');
        fallbacks.insert(SymbolType::Mordent, "∿");

        symbols.insert(SymbolType::Trill, '\u{E566}');
        fallbacks.insert(SymbolType::Trill, "tr");

        Self { symbols, fallbacks }
    }

    pub fn get_symbol(&self, symbol_type: SymbolType, use_smufl: bool) -> &str {
        if use_smufl {
            if let Some(symbol) = self.symbols.get(&symbol_type) {
                return &symbol.to_string();
            }
        }
        self.fallbacks.get(&symbol_type).unwrap_or(&"?")
    }
}
```

### 4.3 Font Hierarchy Strategy

```rust
pub struct FontStrategy {
    pub primary_music_font: &'static str,
    pub fallback_music_font: &'static str,
    pub text_font: &'static str,
    pub use_smufl: bool,
}

impl Default for FontStrategy {
    fn default() -> Self {
        Self {
            primary_music_font: "Bravura",      // Professional, open source
            fallback_music_font: "Leland",       // MuseScore font, more casual
            text_font: "Georgia",                 // For lyrics, labels
            use_smufl: true,                     // Enable SMuFL by default
        }
    }
}
```

### 4.4 Progressive Enhancement

```rust
// Rust implementation with graceful degradation
impl SvgRenderer {
    fn render_barline_symbol(&self, barline_type: BarlineType) -> String {
        let symbol_type = match barline_type {
            BarlineType::Single => SymbolType::BarlineSingle,
            BarlineType::Double => SymbolType::BarlineDouble,
            BarlineType::RepeatStart => SymbolType::RepeatStart,
            BarlineType::RepeatEnd => SymbolType::RepeatEnd,
        };

        let symbol = self.font_mapper.get_symbol(symbol_type, self.config.use_smufl);
        let font_class = if self.config.use_smufl { "smufl-symbol" } else { "unicode-symbol" };

        format!(
            r#"<text x="{}" y="{}" class="barline {}">{}</text>"#,
            self.current_x, self.current_y, font_class, symbol
        )
    }
}
```

## 5. SVG Output Format

### 5.1 Complete SVG Structure with SMuFL

```xml
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg"
     width="800" height="600"
     viewBox="0 0 800 600">

  <!-- Embedded CSS with SMuFL fonts and doremi-script spacing -->
  <style>
  <![CDATA[
    /* SMuFL Font Loading */
    @font-face {
      font-family: 'Bravura';
      src: url('bravura.woff2') format('woff2');
      font-display: swap;
      unicode-range: U+E000-E8FF;
    }

    @font-face {
      font-family: 'Leland';
      src: url('leland.woff2') format('woff2');
      font-display: swap;
      unicode-range: U+E000-E8FF;
    }

    /* Typography Hierarchy */
    .note {
      font-family: sans-serif;
      font-size: 22.4px;
      fill: black;
    }

    .smufl-symbol {
      font-family: 'Bravura', 'Leland', serif;
      font-size: 22.4px;
      fill: black;
    }

    .unicode-symbol {
      font-family: serif;
      font-size: 22.4px;
      fill: black;
    }

    .upper-octave {
      font-family: sans-serif;
      font-size: 13.44px; /* 0.6 * base */
      fill: black;
    }

    .lower-octave {
      font-family: sans-serif;
      font-size: 13.44px;
      fill: black;
    }

    .lyric {
      font-family: serif;
      font-size: 22.624px; /* 1.01 * base */
      fill: black;
    }

    .ornament {
      font-family: 'Bravura', 'Leland', serif;
      font-size: 20.16px; /* 0.9 * base */
      fill: black;
    }

    /* Spacing and positioning rules from doremi-script */
    .barline {
      letter-spacing: -3.2px; /* -0.2em */
    }

    .title {
      font-family: sans-serif;
      font-size: 24px;
      font-weight: bold;
      fill: black;
    }
  ]]>
  </style>

  <!-- Title if present -->
  <text x="400" y="30" class="title" text-anchor="middle">Piece Title</text>

  <!-- Music content with SMuFL symbols -->
  <g class="composition" transform="translate(20, 80)">
    <!-- Note with octave marker -->
    <text x="0" y="0" class="note">1</text>
    <text x="3.2" y="-6.4" class="upper-octave">•</text>

    <!-- Dash extension -->
    <text x="30" y="0" class="note">–</text>

    <!-- Note with sharp accidental (SMuFL) -->
    <text x="50" y="0" class="note">2</text>
    <text x="66" y="-11.2" class="smufl-symbol">&#xE262;</text> <!-- SMuFL sharp -->

    <!-- Repeat end barline (SMuFL) -->
    <text x="90" y="0" class="smufl-symbol barline">&#xE041;</text> <!-- SMuFL repeat end -->
  </g>

</svg>
```

### 5.2 Positioning Logic

Elements are positioned using absolute coordinates within the composition group:

```rust
// Pseudo-code for positioning
let mut current_x = 0.0;
let base_y = 0.0;

for element in elements {
    match element.type {
        "pitch" => {
            // Render main note
            render_text(current_x, base_y, element.value, "note");

            // Render octave markers relative to note
            if element.octave > 0 {
                render_text(current_x + 3.2, base_y - 6.4, "•", "upper-octave");
            }

            // Advance position
            current_x += 30.0; // Approximate note width
        },
        "dash" => {
            render_text(current_x, base_y, "–", "dash");
            current_x += 15.0; // Dash width
        },
        "barline" => {
            render_text(current_x, base_y, "|", "barline");
            current_x += 20.0; // Barline spacing
        }
    }
}
```

## 6. Implementation Architecture

### 6.1 Core Rust Components

```rust
// Main application structure
struct SvgRenderer {
    width: f32,
    height: f32,
    current_x: f32,
    current_y: f32,
}

// Document model (matches JSON exactly)
#[derive(Deserialize)]
struct Document {
    title: Option<String>,
    composer: Option<String>,
    notation_type: String,
    font_size: f32,
    supports_utf8: bool,
    elements: Vec<Element>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum Element {
    #[serde(rename = "pitch")]
    Pitch {
        value: String,
        octave: i8,
        accidental: Option<String>,
        ornaments: Vec<Ornament>,
        lyrics: Vec<String>,
    },
    #[serde(rename = "dash")]
    Dash { is_rest: bool },
    #[serde(rename = "barline")]
    Barline { style: String },
}

// Core rendering methods
impl SvgRenderer {
    fn render(&mut self, doc: &Document) -> String;
    fn render_element(&mut self, element: &Element) -> String;
    fn render_pitch(&mut self, pitch: &PitchElement) -> String;
    fn render_dash(&mut self, dash: &DashElement) -> String;
    fn render_barline(&mut self, barline: &BarlineElement) -> String;
    fn generate_css(&self) -> String;
}
```

### 6.2 HTTP API (using Axum)

```rust
use axum::{extract::Json, response::Html, routing::post, Router};

async fn render_svg(Json(doc): Json<Document>) -> Html<String> {
    let mut renderer = SvgRenderer::new();
    let svg = renderer.render(&doc);
    Html(svg)
}

fn create_router() -> Router {
    Router::new().route("/render/svg", post(render_svg))
}
```

### 6.3 Dependencies (Cargo.toml)

```toml
[dependencies]
axum = "0.7"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
tower = "0.4"                    # For serving static font files
tower-http = { version = "0.4", features = ["fs"] }  # Static file serving

[build-dependencies]
# Optional: For font subsetting during build
ttf-parser = "0.19"  # Parse font files for subsetting
```

### 6.4 Font Asset Management

```rust
// Static font serving
use tower_http::services::ServeDir;

fn create_router() -> Router {
    Router::new()
        .route("/render/svg", post(render_svg))
        .nest_service("/fonts", ServeDir::new("assets/fonts"))
        .layer(
            CorsLayer::new()
                .allow_origin("*".parse::<HeaderValue>().unwrap())
                .allow_headers([CONTENT_TYPE])
                .allow_methods([Method::GET, Method::POST])
        )
}

// Font file organization
// assets/fonts/
//   bravura.woff2     (Primary SMuFL font)
//   leland.woff2      (Fallback SMuFL font)
//   bravura.css       (Font-face declarations)
```

## 7. Test Cases

### 7.1 Basic Elements

```json
{
  "notation_type": "number",
  "font_size": 14.0,
  "supports_utf8": true,
  "elements": [
    {"type": "pitch", "value": "1", "octave": 0, "accidental": null, "ornaments": [], "lyrics": []},
    {"type": "dash", "is_rest": false},
    {"type": "pitch", "value": "2", "octave": 0, "accidental": null, "ornaments": [], "lyrics": []},
    {"type": "barline", "style": "single"}
  ]
}
```

Expected: "1-2|" with proper doremi-script spacing

### 7.2 Octave Markers

```json
{
  "notation_type": "number",
  "font_size": 14.0,
  "supports_utf8": true,
  "elements": [
    {"type": "pitch", "value": "1", "octave": 1, "accidental": null, "ornaments": [], "lyrics": []},
    {"type": "pitch", "value": "2", "octave": -1, "accidental": null, "ornaments": [], "lyrics": []}
  ]
}
```

Expected: "1" with dot above, "2" with dot below

### 7.3 Accidentals

```json
{
  "notation_type": "number",
  "font_size": 14.0,
  "supports_utf8": true,
  "elements": [
    {"type": "pitch", "value": "1", "octave": 0, "accidental": "sharp", "ornaments": [], "lyrics": []},
    {"type": "pitch", "value": "2", "octave": 0, "accidental": "flat", "ornaments": [], "lyrics": []}
  ]
}
```

Expected: "1♯ 2♭" with proper positioning

### 7.4 SMuFL Font Test

```json
{
  "title": "SMuFL Test",
  "notation_type": "number",
  "font_size": 14.0,
  "supports_utf8": true,
  "elements": [
    {"type": "barline", "style": "repeat_start"},
    {"type": "pitch", "value": "1", "octave": 0, "accidental": "sharp", "ornaments": [], "lyrics": []},
    {"type": "pitch", "value": "2", "octave": 0, "accidental": "flat", "ornaments": [{"type": "symbolic_ornament", "symbol": "mordent"}], "lyrics": []},
    {"type": "barline", "style": "repeat_end"}
  ]
}
```

Expected: SMuFL repeat barlines (𝄆 and 𝄇) with SMuFL accidentals (♯ and ♭) and mordent symbol

### 7.5 Complete Example with All Features

```json
{
  "title": "Complete Test Piece",
  "composer": "Test Composer",
  "notation_type": "number",
  "font_size": 14.0,
  "supports_utf8": true,
  "elements": [
    {"type": "pitch", "value": "1", "octave": 1, "accidental": "sharp", "ornaments": [{"type": "on_top_grace_notes", "notes": [{"value": "N", "octave": 0, "accidental": null}, {"value": "R", "octave": 0, "accidental": null}, {"value": "S", "octave": 0, "accidental": null}]}], "lyrics": ["La"]},
    {"type": "dash", "is_rest": false},
    {"type": "pitch", "value": "2", "octave": 0, "accidental": null, "ornaments": [], "lyrics": ["la"]},
    {"type": "barline", "style": "single"}
  ]
}
```

Expected: Complex rendering with SMuFL symbols, proper doremi-script spacing, and all elements correctly positioned

### 7.6 Comprehensive Sargam Ornament Test

```json
{
  "title": "Sargam Ornament Test",
  "notation_type": "sargam",
  "font_size": 14.0,
  "supports_utf8": true,
  "elements": [
    {"type": "pitch", "value": "S", "octave": 0, "accidental": null, "ornaments": [{"type": "before_grace_notes", "notes": [{"value": "G", "octave": 0, "accidental": null}, {"value": "M", "octave": 0, "accidental": null}]}], "lyrics": []},
    {"type": "pitch", "value": "R", "octave": 0, "accidental": null, "ornaments": [{"type": "on_top_grace_notes", "notes": [{"value": "N", "octave": 0, "accidental": null}, {"value": "R", "octave": 0, "accidental": null}, {"value": "S", "octave": 0, "accidental": null}, {"value": "N", "octave": 0, "accidental": null}, {"value": "S", "octave": 0, "accidental": null}]}], "lyrics": []},
    {"type": "pitch", "value": "G", "octave": 0, "accidental": null, "ornaments": [{"type": "after_grace_notes", "notes": [{"value": "P", "octave": 0, "accidental": null}, {"value": "D", "octave": 0, "accidental": null}]}], "lyrics": []},
    {"type": "pitch", "value": "M", "octave": 0, "accidental": null, "ornaments": [{"type": "symbolic_ornament", "symbol": "mordent"}], "lyrics": []},
    {"type": "barline", "style": "single"}
  ]
}
```

Expected: All 4 ornament types rendered correctly with sargam notation - before grace notes (GM before S), on-top grace notes (NRSNS above R), after grace notes (PD after G), and symbolic mordent (~) above M

## 8. Visual Quality Requirements

### 8.1 Spacing Accuracy
- Element spacing must match doremi-script within 2px
- Octave markers positioned exactly per CSS rules
- Accidentals and ornaments properly aligned

### 8.2 Typography Fidelity
- Font sizes and families match doremi-script
- Line heights and positioning preserved
- UTF-8 symbols rendered correctly when supported

### 8.3 CSS Cleanliness
- All styling via CSS classes, no hardcoded positioning
- Relative positioning where possible
- Clean, readable CSS output

## 9. Development Guidelines

### 9.1 JSON Structure Stability
**CRITICAL**: The JSON document format defined in Section 2 should NOT change during development. Any new features must work within this structure. No adapters, transformers, or middleware.

### 9.2 Implementation Order
1. Basic element rendering (pitch, dash, barline)
2. CSS generation and embedding
3. Octave marker positioning
4. Accidental positioning
5. Ornament support
6. Lyrics support
7. HTTP API

### 9.3 Testing Strategy
- Visual comparison with doremi-script output
- Pixel-perfect spacing verification
- Cross-browser SVG compatibility
- Performance with complex documents

## 10. Success Metrics

### 10.1 Typography Quality
- [ ] Spacing matches doremi-script within 2px tolerance
- [ ] All CSS rules properly ported from doremi-script
- [ ] SMuFL symbols render with professional quality
- [ ] Graceful fallback to Unicode when SMuFL unavailable
- [ ] Visual output exceeds doremi-script quality

### 10.2 Technical Quality
- [ ] Clean SVG output with embedded CSS and fonts
- [ ] Stable JSON API (no format changes needed)
- [ ] HTTP endpoint functional with CORS support
- [ ] Font files served efficiently with proper caching
- [ ] Performance acceptable for typical documents

### 10.3 SMuFL Integration
- [ ] Bravura and Leland fonts load correctly
- [ ] All barlines use SMuFL symbols (E030-E041)
- [ ] All accidentals use SMuFL symbols (E260-E262)
- [ ] Ornaments use SMuFL symbols (E560-E5FF)
- [ ] Font fallback chain works reliably

### 10.4 Completeness
- [ ] All test cases render correctly
- [ ] Support for all element types in JSON schema
- [ ] Proper error handling for malformed input
- [ ] Font asset management working
- [ ] Documentation complete with SMuFL examples

## Integration with Stylesheet Specification

**IMPORTANT**: Upon completion of this proof of concept, the `/specs/stylesheet-specification.md` must be updated to ensure visual consistency between the code editor and SVG renderer. The code editor and SVG output should share:

- **Typography hierarchy** (font families, sizes, weights)
- **Color palette** (semantic colors for notes, barlines, ornaments)
- **Spacing rules** (beat grouping, element margins)
- **SMuFL font usage** (consistent symbol rendering)
- **Visual semantics** (what colors and styles mean musically)

This ensures users see the same visual language whether editing in the code editor or viewing rendered SVG output.

## Conclusion

This proof of concept will validate whether doremi-script's proven typography can be successfully ported to a modern Rust SVG implementation with professional SMuFL fonts. The standalone nature allows for rapid iteration and clear success criteria, while the stable JSON format ensures a clean API boundary for future integration. Upon success, the findings will inform updates to the stylesheet specification to maintain visual consistency across the entire music-text ecosystem.