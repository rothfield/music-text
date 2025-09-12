# Git Repository Reorganization Plan

## Current Status Analysis

The repository currently has significant changes that need to be organized into logical commits. The recent "checkpoint" commit (e6aa61a) captured major architectural changes, but there are still pending modifications that should be properly committed.

## Remaining Uncommitted Changes

### Build System Updates
- `Cargo.lock` - Dependency resolution updates (mustache, uuid, getrandom, etc.)
- `Cargo.toml` - Added mustache dependency for templating

### Enhanced Parsing Infrastructure 
- `CLAUDE.md` - Critical parser requirement documented: barlines now required for valid input
- `src/document/model.rs` - Complete PitchCode enum expansion (35 total variants with double sharps/flats)
- `src/lib.rs` - Added converters module export
- `src/web_server.rs` - Enhanced API with direct notation-to-SVG generation

### Build Artifacts (To Be Ignored)
- `target/debug/` files - Build outputs that should not be committed

### Documentation
- `LILYPOND_TEMPLATE_RESEARCH.md` - Research notes (untracked)

## Proposed Reorganization Strategy

### Phase 1: Infrastructure Commit
**Commit Message**: "Enhance parsing infrastructure with complete pitch coverage and converters"

**Files to include**:
- `Cargo.toml` (mustache dependency)
- `Cargo.lock` (dependency updates) 
- `src/document/model.rs` (complete PitchCode enum)
- `src/lib.rs` (converters module)
- `CLAUDE.md` (parser barline requirement)

### Phase 2: API Enhancement Commit  
**Commit Message**: "Add direct notation-to-SVG API endpoint for web interface"

**Files to include**:
- `src/web_server.rs` (enhanced API)

### Phase 3: Documentation Commit (Optional)
**Commit Message**: "Add LilyPond template research documentation"

**Files to include**:
- `LILYPOND_TEMPLATE_RESEARCH.md` (if user wants to commit research notes)

## Rationale

This organization separates concerns clearly:
1. **Infrastructure changes** - Core parsing and data model enhancements that affect the entire system
2. **API changes** - Web interface improvements that use the infrastructure 
3. **Documentation** - Research notes that don't affect code behavior

Each commit represents a logical unit of work that could be reviewed, tested, or reverted independently.

## Implementation Notes

- Build artifacts in `target/debug/` should be added to `.gitignore` if not already present
- The dependency updates in `Cargo.lock` are necessary due to the mustache addition in `Cargo.toml`
- The PitchCode enum expansion is a significant enhancement supporting 35 pitch variants (7 base pitches × 5 accidental states)
- The barline requirement is a critical parser constraint that affects all testing and usage

## Alternative: Single Comprehensive Commit

If preferred, all changes could be combined into one commit:
**Message**: "Major infrastructure enhancements: complete pitch coverage, converters module, and enhanced web API"

This would include all files except build artifacts, providing a single atomic update of the system's capabilities.