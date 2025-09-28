# Technical Plan: WYSIWYG Canvas Editor

This document outlines a phased technical approach for building the WYSIWYG Canvas Editor. The core strategy is to leverage the existing, powerful Rust backend (parser, spatial analysis, rhythm analysis, renderers) while building a new HTML/JavaScript-based web interface.

## Phase 1: Consolidate the Backend into a Stable, UI-Agnostic Library

The goal of this phase is to treat the existing Rust code as a library (`lib.rs`) that can be called by any frontend, whether it's the current web UI or the future HTML canvas editor.

1.  **Define a Clean API Boundary:** Expose a single, powerful function that takes the `music-text` string and returns a complete, structured representation of the document suitable for rendering. The existing `process_notation` function in `pipeline.rs` is a strong starting point.
2.  **Create a "Render Model":** The API should return a serialized version of the `Document` AST after all parsing and analysis is complete. This model will be the "source of truth" for the UI. It must contain all necessary information to draw the notation: note pitches, positions, durations, slurs, lyrics, beat groups, etc.
3.  **Expose via an API:** Continue using the Axum web server (`web.rs`) to expose this API for the new HTML client. For a highly interactive canvas editor, consider WebSockets in the future for lower-latency communication, but a standard HTTP API is sufficient to start.

**Reusable Code:** This phase almost exclusively uses existing code: `lib.rs`, `pipeline.rs`, `parse/`, `rhythm/`, `spatial/`, and `models/`.

## Phase 2: Build the HTML Canvas UI Scaffold and Basic Rendering

This is the "start from scratch" part, but only for the user interface.

1.  **Set up the HTML Application Shell:** Create a new HTML page with a `<canvas>` element as the main drawing surface. Include JavaScript modules for the canvas editor functionality.
2.  **Implement the API Client:** The HTML/JavaScript UI will act as a client to the backend API defined in Phase 1. It will send `music-text` strings via HTTP requests and receive the render model as JSON.
3.  **Focus on Rendering First (Read-Only):** The first major goal is to simply *display* a `music-text` document.
    *   Call the backend API with a sample document using `fetch()`.
    *   Receive the render model as JSON.
    *   Use the existing SVG output from `renderers::svg` modules by embedding the SVG directly in the HTML or converting it to canvas drawing operations. This provides a high-quality visual representation immediately, without needing to write a new renderer from scratch.

**Reusable Code:** This phase reuses the `renderers/` modules for generating the visual output.

## Phase 3: Implement the Core Interactive Loop

With rendering in place, the HTML canvas can now become interactive.

1.  **Element Picking:** Implement JavaScript logic to identify which musical element (note, rest, etc.) the user is clicking on. This involves mapping mouse coordinates to the bounding boxes of the rendered elements, either through SVG element detection or canvas hit-testing.
2.  **Introduce the Edit/Response Cycle:**
    *   When a user performs an action (e.g., clicks a note and presses "delete"), the JavaScript UI sends a command to the backend API (e.g., `{"action": "delete", "element_id": 123}`).
    *   The backend will need a new function to handle these commands. It would load the document, apply the change, and then run the full `process_notation` pipeline again.
    *   It then sends the *new, complete render model* back to the JavaScript UI as JSON.
    *   The JavaScript UI receives the new model and triggers a full re-render of the canvas.

This creates a robust, unidirectional data flow where the Rust backend remains the single source of truth.

## Phase 4: Incrementally Add Editing Features

Once the core loop is working, features from the specification can be added incrementally. Each new feature is a new type of command sent to the backend via JavaScript `fetch()` requests.

*   **Add Note:** Click a position on the canvas, send an `{"action": "add_note", "pitch": "S", "position": ...}` command via HTTP POST.
*   **Drag Note:** Send `{"action": "move_note", "element_id": 123, "new_position": ...}` via HTTP POST.
*   **Apply Slur:** Select multiple notes, send `{"action": "apply_slur", "element_ids": [123, 124, 125]}` via HTTP POST.
*   **Keyboard Shortcuts:** Implement JavaScript event listeners for keyboard shortcuts (Ctrl+C, Ctrl+V, Ctrl+L, etc.) that send appropriate commands to the backend.
*   **Real-time Editing:** Implement debounced text input that sends the current document state to the backend for live parsing and re-rendering.

This approach allows for the methodical construction of the complex editor, building upon a solid foundation and maximizing the reuse of the existing, battle-tested Rust backend logic.
