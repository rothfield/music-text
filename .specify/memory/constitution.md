<!--
Sync Impact Report:
Version change: initial → 1.1.0
Modified principles: Updated governance philosophy and grammar-first development principle
- Governance: multi-cultural notation → monophonic modal music focus
- Grammar-First: formal EBNF → 2D grammar descriptions, hand-written recursive descent parser
- Spatial Rhythm Fidelity → Spatial Sensitivity
Added principles: Rapid Development Discipline, Modern Rust Practices, Tech Stack Philosophy
Added sections: Core Principles, Quality Standards, Development Workflow, Governance
Removed sections: None
Templates requiring updates: ✅ all templates created
Follow-up TODOs: Notation system mixing policy to be determined
-->

# Music-Text Constitution

## Core Principles

### I. Notation System Support
The parser MUST support multiple notation systems including Sargam, Number, Western, Bhatkhande, and Tabla. The handling of mixed notation systems within documents is under active consideration and will be defined based on user needs and musical use cases.

### II. Grammar-First Development
The grammar specification at `specs/grammar-specification.md` remains the single source of truth. It contains 2D grammar descriptions and is not necessarily formal EBNF or pest grammar. No parser generator is used - the hand-written recursive descent parser MUST follow the specification and use similar naming conventions. All syntax changes MUST begin with specification updates before implementation.

### III. Spatial Sensitivity
The music-text language is spatially sensitive. Spatial positioning of notes directly represents rhythmic timing. Horizontal spacing MUST be preserved exactly as input - no normalization or auto-formatting. The parser MUST maintain character-level precision for spatial relationships between musical elements.

### IV. Multi-Interface Architecture
Every parsing capability MUST be accessible through: CLI with text input/output, programmatic library API, and interactive terminal interface. All interfaces MUST produce equivalent results for identical input. No interface-specific parsing behaviors or limitations.

### V. Rapid Development Discipline
This is a side project under rapid development. Avoid adapters and orphan code. Beware creating v2 code alongside v1. Backwards compatibility is not a concern - prefer clean rewrites over compatibility layers when major changes are needed.

### VI. Modern Rust Practices
Avoid `mod.rs` files - use Rust's modern module system with named module files. Follow current Rust idioms and leverage newer language features rather than legacy patterns.

### VII. Tech Stack Philosophy
Avoid JavaScript whenever possible. Move code to server side rather than client side. Don't worry about performance optimization as a WASM version will come later to address speed concerns.

### VIII. Build System Discipline
ALWAYS use `make build` for development builds. NEVER use `--release` flag during development to prevent ramdisk overflow. Development builds MUST be fast and iterative. Release builds only for production deployment after thorough testing.

## Quality Standards

### Testing Requirements
- Grammar changes require corresponding test cases covering new syntax
- Parsing accuracy tests for all supported notation systems
- Spatial rhythm preservation verification
- CLI interface compatibility testing
- Error message clarity and usefulness validation

### Performance Criteria
- Parse time MUST be sub-second for typical music documents (<1000 lines)
- Memory usage MUST remain reasonable for development on resource-constrained systems
- Build times MUST support rapid iteration cycles

### Documentation Standards
- EBNF grammar specification MUST be complete and unambiguous
- CLI help text MUST be comprehensive and examples-driven
- Error messages MUST guide users toward correct syntax

## Development Workflow

### Change Process
1. Identify syntax or behavior modification need
2. Update grammar specification in EBNF format
3. Write tests demonstrating expected behavior
4. Implement parser changes to match specification
5. Verify all interfaces maintain compatibility
6. Test spatial rhythm preservation
7. Validate build system requirements

### Code Review Requirements
- Grammar specification alignment verification
- Multi-notation system compatibility check
- Spatial fidelity preservation confirmation
- Build system compliance validation
- Error handling and user experience review

### Quality Gates
- All tests MUST pass before merge
- Grammar specification MUST be updated before implementation
- Build using `make build` MUST succeed
- No interface regressions allowed

## Governance

Constitutional amendments require documentation of impact on grammar specification, testing strategy, and all supported interfaces. Changes affecting notation system support or spatial rhythm handling require broader review.

All development decisions MUST align with the project's core philosophy: providing useful textual notation tools rooted in monophonic modal music. While staff notation interchange is supported, the system is not designed to replicate staff notation paradigms.

Version control follows semantic versioning with grammar changes driving version increments.

**Version**: 1.1.0 | **Ratified**: 2025-09-22 | **Last Amended**: 2025-09-22