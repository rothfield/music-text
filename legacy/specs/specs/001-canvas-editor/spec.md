# Feature Specification: Canvas Editor (WYSIWYG Music Text Editor)

**Feature Branch**: `001-canvas-editor`
**Created**: 2025-09-24
**Status**: Draft
**Input**: User description: "canvas editor"

## Execution Flow (main)
```
1. Parse user description from Input
   → User wants a WYSIWYG canvas-based sargam music editor using Rust/eGUI with SVG rendering
2. Extract key concepts from description
   → Actors: musicians, composers using sargam notation
   → Actions: visual editing, content line creation, beat placement, typography rendering
   → Data: content lines, musical beats, sargam notation, SVG output
   → Constraints: desktop application, content-line centric, beautiful typography
3. Clarifications resolved:
   → Technology: Rust/eGUI with SVG plugin for desktop application
   → Scope: Content line focused editor with automatic beat looping
   → Typography: Based on typography-specification.md standards
4. Fill User Scenarios & Testing section
   → Primary flow: visual sargam composition with text editor features
5. Generate Functional Requirements
   → Content line editing, SVG rendering, file operations, visual feedback
6. Identify Key Entities
   → Content lines, beats, sargam notation, editor interface
7. Run Review Checklist
   → Requirements clarified and scope defined
8. Return: SUCCESS (spec ready for planning)
```

---

## ⚡ Quick Guidelines
- ✅ Focus on WHAT users need and WHY
- ❌ Avoid HOW to implement (no tech stack, APIs, code structure)
- 👥 Written for business stakeholders, not developers

---

## User Scenarios & Testing *(mandatory)*

### Primary User Story
Musicians and composers want to create sargam music notation through a visual WYSIWYG editor that provides real-time parsing and visual feedback as they type. The key advantage over plain text editing is that as users type, each line is immediately parsed and the screen redraws to show proper alignment of staff notation, beat groupings, note attachments, syllables, and octave dots. Users unfamiliar with staff notation should be able to import existing MusicXML files and have them automatically converted to accessible sargam text notation. They should be able to edit content lines visually with immediate visual feedback, use familiar text editing controls (copy/paste, selection, cursor), and view different output formats in tabs below the editor.

### Acceptance Scenarios
1. **Given** an empty canvas editor, **When** user types sargam notation, **Then** it appears with proper typography and automatic beat grouping with immediate visual feedback as each keystroke is parsed
2. **Given** existing content lines, **When** user selects text and copies it, **Then** the selected notation can be pasted elsewhere with formatting preserved
3. **Given** a content line being edited, **When** user presses Enter, **Then** a new content line is created below the current one
4. **Given** musical notation with beats, **When** user views the content, **Then** automatic beat looping and visual grouping is applied
5. **Given** a completed composition, **When** user clicks on tabs below editor, **Then** they can view LilyPond source, LilyPond SVG, and document JSON visualization
6. **Given** musical content in the editor, **When** user uses control bar buttons, **Then** octave adjustments and slur operations are applied to the notation
7. **Given** musical content in the editor, **When** user presses Ctrl+L, **Then** notes 7654 are set to lower octave and 12345 to middle octave
8. **Given** musical content in the editor, **When** user presses Ctrl+Up, **Then** notes 1234 are set to upper octave and 7654 to middle octave
9. **Given** musical content in the editor, **When** user presses Ctrl+M, **Then** all notes are set to middle octave
10. **Given** musical content in the editor, **When** user clicks force lower octave button, **Then** all notes are set to lower octave
11. **Given** musical content in the editor, **When** user clicks force upper octave button, **Then** all notes are set to upper octave
12. **Given** musical content in the editor, **When** user clicks force highest octave button, **Then** all notes are set to highest octave
13. **Given** musical content in the editor, **When** user clicks force lowest octave button, **Then** all notes are set to lowest octave
14. **Given** cursor is on a content line, **When** user presses Up arrow, **Then** cursor moves to previous content line
15. **Given** cursor is on a content line, **When** user presses Down arrow, **Then** cursor moves to next content line
16. **Given** selected content in editor, **When** user presses Ctrl+C, **Then** content is copied to clipboard with formatting preserved
17. **Given** copied content in clipboard, **When** user presses Ctrl+V, **Then** content is pasted at cursor position with proper formatting
18. **Given** recent changes made in editor, **When** user presses Ctrl+Z, **Then** last action is undone and editor state reverts
19. **Given** user types ":| in editor, **When** text is parsed, **Then** a right barline object is created at that position
20. **Given** user wants to insert barlines, **When** user accesses button palette, **Then** various barline types are available for insertion
21. **Given** cursor positioned to right of a note, **When** user presses Ctrl+O, **Then** mini editor dialog opens for adding grace notes
22. **Given** grace note dialog is open, **When** user enters grace note text, **Then** grace notes appear above the target note with full ornament class styling
23. **Given** a note with grace notes, **When** viewed in editor, **Then** grace notes appear positioned after the note in the notation flow
24. **Given** user wants to create multi-stave notation, **When** user groups staves together, **Then** staves are visually connected with continuous leftmost barlines
25. **Given** grouped staves in multi-stave system, **When** barlines are displayed, **Then** leftmost barlines extend continuously across all grouped staves
26. **Given** ungrouped staves, **When** barlines are displayed, **Then** each stave displays individual barlines without connection
27. **Given** user wants to zoom in, **When** user presses Ctrl+Shift++, **Then** canvas scale increases and notation appears larger
28. **Given** user wants to zoom out, **When** user presses Ctrl+Shift+-, **Then** canvas scale decreases and notation appears smaller
29. **Given** user wants to add lyrics, **When** lyrics are attached to a content line, **Then** syllables automatically map to notes using LilyPond-style syllable alignment
30. **Given** content line with attached lyrics, **When** viewed in editor, **Then** lyrics appear below the notation with proper syllable-to-note correspondence
31. **Given** lyrics with more syllables than notes, **When** displayed, **Then** extra syllables are handled according to automatic mapping rules
32. **Given** user wants to explicitly group beats, **When** user uses lower loop button or nbsp character, **Then** selected beats are grouped with visual loop indicator below
33. **Given** explicitly grouped beats, **When** viewed in editor, **Then** custom beat grouping overrides automatic beat looping
34. **Given** user wants to modify beat grouping, **When** user selects different beats for grouping, **Then** previous explicit grouping is updated or replaced
35. **Given** user enters tala at line level, **When** tala is specified, **Then** tala automatically applies to barlines and measure structure for that line
36. **Given** line with tala specification, **When** barlines are displayed, **Then** barlines follow the tala's rhythmic pattern and measure divisions
37. **Given** multiple lines with different talas, **When** viewed in editor, **Then** each line maintains its own tala-based measure structure
38. **Given** user requests document output, **When** generating final document, **Then** system produces engraving quality music text document with professional layout
39. **Given** document exceeds single page, **When** outputting document, **Then** system automatically inserts page breaks at appropriate musical boundaries
40. **Given** multi-page document, **When** formatted for output, **Then** headers and footers are applied consistently across all pages
41. **Given** completed composition, **When** exporting document, **Then** typography meets professional engraving standards for music publication
42. **Given** notation with octave markers, **When** rendered for engraving, **Then** dots/bullets are positioned precisely to avoid collisions with descenders and ascenders
43. **Given** western musical characters in notation, **When** displaying for engraving quality, **Then** proper music fonts are used for professional appearance
44. **Given** mixed content with descenders, **When** positioning octave markers, **Then** vertical spacing automatically adjusts to prevent typography collisions
45. **Given** engraving quality output, **When** rendered, **Then** all typography elements follow professional music publishing standards for positioning and spacing
46. **Given** user is on LilyPond SVG tab, **When** user types in editor, **Then** SVG automatically redraws with debounced updates to provide real-time visual feedback
47. **Given** active LilyPond SVG tab with debouncing, **When** user pauses typing, **Then** SVG rendering updates after brief delay to reflect current notation
48. **Given** continuous typing on LilyPond SVG tab, **When** user is actively editing, **Then** debounce mechanism prevents excessive redraw operations while maintaining responsiveness
49. **Given** user is on MIDI tab, **When** viewing the tab, **Then** MIDI representation of the current notation is displayed and available for playback
50. **Given** user changes notation while on MIDI tab, **When** content is modified, **Then** MIDI representation updates to reflect current musical content
51. **Given** user wants to import existing music, **When** user imports MusicXML file, **Then** system converts staff notation to sargam text notation for editing
52. **Given** imported MusicXML content, **When** conversion is complete, **Then** resulting sargam notation appears in editor with proper formatting and structure
53. **Given** user unfamiliar with staff notation, **When** importing MusicXML, **Then** system provides accessible sargam text representation of the musical content
54. **Given** complex MusicXML with multiple voices, **When** imported, **Then** system intelligently maps staff notation elements to appropriate sargam text equivalents
55. **Given** user wants to insert non-breaking space, **When** user presses Ctrl+Space, **Then** non-breaking space character is inserted for explicit beat grouping
56. **Given** non-breaking space inserted via Ctrl+Space, **When** viewed in editor, **Then** explicit beat grouping is applied using the inserted nbsp character
57. **Given** content with nbsp characters, **When** parsed by music-text parser, **Then** parser correctly recognizes nbsp as beat grouping delimiter and processes content accordingly
58. **Given** mixed content with regular spaces and nbsp characters, **When** parsing, **Then** parser differentiates between regular whitespace and nbsp-based explicit beat grouping
59. **Given** user types in editor, **When** each keystroke is entered, **Then** current line is immediately parsed and screen redraws with updated visual representation
60. **Given** real-time parsing active, **When** user modifies notation, **Then** staff notation, beat groupings, note attachments, syllables, and octave dots maintain proper alignment automatically
61. **Given** typing in progress, **When** line content changes, **Then** visual elements update in real-time providing immediate feedback superior to plain text editing
62. **Given** complex notation with multiple elements, **When** user edits any part, **Then** all related visual components (dots, groupings, attachments) automatically realign and redraw
63. **Given** user interacts with WYSIWYG interface, **When** editing notation, **Then** music-text format serves as underlying file/storage format while user works with visual representation
64. **Given** file operations (save/load), **When** performed, **Then** music-text format maintains compatibility while abstracting parsing complexity from user editing experience
65. **Given** user wants to add syllables to individual notes, **When** syllables are attached to specific notes, **Then** syllables appear below each note with proper alignment and typography
66. **Given** notes with attached syllables, **When** viewed in editor, **Then** syllables maintain alignment with their parent notes during real-time editing and redrawing
67. **Given** mixed notation with both line-level lyrics and note-level syllables, **When** displayed, **Then** system handles both attachment methods with appropriate visual differentiation
68. **Given** user creates document sections, **When** sections are defined, **Then** document is organized into named sections with clear visual boundaries
69. **Given** user creates variations within sections, **When** variations are added, **Then** system automatically numbers variations sequentially (Variation 1, Variation 2, etc.)
70. **Given** document with multiple sections and variations, **When** viewed, **Then** hierarchical organization is maintained with section headers and variation numbering
71. **Given** user reorders or deletes variations, **When** changes are made, **Then** variation numbering automatically updates to maintain sequential order
72. **Given** user enters notation in editor, **When** system detects notation patterns, **Then** status line displays detected notation system (sargam, western, number, etc.) with forgiving interpretation
73. **Given** mixed or ambiguous notation, **When** parsed by system, **Then** forgiving detection allows user to continue editing while displaying best-guess notation system in status line
74. **Given** user types notation that could match multiple systems, **When** editing, **Then** system provides forgiving interpretation and lets user determine correct notation system through continued editing
75. **Given** user wants to specify notation system, **When** user sets document notation system option, **Then** system uses specified notation system for interpretation while maintaining forgiving approach
76. **Given** document with explicit notation system setting, **When** user edits notation, **Then** system interprets content according to specified system while still allowing mixed notation if detected
77. **Given** a document with changes, **When** user saves, **Then** document is stored using local storage-like mechanism for easy development

### Edge Cases
- What happens when user enters invalid sargam notation that cannot be parsed?
- How does the system handle very long content lines that exceed display width?
- What occurs when copying content between different sections or documents?
- How does Up/Down navigation behave at first/last content lines?
- What happens when trying to undo when no previous actions exist?
- How does paste operation handle invalid or incompatible clipboard content?
- What happens when ":| shorthand conflicts with other notation elements?
- How does button palette handle barline insertion at invalid positions?
- What happens when Ctrl+O is pressed when cursor is not positioned to right of a note?
- How does grace note dialog handle invalid or empty grace note text input?
- What occurs when multiple grace notes are attached to the same note?
- How does stave grouping handle different numbers of measures across staves?
- What happens when user attempts to group incompatible stave types?
- How does the system handle ungrouping staves that are part of a multi-stave system?
- What are the minimum and maximum zoom limits for canvas scaling?
- How does scaling affect cursor positioning and text editing precision?
- What happens when scaling makes notation too small or too large to edit effectively?
- How does syllable mapping handle melismatic passages (multiple notes per syllable)?
- What occurs when lyrics contain special characters or non-standard syllable divisions?
- How does the system handle lyrics in different languages with varying syllable structures?
- What happens when lyrics are attached to content lines with grace notes or ornaments?
- How does explicit beat grouping interact with automatic beat looping algorithms?
- What occurs when nbsp characters conflict with other whitespace or formatting?
- How does the system handle overlapping or nested explicit beat groupings?
- What happens when explicit beat groupings span across barlines or measure boundaries?
- How does tala specification interact with manually placed barlines?
- What occurs when tala patterns don't align with the actual note count in a line?
- How does the system handle invalid or unrecognized tala specifications?
- What happens when changing tala mid-line or applying different talas to grouped staves?
- How does page break algorithm handle complex multi-stave systems that don't fit on single pages?
- What occurs when header/footer content conflicts with musical notation at page boundaries?
- How does the system handle very short or very long compositions for optimal page layout?
- What happens when export format requirements conflict with editor display formatting?
- How does octave marker positioning handle extreme ascender/descender combinations?
- What occurs when music font requirements conflict with system font availability?
- How does collision detection handle complex overlapping typography scenarios?
- What happens when typography adjustments affect overall document layout and pagination?
- How does debounced SVG redrawing handle rapid typing and performance optimization?
- What occurs when MIDI generation fails or encounters unsupported notation elements?
- How does the system handle tab switching during active debounced operations?
- What happens when multiple tabs require updates simultaneously during notation changes?
- How does MusicXML import handle unsupported or incompatible notation elements?
- What occurs when imported MusicXML contains multiple key signatures or time signatures?
- How does the system handle MusicXML files with complex rhythmic patterns that don't map directly to sargam notation?
- What happens when MusicXML import encounters corrupt or malformed files?
- How does the converter handle Western musical concepts that don't have direct sargam equivalents?
- What happens when Ctrl+Space conflicts with system or browser keyboard shortcuts?
- How does the system differentiate between regular spaces and non-breaking spaces in the editor display?
- What occurs when multiple consecutive non-breaking spaces are inserted via Ctrl+Space?
- How does parser update maintain backward compatibility with existing music-text documents?
- What happens when parser encounters nbsp characters in unexpected positions or contexts?
- How does the enhanced parser handle mixed whitespace scenarios with both regular spaces and nbsp characters?
- What occurs when parsing fails due to malformed nbsp usage in beat grouping?
- How does real-time parsing handle performance optimization during rapid typing?
- What happens when parsing encounters incomplete or temporarily invalid notation during typing?
- How does the system handle visual element alignment when screen redraws are interrupted or delayed?
- What occurs when real-time parsing conflicts with other operations like file saving or tab switching?
- How does keystroke-level parsing maintain performance with complex notation containing many visual elements?
- How does format abstraction layer handle edge cases where WYSIWYG editing creates music-text that's difficult to parse?
- What happens when music-text file format evolution conflicts with existing WYSIWYG editor capabilities?
- How does the system maintain backward compatibility when music-text format becomes primarily a storage format?
- How does note-level syllable attachment handle conflicts with line-level lyrics on the same content line?
- What occurs when syllables attached to individual notes are longer than the note spacing allows?
- How does the system handle syllable positioning when notes are moved or edited during real-time editing?
- What happens when both note-level syllables and line-level lyrics contain conflicting text for the same musical content?
- How does section organization handle very large documents with many nested sections and variations?
- What occurs when user moves variations between different sections and automatic numbering conflicts arise?
- How does the system handle section and variation references in cross-references or navigation within the document?
- What happens when section names or variation content create conflicts during automatic numbering updates?
- How does forgiving notation detection handle completely unrecognizable input patterns?
- What occurs when user-specified notation system conflicts with clearly detected different notation patterns?
- How does status line display handle rapid switching between different detected notation systems during typing?
- What happens when mixed notation contains elements that are valid in multiple notation systems simultaneously?

## Requirements *(mandatory)*

### Functional Requirements
- **FR-001**: System MUST provide a desktop WYSIWYG editor for sargam music notation
- **FR-002**: System MUST render content lines with typography standards from typography-specification.md
- **FR-003**: System MUST support essential text editing operations (copy with Ctrl+C, paste with Ctrl+V, selection, cursor movement)
- **FR-004**: System MUST automatically group and loop beats within content lines
- **FR-005**: System MUST maintain webapp layout structure with editor area at top, control bar above editor, and tabbed views below
- **FR-006**: System MUST include octave adjustment buttons in control bar for musical transposition
- **FR-007**: System MUST include slur button in control bar for musical phrasing control
- **FR-008**: System MUST render output using SVG for high-quality typography
- **FR-009**: System MUST focus on content line editing as the primary document structure
- **FR-010**: System MUST parse content lines using existing music-text parsing capabilities
- **FR-011**: System MUST support Enter key to create new content lines
- **FR-012**: System MUST provide visual feedback during editing operations
- **FR-013**: System MUST use subset of existing document model optimized for UI layout
- **FR-014**: System MUST provide tabbed views for LilyPond source, LilyPond SVG, and document JSON visualization
- **FR-015**: System MUST implement an auto-save/session-restore mechanism to prevent data loss and simplify testing workflows.
- **FR-016**: System MUST support Ctrl+L keyboard shortcut to set notes 7654 to lower octave and 12345 to middle octave
- **FR-017**: System MUST support Ctrl+Up keyboard shortcut to set notes 1234 to upper octave and 7654 to middle octave
- **FR-018**: System MUST support Ctrl+M keyboard shortcut to set all notes to middle octave
- **FR-019**: System MUST provide force lower octave button that sets all notes to lower octave
- **FR-020**: System MUST provide force upper octave button that sets all notes to upper octave
- **FR-021**: System MUST provide force highest octave button that sets all notes to highest octave
- **FR-022**: System MUST provide force lowest octave button that sets all notes to lowest octave
- **FR-023**: System MUST support Up/Down arrow keys for navigation between previous/next content lines
- **FR-024**: System MUST support Ctrl+C for copying selected content with formatting preservation
- **FR-025**: System MUST support Ctrl+V for pasting content at cursor position with proper formatting
- **FR-026**: System MUST support Ctrl+Z for undo operations to revert recent changes
- **FR-027**: System MUST recognize ":| shorthand and create right barline objects automatically
- **FR-028**: System MUST provide button palette for inserting various barline types
- **FR-029**: System MUST support Ctrl+O shortcut to open grace note mini editor when cursor is positioned to right of a note
- **FR-030**: System MUST display grace note dialog box for entering grace note text
- **FR-031**: System MUST render grace notes above target notes with full ornament class styling from typography specification
- **FR-032**: System MUST position grace notes after notes in the notation flow
- **FR-033**: System MUST provide functionality for users to group staves together into multi-stave systems
- **FR-034**: System MUST render leftmost barlines continuously across all grouped staves in multi-stave systems
- **FR-035**: System MUST display individual barlines for ungrouped staves without visual connection
- **FR-036**: System MUST provide visual differentiation between multi-stave and single-stave notation systems
- **FR-037**: System MUST support Ctrl+Shift++ keyboard shortcut to increase canvas scale and zoom in on notation
- **FR-038**: System MUST support Ctrl+Shift+- keyboard shortcut to decrease canvas scale and zoom out on notation
- **FR-039**: System MUST maintain notation proportions and typography quality during scaling operations
- **FR-040**: System MUST support attaching lyrics to content lines with automatic syllable-to-note mapping
- **FR-041**: System MUST implement LilyPond-style automatic syllable alignment for lyrics
- **FR-042**: System MUST display lyrics below notation with proper syllable-to-note correspondence
- **FR-043**: System MUST handle cases where syllable count differs from note count using intelligent mapping rules
- **FR-044**: System MUST provide lower loop button for explicit beat grouping with visual loop indicators below beats
- **FR-045**: System MUST support nbsp (non-breaking space) character as alternative method for explicit beat grouping
- **FR-046**: System MUST allow explicit beat grouping to override automatic beat looping behavior
- **FR-047**: System MUST support modification and replacement of existing explicit beat groupings
- **FR-048**: System MUST support tala entry at line level for automatic application to barlines and measure structure
- **FR-049**: System MUST automatically apply tala rhythmic patterns to barline placement and measure divisions
- **FR-050**: System MUST support different talas on different lines with independent measure structures
- **FR-051**: System MUST maintain tala-based measure consistency within each content line
- **FR-052**: System MUST produce clean and readable music text documents suitable for lead sheets, with a professional layout.
- **FR-053**: System MUST automatically insert page breaks at appropriate musical boundaries when content exceeds page limits
- **FR-054**: System MUST apply consistent headers and footers across all pages in multi-page documents
- **FR-055**: System MUST meet professional typography standards, with a primary focus on readability for lead sheets.
- **FR-056**: System MUST position octave marker dots/bullets precisely to avoid collisions with descenders and ascenders
- **FR-057**: System MUST use proper music fonts for western musical characters in engraving quality output
- **FR-058**: System MUST automatically adjust vertical spacing to prevent typography collisions, with the highest priority on ensuring lyrics do not overlap with notes or other musical symbols.
- **FR-059**: System MUST apply professional music publishing standards for all typography element positioning and spacing
- **FR-060**: System MUST automatically redraw LilyPond SVG when user types in editor while on LilyPond SVG tab
- **FR-061**: System MUST implement debounced updates for LilyPond SVG rendering to provide real-time feedback without excessive processing
- **FR-062**: System MUST update SVG rendering after brief delay when user pauses typing on LilyPond SVG tab
- **FR-063**: System MUST prevent excessive redraw operations during continuous typing while maintaining responsive visual feedback
- **FR-064**: System MUST provide MIDI tab displaying MIDI representation of current notation with playback capability
- **FR-065**: System MUST update MIDI representation when user modifies notation content
- **FR-066**: System MUST support MusicXML import functionality to convert staff notation to sargam text notation
- **FR-067**: System MUST convert imported MusicXML content to properly formatted sargam notation in the editor
- **FR-068**: System MUST provide accessible sargam text representation for users unfamiliar with staff notation
- **FR-069**: For the initial implementation (POC), the system MUST import single-voice melodies from MusicXML, focusing on basic notes and rhythms. Complex elements like articulations, dynamics, and multi-voice arrangements are out of scope for the MVP.
- **FR-070**: System MUST support Ctrl+Space keyboard shortcut to insert non-breaking space characters for explicit beat grouping
- **FR-071**: System MUST apply explicit beat grouping when non-breaking space characters are inserted via Ctrl+Space
- **FR-072**: System MUST update existing music-text parser to recognize and handle non-breaking space characters for explicit beat grouping
- **FR-073**: System MUST ensure parser correctly interprets nbsp characters as beat grouping delimiters in content line parsing
- **FR-074**: System MUST perform near real-time parsing of each line as user types, with debounced screen redrawing to ensure a responsive user experience
- **FR-075**: System MUST maintain automatic alignment of staff notation, beat groupings, note attachments, syllables, and octave dots during real-time editing
- **FR-076**: System MUST provide immediate visual feedback superior to plain text editing through near real-time parsing and debounced redrawing
- **FR-077**: System MUST automatically realign and redraw all related visual components when user edits any part of complex notation
- **FR-078**: System MUST treat music-text format as underlying file/storage format while providing WYSIWYG editing interface
- **FR-079**: System MUST maintain music-text format compatibility for file operations while abstracting editing complexity from users
- **FR-080**: System MUST support attaching syllables to individual notes with proper alignment and typography below each note
- **FR-081**: System MUST maintain syllable alignment with parent notes during real-time editing and redrawing operations
- **FR-082**: System MUST handle both line-level lyrics and note-level syllables with appropriate visual differentiation
- **FR-083**: System MUST support document sections with named sections and clear visual boundaries for organizing compositions
- **FR-084**: System MUST provide automatic numbering of variations within sections (Variation 1, Variation 2, etc.)
- **FR-085**: System MUST maintain hierarchical organization with section headers and variation numbering throughout the document
- **FR-086**: System MUST automatically update variation numbering when variations are reordered or deleted
- **FR-087**: System MUST implement forgiving notation detection that identifies notation patterns and displays detected system in status line
- **FR-088**: System MUST display detected notation system (sargam, western, number, etc.) in status line with forgiving interpretation approach
- **FR-089**: System MUST allow user to continue editing with ambiguous or mixed notation while providing best-guess detection feedback
- **FR-090**: System MUST let user determine correct notation system through continued editing rather than enforcing strict validation
- **FR-091**: System MUST provide option to explicitly set document notation system (sargam, western, number, etc.) for interpretation preference
- **FR-092**: System MUST use specified notation system for interpretation while maintaining forgiving approach and allowing mixed notation
- **FR-093**: System MUST provide document export functionality for high-quality music text documents
- **FR-094**: System MUST provide file menu operations including MusicXML import (new, open, save, save as, import MusicXML)
- **FR-095**: System MUST automatically reload the most recently opened document upon startup.

### Key Entities *(include if feature involves data)*
- **Content Line**: Primary editable unit containing sargam notation with automatic beat grouping, typography, optional attached lyrics, and line-level tala specification
- **Beat Group**: Visual grouping of musical beats within content lines with automatic looping behavior or explicit user-defined grouping
- **Editor Area**: Main WYSIWYG editing surface where content lines are visually displayed and edited
- **Control Bar**: Top interface panel containing octave adjustment buttons (including force lowest, force lower, force upper, force highest), slur controls, barline button palette, lower loop button for explicit beat grouping, and other musical operation buttons
- **Status Line**: Interface element displaying detected notation system (sargam, western, number, etc.) with forgiving interpretation feedback
- **Tab Panel**: Bottom interface area displaying different views (LilyPond source, LilyPond SVG, document JSON, engraving quality preview, MIDI)
- **SVG Renderer**: Component that converts parsed content lines into high-quality typographic display
- **Document Model**: Simplified version of existing music-text document structure optimized for visual editing with support for sections and variations
- **Typography Engine**: Renders musical notation according to typography-specification.md standards with engraving quality positioning and spacing
- **Storage System**: Manages document persistence. Primary mechanism is direct file system access (open, save). Includes a secondary auto-save/session-restore feature for data recovery and development convenience.
- **Octave Controller**: Component handling keyboard shortcuts for octave adjustment operations on musical notation
- **Navigation Controller**: Component handling Up/Down arrow key navigation between content lines
- **Clipboard Manager**: Component managing copy/paste operations with formatting preservation
- **Undo System**: Component tracking and reverting editor state changes for undo functionality
- **Barline Object**: Musical notation element representing different types of barlines (right, left, double, repeat, etc.)
- **Button Palette**: Interface component providing access to various barline types for insertion into notation
- **Grace Note Object**: Musical ornament element attached to notes, rendered above with ornament class styling and positioned after notes in notation flow
- **Grace Note Dialog**: A simple, modal dialog (e.g., a JavaScript-style prompt) for entering grace note text. It contains a single text input, a confirmation button, and a cancellation button.
- **Stave Group**: Collection of multiple staves that are visually connected and treated as a unified multi-stave system
- **Multi-Stave System**: Notation layout where grouped staves display with continuous leftmost barlines and unified visual treatment
- **Stave Grouping Interface**: User interface component allowing selection and grouping of individual staves into multi-stave systems
- **Canvas Scaler**: Component managing zoom levels and scaling operations for the editor canvas
- **Scale Controller**: Keyboard shortcut handler for Ctrl+Shift++ and Ctrl+Shift+- zoom operations
- **Lyrics Object**: Text content attached to content lines containing syllables for automatic mapping to notes, or individual syllables attached to specific notes
- **Syllable Mapper**: Component implementing LilyPond-style automatic alignment of syllables to notes
- **Lyrics Renderer**: Component displaying lyrics below notation with proper typography and positioning
- **Explicit Beat Grouper**: Component handling user-defined beat groupings via lower loop button or nbsp character
- **Loop Indicator**: Visual element displaying explicit beat groupings as loop shapes below selected beats
- **Tala Object**: Rhythmic pattern specification applied at line level defining measure structure and barline placement
- **Tala Engine**: Component that automatically applies tala patterns to barlines and measure divisions within content lines
- **Measure Structure**: Layout organization based on tala rhythmic patterns controlling barline positioning and timing
- **Document Layout Engine**: Component managing professional page layout, pagination, and document structure for engraving quality output
- **Page Break Controller**: Component determining optimal page break locations at musical boundaries
- **Header/Footer Manager**: Component applying consistent headers and footers across multi-page documents
- **Engraving Engine**: Component ensuring professional typography and layout standards for music publication quality
- **Export Controller**: Component managing document export functionality for high-quality music text documents
- **Octave Marker Positioner**: Component handling precise positioning of dots/bullets to avoid collisions with descenders and ascenders
- **Music Font Manager**: Component ensuring proper music fonts are applied to western musical characters for engraving quality
- **Collision Detection Engine**: Component automatically adjusting vertical spacing to prevent typography collisions in engraving output
- **Typography Standards Enforcer**: Component applying professional music publishing standards for element positioning and spacing
- **SVG Debouncer**: Component managing debounced automatic redrawing of LilyPond SVG tab with real-time visual feedback
- **MIDI Generator**: Component creating MIDI representation of notation for playback and display in MIDI tab
- **Tab Update Manager**: Component coordinating updates across different tab views when notation content changes
- **MusicXML Importer**: Component handling import and parsing of MusicXML files for conversion to sargam text notation
- **Staff-to-Sargam Converter**: Component intelligently mapping staff notation elements to appropriate sargam text equivalents
- **Import Dialog**: Interface component for selecting and importing MusicXML files with conversion options
- **Keyboard Input Handler**: Component managing special keyboard shortcuts including Ctrl+Space for non-breaking space insertion
- **Enhanced Parser**: Updated music-text parser with nbsp character recognition and handling for explicit beat grouping
- **Beat Delimiter Processor**: Component within parser that differentiates between regular whitespace and nbsp-based beat grouping delimiters
- **Real-Time Parser Engine**: Component performing immediate parsing of each line as user types with keystroke-level responsiveness
- **Live Redraw Manager**: Component coordinating immediate screen redraws and visual element alignment during real-time editing
- **Visual Alignment System**: Component maintaining proper positioning of staff notation, beat groupings, note attachments, syllables, and octave dots
- **Keystroke Handler**: Component capturing and processing each user keystroke for immediate parsing and visual feedback
- **Format Abstraction Layer**: Component managing music-text format as underlying file/storage format while presenting WYSIWYG interface to users
- **Compatibility Manager**: Component maintaining music-text format compatibility for file operations while abstracting parsing complexity from editing experience
- **Note-Syllable Mapper**: Component managing attachment and alignment of individual syllables to specific notes
- **Syllable Positioning Engine**: Component maintaining proper positioning of note-level syllables below their parent notes during real-time editing
- **Document Section Manager**: Component organizing document into named sections with clear visual boundaries and hierarchical structure
- **Variation Numbering Engine**: Component automatically numbering variations within sections and updating numbers when variations are reordered or deleted
- **Hierarchical Navigation**: Component managing navigation between sections and variations within the document structure
- **Notation Detection Engine**: Component implementing forgiving notation pattern recognition and displaying detected system in status line
- **Notation System Selector**: Interface component allowing user to explicitly set document notation system preference
- **Forgiving Parser**: Component providing lenient interpretation of notation. It allows continued editing with ambiguous content and treats any un-parseable token as a distinct text node, which is then rendered with a special error style.

---

## Review & Acceptance Checklist
*GATE: Automated checks run during main() execution*

### Content Quality
- [x] No implementation details (languages, frameworks, APIs)
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
- [x] All mandatory sections completed

### Requirement Completeness
- [x] No [NEEDS CLARIFICATION] markers remain
- [x] Requirements are testable and unambiguous
- [x] Success criteria are measurable
- [x] Scope is clearly bounded
- [x] Dependencies and assumptions identified

---

## Execution Status
*Updated by main() during processing*

- [x] User description parsed
- [x] Key concepts extracted
- [x] Ambiguities marked and resolved
- [x] User scenarios defined
- [x] Requirements generated
- [x] Entities identified
- [x] Review checklist passed

---

## Future Considerations

### Enhanced Typography Features
- **Ligature Support**: For connected note sequences
- **Variable Fonts**: For dynamic weight adjustments
- **Custom Font Loading**: Music-specific typefaces
- **Accessibility**: High contrast and screen reader support
- **Unicode Symbols**: Proper musical ornament characters (`∿`, `𝆝`)

### Intelligent Spacing System *(Future Enhancement)*
- **Adaptive Beat Grouping Spacing**: Automatically adjust horizontal spacing between beat groups based on content complexity and visual density for enhanced readability
- **Syllable-Aware Spacing**: Dynamically expand horizontal space around notes to accommodate syllable text width, ensuring syllables don't overlap with adjacent musical elements
- **Text Width Calculations**: Intelligent measurement of syllable text dimensions to determine minimum required spacing for clean visual presentation
- **Readability Optimization**: Intelligent spacing algorithms that consider note density, ornaments, syllables, and octave markers for optimal visual clarity
- **Dynamic Whitespace Management**: Smart insertion of appropriate whitespace between beat groups to improve musical phrase readability and visual flow, with additional space allocation for syllable text
- **Context-Aware Spacing**: Spacing adjustments that consider surrounding musical elements (barlines, lyrics, grace notes, syllable text) for balanced visual presentation
- **Typography-Driven Spacing**: Spacing calculations based on font metrics and typography standards for professional music notation appearance, including syllable font metrics
- **Phrase-Level Spacing**: Intelligent grouping and spacing at the musical phrase level to enhance comprehension and visual parsing of complex notation with syllable considerations
- **Adaptive Layout Engine**: Dynamic spacing system that adjusts in real-time based on content changes, maintaining optimal readability as users edit and add syllables to notes
- **Comprehensive Collision Avoidance**: System-wide collision detection and prevention across all visual elements including notes, syllables, ornaments, octave dots, grace notes, beat groupings, barlines, and lyrics to avoid unpleasant visual overlaps throughout the layout engine

### Layout Engine & Collision System *(Future Enhancement)*
- **Universal Collision Detection**: Comprehensive system tracking bounding boxes and spatial relationships of all visual elements (notes, syllables, ornaments, octave dots, grace notes, beat groupings, barlines, lyrics, tala markers)
- **Hierarchical Collision Priority**: Intelligent prioritization system determining which elements should move or resize when collisions are detected (e.g., spacing adjusts before font size reduction)
- **Multi-Layer Collision Management**: Separate collision handling for different visual layers (baseline notes, above-note ornaments, below-note syllables, beat group arcs, measure-level elements)
- **Predictive Collision Avoidance**: Anticipate potential collisions during real-time editing and proactively adjust layout before conflicts occur
- **Graceful Degradation**: When space is extremely limited, provide intelligent fallback strategies (syllable abbreviation, stacked elements, font size reduction) rather than overlapping
- **Cross-Element Spacing**: Maintain minimum clearance between different types of elements based on typography standards and visual design principles

### Performance Optimizations
- **CSS Grid**: For complex notation layouts
- **GPU Acceleration**: For smooth arc rendering and collision detection
- **Lazy Loading**: For large notation documents
- **Spatial Indexing**: Efficient data structures for rapid collision detection across many elements
- **Intelligent Caching**: Cache spacing calculations, typography metrics, syllable text width measurements, and collision detection results for improved performance
- **Progressive Enhancement**: Gradually apply intelligent spacing as processing resources allow, prioritizing critical collision avoidance for immediate readability
- **Text Measurement Optimization**: Efficient syllable text width calculation algorithms to minimize performance impact during real-time editing
- **Incremental Layout Updates**: Only recalculate collisions and spacing for affected areas during editing, not the entire document

---