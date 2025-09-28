// IO and persistence abstraction wrapping localStorage and server calls

export class Persistence {
  constructor(storageKey = 'musicTextDocument') {
    this.storageKey = storageKey;
    this.initialFormatsFetched = false;
  }

  loadLocal() {
    try {
      console.log('Persistence: Loading from localStorage key:', this.storageKey);

      // Try the legacy single key first
      const raw = localStorage.getItem(this.storageKey);
      if (raw) {
        console.log('Persistence: Found legacy document data');
        return JSON.parse(raw);
      }

      // Try the new UUID-based system
      if (window.LocalStorage) {
        const currentDoc = window.LocalStorage.loadCurrentDocument();
        if (currentDoc) {
          console.log('Persistence: Found UUID-based document:', currentDoc.documentUUID || 'no UUID');
          return currentDoc;
        }
      }

      console.log('Persistence: No document found in localStorage');
      return null;
    } catch (error) {
      console.error('Persistence: Error loading from localStorage:', error);
      return null;
    }
  }

  saveLocal(document) {
    try {
      console.log('Persistence: Saving document to localStorage');

      // Save to legacy key for compatibility
      localStorage.setItem(this.storageKey, JSON.stringify(document));

      // Also save to UUID-based system if document has UUID
      if (document && document.documentUUID && window.LocalStorage) {
        window.LocalStorage.saveDocument(document.documentUUID, document);
        console.log('Persistence: Saved to UUID-based storage:', document.documentUUID);
      }

      console.log('Persistence: Document saved successfully');
      return true;
    } catch (error) {
      console.error('Persistence: Error saving to localStorage:', error);
      return false;
    }
  }

  async fetchFormatsOnce(documentUUID) {
    if (this.initialFormatsFetched || !documentUUID) return null;
    this.initialFormatsFetched = true;
    // Placeholder: integrate with existing formats endpoint if present
    return null;
  }

  async renderSvg(document, ui_state) {
    // Nuke this - it's brain dead
    console.log('Persistence: renderSvg called but disabled');
    return { svg: '' };
  }
}

