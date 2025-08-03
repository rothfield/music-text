# Technical Note: Data Model Evolution and Grid Editor Implications

## Overview
Analysis of the evolution from the old `notation_engine` system to the current `notation_parser`, examining how architectural differences inform the design of a grid-based editor.

## Data Model Comparison

### Old System (notation_engine): Typography-Focused
```rust
Composition {
    title: String,
    author: String,
    lines: Vec<Line>,              // Sequential lines of flowing text
    width: f32, height: f32,       // Canvas dimensions
    selected_element_id: Option<u32>, // Selection state
    letter_font: Font,
    symbol_font: Font,
    font_size: f32,
}

Line {
    name: String,
    elements: Vec<TextElement>,    // Linear sequence
}

TextElement {
    id: u32,                      // Unique identifier
    text: String,                 // Single character/symbol
    x: f32, y: f32,              // Continuous pixel coordinates
    width: f32, height: f32,      // Precise dimensions
    font: FontFamily,
    y_offset: f32,               // Baseline adjustment
}
```

### Current System (notation_parser): Structure-Focused
```rust
Document {
    metadata: Metadata,           // Separated metadata layer
    nodes: Vec<Node>,            // Hierarchical tree
}

Node {
    node_type: String,           // Semantic type: "PITCH", "BEAT", "ORNAMENT"
    value: String,               // Text content
    row: usize, col: usize,      // Discrete grid coordinates
    divisions: usize,            // Musical timing
    dash_consumed: bool,         // Processing state
    nodes: Vec<Node>,            // Recursive children
    pitch_code: Option<PitchCode>, // Musical semantics
    octave: Option<i8>,
}
```

## Key Architectural Differences

### 1. Coordinate Systems
- **Old**: Continuous coordinates (`x: 45.7, y: 120.3`)
- **New**: Discrete grid coordinates (`row: 3, col: 15`)

### 2. Element Identity
- **Old**: Unique numeric IDs for selection (`id: u32`)
- **New**: Positional identity by coordinates (`row, col`)

### 3. Data Structure
- **Old**: Flat sequential elements within lines
- **New**: Recursive tree with parent-child relationships

### 4. Focus
- **Old**: Typography (font metrics, baseline offsets, rendering)
- **New**: Semantics (musical meaning, pitch codes, structural relationships)

### 5. Processing Model
- **Old**: Layout calculation → Positioning → Rendering
- **New**: Lexing → Spatial flattening → Beat grouping → Conversion

## Layout Model from Old System

### Three-Stage Layout Pipeline
```rust
// Stage 1: Font Metrics
get_element_metrics() -> ElementMetrics { width, height, y_offset }

// Stage 2: Position Calculation  
calculate_next_position() -> Position { id, x, y }

// Stage 3: Bounding Box Assignment
assign_bounding_boxes() -> Updates element dimensions
```

### Precise Font Metrics
```rust
ElementMetrics {
    width: f32,      // Actual glyph pixel width
    height: f32,     // Actual glyph pixel height
    y_offset: f32,   // Baseline offset for rendering
}
```

**Features:**
- Pixel-perfect measurement using `rusttype`
- Baseline calculation: `y_offset = v_metrics.ascent - min_y`
- Padding for interaction (2px for better click targets)
- Font-aware metrics for letter vs symbol fonts

### Flow-Based Positioning
```rust
const START_X: f32 = 50.0;
const START_Y: f32 = 100.0;
const LINE_SPACING_Y: f32 = 80.0;
const LETTER_SPACING: f32 = 10.0;
```

**Algorithm:**
- Horizontal flow with automatic spacing
- Line wrapping when exceeding canvas width
- Fixed vertical line spacing
- Auto-positioning relative to previous elements

### Adaptive Bounding Boxes
- **Element width** = Distance to next element (proportional spacing)
- **Uniform height** = Maximum height in line (using "Mg" reference)
- **Last element** = Fallback width (width of "M")

## Grid Editor Implications

### Relevant Concepts from Old System

#### 1. Selection Model
```rust
selected_element_id: Option<u32>
```
**Grid needs:** Current cell selection, multi-cell ranges, persistent selection state

#### 2. Precise Positioning & Dimensions
```rust
x: f32, y: f32, width: f32, height: f32
```
**Grid needs:** Fixed cell dimensions, pixel mapping, bounding boxes for interaction

#### 3. Font Integration
```rust
letter_font: Font, symbol_font: Font, y_offset: f32
```
**Grid needs:** Dual font system, baseline alignment, font metrics for cell sizing

#### 4. Canvas Management
```rust
width: f32, height: f32
```
**Grid needs:** Viewport size, scrolling bounds, dynamic resizing

#### 5. Hit Testing
```rust
let hit = x >= el.x && x <= el.x + el.width && 
          y >= box_y && y <= box_y + el.height;
```
**Grid needs:** Mouse-to-cell coordinate conversion, precise click detection

#### 6. Command-Based Rendering
```rust
render_to_canvas() -> Vec<DrawCommand>
```
**Grid needs:** Separation of state generation from canvas execution

### Missing in Current System for Grid
1. **No selection state** - No cursor/selection concept
2. **No pixel positioning** - Only discrete coordinates
3. **No font metrics** - No rendering information
4. **No interactive state** - No "current editing position"
5. **No viewport concept** - No bounds or scrolling management

## Recommended Hybrid Architecture

### Grid Layout Structure
```rust
struct GridLayout {
    // From old system: precise font handling
    letter_font: Font,
    symbol_font: Font,
    font_size: f32,
    
    // Grid-specific: uniform structure
    cell_width: f32,
    cell_height: f32,
    rows: usize,
    cols: usize,
    
    // Performance optimization
    metrics_cache: HashMap<(char, FontFamily), ElementMetrics>,
}

struct GridEditor {
    // From old system: interaction & rendering
    selected_row: usize,
    selected_col: usize,
    canvas_width: f32,
    canvas_height: f32,
    layout: GridLayout,
    
    // From new system: musical intelligence
    document: Document,
    raw_text: String,    // Source of truth for round-trip
}
```

### Integration Workflow
```
Grid Edit → Update raw_text → Re-parse to Document → Generate LilyPond
```

### Two-Phase Layout for Grid
```rust
// Phase 1: Calculate metrics for all cells
for cell in grid { 
    cell.metrics = get_element_metrics(cell.char, cell.font) 
}

// Phase 2: Position cells in uniform grid
for cell in grid { 
    cell.position = calculate_grid_position(row, col, cell_dimensions) 
}
```

## Benefits of Hybrid Approach
- **Professional typography**: Pixel-perfect font rendering from old system
- **Musical intelligence**: Structural understanding from new system
- **Predictable layout**: Excel-like grid structure for editing
- **Performance**: Cached font metrics, efficient rendering
- **Round-trip capability**: Maintains compatibility with existing pipeline

## Conclusion
The old system's layout model provides essential foundation for grid editing (font metrics, positioning, interaction), while the new system provides musical semantic understanding. A hybrid approach leveraging both architectures would create a sophisticated grid editor that maintains the current system's analytical capabilities while providing intuitive direct editing.