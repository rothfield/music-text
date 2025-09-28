# Technical Note: WASM Integration Strategy

**Priority: LOW** - Defer until core application features are complete
**Created**: 2025-01-20
**Status**: Planning/Research

## Executive Summary

This document outlines the strategy for migrating the music-text parser from server-side Rust to WebAssembly (WASM) for client-side execution. The migration would eliminate server round-trips for parsing, improving responsiveness and reducing server load.

## Current Architecture

- **Parser**: Rust binary compiled as native executable
- **Web Interface**: JavaScript client calls server API
- **Data Flow**: Client → HTTP POST → Server (Rust) → JSON → Client

## Proposed WASM Architecture

- **Parser**: Rust compiled to WASM, runs in browser
- **Web Interface**: JavaScript directly calls WASM functions
- **Data Flow**: Client → WASM → JSON → Client (no network)

## Migration Approach Comparison

### Option 1: Self-Executing JS (Emscripten) - RECOMMENDED FOR INITIAL MIGRATION

**Advantages:**
- Minimal code changes required
- Existing `main()` function works as-is
- stdio (println!, stdin/stdout) works via Emscripten FS layer
- CLI arguments work: `Module.callMain(["document", "input"])`
- Quick proof of concept possible
- Same binary interface as server version

**Disadvantages:**
- Larger bundle size (~200KB overhead for Emscripten runtime)
- Less efficient memory usage
- No direct DOM access
- Manual memory management for complex interactions

**Implementation Steps:**
1. Add Emscripten target to build
2. Compile with: `emcc` or `cargo build --target wasm32-unknown-emscripten`
3. Load in browser with Emscripten's generated JS loader
4. Call via: `Module.callMain(["document", musicText])`

### Option 2: wasm-bindgen - BETTER FOR PRODUCTION

**Advantages:**
- Type-safe JS/Rust boundary
- Automatic memory management
- Direct DOM/Web API access
- Smaller bundle (with wee_alloc)
- Better debugging support
- Clean async/Promise handling

**Disadvantages:**
- Requires refactoring to expose functions instead of CLI
- Need to restructure I/O (no stdio)
- More initial work

**Implementation Steps:**
1. Refactor core parsing into library functions
2. Add wasm-bindgen attributes
3. Generate JS bindings
4. Import as ES6 module

## Recommended Migration Path

### Phase 1: Proof of Concept (Emscripten)
```javascript
// Minimal changes - use existing CLI binary
const result = Module.callMain(["document", userInput]);
// Parse stdout to get JSON result
```

### Phase 2: Refactor for Production (wasm-bindgen)
```rust
#[wasm_bindgen]
pub fn parse_document(input: &str) -> Result<JsValue, JsValue> {
    let doc = parse::parse_document(input)?;
    Ok(serde_wasm_bindgen::to_value(&doc)?)
}
```

## Performance Considerations

### Expected Benefits:
- **Latency**: ~1ms parsing vs ~50-200ms server round-trip
- **Scalability**: Parsing happens on client machines
- **Offline**: Works without internet connection

### Trade-offs:
- **Initial Load**: +500KB-1MB download (one-time, cacheable)
- **Memory**: Client RAM usage instead of server
- **CPU**: Client CPU for parsing (generally negligible)

## Implementation Checklist (When Ready)

- [ ] Benchmark current server parsing performance
- [ ] Create Emscripten build configuration
- [ ] Test CLI compatibility in WASM
- [ ] Measure WASM bundle size
- [ ] Create fallback to server API for unsupported browsers
- [ ] Implement caching strategy for WASM module
- [ ] Add feature detection for WASM support
- [ ] Refactor to wasm-bindgen (if needed for production)

## Browser Compatibility

- Chrome 57+ (2017)
- Firefox 52+ (2017)
- Safari 11+ (2017)
- Edge 16+ (2017)

All modern browsers have excellent WASM support.

## Decision Criteria for Migration

Migrate to WASM when:
1. Core parsing features are stable
2. Server costs become significant
3. User base grows beyond single server capacity
4. Offline support becomes important
5. Real-time parsing feedback is needed

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Large bundle size | Use compression, lazy loading |
| Browser incompatibility | Fallback to server API |
| Memory limitations | Set input size limits |
| Debugging complexity | Use source maps, console logging |

## Conclusion

WASM integration is a valuable optimization but should be deferred until:
1. Core application features are complete and stable
2. The benefits outweigh the implementation cost
3. Performance bottlenecks are identified and measured

The self-executing JS approach (Emscripten) offers the easiest migration path from the current server-side architecture, allowing us to test WASM viability with minimal code changes. Once proven, we can refactor to wasm-bindgen for production optimization.

## References

- [Emscripten Documentation](https://emscripten.org/docs/compiling/WebAssembly.html)
- [wasm-bindgen Book](https://rustwasm.github.io/wasm-bindgen/)
- [Rust WASM Working Group](https://rustwasm.github.io/)
- [WebAssembly MDN](https://developer.mozilla.org/en-US/docs/WebAssembly)