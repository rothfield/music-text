# Document-First Architecture

## Overview

The system is transitioning from a **text-format-centric** to a **document-model-centric** architecture. This fundamentally changes how we think about the music-text format and interactive editing.

## Architectural Shift

### Before: Format-Locked
```
User Input → Music-Text Parser → Document → Render
```
- Music-text format is the primary interface
- All operations require parsing
- Tied to specific syntax forever

### After: Document-First
```
User Interaction → Document Model → Render
Music-Text Import/Export ← Document Model → Other Formats
```
- Document model is the primary interface
- Parsing only at format boundaries
- Format-agnostic core operations

## Key Changes

### Interactive Editing
- **Document transforms** instead of text manipulation
- **UUID-based selection** instead of character indices
- **Incremental re-flow** instead of full re-parsing
- **Server-owned document state** instead of text synchronization

### Format Support
- Music-text becomes **one supported format** among many
- Core editor operations work on **document model directly**
- Import/export adapters for **multiple notation formats**
- **Legacy format support** without architectural constraints

## Performance Benefits

### Parser Avoidance
- Recursive descent parser only runs at format boundaries
- Interactive operations are O(1) or O(affected_elements)
- No grammar validation overhead for model operations

### Incremental Processing
- Only re-analyze spatial/rhythm when needed
- Cache expensive renders (SVG, LilyPond) until invalidated
- Smart re-flow based on transform type

## WASM Readiness

This architecture maps perfectly to WASM deployment:
- Document model lives in WASM memory
- No serialization overhead for operations
- Same semantic command interface
- Format conversion at WASM boundaries

## Strategic Impact

The editor evolves from a **"music-text editor"** to a **"universal structured music notation editor"** that happens to support music-text format. This enables:

- Multi-format interoperability (MusicXML, ABC, etc.)
- Future format support without core changes
- Performance optimization opportunities
- Clean separation of concerns

**Date**: 2025-09-25
**Status**: Architecture direction established, implementation in progress