/**
 * Music Text Web Interface - Main Application Module
 * Orchestrates the entire web application
 */

import { LocalStorage } from './localStorage.js';
import { UI } from './ui.js';
import { API } from './api.js';
import { FontManager } from './fontManager.js';
import { CanvasEditor } from './canvasEditor.js';
import { DocumentModel } from './documentModel.js';
import { MusicTextPlayer } from './midiPlayer.js';

// Make LocalStorage available globally for DocumentPersistence
window.LocalStorage = LocalStorage;

class MusicTextApp {
    constructor() {
        this.currentParseResult = null;
        this.inputTimer = null;
        this.canvasEditor = new CanvasEditor();
        this.midiPlayer = null;
    }

    // Initialize the application
    async init() {
        try {
            console.log('🎵 Starting Music Text App initialization...');
            await this.setupUI();
            console.log('✅ UI setup complete');

            // Don't initialize MIDI until needed
            this.midiInitialized = false;
            console.log('✅ MIDI initialization deferred');

            this.setupEventListeners();
            console.log('✅ Event listeners setup complete');

            this.restoreState();
            console.log('✅ State restoration complete');

            console.log('✅ Music Text App initialized with modular architecture');
        } catch (error) {
            console.error('❌ Failed to initialize app:', error);
            UI.setStatus('Failed to initialize application', 'error');
        }
    }

    // Setup UI components
    async setupUI() {
        try {
            // Initialize canvas editor
            console.log('🎨 Initializing canvas editor...');
            await this.canvasEditor.init('canvasEditor');
            console.log('✅ Canvas editor initialized');

            // Initialize font manager
            console.log('🔤 Initializing font manager...');
            FontManager.init();
            console.log('✅ Font manager initialized');

            // Load and set notation type from localStorage (if dropdown exists)
            const savedNotationType = LocalStorage.loadNotationType();
            const notationSelect = document.getElementById('notationTypeSelect');
            if (notationSelect) {
                notationSelect.value = savedNotationType;
                console.log('✅ Notation type restored:', savedNotationType);
            } else {
                console.log('⚠️ Notation type select not found');
            }

            // Setup initial UI state
            console.log('🎯 Setting up initial tab state...');
            this.setupInitialTabState();
            console.log('✅ Initial tab state set');

        } catch (error) {
            console.error('❌ Error in setupUI:', error);
            throw error;
        }
    }

    // Setup initial tab state - always default to vexflow
    setupInitialTabState() {
        const defaultTab = 'vexflow';

        // Set the active tab without calling switchTab
        document.querySelectorAll('.tab').forEach(tab => tab.classList.remove('active'));
        document.querySelectorAll('.tab-content').forEach(content => content.classList.remove('active'));
        document.querySelector(`[onclick*="${defaultTab}"]`)?.classList.add('active');
        document.getElementById(`${defaultTab}-tab`)?.classList.add('active');
    }

    // Setup MIDI player
    async setupMIDI() {
        try {
            this.midiPlayer = new MusicTextPlayer();
            await this.midiPlayer.init();

            // Setup MIDI event listeners
            this.midiPlayer.on('play', () => {
                document.getElementById('playBtn').disabled = true;
                document.getElementById('pauseBtn').disabled = false;
                document.getElementById('stopBtn').disabled = false;
                UI.setStatus('🎵 MIDI playback started', 'success');
            });

            this.midiPlayer.on('pause', () => {
                document.getElementById('playBtn').disabled = false;
                document.getElementById('pauseBtn').disabled = true;
                UI.setStatus('⏸️ MIDI playback paused', 'loading');
            });

            this.midiPlayer.on('stop', () => {
                document.getElementById('playBtn').disabled = false;
                document.getElementById('pauseBtn').disabled = true;
                document.getElementById('stopBtn').disabled = true;
                UI.setStatus('⏹️ MIDI playback stopped', 'success');
            });

            console.log('✅ MIDI player setup complete');
        } catch (error) {
            console.warn('MIDI player setup failed:', error);
            // Hide MIDI controls if setup fails
            const midiControls = document.querySelector('.midi-controls');
            if (midiControls) {
                midiControls.style.display = 'none';
            }
        }
    }

    // Setup event listeners
    setupEventListeners() {
        // Setup event listeners for main control buttons
        document.getElementById('newDocButton').addEventListener('click', () => this.createNewDocument());
        document.getElementById('clearButton').addEventListener('click', () => this.clearAll());
        document.getElementById('clearStorageButton').addEventListener('click', () => this.clearLocalStorage());
        document.getElementById('copyButton').addEventListener('click', () => this.canvasEditor.copySelection());
        document.getElementById('pasteButton').addEventListener('click', () => this.canvasEditor.paste());
        document.getElementById('lilypondButton').addEventListener('click', () => this.generateSVG());
        document.getElementById('playButton').addEventListener('click', () => this.playMidi());
        document.getElementById('stopButton').addEventListener('click', () => this.stopMidi());

        // Set up canvas editor event listeners
        this.canvasEditor.onContentChange = (content) => {
            // Debounced parsing for real-time updates
            clearTimeout(this.inputTimer);
            this.inputTimer = setTimeout(() => {
                if (content.trim()) {
                    this.parseAndUpdatePreview();
                }
            }, 300);
        };

        this.canvasEditor.onSelectionChange = (selection) => {
            this.updateOctaveButtonStates();
        };
    }

    // Restore application state from localStorage
    restoreState() {
        // Document state is restored automatically by CanvasEditor.
        // Restore active tab.
        const activeTab = LocalStorage.loadActiveTab();
        if (activeTab && activeTab !== 'vexflow') {
            UI.switchTab(activeTab);
        }
    }

    // Parse and update preview (real-time, no status messages)
    async parseAndUpdatePreview() {
        const input = this.canvasEditor.getValue();
        
        if (!input.trim()) {
            return;
        }

        try {
            const result = await API.parseForPreview(input);
            this.currentParseResult = result;
            
            // Update all outputs
            UI.updatePipelineData(result);
            UI.updateLilyPondOutput(result);
            UI.updateSourceOutput(result);
            await UI.updateVexFlowOutput(result);
            
            // Update canvas editor with parse results
            if (result.success) {
                this.canvasEditor.updateParseResult(result);
            }
            
        } catch (error) {
            console.warn('Parse error during preview:', error.message);
            document.getElementById('vexflow-output').innerHTML = `<p>Parse error: ${error.message}</p>`;
        }
    }

    // Manual parse (triggered by Parse button)
    async parseMusic() {
        const input = this.canvasEditor.getValue();
        
        // Validate input
        const validation = API.validateInput(input);
        if (!validation.valid) {
            UI.setStatus(validation.error, 'error');
            return;
        }
        
        UI.setStatus('Parsing notation...', 'loading');
        
        try {
            const result = await API.parse(input);
            this.currentParseResult = result;

            // Update all outputs
            UI.updatePipelineData(result);
            UI.updateLilyPondOutput(result);
            UI.updateSourceOutput(result);

            if (API.hasVexFlowData(result)) {
                await UI.updateVexFlowOutput(result);
                UI.setStatus('Parse successful! VexFlow preview updated.', 'success');
            } else if (API.isSuccessfulResult(result)) {
                await UI.updateVexFlowOutput(result);
                UI.setStatus('Parse successful! (No VexFlow data)', 'success');
            } else {
                await UI.updateVexFlowOutput(result);
                UI.setStatus('Parse failed.', 'error');
            }
            
            // Auto-switch to Editor SVG tab
            UI.switchTab('editor_svg');
            
        } catch (error) {
            UI.setStatus(`Error: ${error.message}`, 'error');
            document.getElementById('vexflow-output').innerHTML = `<p>Error: ${error.message}</p>`;
        }
    }



    // Generate LilyPond PNG (triggered by LilyPond button)
    async generateSVG() {
        // Check if we have a document UUID
        const documentUUID = this.canvasEditor.document.documentUUID;
        if (!documentUUID) {
            UI.setStatus('No document loaded', 'error');
            return;
        }

        UI.setStatus('Generating LilyPond PNG...', 'loading');

        try {
            // Call the RESTful API to export as LilyPond PNG
            const response = await fetch('/api/documents/export', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    document: this.canvasEditor.document.toJSON(),
                    format: 'lilypond-png'
                })
            });

            if (!response.ok) {
                const error = await response.json();
                throw new Error(error.message || `HTTP ${response.status}`);
            }

            const result = await response.json();

            if (result.success && result.content) {
                // Display the PNG in the LilyPond SVG tab
                const lilypondSvgOutput = document.getElementById('lilypond_svg-output');
                if (lilypondSvgOutput) {
                    // result.content is a data URL (base64 encoded PNG)
                    lilypondSvgOutput.innerHTML = `<img src="${result.content}" alt="LilyPond notation" style="max-width: 100%; height: auto;">`;
                }

                UI.setStatus('LilyPond PNG generated successfully!', 'success');
                UI.switchTab('lilypond_svg');
            } else {
                UI.setStatus(result.message || 'LilyPond generation failed', 'error');
            }

        } catch (error) {
            console.error('LilyPond generation error:', error);
            UI.setStatus(`Error: ${error.message}`, 'error');

            // Show error in the output tab
            const lilypondSvgOutput = document.getElementById('lilypond_svg-output');
            if (lilypondSvgOutput) {
                lilypondSvgOutput.innerHTML = `<p style="color: red;">Error: ${error.message}</p>`;
            }
        }
    }


    // Clear all content and localStorage
    clearAll() {
        UI.clearAllContent();
        this.canvasEditor.setValue('');
        this.currentParseResult = null;

        // Document clearing is now handled by DocumentPersistence

        // Switch back to Editor SVG tab and restore focus
        UI.switchTab('editor_svg');
    }

    // Clear localStorage completely
    clearLocalStorage() {
        if (confirm('Clear all localStorage data? This will remove all saved documents.')) {
            try {
                // Clear all musictext-related localStorage keys
                const keysToRemove = [];
                for (let i = 0; i < localStorage.length; i++) {
                    const key = localStorage.key(i);
                    if (key && (key.startsWith('musictext_') || key.startsWith('musicText'))) {
                        keysToRemove.push(key);
                    }
                }

                keysToRemove.forEach(key => localStorage.removeItem(key));

                // Reset the current document
                this.canvasEditor.document = null;
                this.canvasEditor.clearCanvas();

                UI.setStatus('LocalStorage cleared', 'success');
                console.log('Cleared localStorage keys:', keysToRemove);
            } catch (error) {
                console.error('Failed to clear localStorage:', error);
                UI.setStatus('Failed to clear localStorage', 'error');
            }
        }
    }

    // Create a new document using the API
    async createNewDocument() {
        try {
            UI.setStatus('Creating new document...', 'loading');

            // Create a blank document immediately without dialog
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
                // Clear current content
                this.clearAll();

                // Set up new document from API response - UUID is now in document itself
                this.canvasEditor.document.documentUUID = result.document.documentUUID;

                // Document starts blank - no need to load content

                // Update document metadata
                if (result.document.metadata) {
                    this.canvasEditor.document.metadata = result.document.metadata;
                }

                // Save the new document state
                this.canvasEditor.saveToLocalStorage();

                UI.setStatus(`New document created (ID: ${result.document.documentUUID.slice(0, 8)}...)`, 'success');

                // Fetch the complete document data by UUID for proper server-authoritative data
                try {
                    const docResponse = await fetch('/api/documents/render?representations=editor_svg,vexflow,lilypond', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ document: result.document })
                    });
                    if (docResponse.ok) {
                        const docData = await docResponse.json();

                        // Update the document model with server data
                        if (docData.document) {
                            this.canvasEditor.document.fromJSON(docData.document);
                        }

                        // Update UI tabs with complete server-side document data
                        const uiResult = {
                            success: true,
                            document: docData.document,
                            editor_svg: docData.formats.editor_svg
                        };

                        // Update document tab
                        if (UI.updatePipelineData) {
                            UI.updatePipelineData(uiResult);
                        }

                        // Update all format tabs if formats are available
                        if (docData.formats) {
                            // Update all tabs with the formats
                            if (window.UI && window.UI.updateFormatsFromBackend) {
                                window.UI.updateFormatsFromBackend(docData.formats);
                            }

                            // Render editor SVG if available
                            if (docData.formats.editor_svg && this.canvasEditor.renderEditorSvg) {
                                console.log('Rendering new document SVG, length:', docData.formats.editor_svg.length);
                                this.canvasEditor.renderEditorSvg(docData.formats.editor_svg);
                            }
                        } else {
                            console.warn('No formats in response');
                        }
                    } else {
                        // Fallback to creation response data if GET fails
                        if (result.document) {
                            this.canvasEditor.document.fromJSON(result.document);
                        }

                        if (result.formats) {
                            const uiResult = {
                                success: true,
                                document: result.document,
                                editor_svg: result.formats.editor_svg
                            };

                            if (UI.updatePipelineData) {
                                UI.updatePipelineData(uiResult);
                            }

                            if (result.formats && result.formats.editor_svg && this.canvasEditor.renderEditorSvg) {
                                this.canvasEditor.renderEditorSvg(result.formats.editor_svg);
                            }
                        }
                    }
                } catch (fetchError) {
                    console.warn('Failed to fetch document by UUID, using creation response:', fetchError);
                    // Fallback to creation response data
                    if (result.document) {
                        this.canvasEditor.document.fromJSON(result.document);
                    }

                    // Update all format tabs from creation response
                    if (result.formats) {
                        // Update all tabs with the formats
                        if (window.UI && window.UI.updateFormatsFromBackend) {
                            window.UI.updateFormatsFromBackend(result.formats);
                        }

                        // Render editor SVG if available
                        if (result.formats.editor_svg && this.canvasEditor.renderEditorSvg) {
                            this.canvasEditor.renderEditorSvg(result.formats.editor_svg);
                        }

                        // Update document display
                        const uiResult = {
                            success: true,
                            document: result.document
                        };
                        if (UI.updatePipelineData) {
                            UI.updatePipelineData(uiResult);
                        }
                    }
                }

                // Focus the canvas for immediate editing
                this.canvasEditor.focus();

            } else {
                throw new Error('Invalid response from document creation API');
            }

        } catch (error) {
            console.error('Failed to create new document:', error);
            UI.setStatus('Failed to create new document: ' + error.message, 'error');

            // Fallback: just clear everything locally
            console.log('Falling back to local clear...');
            this.clearAll();
            UI.setStatus('Created new document (offline)', 'success');
        }
    }

    // Update octave button states based on text selection
    updateOctaveButtonStates() {
        const selectedUuids = this.canvasEditor.getSelectedUuids();
        const hasSelection = selectedUuids.length > 0;

        const octaveButtons = [
            'btn-lowest', 'btn-lower', 'btn-middle',
            'btn-higher', 'btn-highest'
        ];

        octaveButtons.forEach(buttonId => {
            const button = document.getElementById(buttonId);
            if (button) {
                button.disabled = !hasSelection;
            }
        });
    }

    // Apply octave adjustment to selected elements
    async applyOctaveAdjustment(octaveType) {
        const selectedUuids = this.canvasEditor.getSelectedUuids();

        if (selectedUuids.length === 0) {
            UI.setStatus('Please select some notes first', 'error');
            return;
        }

        UI.setStatus(`Applying ${octaveType} octave to ${selectedUuids.length} elements...`, 'loading');

        try {
            // Use the new transform endpoint
            await this.canvasEditor.executeTransform('set_octave', {
                octave_type: octaveType // 'lowest', 'lower', 'middle', 'higher', 'highest'
            });

            UI.setStatus(`Applied ${octaveType} octave to ${selectedUuids.length} elements`, 'success');

        } catch (error) {
            console.error('Octave adjustment error:', error);
            UI.setStatus('Failed to apply octave adjustment: ' + error.message, 'error');
        }
    }

    // MIDI Control Methods
    playMidi() {
        if (!this.midiPlayer) {
            UI.setStatus('MIDI player not available', 'error');
            return;
        }

        // Use the document from canvasEditor (document-first architecture)
        const document = this.canvasEditor.document;

        if (!document || !document.elements || document.elements.length === 0) {
            UI.setStatus('No music content to play', 'error');
            return;
        }

        console.log('🎵 Using document from canvasEditor:', document);
        console.log('🎵 Document has elements:', document.elements?.length);
        console.log('🎵 Document structure:', {
            hasElements: !!document.elements,
            elementCount: document.elements?.length,
            firstElement: document.elements?.[0]
        });

        try {
            this.midiPlayer.play(document);
        } catch (error) {
            console.error('MIDI playback error:', error);
            UI.setStatus('MIDI playback failed: ' + error.message, 'error');
        }
    }

    pauseMidi() {
        if (this.midiPlayer) {
            this.midiPlayer.pause();
        }
    }

    stopMidi() {
        if (this.midiPlayer) {
            this.midiPlayer.stop();
        }
    }

    setTempo(bpm) {
        if (this.midiPlayer) {
            this.midiPlayer.setTempo(parseInt(bpm));
            document.getElementById('tempoDisplay').textContent = `${bpm} BPM`;
        }
    }
}

// Create global app instance
const app = new MusicTextApp();
window.appInstance = app;  // Make app instance accessible globally
window.MusicTextApp = app;
window.musicApp = app; // For UI access

// Global functions for onclick handlers (to maintain compatibility with existing HTML)
window.parseMusic = () => app.parseMusic();
window.generateSVG = () => app.generateSVG();
window.generateSVGPOC = () => app.generateSVGPOC();
window.updateNotationType = () => {
    // Handle notation type change (if dropdown exists)
    const notationSelect = document.getElementById('notationTypeSelect');
    if (!notationSelect) return;

    const notationType = notationSelect.value;
    console.log('Notation type changed to:', notationType);

    // Save to localStorage
    LocalStorage.saveNotationType(notationType);

    // Re-parse with the new notation type if there's content
    const input = app.canvasEditor.getValue();
    if (input && input.trim()) {
        app.parseAndUpdatePreview();
    }
};
window.clearAll = () => app.clearAll();
window.createNewDocument = () => app.createNewDocument();
window.switchTab = (tabName, clickedTab) => UI.switchTab(tabName, clickedTab);
window.changeFontFamily = (fontClass) => FontManager.changeFont(fontClass);
window.toggleSlur = async () => {
    const app = window.appInstance;
    if (!app) {
        console.error('App instance not found');
        return;
    }

    const selectedUuids = app.canvasEditor.getSelectedUuids();

    // Check if there are selected elements
    if (selectedUuids.length === 0) {
        UI.setStatus('Please select some notes first', 'error');
        return;
    }

    try {
        // Use semantic command for document-first operation
        await app.canvasEditor.executeSemanticCommand('apply_slur', {
            slur_type: 'standard' // Could be 'tie', 'phrase', etc.
        });

        UI.setStatus(`Applied slur to ${selectedUuids.length} elements`, 'success');

    } catch (error) {
        console.error('Slur application error:', error);
        UI.setStatus('Failed to apply slur', 'error');
    }
};
window.applyOctaveAdjustment = (octaveType) => app.applyOctaveAdjustment(octaveType);

// MIDI control functions
window.playMidi = () => app.playMidi();
window.pauseMidi = () => app.pauseMidi();
window.stopMidi = () => app.stopMidi();
window.setTempo = (bpm) => app.setTempo(bpm);

window.UI = UI;

// Initialize when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    app.init();
});

// Export for potential use in other modules
export { app as MusicTextApp };
