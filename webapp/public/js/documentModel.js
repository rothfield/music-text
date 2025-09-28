/**
 * Document Model for Music-Text
 * Client-side document representation with UI state
 */

export class DocumentModel {
    constructor() {
        // UUID must come from server - never generate client-side
        this.documentUUID = null;
        this.version = "1.0.0";
        this.timestamp = new Date().toISOString();

        // Core document structure - matches server Document struct
        this.elements = []; // Array of elements from server
        this.content = []; // Content lines
        this.metadata = {};
        this.title = null;
        this.author = null;
        this.directives = {};
        this.char_index = 0;
        this.value = null;

        // UI state (not part of music data, but persisted for UX)
        this.ui_state = {
            selection: {
                selected_uuids: [],
                cursor_position: 0 // Character position in text
            },
            viewport: {
                scroll_x: 0,
                scroll_y: 0,
                zoom_level: 1.0
            },
            active_tab: 'editor_svg' // Which tab is currently visible
        };

        // Formats are generated on server - not stored in client model
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
        this.ui_state.selection.cursor_position = position;
        this.updateTimestamp();
    }

    // Update viewport state
    setViewport(viewport) {
        this.ui_state.viewport = { ...this.ui_state.viewport, ...viewport };
        this.updateTimestamp();
    }


    // Update timestamp
    updateTimestamp() {
        this.timestamp = new Date().toISOString();
    }

    // Export to JSON for persistence - return document as-is
    toJSON() {
        return {
            documentUUID: this.documentUUID,
            version: this.version,
            timestamp: this.timestamp,
            elements: this.elements,
            content: this.content,
            metadata: this.metadata,
            title: this.title,
            author: this.author,
            directives: this.directives,
            char_index: this.char_index,
            value: this.value,
            ui_state: this.ui_state
        };
    }

    // Static factory method to create DocumentModel from JSON
    static fromJSON(data) {
        const doc = new DocumentModel();
        return doc.fromJSON(data);
    }

    // Import from JSON - store server document as-is
    fromJSON(data) {
        this.documentUUID = data.documentUUID || null;
        this.version = data.version || "1.0.0";
        this.timestamp = data.timestamp || new Date().toISOString();
        // Store elements array directly from server
        this.elements = data.elements || [];
        this.content = data.content || [];
        this.metadata = data.metadata || {};
        // Store other document fields from server
        this.title = data.title || null;
        this.author = data.author || null;
        this.directives = data.directives || {};
        this.char_index = data.char_index || 0;
        this.value = data.value || null;
        // Deep merge UI state with defaults
        this.ui_state = {
            selection: {
                selected_uuids: [],
                cursor_position: 0,
                ...(data.ui_state?.selection || {})
            },
            viewport: {
                scroll_x: 0,
                scroll_y: 0,
                zoom_level: 1.0,
                ...(data.ui_state?.viewport || {})
            },
            active_tab: data.ui_state?.active_tab || 'editor_svg'
        };
        return this;
    }


    // Create a new document from text content
    static async fromMusicText(textContent) {
        // Server creates the document with UUIDs
        try {
            const response = await fetch('/api/documents?representations=editor_svg,vexflow,lilypond', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    content: textContent,
                    metadata: { title: 'Document from Text' }
                })
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
        if (Array.isArray(this.elements)) {
            for (const element of this.elements) {
                const type = Object.keys(element)[0] || 'unknown';
                elementTypes[type] = (elementTypes[type] || 0) + 1;
            }
        }

        return {
            total_elements: Array.isArray(this.elements) ? this.elements.length : 0,
            element_types: elementTypes,
            has_selection: this.ui_state.selection.selected_uuids.length > 0,
            last_modified: this.timestamp
        };
    }
}

// Document persistence manager
export class DocumentPersistence {
    constructor() {
        // No longer need storageKey - using UUID-based keys
    }

    // Save document to localStorage using its UUID
    saveDocument(document) {
        try {
            if (!document.documentUUID) {
                console.error('Cannot save document without documentUUID');
                return false;
            }

            const documentData = document.toJSON();
            // Use LocalStorage's new UUID-based storage
            const success = window.LocalStorage?.saveDocument(document.documentUUID, documentData);
            if (success) {
                console.log('Document saved to localStorage:', document.getStats());
            }
            return success;
        } catch (error) {
            console.error('Failed to save document to localStorage:', error);
            return false;
        }
    }

    // Load current document from localStorage
    loadDocument() {
        try {
            // Load the current document (whatever is set as active)
            const documentData = window.LocalStorage?.loadCurrentDocument();
            if (!documentData) {
                return null;
            }

            const document = new DocumentModel().fromJSON(documentData);
            console.log('Document loaded from localStorage:', document.getStats());
            return document;
        } catch (error) {
            console.error('Failed to load document from localStorage:', error);
            return null;
        }
    }

    // Load specific document by UUID
    loadDocumentByUUID(documentUUID) {
        try {
            const documentData = window.LocalStorage?.loadDocument(documentUUID);
            if (!documentData) {
                return null;
            }

            const document = new DocumentModel().fromJSON(documentData);
            console.log('Document loaded from localStorage by UUID:', document.getStats());

            // Set as current document
            this.setCurrentDocumentUUID(documentUUID);

            return document;
        } catch (error) {
            console.error('Failed to load document by UUID from localStorage:', error);
            return null;
        }
    }

    // Clear current document from localStorage
    clearDocument() {
        try {
            const currentUUID = window.LocalStorage?.loadCurrentDocumentUUID();
            if (currentUUID) {
                const success = window.LocalStorage?.deleteDocument(currentUUID);
                if (success) {
                    console.log('Current document cleared from localStorage');
                }
                return success;
            }
            return true;
        } catch (error) {
            console.error('Failed to clear document from localStorage:', error);
            return false;
        }
    }

    // Check if current document exists in localStorage
    hasDocument() {
        const currentUUID = window.LocalStorage?.loadCurrentDocumentUUID();
        return currentUUID !== null;
    }

    // Get current document UUID
    getCurrentDocumentUUID() {
        return window.LocalStorage?.loadCurrentDocumentUUID();
    }

    // Set current document UUID (switch to different document)
    setCurrentDocumentUUID(documentUUID) {
        window.LocalStorage?.saveCurrentDocumentUUID(documentUUID);
    }

    // List all stored documents
    listStoredDocuments() {
        return window.LocalStorage?.listDocuments() || [];
    }
}