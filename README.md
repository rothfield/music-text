# Music Text Project

## Vision: Tool Suite for Textual Music Notation

We're building a comprehensive suite of tools for textual music notation - an ecosystem that provides an alternative to staff-notation-centric music software.

## The Philosophy

**Build a suite of small, focused, interoperable tools**
- Each tool does one thing well
- Tools work together via standard interfaces
- Create a working environment for textual notation

**Bidirectional workflow:**
- **Forward**: Jot down notation → Typeset beautifully → Interchange with rest of world (LilyPond, MusicXML, etc.)
- **Reverse**: World → Sargam (convert existing formats into practical textual notation)

## Tools

- **Editor**: Input and edit notation in real-time
- **Typesetter**: Beautiful visual output
- **Converter**: Import/export to other music formats
- **Validator**: Check notation syntax and structure
- **Transformer**: Manipulate notation programmatically
- **Renderer**: Multiple output formats (PDF, SVG, etc.)
- **Library**: Shared notation processing functions

## Background

This builds on the notation system used at the AACM (Association for the Advancement of Creative Musicians) since the 1970s for capturing music in real-time. Musicians developed practical ways to write down what they heard as it happened - whether in lessons, performances, or jam sessions.

The tools help with the same things musicians have always needed:
- Quick notation that keeps up with musical ideas
- Clean scores for performance and study
- Ways to share music with other musicians and software
- Preserving musical knowledge for future generations

## The Goal

Create a rich, comprehensive ecosystem where musicians can work entirely in textual notation while seamlessly interfacing with the broader music world.

## Status

**What is done:**
- Basic editor interface
- Core typesetting engine
- Export to SVG/PDF formats
- Export to LilyPond
- Real-time staff notation preview
- Rhythm complexities output to LilyPond

**What needs to be done:**
- Import from other music formats
- Advanced notation validation
- Programmatic transformation tools
- Extended rendering options
- OCR for manuscript digitization (basic framework built, LLMs make this achievable, synthetic training data generation ready)

---

*Modern tools.*