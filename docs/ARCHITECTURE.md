## **Technical Design: Model-Driven Editor Architecture**

### 1. Overview

This document outlines a new architectural design for the Music Text web application. The current architecture, which relies on parsing a plain text string (`textContent`) on every user interaction, is brittle and inefficient for a rich editing experience.

The proposed architecture transitions the application to a **model-driven** approach. The client will directly manipulate a structured JSON object representing the musical document. This change will make the editor more robust, significantly improve performance by eliminating redundant parsing, and create a clear and logical path toward the ultimate goal of a WASM-powered client.

### 2. Core Architectural Principles

*   **Model is the Source of Truth:** The client will hold a structured JSON object (the `Document` model) as the definitive state of the musical piece. All user actions will mutate this object.
*   **View is a Function of the Model:** The SVG rendered on the canvas is a stateless, visual representation of the current `Document` model. The view is always derived from the model.
*   **Editing Logic is Client-Side:** All editing operations (inserting notes, splitting lines, etc.) will be implemented in client-side JavaScript that directly manipulates the local JSON model. This provides an instantaneous user experience.
*   **Server is a Stateless Rendering Engine:** The server's role in the interactive loop is to be a "dumb" but powerful rendering service. It accepts a complete `Document` model and returns an SVG. It does not maintain any session state between requests.
*   **Clear Separation of Document and UI State:** The core `Document` model will contain only pure musical data. Transient UI state (like cursor position or selection) will be managed separately and passed to the rendering API as needed, but will not be part of the persisted document model itself.

### 3. State Management

*   **Client-Side State:** The `CanvasEditor.js` module will be the primary state holder. It will manage two key pieces of state:
    1.  `this.document`: A JSON object representing the entire musical score. This is the data that gets saved.
    2.  `this.uiState`: A simple JavaScript object containing transient information needed for rendering, such as `{ cursorPosition: 42, selectionStart: 40, selectionEnd: 45 }`. This data is *not* saved.

*   **Server-Side State:** The server is **stateless**. Each API request is atomic and must contain all the information necessary to process it. The server does not store or remember any document or UI state between API calls.

### 4. Data Models

#### 4.1. The `Document` Model
This is the central data structure, a JSON object that directly mirrors the Rust structs on the server. It represents the pure, structured data of the music.

*   **Example (Simplified):**
    ```json
    {
      "id": "doc_1a7b3c",
      "title": "My Song",
      "author": "J. Doe",
      "directives": { "notation": "number" },
      "elements": [
        {
          "Stave": {
            "lines": [
              {
                "ContentLine": {
                  "elements": [ /* notes, barlines, etc. */ ]
                }
              },
              {
                "LyricsLine": {
                  "syllables": [ /* ... */ ]
                }
              }
            ]
          }
        }
      ]
    }
    ```

#### 4.2. The `UIState` Model
This is a simple structure representing the state of the user's interaction with the editor. It is managed by the client and passed to the server only for rendering purposes.

*   **Example:**
    ```json
    {
      "cursor_position": 55,
      "selection_start": 50,
      "selection_end": 60
    }
    ```

### 5. Application Flow: The Editing Loop

This flow describes a typical user interaction, such as adding a note.

1.  **Initial Load:** The client calls `GET /api/documents/{id}`. The server responds with the full `Document` JSON, which is loaded into `canvasEditor.document`.
2.  **Initial Render:** The client immediately calls `POST /api/render/svg` with the `Document` and initial `UIState` to draw the initial view on the canvas.
3.  **User Action:** The user presses a key or clicks a button (e.g., "Insert Note").
4.  **Client-Side Model Mutation:** A JavaScript function in `CanvasEditor` is triggered. It directly modifies the `this.document` JSON object by, for example, splicing a new note object into the correct `elements` array. The `this.uiState` is also updated with the new cursor position.
5.  **Re-render Request:** After the local model is mutated, the `CanvasEditor` automatically sends a new request to `POST /api/render/svg`, containing the *entire, updated* `Document` JSON and the new `UIState`.
6.  **Server Renders:** The server receives the JSON, deserializes it into Rust structs (no parsing needed), and calls the SVG renderer.
7.  **View Update:** The server returns the new SVG string. The client receives it and repaints the canvas. The user sees the new note appear instantly.

### 6. API Endpoint Specification

The API is divided into two concerns: document persistence (CRUD) and interactive rendering.

#### 6.1. Document Persistence API

| Method | Path | Description |
| :--- | :--- | :--- |
| `GET` | `/api/documents` | List all saved documents. |
| `POST` | `/api/documents` | Create a new document. Can be from scratch or by importing from `source_text`. Returns the new `Document` with an `id`. |
| `GET` | `/api/documents/{id}` | Retrieve a single `Document` by its ID. |
| `PUT` | `/api/documents/{id}` | Save/update a document. The request body is the full `Document` JSON. |
| `DELETE` | `/api/documents/{id}` | Delete a document. |

#### 6.2. Interactive Rendering API

This is the workhorse for the editor.

| Method | Path | Description |
| :--- | :--- | :--- |
| `POST` | `/api/render/svg` | Renders a `Document` model to SVG for canvas display. |

*   **Request Body for `POST /api/render/svg`:**
    ```json
    {
      "document": { /* A full Document object */ },
      "ui_state": { /* A UIState object */ },
      "notation_type": "number"
    }
    ```
*   **Success Response (`200 OK`):**
    *   **Headers:** `Content-Type: image/svg+xml`
    *   **Body:** The raw SVG string.

### 7. Path to WASM

This architecture is the ideal stepping stone to a full WASM implementation.
1.  **Logic Prototyping:** The client-side editing logic written in JavaScript (e.g., the function to split a line in the JSON model) serves as a direct, working prototype for the final Rust functions.
2.  **Seamless Transition:** To move to WASM, the state (`Document` struct) will be moved into the WASM module's memory. The JavaScript editing functions will be ported to Rust. The `fetch` call to `/api/render/svg` will be replaced by a direct, in-memory call to a `render_svg()` function within the WASM module.
3.  **Reusable Backend:** The server-side Rust code for rendering is directly reusable in the WASM library.

This plan allows for immediate progress on the model-driven editor using a stateless server, while ensuring that all the core logic being developed is directly applicable to the final WASM architecture.
