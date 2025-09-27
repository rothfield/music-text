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
            await this.setupUI();
            // Don't initialize MIDI until needed
            this.midiInitialized = false;
            this.setupEventListeners();
            // this.restoreState(); // Disabled localStorage restoration
            console.log('✅ Music Text App initialized with modular architecture');
        } catch (error) {
            console.error('Failed to initialize app:', error);
            UI.setStatus('Failed to initialize application', 'error');
        }
    }

    // Setup UI components
    async setupUI() {
        // Initialize canvas editor
        this.canvasEditor.init('canvasEditor');

        // Initialize font manager
        FontManager.init();

        // Load and set notation type from localStorage (if dropdown exists)
        const savedNotationType = LocalStorage.loadNotationType();
        const notationSelect = document.getElementById('notationTypeSelect');
        if (notationSelect) {
            notationSelect.value = savedNotationType;
        }

        // Setup initial UI state
        this.setupInitialTabState();
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
            // Update the backing text tab
            UI.updateBackingTextOutput(content);

            // Debounced parsing for real-time updates
            clearTimeout(this.inputTimer);
            this.inputTimer = setTimeout(() => {
                if (content.trim()) {
                    this.parseAndUpdatePreview();
                }
            }, 300);
        };

        this.canvasEditor.onSelectionChange = (selection) => {
            this.saveCursorPosition();
            this.updateOctaveButtonStates();

            // Backing text display removed - no longer needed
        };
    }

    // Handle input events
    handleInput(event) {
        
        const textarea = event.target;

        // Debounced parsing for real-time updates
        clearTimeout(this.inputTimer);
        this.inputTimer = setTimeout(() => {
            if (textarea.value.trim()) {
                this.parseAndUpdatePreview();
            } else {
                UI.clearEmptyInputs();
            }
        }, 300);
    }

    // Save current cursor position
    saveCursorPosition() {
        // Cursor position is now saved automatically with document state
    }

    // Restore application state from localStorage
    restoreState() {
        
        // Document state is now restored automatically by CanvasEditor.loadFromLocalStorage()
        
        // Restore active tab
        const activeTab = LocalStorage.loadActiveTab();
        if (activeTab !== 'vexflow') {
            UI.switchTab(activeTab);
        }
        
        musicInput.focus();
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
        // Save cursor position before processing
        this.saveCursorPosition();
        
        const input = this.canvasEditor.getValue();
        
        // Validate input
        const validation = API.validateInput(input);
        if (!validation.valid) {
            UI.setStatus(validation.error, 'error');
            UI.restoreFocusAndCursor();
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
            UI.restoreFocusAndCursor();
        }
    }

    // Generate SVG POC (triggered by SVG POC button)
    async generateSVGPOC() {
        // Save cursor position before processing
        this.saveCursorPosition();

        const input = this.canvasEditor.getValue();
        const notationType = document.getElementById('notationTypeSelect')?.value || 'auto';

        // Validate input
        const validation = API.validateInput(input);
        if (!validation.valid) {
            UI.setStatus(validation.error, 'error');
            UI.restoreFocusAndCursor();
            return;
        }

        UI.setStatus('Generating SVG POC...', 'loading');

        try {
            // Send text and notation type directly to server for SVG POC generation
            const svgResponse = await fetch('/api/render-svg-poc', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    input: input,
                    notation_type: notationType
                })
            });

            if (!svgResponse.ok) {
                throw new Error(`SVG POC API error: ${svgResponse.status}`);
            }

            const svgContent = await svgResponse.text();

            // Display the SVG
            document.getElementById('svgpoc-output').innerHTML = svgContent;
            UI.setStatus('SVG POC generated successfully!', 'success');
            UI.switchTab('svgpoc');

        } catch (error) {
            document.getElementById('svgpoc-output').innerHTML = `<p>Error: ${error.message}</p>`;
            UI.setStatus(`Error: ${error.message}`, 'error');
            UI.restoreFocusAndCursor();
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
            const response = await fetch(`/api/documents/${documentUUID}/export`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
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

            const response = await fetch('/api/documents', {
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
                    const docResponse = await fetch(`/api/documents/${result.document.documentUUID}`);
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

        // Check if there are selected elements
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

            // Fallback to semantic command if transform not available
            console.log('Falling back to semantic command...');
            try {
                await this.canvasEditor.executeSemanticCommand('set_octave', {
                    octave_type: octaveType
                });
                UI.setStatus(`Applied ${octaveType} octave (semantic command)`, 'success');
            } catch (fallbackError) {
                console.error('Semantic command fallback also failed:', fallbackError);

                // Last fallback to legacy text-based transform
                console.log('Falling back to legacy octave transform...');
                try {
                    await this.canvasEditor.applyTransformation('/api/transform/octave', {
                        action: 'octave',
                        octave_type: octaveType
                    });
                    UI.setStatus(`Applied ${octaveType} octave (legacy mode)`, 'success');
                } catch (legacyError) {
                    console.error('All fallbacks failed:', legacyError);
                    UI.setStatus('Failed to apply octave adjustment', 'error');
                }
            }
        }
    }

    // Check if text contains musical notes
    containsMusicalNotes(text) {
        // Match Sargam (S R G M P D N), Numbers (1-7), Western (A-G), with accidentals
        const notePattern = /[SRGMPDNsrgmpdn1-7A-Ga-g][#b♯♭♮]*/;
        return notePattern.test(text);
    }

    // Process octave adjustment using full document context for proper column alignment
    processOctaveAdjustmentWithColumns(fullText, selectionStart, selectionEnd, octaveType) {
        const lines = fullText.split('\n');
        const marker = this.getOctaveMarker(octaveType);
        let upperLineWasAdded = false;

        // Find which lines and columns contain the selected notes
        const selectedNotePositions = this.findSelectedNotePositions(fullText, selectionStart, selectionEnd);

        if (selectedNotePositions.length === 0) {
            return { modifiedText: fullText, upperLineWasAdded: false }; // No notes found
        }

        // Group note positions by line
        const notesByLine = new Map();
        for (const pos of selectedNotePositions) {
            if (!notesByLine.has(pos.lineIndex)) {
                notesByLine.set(pos.lineIndex, []);
            }
            notesByLine.get(pos.lineIndex).push(pos.column);
        }

        // Create a list of line indices to process, sorted, to handle multiple line selections
        const sortedLineIndices = Array.from(notesByLine.keys()).sort((a, b) => a - b);
        let linesAdded = 0; // Track how many lines we've added to adjust indices

        for (const lineIndex of sortedLineIndices) {
            const columns = notesByLine.get(lineIndex);
            const adjustedLineIndex = lineIndex + linesAdded;

            if (octaveType === 'middle') {
                // Remove octave markers at these columns
                const upperLineIndex = adjustedLineIndex - 1;
                const lowerLineIndex = adjustedLineIndex + 1;

                // Remove from upper line if it exists
                if (upperLineIndex >= 0 && this.isUpperLine(lines[upperLineIndex])) {
                    lines[upperLineIndex] = this.removeMarkersAtColumns(lines[upperLineIndex], columns);
                }

                // Remove from lower line if it exists
                if (lowerLineIndex < lines.length && this.isLowerLine(lines[lowerLineIndex])) {
                    lines[lowerLineIndex] = this.removeMarkersAtColumns(lines[lowerLineIndex], columns);
                }
            } else if (this.isUpperOctave(octaveType)) {
                const upperLineIndex = adjustedLineIndex - 1;
                // Check if there's already an upper line
                if (upperLineIndex >= 0 && this.isUpperLine(lines[upperLineIndex])) {
                    lines[upperLineIndex] = this.addMarkersAtColumns(lines[upperLineIndex], columns, marker);
                } else {
                    const newUpperLine = this.createLineWithMarkersAtColumns(columns, marker);
                    lines.splice(adjustedLineIndex, 0, newUpperLine);
                    linesAdded++;
                    upperLineWasAdded = true;
                }
            } else { // Lower Octave
                const lowerLineIndex = adjustedLineIndex + 1;
                // Check if there's already a lower line
                if (lowerLineIndex < lines.length && this.isLowerLine(lines[lowerLineIndex])) {
                    lines[lowerLineIndex] = this.addMarkersAtColumns(lines[lowerLineIndex], columns, marker);
                } else {
                    const newLowerLine = this.createLineWithMarkersAtColumns(columns, marker);
                    lines.splice(lowerLineIndex, 0, newLowerLine);
                    linesAdded++;
                }
            }
        }

        return { modifiedText: lines.join('\n'), upperLineWasAdded };
    }

    // Add markers to upper line at specific columns
    addToUpperLineWithColumns(lines, contentLineIndex, columns, marker) {
        const upperLineIndex = contentLineIndex - 1;

        // Check if there's already an upper line
        if (upperLineIndex >= 0 && this.isUpperLine(lines[upperLineIndex])) {
            // Add markers to existing upper line at specified columns
            lines[upperLineIndex] = this.addMarkersAtColumns(lines[upperLineIndex], columns, marker);
        } else {
            // Create new upper line with markers at specified columns
            const newUpperLine = this.createLineWithMarkersAtColumns(columns, marker);
            lines.splice(contentLineIndex, 0, newUpperLine);
        }
    }

    // Add markers to lower line at specific columns
    addToLowerLineWithColumns(lines, contentLineIndex, columns, marker) {
        const lowerLineIndex = contentLineIndex + 1;

        // Check if there's already a lower line
        if (lowerLineIndex < lines.length && this.isLowerLine(lines[lowerLineIndex])) {
            // Add markers to existing lower line at specified columns
            lines[lowerLineIndex] = this.addMarkersAtColumns(lines[lowerLineIndex], columns, marker);
        } else {
            // Create new lower line with markers at specified columns
            const newLowerLine = this.createLineWithMarkersAtColumns(columns, marker);
            lines.splice(contentLineIndex + 1, 0, newLowerLine);
        }
    }

    // Check if a line looks like an upper line (contains dots, colons, asterisks)
    isUpperLine(line) {
        const upperMarkers = /[.:*]/;
        const hasMarkers = upperMarkers.test(line);
        const hasNotes = this.containsMusicalNotes(line);
        return hasMarkers && !hasNotes;
    }

    // Check if a line looks like a lower line (contains dots, colons)
    isLowerLine(line) {
        const lowerMarkers = /[.:]/;
        const hasMarkers = lowerMarkers.test(line);
        const hasNotes = this.containsMusicalNotes(line);
        return hasMarkers && !hasNotes;
    }

    // Add markers to existing line at specific columns
    addMarkersAtColumns(existingLine, columns, marker) {
        let result = existingLine;
        const markerLength = marker.length;

        for (const column of columns) {
            // Ensure the line is long enough for the marker
            while (result.length < column + markerLength) {
                result += ' ';
            }
            // Replace spaces with the marker at the specified column
            result = result.substring(0, column) + marker + result.substring(column + markerLength);
        }

        return result;
    }

    // Remove markers at specified columns
    removeMarkersAtColumns(existingLine, columns) {
        let result = existingLine;

        for (const column of columns) {
            if (column < result.length) {
                // Replace marker with space
                result = result.substring(0, column) + ' ' + result.substring(column + 1);
            }
        }

        // Trim trailing spaces
        return result.replace(/\s+$/, '');
    }

    // Create new line with markers at specific columns
    createLineWithMarkersAtColumns(columns, marker) {
        if (columns.length === 0) {
            return '';
        }

        const maxColumn = Math.max(...columns);
        // For two-dot markers, ensure we have enough space
        const lineLength = marker === '..' ? maxColumn + 2 : maxColumn + 1;
        let line = ' '.repeat(lineLength);

        // Handle two-dot markers specially
        if (marker === '..') {
            for (const column of columns) {
                line = line.substring(0, column) + '..' + line.substring(column + 2);
            }
        } else {
            for (const column of columns) {
                line = line.substring(0, column) + marker + line.substring(column + marker.length);
            }
        }

        // Remove trailing spaces to keep spatial lines clean
        return line.trimEnd();
    }

    // Find the line containing musical content
    findContentLineIndex(lines) {
        for (let i = 0; i < lines.length; i++) {
            if (this.containsMusicalNotes(lines[i])) {
                return i;
            }
        }
        return -1;
    }

    // Check if octave type is for upper octaves
    isUpperOctave(octaveType) {
        return ['higher', 'highish', 'highest'].includes(octaveType);
    }



    // Find positions of selected notes with line and column information
    findSelectedNotePositions(fullText, selectionStart, selectionEnd) {
        const positions = [];
        let currentPos = 0;
        let lineIndex = 0;
        let columnInLine = 0;

        for (let i = 0; i < fullText.length; i++) {
            if (i >= selectionStart && i < selectionEnd) {
                const char = fullText[i];
                if (this.containsMusicalNotes(char)) {
                    positions.push({
                        globalPos: i,
                        lineIndex: lineIndex,
                        column: columnInLine
                    });
                }
            }

            if (fullText[i] === '\n') {
                lineIndex++;
                columnInLine = 0;
            } else {
                columnInLine++;
            }
        }

        return positions;
    }

    // Get the appropriate octave marker for the octave type
    getOctaveMarker(octaveType) {
        const markerMap = {
            'lowest': ':',    // LL - Lowest octave (: below)
            'lowish': '.',    // L - Lower octave (. below)
            'lower': '.',     // L - Lower octave (. below)
            'middle': '',     // M - Middle octave (no markers)
            'higher': '.',    // U - Upper octave (. above)
            'highish': ':',   // HH - Higher octave (: above)
            'highest': ':'    // HH - Highest octave (: above)
        };

        return markerMap[octaveType] || '.';
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

        // Fallback to legacy text-based transform if semantic command not available
        console.log('Falling back to legacy text transform...');
        try {
            await app.canvasEditor.applyTransformation('/api/transform/slur', {
                action: 'slur'
            });
            UI.setStatus('Applied slur (legacy mode)', 'success');
        } catch (fallbackError) {
            console.error('Legacy fallback also failed:', fallbackError);
            UI.setStatus('Failed to apply slur', 'error');
        }
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
