# CodeMirror 5 Context for AI Agent

This document provides a high-level overview of the CodeMirror 5 integration for the `music-text` project. The goal is to guide the AI agent in manipulating and extending the editor functionality efficiently.

## Core Architecture

- **Editor Type**: CodeMirror is a **syntax-aware plain-text editor**. It is NOT a rich text editor.
- **Data Model**: The source of truth is always the plain text content. Syntax highlighting and other features are a non-destructive visual layer.
- **Primary Goal**: To provide users with a helpful, visually rich environment for writing `doremi-script` plain-text notation. The editor's output must always be a clean string for the parser.

## Key Files

- **Library (self-hosted)**:
  - `webapp/public/assets/codemirror.min.js`
  - `webapp/public/assets/codemirror.min.css`
- **Implementation Module**:
  - `webapp/public/js/editor.js`: All CodeMirror-specific logic is encapsulated here.
- **Main Application**:
  - `webapp/public/js/app.js`: Initializes the editor and provides the callback for handling input.
- **Documentation**:
  - `docs/codemirror_manual.md`: The full, local copy of the CodeMirror 5 API documentation.

## Common Tasks & API Usage

Based on the project's needs, the AI agent will frequently need to perform the following tasks. Refer to `docs/codemirror_manual.md` for detailed API information.

### 1. Initialization
- **Entry Point**: `Editor.init(callback)` in `editor.js`.
- **How**: A new CodeMirror instance is created and attached to the `<div id="musicInput">`.
- **Key Options**: `lineNumbers`, `mode`, `value`, `autofocus`.

### 2. Getting and Setting Content
- **Get**: `Editor.getValue()` which internally calls `instance.getValue()`.
- **Set**: `Editor.setValue(text)` which internally calls `instance.setValue(text)`.
- **Note**: This is the primary way the editor interacts with the application's state.

### 3. Handling User Input
- **Event**: The `change` event is the most important.
- **How**: `instance.on('change', callback)` is used to wire the editor to the main application's `handleInput` logic. The callback receives the editor instance.

### 4. Managing the Cursor and Selections
- **Get Cursor**: `instance.getCursor()` returns a `{line, ch}` object.
- **Set Cursor**: `instance.setCursor({line, ch})`.
- **Get Selection**: `instance.getSelection()` returns the selected string.
- **Replace Selection**: `instance.replaceSelection('new text')` is the primary method for programmatic text manipulation (e.g., from a UI button).

### 5. Syntax Highlighting (Future Task)
- **Mechanism**: A custom language mode will need to be created.
- **API**: `CodeMirror.defineMode(name, modeFactory)`.
- **Goal**: The mode will parse the text line by line and return CSS class names for different tokens (pitches, barlines, comments, etc.), which are then styled in `webapp/public/css/style.css`.

### 6. Programmatic Text Manipulation (Future Task)
- **Scenario**: A user clicks a "Slur" button.
- **Implementation**:
  1. Get the current selection: `const selection = Editor.instance.getSelection();`
  2. Modify the text: `const slurText = '(' + selection + ')';`
  3. Replace the selection with the modified text: `Editor.instance.replaceSelection(slurText);`
- **Key API**: `getSelection()`, `replaceSelection()`.

This context should be sufficient for most common tasks. For anything more advanced, the full manual is available locally.
