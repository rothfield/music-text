/**
 * Document Model for Music-Text
 * Client-side document representation with UI state
 */

export class DocumentModel {
    constructor() {
        this.version = "1.0.0";
        this.timestamp = new Date().toISOString();

        // Core document structure
        this.elements = new Map(); // UUID -> element data
        this.content = []; // Ordered list of element UUIDs
        this.metadata = {
            title: null,
            composer: null,
            key_signature: null,
            time_signature: null,
            tempo: null
        };

        // UI state (not part of music data, but persisted for UX)
        this.ui_state = {
            selection: {
                selected_uuids: [],
                cursor_uuid: null,
                cursor_position: 0 // Character position for fallback
            },
            viewport: {
                scroll_x: 0,
                scroll_y: 0,
                zoom_level: 1.0
            },
            editor_mode: 'text', // 'text' or 'visual'
            active_tab: 'vexflow'
        };

        // Format-specific representations (cached)
        this.format_cache = {
            music_text: null, // Original text format
            lilypond: null,   // Generated LilyPond
            svg: null,        // Rendered SVG
            midi: null        // MIDI data
        };
    }

    // Add an element to the document
    addElement(uuid, elementData) {
        this.elements.set(uuid, {
            uuid,
            type: elementData.type, // 'note', 'rest', 'barline', 'lyric', etc.
            content: elementData.content,
            properties: elementData.properties || {},
            position: elementData.position || { line: 0, column: 0 },
            created_at: new Date().toISOString(),
            ...elementData
        });

        // Add to content order if not already present
        if (!this.content.includes(uuid)) {
            this.content.push(uuid);
        }

        this.invalidateCache();
    }

    // Remove an element
    removeElement(uuid) {
        this.elements.delete(uuid);
        this.content = this.content.filter(id => id !== uuid);
        this.invalidateCache();
    }

    // Get element by UUID
    getElement(uuid) {
        return this.elements.get(uuid);
    }

    // Get all elements of a specific type
    getElementsByType(type) {
        return Array.from(this.elements.values()).filter(el => el.type === type);
    }

    // Get selected elements
    getSelectedElements() {
        return this.ui_state.selection.selected_uuids
            .map(uuid => this.elements.get(uuid))
            .filter(Boolean);
    }

    // Update selection
    setSelection(uuids) {
        this.ui_state.selection.selected_uuids = Array.isArray(uuids) ? uuids : [uuids];
        this.updateTimestamp();
    }

    // Update cursor position
    setCursor(uuid, position = 0) {
        this.ui_state.selection.cursor_uuid = uuid;
        this.ui_state.selection.cursor_position = position;
        this.updateTimestamp();
    }

    // Update viewport state
    setViewport(viewport) {
        this.ui_state.viewport = { ...this.ui_state.viewport, ...viewport };
        this.updateTimestamp();
    }

    // Cache format representations
    cacheFormat(format, data) {
        this.format_cache[format] = data;
        this.updateTimestamp();
    }

    // Get cached format
    getCachedFormat(format) {
        return this.format_cache[format];
    }

    // Invalidate all cached formats (call after document changes)
    invalidateCache() {
        this.format_cache = {
            music_text: null,
            lilypond: null,
            svg: null,
            midi: null
        };
        this.updateTimestamp();
    }

    // Update timestamp
    updateTimestamp() {
        this.timestamp = new Date().toISOString();
    }

    // Export to JSON for persistence
    toJSON() {
        return {
            version: this.version,
            timestamp: this.timestamp,
            elements: Object.fromEntries(this.elements),
            content: this.content,
            metadata: this.metadata,
            ui_state: this.ui_state,
            format_cache: this.format_cache
        };
    }

    // Import from JSON
    fromJSON(data) {
        this.version = data.version || "1.0.0";
        this.timestamp = data.timestamp || new Date().toISOString();
        this.elements = new Map(Object.entries(data.elements || {}));
        this.content = data.content || [];
        this.metadata = data.metadata || {};
        this.ui_state = {
            selection: { selected_uuids: [], cursor_uuid: null, cursor_position: 0 },
            viewport: { scroll_x: 0, scroll_y: 0, zoom_level: 1.0 },
            editor_mode: 'text',
            active_tab: 'vexflow',
            ...data.ui_state
        };
        this.format_cache = {
            music_text: null,
            lilypond: null,
            svg: null,
            midi: null,
            ...data.format_cache
        };
        return this;
    }

    // Create a new document from music-text format
    static async fromMusicText(textContent) {
        const doc = new DocumentModel();
        doc.cacheFormat('music_text', textContent);

        // Parse the text content into document structure
        // This would normally call the server parser, but for now we'll create a minimal structure
        try {
            const response = await fetch('/api/parse-music-text', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ text: textContent })
            });

            if (response.ok) {
                const parseResult = await response.json();
                if (parseResult.success && parseResult.document) {
                    // Populate document from server-parsed structure
                    doc.populateFromServerDocument(parseResult.document);
                }
            }
        } catch (error) {
            console.warn('Failed to parse music text on server, using minimal structure:', error);
        }

        return doc;
    }

    // Populate document from server-parsed document structure
    populateFromServerDocument(serverDoc) {
        // This is where we'd map the server's document model to our client model
        // For now, just store basic metadata
        if (serverDoc.metadata) {
            this.metadata = { ...this.metadata, ...serverDoc.metadata };
        }

        if (serverDoc.elements) {
            for (const [uuid, element] of Object.entries(serverDoc.elements)) {
                this.addElement(uuid, element);
            }
        }
    }

    // Get statistics about the document
    getStats() {
        const elementTypes = {};
        for (const element of this.elements.values()) {
            elementTypes[element.type] = (elementTypes[element.type] || 0) + 1;
        }

        return {
            total_elements: this.elements.size,
            element_types: elementTypes,
            has_selection: this.ui_state.selection.selected_uuids.length > 0,
            last_modified: this.timestamp
        };
    }
}

// Document persistence manager
export class DocumentPersistence {
    constructor(storageKey = 'musicTextDocument') {
        this.storageKey = storageKey;
    }

    // Save document to localStorage
    saveDocument(document) {
        try {
            const jsonData = JSON.stringify(document.toJSON());
            localStorage.setItem(this.storageKey, jsonData);
            console.log('Document saved to localStorage:', document.getStats());
            return true;
        } catch (error) {
            console.error('Failed to save document to localStorage:', error);
            return false;
        }
    }

    // Load document from localStorage
    loadDocument() {
        try {
            const jsonData = localStorage.getItem(this.storageKey);
            if (!jsonData) {
                return null;
            }

            const data = JSON.parse(jsonData);
            const document = new DocumentModel().fromJSON(data);
            console.log('Document loaded from localStorage:', document.getStats());
            return document;
        } catch (error) {
            console.error('Failed to load document from localStorage:', error);
            return null;
        }
    }

    // Clear document from localStorage
    clearDocument() {
        try {
            localStorage.removeItem(this.storageKey);
            console.log('Document cleared from localStorage');
            return true;
        } catch (error) {
            console.error('Failed to clear document from localStorage:', error);
            return false;
        }
    }

    // Check if document exists in localStorage
    hasDocument() {
        return localStorage.getItem(this.storageKey) !== null;
    }
}