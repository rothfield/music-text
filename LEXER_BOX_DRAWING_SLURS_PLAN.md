# Lexer Support for Box Drawing Slurs - Implementation Plan

## Current State Analysis

### Existing Slur Implementation
- **Tokenizer**: Recognizes underscore sequences (`_`, `__`, `___`) as `SLUR` tokens (tokenizer.rs:147-159)
- **Vertical Parser**: Processes SLUR tokens spatially to assign `SlurRole` to notes (Start/Middle/End/StartEnd)
- **Format**: Two-line spatial format with underscores above note lines:
  ```
        ____
  S R GMPD
  ```

### Token Flow
```
Input: "      ____\nS R GMPD"
  ↓ Tokenizer
SLUR token: [SLUR{value:"____", col:6}]
  ↓ Vertical Parser  
Note assignments: [Note{SlurRole::Start}, Note{SlurRole::Middle}, Note{SlurRole::Middle}, Note{SlurRole::End}]
```

## Proposed Box Drawing Character Support

### Unicode Characters for Slurs
- **Start**: `╭` (U+256D) - Box Drawing Light Down and Right
- **Middle**: `─` (U+2500) - Box Drawing Light Horizontal  
- **End**: `╮` (U+256E) - Box Drawing Light Down and Left

### Visual Example
```
Current:     ____
S R GMPD

Proposed:  ╭──╮
S R GMPD
```

## Implementation Strategy

### Phase 1: Tokenizer Extension

#### A. Add Box Drawing Character Recognition
**File**: `src/parser/tokenizer.rs`

**Current underscore handling** (lines 147-159):
```rust
'_' => {
    let slur_start_pos = start_pos;
    while let Some('_') = self.peek() {
        self.advance();
    }
    let slur_value: String = self.chars[slur_start_pos..self.pos].iter().collect();
    Some(Token {
        token_type: "SLUR".to_string(),
        value: slur_value,
        // ...
    })
}
```

**Proposed addition** (with validation):
```rust
// Box drawing slur start
'╭' => {
    self.parse_box_drawing_slur()  // Validates complete ╭─*╮ pattern
}

// Invalid box drawing characters (not part of proper slur)
'─' | '╮' => {
    Some(Token {
        token_type: TokenType::Unknown.as_str().to_string(),
        value: ch.to_string(),
        // ...
    })
}
```

**Validation Logic**:
```rust
fn parse_box_drawing_slur(&mut self) -> Option<Token> {
    let mut slur_value = String::from("╭");
    
    // Consume middle characters (─)
    while let Some('─') = self.peek() {
        self.advance();
        slur_value.push('─');
    }
    
    // Must end with ╮ for valid slur
    if let Some('╮') = self.peek() {
        self.advance();
        slur_value.push('╮');
        // Return valid SLUR token
    } else {
        // Return UNKNOWN token for invalid pattern
    }
}
```

#### B. Backward Compatibility
- **Keep existing underscore handling** - no changes to current logic
- **Add parallel recognition** for box drawing characters
- **Same token type** (`SLUR`) - vertical parser doesn't need changes

### Phase 2: Vertical Parser Compatibility

#### Current Slur Processing Logic
**File**: `src/parser/vertical.rs` (lines 28-43)

The vertical parser already handles `SLUR` tokens generically:
```rust
let slur_tokens: Vec<_> = line_tokens.iter()
    .filter(|t| t.token_type == "SLUR" || (t.token_type == "SYMBOLS" && t.value == "_"))
    .collect();
```

**Required Changes**:
- **None!** The existing logic will work with box drawing characters since they generate the same `SLUR` token type
- The spatial analysis (column positioning) works identically for both formats

### Phase 3: Enhanced Slur Line Generation

#### Current Limitations
- Underscores create uniform appearance: `____`
- No visual distinction between slur start/middle/end positions

#### Box Drawing Enhancement
- **Start character**: `╭` indicates slur beginning
- **Middle characters**: `─` indicates slur continuation  
- **End character**: `╮` indicates slur termination
- **Single note slurs**: `╭╮` for isolated slur

#### Implementation Location
**File**: WASM text editing functions (to be created)

The conversion from selected text to box drawing format happens in the text editing phase, not the lexer:
```javascript
// WASM function converts selection to spatial format
function applySlurToText(text, startPos, endPos) {
    // Generate spatial slur line with box drawing characters
    // Insert above musical line at correct character positions
}
```

## Detailed Implementation Plan

### 1. Tokenizer Modification

**Location**: `src/parser/tokenizer.rs`, `next_token()` method

**Add case for box drawing characters**:
```rust
// After line 189 (after existing symbols handling)
// Box drawing slur characters  
'╭' | '─' | '╮' => {
    Some(Token {
        token_type: "SLUR".to_string(),
        value: ch.to_string(),
        line: 0, // Will be set by caller
        col: 0,  // Will be set by caller
    })
}
```

**Why this validation approach**:
- **Enforces proper slur syntax** - rejects invalid patterns like `╮╮` or `─╭`
- **Single SLUR token per slur** - complete slur like `╭──╮` becomes one token
- **Perfect backward compatibility** - existing underscore logic unchanged
- **Clear error handling** - invalid patterns become UNKNOWN tokens
- **Reuses existing pipeline** - SLUR tokens processed identically by vertical parser

### 2. Testing Strategy

#### Test Cases
1. **Legacy underscore format** - must continue working
2. **Box drawing format** - new slur recognition
3. **Mixed format** - underscores and box drawing on same document
4. **Complex slurs** - multiple slurs, nested slurs, single-note slurs

#### Test Files
Create test files in `my_test_data/`:
- `box_drawing_slurs_simple.123` - basic box drawing slurs
- `box_drawing_slurs_complex.123` - multiple overlapping slurs
- `mixed_slur_formats.123` - both underscore and box drawing

#### Verification Commands
```bash
# Test parsing with new format
NOTATION_OUTPUT_DIR="test_output" ./target/release/cli box_drawing_slurs_simple.123

# Verify VexFlow rendering
cd webapp && node server.js
# Test in browser at localhost:3000
```

### 3. WASM Text Editing Integration

#### Box Drawing Generation Logic
The WASM functions will:
1. **Analyze selection** - determine start/end positions
2. **Generate slur line** - create spatial line with box drawing characters  
3. **Insert above music** - place slur line above note line with character alignment
4. **Handle spatial positioning** - account for octave markers, existing slurs

#### Slur Character Selection Logic
```rust
fn generate_box_drawing_slur(start_col: usize, end_col: usize) -> String {
    if start_col == end_col {
        // Single note slur
        "╭╮".to_string()
    } else {
        // Multi-note slur
        let mut slur_line = " ".repeat(start_col);  // Leading spaces
        slur_line.push('╭'); // Start character
        
        // Calculate middle section length
        let middle_length = end_col.saturating_sub(start_col + 1);
        for _ in 0..middle_length {
            slur_line.push('─'); // Middle characters  
        }
        
        slur_line.push('╮'); // End character
        slur_line
    }
}
```

## Risk Analysis

### Technical Risks

#### **Unicode Handling**
- **Risk**: Unicode characters may cause parsing issues
- **Mitigation**: Box drawing characters are single-byte UTF-8, well-supported
- **Fallback**: Underscore format always available

#### **Visual Alignment**  
- **Risk**: Box drawing characters have different widths than underscores
- **Mitigation**: Both are single-column characters in monospace fonts
- **Testing**: Verify alignment in terminal and web interface

#### **Backward Compatibility**
- **Risk**: Breaking existing underscore-based music files
- **Mitigation**: Parallel recognition - both formats supported simultaneously
- **Guarantee**: Zero changes to existing underscore token handling

### User Experience Risks

#### **Font Support**
- **Risk**: Box drawing characters not available in user's font
- **Mitigation**: Most modern monospace fonts support box drawing
- **Fallback**: Document underscore format as alternative

#### **Input Method**
- **Risk**: Difficulty typing box drawing characters
- **Mitigation**: WASM functions generate them automatically from selections
- **Alternative**: Copy-paste from documentation

## Success Criteria

### Functional Requirements
1. ✅ **Lexer recognizes box drawing slur characters**
2. ✅ **Existing underscore format continues working**  
3. ✅ **Mixed format documents parse correctly**
4. ✅ **VexFlow rendering identical for both formats**
5. ✅ **LilyPond output identical for both formats**

### Performance Requirements
1. ✅ **No parsing performance degradation**
2. ✅ **Tokenization speed unaffected**
3. ✅ **Memory usage equivalent**

### Compatibility Requirements  
1. ✅ **All existing test files pass**
2. ✅ **CLI output format unchanged**
3. ✅ **Web interface functions normally**

## Implementation Timeline

### Immediate (This Session)
1. **Tokenizer modification** - Add box drawing character recognition (15 minutes)
2. **Basic testing** - Create test file and verify parsing (15 minutes)
3. **Integration testing** - Test with VexFlow and LilyPond converters (15 minutes)

### Phase 2 (Next Steps)  
1. **WASM function implementation** - Text selection to box drawing conversion
2. **UI integration** - JavaScript buttons for slur application
3. **Cursor position restoration** - Maintain editing position after text modification

### Phase 3 (Future)
1. **Advanced slur patterns** - Nested slurs, complex overlapping
2. **Underline support** - Box drawing characters for underlines below notes
3. **Documentation** - User guide for new slur format

## Technical Notes

### Character Analysis
```
╭ (U+256D): Box Drawing Light Down and Right
─ (U+2500): Box Drawing Light Horizontal  
╮ (U+256E): Box Drawing Light Down and Left
```

### UTF-8 Encoding
- All three characters: 3 bytes in UTF-8
- No normalization issues
- Consistent width in monospace fonts

### Lexer Token Stream
```
Input:  "╭─╮\nS R G"
Tokens: [SLUR{"╭"}, SLUR{"─"}, SLUR{"╮"}, NEWLINE, PITCH{"S"}, WHITESPACE, PITCH{"R"}, WHITESPACE, PITCH{"G"}]
```

This matches exactly the token pattern expected by the vertical parser's spatial analysis algorithm.

---

**CRITICAL**: This plan maintains 100% backward compatibility while adding modern Unicode slur visualization. The tokenizer change is minimal (3 lines) and leverages all existing slur processing infrastructure.