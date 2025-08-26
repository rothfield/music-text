# TypeScript Modular Architecture

This document describes the TypeScript modular architecture for the notation parser web interface.

## Overview

The original 2500+ line monolithic HTML file has been refactored into a clean, type-safe TypeScript modular architecture. This provides better maintainability, debugging, and LLM-assisted development.

## Architecture

### Type Safety First
- **Strong typing** for all VexFlow objects and notation data structures
- **Compile-time error checking** prevents runtime crashes
- **IntelliSense support** for better development experience
- **Interface contracts** between modules

### Module Structure

```
src-ts/
├── types/
│   └── notation.ts           # Core type definitions
├── modules/
│   ├── vexflow-renderer.ts   # VexFlow rendering engine
│   ├── wasm-interface.ts     # WASM module integration
│   └── ui-controller.ts      # UI event handling & coordination
└── main.ts                   # Application entry point
```

## Key Types

### `NotationElement`
Core interface for parsed notation elements:
```typescript
interface NotationElement {
  type: 'Note' | 'Rest' | 'BarLine' | 'SlurStart' | 'SlurEnd' | 'Tuplet';
  keys?: string[];              // VexFlow note keys
  duration?: string;            // Note duration
  dots?: number;               // Dotted notes
  tied?: boolean;              // Tie to next note
  // ... additional properties
}
```

### `StaveData`
Represents a complete musical stave:
```typescript
interface StaveData {
  notes: NotationElement[];
  key_signature?: string | null;
}
```

### `ParserResponse`
WASM parser output format:
```typescript
interface ParserResponse {
  staves?: StaveData[];
  error?: string;
}
```

## Core Modules

### 🎼 VexFlowRenderer
**Purpose:** Converts parsed notation data to VexFlow visual output

**Key Features:**
- **Separated indexing** - `vexNotes[]` vs `noteOnlyArray[]` to fix slur crashes
- **Stem validation** - Ensures notes have proper stems before creating slurs
- **Cross-barline slurs** - Properly handles slurs spanning measures
- **Type-safe VexFlow** - Strong typing for all VexFlow objects

**Critical Fix:** Prevents VexFlow "NoStem" crashes by separating barlines from note indexing.

### 🔧 WasmInterface
**Purpose:** Manages WASM module loading and notation parsing

**Key Features:**
- **Async initialization** - Proper WASM loading with timeout handling
- **Error handling** - Graceful fallback for WASM failures
- **Debug commands** - Exposes version info and parsing functions

### 🖱️ UIController
**Purpose:** Coordinates UI interactions and data flow

**Key Features:**
- **Debounced parsing** - Prevents excessive parsing on input
- **Live preview** - Real-time VexFlow rendering
- **Event management** - Clean separation of UI logic

## Development Commands

### Build & Development
```bash
# One-time build
npm run build

# Watch mode (auto-rebuild on changes)
npm run watch

# Development (watch + server)
npm run dev

# Server only
npm run start
```

### File Watching
The TypeScript compiler watches `src-ts/` and outputs to `web/js/`:
```
src-ts/main.ts → web/js/main.js
src-ts/modules/vexflow-renderer.ts → web/js/modules/vexflow-renderer.js
```

## Testing

### Available Versions
- **Original**: http://localhost:3000 (`web/index.html`)
- **TypeScript**: http://localhost:3000/index-ts.html (`web/index-ts.html`)

### Development Workflow
1. Start TypeScript watch: `npm run watch`
2. Start server: `npm run start`
3. Edit TypeScript files in `src-ts/`
4. Test at http://localhost:3000/index-ts.html
5. Check browser console for debug output

### Debug Commands
Available in browser console:
```javascript
// WASM functions
wasm.get_version()
wasm.get_build_timestamp()  
wasm.parse_notation("| S R G M |")

// Self-test functions
runSelfTest()
showSelfTestResults()
```

## Key Fixes Implemented

### 🎯 Slur Rendering Fix
**Problem:** VexFlow crashed with "NoStem: No stem attached to this note"
**Root Cause:** Barlines were included in note indexing for slurs
**Solution:** 
- Created `noteOnlyArray[]` for slur indexing (excludes barlines)
- Added stem validation before creating curves
- Fixed slur end placement in beats with multiple notes

### 🎵 LilyPond Output Fix  
**Problem:** Slurs ended at wrong notes (E to G instead of E to C)
**Solution:** Fixed beat pitch counting in `add_slur_markers_to_line`

## Configuration

### TypeScript Config (`tsconfig.json`)
```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ES2020", 
    "outDir": "./web/js",
    "rootDir": "./src-ts",
    "strict": true,
    "sourceMap": true
  }
}
```

### Package Scripts
```json
{
  "scripts": {
    "build": "tsc",
    "watch": "tsc --watch", 
    "dev": "tsc --watch & node server.js",
    "start": "node server.js"
  }
}
```

## Benefits of TypeScript Architecture

### For Development
- ✅ **Type safety** prevents runtime errors
- ✅ **Modular code** is easier to test and debug
- ✅ **Clear interfaces** between components
- ✅ **Better IDE support** with autocomplete

### For LLMs
- ✅ **Self-documenting** types explain data structures
- ✅ **Safer refactoring** with compile-time checks
- ✅ **Clear module boundaries** for focused changes
- ✅ **Better code generation** with type context

## Migration Notes

The TypeScript version maintains **full compatibility** with the original functionality while providing:

1. **Same features** - All original capabilities preserved
2. **Better structure** - Clean separation of concerns  
3. **Type safety** - Prevents many categories of bugs
4. **Easier maintenance** - Modular code is easier to modify

## Future Improvements

- [ ] Add unit tests for each module
- [ ] Implement proper VexFlow type definitions (currently using basic declarations)
- [ ] Add error boundaries and better error handling
- [ ] Create build pipeline for production bundling
- [ ] Add hot reload for faster development

---

**Remember:** The job is not done until the web UI is tested! 🎯
Always verify changes work in both browser versions before considering a task complete.