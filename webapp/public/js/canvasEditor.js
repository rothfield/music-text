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

  // Load document from localStorage (DISABLED - always create new)
  async loadFromLocalStorage() {
    try {
      console.log('CanvasEditor: localStorage disabled, creating new document via API');
      // Always create a new document via API (localStorage disabled)
      await this.createNewDocument();
    } catch (error) {
      console.error('CanvasEditor: Failed to create new document:', error);
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
    // localStorage disabled - no-op
    console.log('CanvasEditor: localStorage disabled, skipping save');
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

      // Use unified response handler
      if (this.handleServerResponse(result, 'transform')) {
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

  // Capture current UI state from browser
  getCurrentUiState() {
    // Get currently active tab
    const activeTabElement = document.querySelector('.tab.active');
    const activeTab = activeTabElement ? activeTabElement.id : 'editor_svg';

    // Get current viewport state (could be extended to read from CSS transforms, scroll positions, etc.)
    const viewport = {
      scroll_x: 0,
      scroll_y: 0,
      zoom_level: 1.0
    };

    // Get current selection state (preserve from existing document if available)
    const currentSelection = this.document?.ui_state?.selection || {
      cursor_position: 0,
      cursor_uuid: null,
      selected_uuids: []
    };

    return {
      active_tab: activeTab,
      editor_mode: 'text',
      selection: currentSelection,
      viewport: viewport
    };
  }

  // Create a new document via the API (document-first architecture)
  async createNewDocument() {
    try {
      console.log('CanvasEditor: Creating new document via API...');

      // Capture current UI state to preserve user's tab selection and other preferences
      const currentUiState = this.getCurrentUiState();
      console.log('CanvasEditor: Preserving UI state:', currentUiState);

      const requestBody = {
        metadata: {
          title: 'Untitled Document',
          created_at: new Date().toISOString(),
          created_by: 'Web Interface'
        },
        ui_state: currentUiState  // Include current UI state in new document
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

        // Set document creation success flag for unified handler
        result.success = true;

        // Use unified response handler
        if (this.handleServerResponse(result, 'document_creation')) {
          this.content = this.document.content || '';
          console.log('CanvasEditor: New document setup completed');
        }
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
            position: inputCmd.position || 0, // Keep for backward compatibility
            target_uuid: inputCmd.target_uuid || this.determineTargetUuid(), // Determine target UUID automatically
            element_position: inputCmd.element_position || 0, // Position within the target element
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

      // Use unified response handler
      if (this.handleServerResponse(result, 'text_input')) {
        console.log('CanvasEditor: Text input transform completed successfully');
        console.log('CanvasEditor: Updated elements:', result.updated_elements);
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

      // Use unified response handler
      if (this.handleServerResponse(result, 'sync')) {
        console.log('CanvasEditor: Document sync completed successfully');
      }

    } catch (error) {
      console.error('CanvasEditor: Error syncing with server:', error);
    }
  }

  // Unified method to handle server responses and update UI
  handleServerResponse(result, context = 'general') {
    if (!result.success || !result.document) {
      console.error(`CanvasEditor: Server response failed in ${context}:`, result.message);
      return false;
    }

    console.log(`CanvasEditor: Processing successful server response from ${context}`);

    // Update document model
    this.document = DocumentModel.fromJSON(result.document);
    this.saveToLocalStorage();

    // Render editor SVG in main canvas (prioritize result.svg, fallback to formats)
    const editorSvg = result.svg || (result.formats && result.formats.editor_svg);
    if (editorSvg && this.renderEditorSvg) {
      console.log(`CanvasEditor: Rendering editor SVG from ${context}, length:`, editorSvg.length);
      this.renderEditorSvg(editorSvg);
    } else {
      console.log(`CanvasEditor: No editor SVG to render from ${context}`);
    }

    // Restore UI state from document (especially tab selection)
    if (result.document.ui_state && result.document.ui_state.active_tab) {
      const targetTab = result.document.ui_state.active_tab;
      console.log(`CanvasEditor: Restoring active tab from ${context}:`, targetTab);
      console.log(`CanvasEditor: window.UI available:`, !!window.UI);
      console.log(`CanvasEditor: window.UI.switchTab available:`, !!(window.UI && window.UI.switchTab));

      // Switch to the tab specified in the document's UI state
      if (window.UI && window.UI.switchTab) {
        console.log(`CanvasEditor: Calling switchTab(${targetTab})`);
        window.UI.switchTab(targetTab);
        console.log(`CanvasEditor: switchTab call completed`);
      } else {
        console.warn(`CanvasEditor: Unable to switch tab - UI or switchTab not available`);
      }
    } else {
      console.log(`CanvasEditor: No active tab to restore from ${context} - ui_state:`, result.document.ui_state);
    }

    // Update UI tabs and formats
    if (window.UI) {
      // Update formats in all tabs
      if (result.formats && window.UI.updateFormatsFromBackend) {
        console.log(`CanvasEditor: Updating formats from ${context}:`, Object.keys(result.formats));
        window.UI.updateFormatsFromBackend(result.formats);
      }

      // Update document tab
      if (window.UI.updatePipelineData) {
        console.log(`CanvasEditor: Updating pipeline data from ${context}`);
        window.UI.updatePipelineData({
          success: true,
          document: result.document,
          formats: result.formats  // Always include formats for tabs
        });
      }
    }

    console.log(`CanvasEditor: Server response handling completed for ${context}`);
    return true;
  }

  // Determine the target UUID for text insertion
  // Find the first available ContentLine UUID from the document
  determineTargetUuid() {
    try {
      if (!this.document || !this.document.elements || this.document.elements.length === 0) {
        console.warn('CanvasEditor: No document elements for UUID determination');
        return null;
      }

      // Look for the first Stave with ContentLine
      for (const element of this.document.elements) {
        if (element.Stave && element.Stave.lines) {
          for (const line of element.Stave.lines) {
            if (line.ContentLine && line.ContentLine.id) {
              console.log('CanvasEditor: Using ContentLine UUID for insertion:', line.ContentLine.id);
              return line.ContentLine.id;
            }
          }
        }
      }

      console.warn('CanvasEditor: No ContentLine found for UUID targeting');
      return null;
    } catch (error) {
      console.error('CanvasEditor: Error determining target UUID:', error);
      return null;
    }
  }
}
