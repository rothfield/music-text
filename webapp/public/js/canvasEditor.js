import { CanvasController } from './canvas/controller.js';
import { DocumentModel } from './documentModel.js';

// Full-featured CanvasEditor backed by the modular controller
export class CanvasEditor {
  constructor() {
    this._controller = null;
    this.document = new DocumentModel();
    this.onContentChange = null;
    this.onSelectionChange = null;
  }

  async init(containerId = 'canvasEditor') {
    this.container = document.getElementById(containerId);
    if (!this.container) {
      console.error('CanvasEditor: Container not found:', containerId);
    } else {
      console.log('CanvasEditor: Container found:', containerId);
    }

    // Load document from localStorage first
    await this.loadFromLocalStorage();

    try {
      this._controller = new CanvasController('svg-container').init();

      // Set up text input callback to handle server communication
      this._controller.onTextInput = (inputCmd) => {
        console.log('CanvasEditor: Text input from controller:', inputCmd);
        this.handleTextInput(inputCmd);
      };

      console.log('CanvasEditor: CanvasController initialized successfully');
    } catch (error) {
      console.error('CanvasEditor: CanvasController init failed:', error);
      // Continue without controller for basic functionality
    }

    window.musicApp = window.musicApp || {};
    window.musicApp.canvasEditor = this;
    return this;
  }

  // Load document from localStorage
  async loadFromLocalStorage() {
    try {
      console.log('CanvasEditor: Loading document from localStorage...');
      if (!window.LocalStorage) {
        console.error('CanvasEditor: LocalStorage not available');
        return;
      }

      const currentDoc = window.LocalStorage.loadCurrentDocument();
      if (currentDoc) {
        console.log('CanvasEditor: Found document in localStorage:', currentDoc.documentUUID || 'no UUID');
        this.document.fromJSON(currentDoc);
        this.content = this.document.content || '';
        console.log('CanvasEditor: Document loaded from localStorage successfully');

        // Document-first architecture: Send to server for processing
        if (this.document.documentUUID) {
          console.log('CanvasEditor: Document has UUID, will sync with server:', this.document.documentUUID);
          console.log('CanvasEditor: Document content before sync:', this.document.content);
          console.log('CanvasEditor: Full document before sync:', this.document);
          this.syncWithServer();
        } else {
          console.warn('CanvasEditor: Document has no UUID, cannot sync with server');
        }
      } else {
        console.log('CanvasEditor: No document found in localStorage, creating new document via API');
        // Create a new document via API if none exists
        await this.createNewDocument();
      }
    } catch (error) {
      console.error('CanvasEditor: Failed to load document from localStorage:', error);
      this.document = new DocumentModel();
      this.content = '';
    }
  }

  async loadDocument(doc) {
    if (doc) {
      console.log('CanvasEditor: Loading document:', doc.documentUUID || 'no UUID');
      try {
        this.document.fromJSON(doc);
        console.log('CanvasEditor: Document loaded successfully');
      } catch (error) {
        console.error('CanvasEditor: Failed to load document:', error);
      }
    } else {
      console.warn('CanvasEditor: loadDocument called with null/undefined doc');
    }
    return this._controller?.loadDocument(doc);
  }

  // Expected by app.js: set the editor text value directly
  async setValue(value) {
    if (!this._controller) {
      console.log('CanvasEditor: Controller not initialized, initializing...');
      this.init();
    } else {
      console.log('CanvasEditor: Setting value:', (value || '').length, 'characters');
    }

    // Update our document model
    try {
      this.document.content = value || '';
      console.log('CanvasEditor: Document content updated');
    } catch (error) {
      console.error('CanvasEditor: Failed to set content:', error);
    }

    const state = this._controller?.store?.getState?.();
    const doc = state?.document;

    if (!doc) {
      console.log('CanvasEditor: No document in controller, creating basic document');
      // Create a basic document if controller doesn't have one
      const newDoc = {
        content: value || '',
        ui_state: { selection: { cursor_position: (value || '').length } }
      };
      await this._controller?.loadDocument(newDoc);
      return;
    } else {
      console.log('CanvasEditor: Using existing document from controller');
    }

    // Create a copy to modify
    const newDoc = { ...doc };

    if (typeof value === 'string') {
      newDoc.content = value;
      // place cursor at end
      newDoc.ui_state = newDoc.ui_state || { selection: {} };
      newDoc.ui_state.selection = newDoc.ui_state.selection || {};
      newDoc.ui_state.selection.cursor_position = value.length;
    }
    await this._controller?.loadDocument(newDoc);

    // Trigger content change callback
    if (this.onContentChange) {
      console.log('CanvasEditor: Triggering content change callback');
      this.onContentChange(value || '');
    } else {
      console.log('CanvasEditor: No content change callback registered');
    }
  }

  // Optional getter symmetry
  getValue() {
    const state = this._controller?.store?.getState?.();
    return state?.document?.content ?? this.document.content ?? '';
  }

  // Update with parse results from the server
  updateParseResult(result) {
    if (result && result.success && result.document) {
      try {
        this.document = DocumentModel.fromJSON(result.document);
        this.saveToLocalStorage();

        // Update document tab with the new document data
        if (window.UI && window.UI.updatePipelineData) {
          window.UI.updatePipelineData(result);
        }
      } catch (error) {
        console.error('Failed to update parse result:', error);
      }
    }
  }

  // Get selected UUIDs (stub for now)
  getSelectedUuids() {
    // For now, return empty array - selection will be implemented later
    return [];
  }

  // Copy selection (stub)
  copySelection() {
    console.log('Copy selection - not implemented yet');
  }

  // Paste content (stub)
  paste() {
    console.log('Paste - not implemented yet');
  }

  // Focus the editor
  focus() {
    const container = document.getElementById('svg-container') || document.getElementById('canvasEditor');
    if (container) {
      container.focus();
    }
  }

  // Clear the canvas/content
  clearCanvas() {
    this.setValue('');
  }

  // Save to localStorage
  saveToLocalStorage() {
    try {
      if (!this.document) {
        console.warn('CanvasEditor: No document to save');
        return;
      }

      if (!window.LocalStorage) {
        console.error('CanvasEditor: LocalStorage not available');
        return;
      }

      const documentUUID = this.document.documentUUID;
      if (!documentUUID) {
        console.warn('CanvasEditor: Document has no UUID, cannot save to localStorage');
        return;
      }

      const documentData = this.document.toJSON();
      const success = window.LocalStorage.saveDocument(documentUUID, documentData);

      if (success) {
        console.log('CanvasEditor: Document saved to localStorage successfully:', documentUUID);
      } else {
        console.error('CanvasEditor: Failed to save document to localStorage');
      }
    } catch (error) {
      console.error('CanvasEditor: Error saving to localStorage:', error);
    }
  }

  // Execute a transform operation
  async executeTransform(transformType, params) {
    if (!this.document || !this.document.documentUUID) {
      throw new Error('No document loaded');
    }

    const selectedUuids = this.getSelectedUuids();
    if (selectedUuids.length === 0) {
      throw new Error('No elements selected');
    }

    try {
      // POST to transform endpoint using new document-first API
      const response = await fetch('/api/documents/transform', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          document: this.document.toJSON(),
          command_type: transformType,
          target_uuids: selectedUuids,
          parameters: params
        })
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.message || `HTTP ${response.status}`);
      }

      const result = await response.json();

      if (result.success && result.document) {
        // Replace document with server response
        this.document = DocumentModel.fromJSON(result.document);
        this.saveToLocalStorage();

        // Update UI with any SVG from transform
        if (result.svg && this.renderEditorSvg) {
          this.renderEditorSvg(result.svg);
        }

        return result;
      } else {
        throw new Error(result.message || 'Transform failed');
      }
    } catch (error) {
      console.error('Transform error:', error);
      throw error;
    }
  }

  // Execute a semantic command
  async executeSemanticCommand(command, params) {
    // For now, delegate to executeTransform
    return this.executeTransform(command, params);
  }

  // Render editor SVG (stub)
  renderEditorSvg(svgContent) {
    console.log('Render editor SVG - received content length:', svgContent?.length);
    // This would render SVG in the canvas area
    const container = document.getElementById('svg-container');
    if (container && svgContent) {
      container.innerHTML = svgContent;
    }
  }

  // Create a new document via the API (document-first architecture)
  async createNewDocument() {
    try {
      console.log('CanvasEditor: Creating new document via API...');

      const requestBody = {
        metadata: {
          title: 'Untitled Document',
          created_at: new Date().toISOString(),
          created_by: 'Web Interface'
        }
      };

      const response = await fetch('/api/documents?representations=editor_svg,vexflow,lilypond', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(requestBody)
      });

      if (!response.ok) {
        throw new Error(`Failed to create document: ${response.status}`);
      }

      const result = await response.json();

      if (result.document && result.document.documentUUID) {
        console.log('CanvasEditor: New document created:', result.document.documentUUID);
        console.log('CanvasEditor: Document data:', result.document);

        // Update local document with server response
        this.document.fromJSON(result.document);
        this.content = this.document.content || '';

        // Save to localStorage
        this.saveToLocalStorage();

        // Update UI with any formats from server
        if (result.formats && window.UI) {
          if (window.UI.updateFormatsFromBackend) {
            window.UI.updateFormatsFromBackend(result.formats);
          }

          // Render editor SVG in main canvas area
          if (result.formats.editor_svg && this.renderEditorSvg) {
            this.renderEditorSvg(result.formats.editor_svg);
          } else {
            console.log('CanvasEditor: No editor_svg to render or method missing');
          }
        }

        console.log('CanvasEditor: New document setup completed');
      } else {
        throw new Error('Invalid response from document creation API');
      }

    } catch (error) {
      console.error('CanvasEditor: Failed to create new document:', error);
      // Fallback: create local document without UUID
      this.document = new DocumentModel();
      this.content = '';
    }
  }

  // Handle text input commands from CanvasController
  async handleTextInput(inputCmd) {
    console.log('CanvasEditor: Handling text input command:', inputCmd);

    if (!this.document || !this.document.documentUUID) {
      console.warn('CanvasEditor: No document to modify');
      return;
    }

    try {
      // Determine command type based on input operation
      let commandType = '';
      switch (inputCmd.type) {
        case 'insert':
          commandType = 'insert_text';
          break;
        case 'deleteBackward':
          commandType = 'delete_backward';
          break;
        case 'deleteForward':
          commandType = 'delete_forward';
          break;
        default:
          console.warn('CanvasEditor: Unknown input command type:', inputCmd.type);
          return;
      }

      // Send to transform endpoint using new document-first API
      const response = await fetch('/api/documents/transform', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          document: this.document.toJSON(),
          command_type: commandType, // insert_text, delete_backward, etc.
          target_uuids: [], // Empty for text operations
          parameters: {
            position: inputCmd.position || 0,
            text: inputCmd.value || '',
            direction: inputCmd.type === 'deleteBackward' ? 'backward' :
                      inputCmd.type === 'deleteForward' ? 'forward' : undefined
          }
        })
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.message || `HTTP ${response.status}`);
      }

      const result = await response.json();

      if (result.success && result.document) {
        // Replace document with server response (mutated document)
        this.document = DocumentModel.fromJSON(result.document);
        this.saveToLocalStorage();

        // MOST IMPORTANT: Render editor SVG first
        if (result.formats && result.formats.editor_svg && this.renderEditorSvg) {
          console.log('CanvasEditor: Rendering editor SVG after text input, length:', result.formats.editor_svg.length);
          this.renderEditorSvg(result.formats.editor_svg);
        }

        // Update UI with server response formats
        if (result.formats && window.UI) {
          if (window.UI.updateFormatsFromBackend) {
            window.UI.updateFormatsFromBackend(result.formats);
          }

          // Update document tab with mutated document
          if (window.UI.updatePipelineData) {
            window.UI.updatePipelineData({
              success: true,
              document: result.document
            });
          }
        }

        console.log('CanvasEditor: Text input transform completed successfully');
        console.log('CanvasEditor: Updated elements:', result.updated_elements);
      } else {
        console.error('CanvasEditor: Transform returned error:', result.message);
      }

    } catch (error) {
      console.error('CanvasEditor: Error processing text input:', error);
    }
  }

  // Document-first architecture: Send complete document to server for processing
  async syncWithServer() {
    try {
      if (!this.document || !this.document.documentUUID) {
        console.warn('CanvasEditor: Cannot sync - no document or UUID');
        return;
      }

      console.log('CanvasEditor: Syncing document with server:', this.document.documentUUID);

      // Send entire document to server following document-first pattern
      const documentData = this.document.toJSON();
      console.log('CanvasEditor: Sending document data:', documentData);

      // POST to render endpoint for representations (no save)
      const response = await fetch('/api/documents/render?representations=editor_svg,vexflow,lilypond', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ document: documentData })
      });

      if (!response.ok) {
        console.error('CanvasEditor: Server sync failed:', response.status);
        // Try to get error details
        try {
          const errorText = await response.text();
          console.error('CanvasEditor: Server error details:', errorText);
        } catch (e) {
          console.error('CanvasEditor: Could not read error response');
        }
        return;
      }

      const result = await response.json();

      if (result.success) {
        console.log('CanvasEditor: Server sync successful');

        // Replace document with server response (e.g., timestamp updates)
        if (result.document) {
          this.document = DocumentModel.fromJSON(result.document);
          console.log('CanvasEditor: Document replaced from server');
        }

        // Save updated document to localStorage
        this.saveToLocalStorage();

        // Update UI with any format data from server
        if (result.formats && window.UI) {
          if (window.UI.updateFormatsFromBackend) {
            window.UI.updateFormatsFromBackend(result.formats);
          }

          // Update document tab with current document data
          if (window.UI.updatePipelineData) {
            const uiResult = {
              success: true,
              document: this.document.toJSON()
            };
            window.UI.updatePipelineData(uiResult);
          }

          // Render editor SVG in main canvas area
          if (result.formats.editor_svg) {
            this.renderEditorSvg(result.formats.editor_svg);
          } else {
            console.log('CanvasEditor: No editor_svg in sync response');
          }
        }

        console.log('CanvasEditor: Document sync completed successfully');
      } else {
        console.error('CanvasEditor: Server returned error:', result.message);
      }

    } catch (error) {
      console.error('CanvasEditor: Error syncing with server:', error);
    }
  }
}
