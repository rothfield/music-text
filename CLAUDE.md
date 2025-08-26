# Claude Code Assistant Memory

## Important Project Guidelines

### Development Workflow
- **Testing Requirement**: The job is not done until the web UI is actually tested
- Always verify fixes work in the browser interface, not just in backend code
- Check console output and visual rendering to confirm solutions

### Architecture
- **Backend**: Rust notation parser that generates VexFlow JSON
- **Frontend**: Web interface at http://localhost:3000 that renders VexFlow notation
- **Integration**: Client-side JavaScript processes backend output for VexFlow rendering

### Recent Issues Fixed
1. **Slur positioning** - Fixed spatial analysis to prevent multiple slur end markers
2. **Tie logic** - Only create ties between notes of same pitch (correct musical definition)
3. **VexFlow crash** - Fixed slur indexing by separating notes from barlines in array handling

### Key Files
- `src/spatial_analysis.rs` - Slur positioning logic
- `src/vexflow_fsm_converter.rs` - Tie detection and VexFlow JSON generation  
- `web/index.html` - Client-side VexFlow rendering and slur creation
- `src/lilypond_converter.rs` - LilyPond output generation

### Testing Commands
- `cargo build --release` - Build backend
- `node server.js` - Start web server (port 3000)
- Test with notation like `_________` over `G -P | S` in the web interface

### Current Status
- Fixed VexFlow NoStem crash by creating separate noteOnlyArray for slur indexing
- Need to test the web UI to verify slurs render correctly across barlines