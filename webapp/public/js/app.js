/**
 * Music Text Web Interface - Main Application Module
 * Orchestrates the entire web application
 */

import { LocalStorage } from './localStorage.js';
import { UI } from './ui.js';
import { API } from './api.js';
import { FontManager } from './fontManager.js';
import { CanvasEditor } from './canvasEditor.js';
import { MusicTextPlayer } from './midiPlayer.js';

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
        // Set up canvas editor event listeners
        this.canvasEditor.onContentChange = (content) => {
            // Update the backing text tab
            UI.updateBackingTextOutput(content);

            // Save input text
            LocalStorage.saveInputText(content);

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

        // Save input text
        LocalStorage.saveInputText(textarea.value);
        
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
        const cursorPos = this.canvasEditor.getCursorPosition();
        if (cursorPos.start >= 0 && cursorPos.end >= 0) {
            LocalStorage.saveCursorPosition(cursorPos.start, cursorPos.end);
        }
    }

    // Restore application state from localStorage
    restoreState() {
        
        // Restore saved input text
        const savedText = LocalStorage.loadInputText();
        if (savedText) {
            this.canvasEditor.setValue(savedText);
            // Trigger parsing for restored content
            if (savedText.trim()) {
                this.parseAndUpdatePreview();
            }
        }
        
        // Restore cursor position
        const cursorPos = LocalStorage.loadCursorPosition();
        if (cursorPos.start >= 0 && cursorPos.end >= 0) {
            setTimeout(() => {
                this.canvasEditor.setSelection(cursorPos.start, cursorPos.end);
            }, 100);
        }
        
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
            UI.updateSVGSourceOutput(result);
            
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
            UI.updateSVGSourceOutput(result);

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
            
            // Auto-switch to VexFlow tab
            UI.switchTab('vexflow');
            
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


    // Generate SVG (triggered by LilyPond button)
    async generateSVG() {
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
        
        UI.setStatus('Generating LilyPond SVG...', 'loading');
        
        try {
            const result = await API.parseWithSVG(input);
            this.currentParseResult = result;

            // Update all outputs
            UI.updatePipelineData(result);
            UI.updateLilyPondOutput(result);
            UI.updateSourceOutput(result);
            UI.updateSVGSourceOutput(result);

            if (API.hasVexFlowData(result)) {
                await UI.updateVexFlowOutput(result);
            }
            
            // Handle SVG output
            if (UI.updateSVGOutput(result)) {
                UI.setStatus('LilyPond SVG generated successfully!', 'success');
                UI.switchTab('svg');
            } else {
                UI.setStatus('SVG generation failed.', 'error');
                UI.restoreFocusAndCursor();
            }
            
        } catch (error) {
            document.getElementById('svg-output').innerHTML = `<p>Error: ${error.message}</p>`;
            UI.setStatus(`Error: ${error.message}`, 'error');
            UI.restoreFocusAndCursor();
        }
    }


    // Clear all content and localStorage
    clearAll() {
        UI.clearAllContent();
        this.canvasEditor.setValue('');
        this.currentParseResult = null;

        // Clear localStorage
        LocalStorage.saveInputText('');
        LocalStorage.saveCursorPosition(0, 0);

        // Switch back to VexFlow tab and restore focus
        UI.switchTab('vexflow');
    }

    // Create a new document using the API
    async createNewDocument() {
        try {
            UI.setStatus('Creating new document...', 'loading');

            // Ask user if they want to start with a template or blank document
            const createWithTemplate = confirm('Start with a template? (S R G M)\n\nOK = Template\nCancel = Blank document');

            const requestBody = {
                music_text: createWithTemplate ? 'S R G M' : null,
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

            if (result.document_id && result.document) {
                // Clear current content
                this.clearAll();

                // Set up new document from API response
                this.canvasEditor.document.id = result.document_id;

                // Load the document content if it was created with template
                if (createWithTemplate && result.document.format_cache && result.document.format_cache.music_text) {
                    const content = result.document.format_cache.music_text;
                    this.canvasEditor.setValue(content);
                    this.canvasEditor.throttledSubmitToServer(content); // Render immediately
                }

                // Update document metadata
                if (result.document.metadata) {
                    this.canvasEditor.document.metadata = result.document.metadata;
                }

                // Save the new document state
                this.canvasEditor.saveToLocalStorage();

                const hasContent = createWithTemplate ? ' with template content' : ' (blank)';
                UI.setStatus(`New document created${hasContent} (ID: ${result.document_id.slice(0, 8)}...)`, 'success');

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

        if (!this.currentParseResult || !this.currentParseResult.success) {
            UI.setStatus('Please parse music notation first', 'error');
            return;
        }

        console.log('Current parse result:', this.currentParseResult);
        console.log('Parsed document:', this.currentParseResult.parsed_document);

        try {
            // Get the correct document structure from API response
            const document = this.currentParseResult.document;

            console.log('🎵 Passing document to MIDI player:', document);
            console.log('🎵 Document type:', typeof document);
            console.log('🎵 Document keys:', document ? Object.keys(document) : 'null');

            if (!document) {
                UI.setStatus('No parsed document available for playback', 'error');
                return;
            }

            this.midiPlayer.play(document);
        } catch (error) {
            console.error('MIDI playback error:', error);
            UI.setStatus('MIDI playback failed', 'error');
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