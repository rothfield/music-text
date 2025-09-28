# Refactor Stave Processing into Clear Phases Plan

## Current State Analysis

**Current `parse_stave_from_paragraph()`** does everything mixed together:
- Line identification, content parsing, spatial analysis all interleaved
- Returns `Stave` but rhythm processing happens elsewhere per-stave

**Desired Clean Phase Architecture:**
```rust
pub fn process_stave(paragraph: &str, start_line: usize) -> Result<ProcessedStave, ParseError> {
    // Phase 1: Identify content line
    let content_line_index = identify_content_line(lines)?;
    
    // Phase 2: Parse main musical content (tokenization)
    let content_elements = parse_content_line(content_line_text)?;
    
    // Phase 3: Parse spatial annotations above content
    let upper_lines = parse_upper_lines(&lines[..content_line_index])?;
    
    // Phase 4: Parse spatial annotations below content  
    let lower_lines = parse_lower_lines(&lines[content_line_index + 1..])?;
    
    // Return spatially-complete stave (ready for batch rhythm processing)
    Ok(ProcessedStave { content_elements, upper_lines, lower_lines, ... })
}
```

## Implementation Plan

### Phase 1: Extract Line Identification Logic
1. **Create `identify_content_line()`** in `stave.rs`:
   - Extract the content line detection logic from current function
   - Use existing `is_content_line()` logic with 3+ musical elements
   - Return content line index and validation

### Phase 2: Restructure Content Line Parsing
1. **Rename `parse_main_line()` → `parse_content_line()`**:
   - This is the tokenization phase that converts text → `Vec<ParsedElement>`
   - Already implemented, just needs renaming for consistency

### Phase 3: Implement Spatial Analysis Functions
1. **Create `parse_upper_lines()`** in `stave.rs`:
   - Process lines above content line using existing `parse_upper_line()`
   - Handle octave markers, slurs, ornaments spatially positioned above notes

2. **Create `parse_lower_lines()`** in `stave.rs`:
   - Process lines below content line using existing `parse_lower_line()`
   - Handle octave markers, beat groups, lyrics spatially positioned below notes

### Phase 4: Update Pipeline Integration
1. **Modify `parse_document()`** to call new `process_stave()`:
   - Replace current `parse_stave_from_paragraph()` calls
   - Each stave comes out fully spatially analyzed

2. **Update batch rhythm processing**:
   - Move rhythm FSM processing out of per-stave loop
   - Create `process_rhythm_batch(all_spatially_complete_staves)`

### Phase 5: Clean Architecture Separation
1. **Clear Pipeline Flow**:
   ```
   Text → parse_document() → 
     for each paragraph: process_stave() → Document{spatially_complete_staves} →
     process_rhythm_batch(all_staves) → 
     Vec<RhythmProcessedStave> → Renderers
   ```

2. **Function Responsibilities**:
   - `identify_content_line()`: Find the musical content
   - `parse_content_line()`: Tokenize musical elements  
   - `parse_upper_lines()`: Spatial analysis above content
   - `parse_lower_lines()`: Spatial analysis below content
   - `process_rhythm_batch()`: FSM rhythm processing across all staves

## Expected Benefits

### ✅ **Clear Separation of Concerns:**
- **Line Identification**: Distinct phase
- **Content Tokenization**: Distinct phase  
- **Spatial Analysis**: Distinct phases (upper/lower)
- **Rhythm Processing**: Separate batch operation

### ✅ **Better Testability:**
- Each phase can be unit tested independently
- Easier to debug spatial analysis issues
- Clear interfaces between phases

### ✅ **Maintainability:**
- Each function has single responsibility
- Easier to extend spatial analysis features
- Clear data flow through pipeline

## Risk Mitigation
- **Preserve existing logic**: Extract and reorganize, don't rewrite
- **Maintain compatibility**: Same input/output behavior for pipeline
- **Gradual refactor**: Move functions but keep same processing results

## Implementation Status

### ✅ Completed:
- [x] Plan documented
- [x] Todo list created

### 🔄 In Progress:
- [ ] Extract identify_content_line() logic
- [ ] Rename parse_main_line() to parse_content_line()
- [ ] Create parse_upper_lines() function
- [ ] Create parse_lower_lines() function
- [ ] Refactor process_stave() with clear phases
- [ ] Move rhythm processing to batch operation
- [ ] Test the refactored pipeline

## Next Steps
1. Start with extracting `identify_content_line()` from existing function
2. Rename `parse_main_line()` for consistency
3. Implement spatial analysis functions
4. Refactor main `process_stave()` function
5. Update pipeline integration
6. Test complete refactored architecture