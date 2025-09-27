/**
 * UUID-based Document Storage Management
 * Handles document persistence with UUID-based keys
 */

export const LocalStorage = {
    // Current document management
    saveCurrentDocumentUUID(documentUUID) {
        try {
            localStorage.setItem('musictext_current_document', documentUUID);
        } catch (e) {
            console.warn('Failed to save current document UUID:', e);
        }
    },

    loadCurrentDocumentUUID() {
        try {
            return localStorage.getItem('musictext_current_document');
        } catch (e) {
            console.warn('Failed to load current document UUID:', e);
            return null;
        }
    },

    // Document storage by UUID
    saveDocument(documentUUID, documentData) {
        try {
            const key = `musictext_document_${documentUUID}`;
            localStorage.setItem(key, JSON.stringify(documentData));
            // Also set as current document
            this.saveCurrentDocumentUUID(documentUUID);
            return true;
        } catch (e) {
            console.warn('Failed to save document:', e);
            return false;
        }
    },

    loadDocument(documentUUID) {
        try {
            const key = `musictext_document_${documentUUID}`;
            const data = localStorage.getItem(key);
            return data ? JSON.parse(data) : null;
        } catch (e) {
            console.warn('Failed to load document:', e);
            return null;
        }
    },

    // Load current document (without needing UUID)
    loadCurrentDocument() {
        const currentUUID = this.loadCurrentDocumentUUID();
        if (!currentUUID) {
            return null;
        }
        return this.loadDocument(currentUUID);
    },

    // Document management
    deleteDocument(documentUUID) {
        try {
            const key = `musictext_document_${documentUUID}`;
            localStorage.removeItem(key);

            // If this was the current document, clear current pointer
            const currentUUID = this.loadCurrentDocumentUUID();
            if (currentUUID === documentUUID) {
                localStorage.removeItem('musictext_current_document');
            }
            return true;
        } catch (e) {
            console.warn('Failed to delete document:', e);
            return false;
        }
    },

    // List all stored documents
    listDocuments() {
        try {
            const documents = [];
            for (let i = 0; i < localStorage.length; i++) {
                const key = localStorage.key(i);
                if (key && key.startsWith('musictext_document_')) {
                    const documentUUID = key.replace('musictext_document_', '');
                    const data = JSON.parse(localStorage.getItem(key));
                    documents.push({
                        documentUUID,
                        timestamp: data.timestamp,
                        title: data.metadata?.title || 'Untitled'
                    });
                }
            }
            return documents.sort((a, b) => new Date(b.timestamp) - new Date(a.timestamp));
        } catch (e) {
            console.warn('Failed to list documents:', e);
            return [];
        }
    },

    // Settings that are not document-specific
    saveActiveTab(tabName) {
        try {
            localStorage.setItem('musictext_active_tab', tabName);
        } catch (e) {
            console.warn('Failed to save active tab:', e);
        }
    },

    loadActiveTab() {
        try {
            return localStorage.getItem('musictext_active_tab') || 'vexflow';
        } catch (e) {
            console.warn('Failed to load active tab:', e);
            return 'vexflow';
        }
    },

    saveFontPreference(fontClass) {
        try {
            localStorage.setItem('musictext_font', fontClass);
        } catch (e) {
            console.warn('Failed to save font preference:', e);
        }
    },

    loadFontPreference() {
        try {
            return localStorage.getItem('musictext_font') || 'font-default';
        } catch (e) {
            console.warn('Failed to load font preference:', e);
            return 'font-default';
        }
    },

    saveNotationType(notationType) {
        try {
            localStorage.setItem('musictext_notation_type', notationType);
        } catch (e) {
            console.warn('Failed to save notation type:', e);
        }
    },

    loadNotationType() {
        try {
            // Notation type is no longer used by the API; keep stored value for UI only
            return localStorage.getItem('musictext_notation_type') || 'number';
        } catch (e) {
            console.warn('Failed to load notation type:', e);
            return 'number';
        }
    },

    // Clear all document data (keep settings)
    clearAllDocuments() {
        try {
            const keysToRemove = [];
            for (let i = 0; i < localStorage.length; i++) {
                const key = localStorage.key(i);
                if (key && (key.startsWith('musictext_document_') || key === 'musictext_current_document')) {
                    keysToRemove.push(key);
                }
            }
            keysToRemove.forEach(key => localStorage.removeItem(key));
        } catch (e) {
            console.warn('Failed to clear documents:', e);
        }
    },

    // Clear everything
    clearAll() {
        try {
            const keysToRemove = [];
            for (let i = 0; i < localStorage.length; i++) {
                const key = localStorage.key(i);
                if (key && key.startsWith('musictext_')) {
                    keysToRemove.push(key);
                }
            }
            keysToRemove.forEach(key => localStorage.removeItem(key));
        } catch (e) {
            console.warn('Failed to clear localStorage:', e);
        }
    }
};
