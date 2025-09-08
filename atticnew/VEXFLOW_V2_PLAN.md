# VexFlow V2 Analysis and Plan

## Current State Analysis

### 🔍 What We Found

1. **VexFlow Converter Exists** but produces empty output `[]`
2. **Root Cause**: VexFlow converter expects `LINE` nodes but V2 parser produces flat structure
3. **Data Structure Mismatch**: 
   - VexFlow converter: `document.nodes` → looking for `node_type == "LINE"`
   - V2 Parser output: Flat `ParsedElement` list → converted to flat `Node` list
4. **FSM V2 Output Available**: We have beat structure with timing information
5. **Current Flow**: V2 Parser → V1 Document (flat) → VexFlow Converter (expects hierarchical)

### 📊 Debug Evidence
```
--- VexFlow JSON Output ---
[]
```

The converter finds no `LINE` nodes because V2 produces:
```
document.nodes: [pitch{G}, pitch{S}, pitch{P}, barline{|}, pitch{D}]
```

But VexFlow converter expects:
```
document.nodes: [LINE{ nodes: [BEAT{nodes: [...]}, BARLINE, BEAT{nodes: [...]}] }]
```

## 🎯 Simplified Plan

### Phase 1: Create VexFlow V2 Converter (Priority: HIGH)
**Goal**: Direct FSM → VexFlow conversion, no V1 compatibility

1. **Replace vexflow_fsm_converter.rs entirely**
   - Input: `Vec<OutputItemV2>` from FSM
   - Output: `Vec<VexFlowStave>` 
   - **Delete all V1 compatibility code** - no LINE/BEAT node processing

2. **Focus on Single Notation System**
   - **Sargam only** (since everything is PitchCodes internally)
   - Western/Numbers can be added later once core works

3. **Basic Testing Strategy**
   - `G S P D` → 4 quarter notes in VexFlow JSON
   - `G S | P D` → verify barline creates new measure
   - Verify JSON structure matches what web UI expects

### Phase 2: Integration
4. **Update CLI** - remove old VexFlow path, use V2 only
5. **Update Web Server** - use V2 VexFlow output directly
6. **Browser Test** - verify one simple case renders

### Core Conversion Logic
```rust
pub fn convert_fsm_v2_to_vexflow(fsm_output: &[OutputItemV2]) -> Vec<VexFlowStave> {
    // Process beats directly, no LINE nodes needed
    // OutputItemV2::Beat → VexFlow notes with durations
    // OutputItemV2::Barline → new measure
}
```

**Key Insight**: Skip all hierarchical document structure - work directly with FSM beats and PitchCodes.

## Implementation Notes

### Data Flow
```
Raw Text → V2 Parser → ParsedElements → FSM V2 → OutputItemV2 → VexFlow JSON
```

### VexFlow JSON Target Structure
```json
[
  {
    "notes": [
      {
        "keys": ["g/4"],
        "duration": "q"
      },
      {
        "keys": ["c/4"], 
        "duration": "q"
      }
    ],
    "key_signature": null
  }
]
```

### Testing Strategy
- **Single system focus**: Sargam notation only
- **Core functionality**: Notes + durations + barlines
- **Validation**: CLI output → Web UI rendering
- **No backward compatibility**: Clean slate implementation

### Success Criteria
- [ ] CLI produces non-empty VexFlow JSON for simple input
- [ ] Web UI can render the JSON without errors  
- [ ] Basic notation displays correctly in browser