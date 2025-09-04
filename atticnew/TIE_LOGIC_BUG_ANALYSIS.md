# 🐛 Tie Logic Bug Analysis & Fix Plan

## Problem Statement

The current tie logic in the LilyPond converter is flawed. It simply appends a tie (`~`) to the last string in the `lilypond_notes` vector, which could be a barline instead of a note.

## Root Cause Analysis

### 🔍 **Current Faulty Logic** (`src/to_lilypond_src.rs:25-29`)

```rust
if beat.tied_to_previous && !previous_beat_notes.is_empty() && !beat_notes.is_empty() {
    // Add tie marker to the last note of the previous beat
    if let Some(last_note) = lilypond_notes.last_mut() {
        if !last_note.ends_with('~') {
            *last_note = format!("{}~", last_note);  // ❌ BUG: Could be barline!
        }
    }
}
```

### 🧪 **Test Case Demonstrating Bug**

**Input**: `S- | S` (should produce tied notes across barline)

**Current FSM Output**:
```
Beat(S, tied_to_previous: false)  ← ❌ Should be true!
Barline("|")
Beat(S, tied_to_previous: false)
```

**Current LilyPond Output**: `c4 \bar "|" d4` (no tie)  
**Expected LilyPond Output**: `c4~ \bar "|" c4` (with tie)

## 🎯 **Two-Level Problem**

### Level 1: FSM Logic Issue (Primary)
The FSM doesn't properly detect when notes should be tied across barlines.

**Current FSM Processing for `S- | S`**:
1. `S` → `start_beat_pitch()` → State::InBeat, `tied_to_previous: false`
2. `-` → (InBeat processing) → extends current beat (correct)
3. `|` → `finish_beat()` → emit barline → State::S0 
4. `S` → `start_beat_pitch()` → **❌ `tied_to_previous: false` (WRONG!)**

**The Issue**: When starting a new beat after a barline, the FSM doesn't check if the previous beat had an extension that should create a tie.

### Level 2: LilyPond Converter Issue (Secondary)  
Even if FSM were fixed, the tie application logic is unsafe.

**Current Logic Issues**:
- Blindly modifies `lilypond_notes.last_mut()`
- Doesn't verify the last item is actually a note
- Could append `~` to barlines: `\bar "|"~` (invalid LilyPond)

## 🔧 **Fix Plan**

### Phase 1: Fix FSM Tie Detection Logic

**File**: `src/rhythm_fsm.rs`

**Problem**: Need to detect when a dash-extended note should tie to the next note of the same pitch across barlines.

**Solution**: Track "pending tie" state across barlines.

```rust
struct RhythmFSM {
    // ... existing fields
    pending_tie_pitch: Option<PitchCode>,  // NEW: Track pitch that needs tying
}

impl RhythmFSM {
    fn finish_beat(&mut self) {
        if let Some(mut beat) = self.current_beat.take() {
            // Check if this beat ends with an extended note (from dash)
            if let Some(last_element) = beat.elements.last() {
                if last_element.is_note() && /* was extended by dash */ {
                    self.pending_tie_pitch = last_element.degree;
                }
            }
            // ... existing logic
        }
    }
    
    fn start_beat_pitch(&mut self, element: &ParsedElement) {
        let tied_to_previous = if let Some(pending_pitch) = self.pending_tie_pitch {
            // Check if this note matches the pending tie pitch
            element.degree == Some(pending_pitch)
        } else {
            false
        };
        
        let mut beat = Beat { 
            divisions: 1, 
            elements: vec![], 
            tied_to_previous,  // ✅ Correctly set based on pending tie
            is_tuplet: false, 
            tuplet_ratio: None 
        };
        
        // Clear pending tie after processing
        if tied_to_previous {
            self.pending_tie_pitch = None;
        }
        
        // ... rest of logic
    }
}
```

### Phase 2: Fix LilyPond Converter Safety

**File**: `src/to_lilypond_src.rs`

**Problem**: Unsafe tie application to potentially non-note items.

**Solution**: Find and modify only actual notes, not barlines.

```rust
fn find_last_note_index(lilypond_notes: &[String]) -> Option<usize> {
    // Search backwards for the last actual note (not barline, breathmark, etc.)
    for (i, note) in lilypond_notes.iter().enumerate().rev() {
        if !note.starts_with("\\bar") && !note.starts_with("\\breathe") 
           && !note.starts_with("\\") {  // Skip LilyPond commands
            return Some(i);
        }
    }
    None
}

// In main loop:
if beat.tied_to_previous && !previous_beat_notes.is_empty() && !beat_notes.is_empty() {
    // ✅ SAFE: Find last actual note, not just last item
    if let Some(last_note_index) = find_last_note_index(&lilypond_notes) {
        let last_note = &mut lilypond_notes[last_note_index];
        if !last_note.ends_with('~') {
            *last_note = format!("{}~", last_note);
        }
    }
}
```

## 🧪 **Test Cases**

### Primary Test Cases
1. **Cross-barline tie**: `S- | S` → `c4~ \bar "|" c4`
2. **No tie (different pitches)**: `S- | R` → `c4 \bar "|" d4` 
3. **Multiple barlines**: `S- | | S` → `c4~ \bar "|" \bar "|" c4`
4. **Complex sequence**: `S- | R | S-` → `c4~ \bar "|" d4 \bar "|" c4`

### Edge Cases
1. **Barline without following note**: `S- |` → `c4 \bar "|"`
2. **Multiple extensions**: `S-- | S` → `c2~ \bar "|" c4`
3. **With breath marks**: `S- ' | S` → `c4 \breathe \bar "|" c4` (no tie - breath breaks)

## 🎯 **Expected Outcomes**

### After FSM Fix:
```
Beat(S-, tied_to_previous: false, extended: true)
Barline("|")  
Beat(S, tied_to_previous: true)  ← ✅ Correctly detects tie needed
```

### After LilyPond Fix:
```lilypond
c4~ \bar "|" c4  ← ✅ Tie applied to note, not barline
```

### After Both Fixes:
- ✅ Correct tie detection across barlines
- ✅ Safe tie application (never to barlines)
- ✅ Proper musical semantics (ties only between same pitches)
- ✅ Handles complex sequences with multiple barlines

## 🚀 **Implementation Priority**

1. **Phase 1 (Critical)**: Fix FSM tie detection - this is the root cause
2. **Phase 2 (Safety)**: Fix LilyPond converter safety - prevents invalid output
3. **Phase 3 (Validation)**: Comprehensive testing with all edge cases

The FSM fix is more critical because even if the LilyPond converter were made safe, it would still produce wrong output (no ties when there should be ties) without the FSM fix.

---

*This analysis demonstrates that the bug has two components: incorrect FSM logic (primary) and unsafe tie application (secondary). Both need fixing for correct tie rendering across barlines.*