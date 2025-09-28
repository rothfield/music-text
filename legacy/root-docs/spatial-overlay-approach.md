# The Spatial Overlay Approach: 2D Canvas over Linear Text

## The Core Insight

Music notation is fundamentally 2D:
- **Horizontal**: Time progression (notes, beats, measures)
- **Vertical**: Pitch (octave dots above/below), lyrics below, ornaments above, multiple staves

Text/JSON structures are fundamentally linear/hierarchical - they don't naturally map to 2D spatial relationships.

## The Original CodeMirror Attempt

### What You Were Trying
- Keep the text as the editable source
- Overlay visual enhancements via CSS classes
- Monospace grid as the underlying coordinate system
- Make it "prettier" without changing the editing model

### Why CodeMirror Failed
- **Monospace limitations**: Music needs variable spacing (whole notes vs sixteenths)
- **Vertical alignment issues**: Can't properly position octave dots, lyrics, ornaments
- **No true 2D editing**: Still fundamentally line-based
- **CSS limitations**: Can't draw arcs, beams, complex musical symbols
- **Performance**: DOM manipulation for every visual update

## The Canvas Overlay Vision

### Core Concept
```
Plain Text Layer (Editable)
    ↓
Canvas Overlay (Visual Enhancement)
    ↓
User sees beautiful 2D music notation
```

### How It Would Work

1. **Text remains the source of truth**
   ```
   Ṡ Ṙ G M | P D Ṅ Ṡ
   sa re ga ma pa da ni sa
   ```

2. **Canvas renders on top**
   - Parse text to identify note positions
   - Draw pretty notes at calculated x,y coordinates
   - Add visual beats groups, barlines, ornaments
   - Position lyrics with proper spacing

3. **Editing stays text-based**
   - User clicks on canvas
   - Map click position back to text character position
   - Show text cursor at that position
   - User types normally
   - Canvas redraws with new visual

### The Spatial Mismatch Problem

**The fundamental issue**: A user clicks on a visual note at position (x: 245px, y: 120px), but that maps to character 5 in a string. The spatial relationship is lost.

```
Visual Space (Canvas):
   S    R    G    M    |    P    D    N    S
   ↑
   (x: 245, y: 120)

Text Space (String):
   "S R G M | P D N S"
        ↑
        char 5
```

### Problems with Pure Overlay

1. **Cursor positioning**: Text cursor doesn't align with visual elements
2. **Selection**: Text selection looks wrong when overlaid
3. **Insertion point ambiguity**: Where does a new character go visually?
4. **Whitespace handling**: Spaces in text don't map to visual spacing
5. **Line wrapping**: Text wraps differently than musical phrases

## A Hybrid Approach: Spatial-Aware Text Editing

### The Idea
Instead of pure text overlay, maintain **spatial metadata** alongside text:

```rust
struct SpatialText {
    text: String,           // "S R G M | P D N S"
    char_positions: Vec<Position>,  // Where each char appears on canvas
    visual_elements: Vec<VisualElement>,  // What to draw at each position
}
```

### Editing Operations Become Spatial

1. **Click on canvas** → Find nearest visual element → Map to text position
2. **Type character** → Insert in text → Recalculate spatial layout
3. **Selection** → Draw custom selection rectangle on canvas (not text selection)
4. **Cursor** → Draw custom cursor on canvas at musical position

### Keep Some Things as Pure Text

Your insight about upper/lower lines is key:
- **Content lines**: Editable with spatial awareness
- **Lyrics lines**: Display only, positioned algorithmically
- **Title/metadata**: Pure text, no spatial handling needed

## The Real Question: What Are We Optimizing For?

### Option A: Text-First (with visual enhancement)
- **Pro**: Version control, portability, simplicity
- **Pro**: Users understand they're editing text
- **Con**: Spatial mismatch always present
- **Con**: Limited visual fidelity

### Option B: Visual-First (with text serialization)
- **Pro**: True WYSIWYG, no spatial mismatch
- **Pro**: Professional music notation quality
- **Con**: Complex implementation
- **Con**: Text becomes secondary

### Option C: Dual Mode (Your Original Vision?)
- **Edit mode**: Show raw text, monospace, simple
- **View mode**: Beautiful canvas rendering
- **Toggle between them**: Like Markdown editors
- **Pro**: Clear mental model
- **Con**: Not truly WYSIWYG

## Rethinking the Canvas Approach

### What if editing stays in "text space" but displays in "music space"?

```
User types: "S R G M"
           ↓
Canvas shows: ♪ ♪ ♪ ♪ (with proper spacing, fonts, etc.)
           ↓
User clicks on third note
           ↓
Cursor appears in text at position 4 (after "G")
           ↓
User types "#"
           ↓
Text becomes: "S R G# M"
           ↓
Canvas redraws with G♯
```

### The Key: Maintain Dual Coordinate Systems

```rust
struct DualCoordinates {
    text_position: usize,      // Character index in string
    visual_position: Point,    // x,y on canvas
    visual_bounds: Rect,       // Bounding box of visual element
}
```

Every element tracks both its text position AND its visual position.

## Practical Implementation Path

### Phase 1: Read-Only Canvas Overlay
- Parse text into visual elements
- Render beautiful music notation
- No editing yet

### Phase 2: Click-to-Position
- Map canvas clicks to text positions
- Show text cursor at correct position
- Still edit as text

### Phase 3: Visual Cursor
- Draw cursor on canvas
- Hide text cursor
- Cursor moves musically (note to note)

### Phase 4: Visual Selection
- Custom selection rendering
- Select musical elements, not characters

### Phase 5: Direct Manipulation
- Drag notes up/down for octave
- Click-and-drag for selection
- But still updating underlying text

## The CodeMirror Lesson

CodeMirror failed because it's fundamentally a **text editor** trying to be spatial. The canvas approach could succeed because it's a **spatial renderer** with text as the data model.

The key insight: **Don't try to make text spatial. Instead, make spatial things that happen to be stored as text.**

## Conclusion: Embrace the Duality

Music notation has always been dual:
- **Symbolic** (the notes, rhythms, pitches)
- **Spatial** (where they appear on the page)

Your overlay approach acknowledges this: text for the symbolic, canvas for the spatial. The challenge is keeping them synchronized while making editing feel natural.

The question isn't "JSON vs text" or "struct vs string" - it's **"how do we map between symbolic and spatial in a way that feels intuitive to edit?"**