/**
 * Music Text Web Interface - Main Application Module
 * Orchestrates the entire web application
 */

import { LocalStorage } from './localStorage.js';
import { UI } from './ui.js';
import { API } from './api.js';
import { FontManager } from './fontManager.js';
import { EditorManager } from './editorManager.js';
import { MusicTextPlayer } from './midiPlayer.js';

class MusicTextApp {
    constructor() {
        this.currentParseResult = null;
        this.inputTimer = null;
        this.editorManager = new EditorManager();
        this.midiPlayer = null;
    }

    // Initialize the application
    async init() {
        try {
            await this.setupUI();
            await this.setupMIDI();
            this.setupEventListeners();
            this.restoreState();
            console.log('✅ Music Text App initialized with modular architecture');
        } catch (error) {
            console.error('Failed to initialize app:', error);
            UI.setStatus('Failed to initialize application', 'error');
        }
    }

    // Setup UI components
    async setupUI() {
        // Initialize editor
        this.editorManager.init('musicInput');
        
        // Initialize font manager
        FontManager.init();
        
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
        // Get the musicInput container (CodeMirror is attached to this)
        const musicInput = document.getElementById('musicInput');
        if (!musicInput) {
            throw new Error('Music input element not found');
        }

        // Set up CodeMirror-specific event listeners
        const editor = this.editorManager.editor;
        if (editor) {
            // Input events
            editor.on('change', () => {
                this.handleInput({ target: musicInput });
            });

            // Selection and cursor events for octave buttons
            editor.on('cursorActivity', () => {
                this.saveCursorPosition();
                this.updateOctaveButtonStates();
            });
        }
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
        const musicInput = document.getElementById('musicInput');
        const start = musicInput.selectionStart;
        const end = musicInput.selectionEnd;
        if (start !== undefined && end !== undefined && start >= 0 && end >= 0) {
            LocalStorage.saveCursorPosition(start, end);
        }
    }

    // Restore application state from localStorage
    restoreState() {
        const musicInput = document.getElementById('musicInput');
        
        // Restore saved input text
        const savedText = LocalStorage.loadInputText();
        if (savedText) {
            musicInput.value = savedText;
            // Trigger parsing for restored content
            if (savedText.trim()) {
                this.parseAndUpdatePreview();
            }
        }
        
        // Restore cursor position
        const cursorPos = LocalStorage.loadCursorPosition();
        if (cursorPos.start >= 0 && cursorPos.end >= 0) {
            setTimeout(() => {
                musicInput.setSelectionRange(cursorPos.start, cursorPos.end);
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
        const input = document.getElementById('musicInput').value;
        
        if (!input.trim()) {
            return;
        }

        try {
            const result = await API.parseForPreview(input);
            this.currentParseResult = result;
            
            // Update all outputs
            UI.updatePipelineData(result);
            UI.updateLilyPondOutput(result);
            UI.updateTokensOutput(result);
            UI.updateStylesOutput(result);
            UI.updateSourceOutput(result);
            await UI.updateVexFlowOutput(result);
            
            // Update editor highlighting based on parse results
            this.editorManager.highlightFromParseResult(result);

            // Apply span styles using server-generated span styles (preferred)
            if (result.success && result.character_styles) {
                this.editorManager.applySpanStyles(result.character_styles);
            }
            // Fallback to span-based highlighting if character styles not available
            else if (result.success && result.syntax_spans) {
                this.editorManager.applySyntaxSpans(result.syntax_spans);
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
        
        const input = document.getElementById('musicInput').value;
        
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
            UI.updateTokensOutput(result);
            UI.updateStylesOutput(result);
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
            
            // Auto-switch to VexFlow tab
            UI.switchTab('vexflow');
            
        } catch (error) {
            UI.setStatus(`Error: ${error.message}`, 'error');
            document.getElementById('vexflow-output').innerHTML = `<p>Error: ${error.message}</p>`;
            UI.restoreFocusAndCursor();
        }
    }

    // Generate SVG (triggered by LilyPond button)
    async generateSVG() {
        // Save cursor position before processing
        this.saveCursorPosition();
        
        const input = document.getElementById('musicInput').value;
        
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
            UI.updateTokensOutput(result);
            UI.updateStylesOutput(result);
            UI.updateSourceOutput(result);

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
        this.currentParseResult = null;

        // Clear localStorage
        LocalStorage.saveInputText('');
        LocalStorage.saveCursorPosition(0, 0);

        // Switch back to VexFlow tab and restore focus
        UI.switchTab('vexflow');
    }

    // Update octave button states based on text selection
    updateOctaveButtonStates() {
        const musicInput = document.getElementById('musicInput');
        // The editorManager adds compatibility methods to musicInput
        const hasSelection = musicInput.selectionStart !== musicInput.selectionEnd;

        const octaveButtons = [
            'btn-lowest', 'btn-lowish', 'btn-lower',
            'btn-higher', 'btn-highish', 'btn-highest'
        ];

        octaveButtons.forEach(buttonId => {
            const button = document.getElementById(buttonId);
            if (button) {
                button.disabled = !hasSelection;
            }
        });
    }

    // Apply octave adjustment to selected text
    applyOctaveAdjustment(octaveType) {
        const musicInput = document.getElementById('musicInput');
        const selectionStart = musicInput.selectionStart;
        const selectionEnd = musicInput.selectionEnd;

        // Check if there's a selection
        if (selectionStart === selectionEnd) {
            UI.setStatus('Please select some notes first', 'error');
            return;
        }

        const fullText = musicInput.value;
        const selectedText = fullText.substring(selectionStart, selectionEnd);

        // Check if selection contains musical notes
        if (!this.containsMusicalNotes(selectedText)) {
            UI.setStatus('Selection contains no musical notes', 'error');
            return;
        }

        try {
            // Apply octave modification using full document context
            const modifiedText = this.processOctaveAdjustmentWithColumns(fullText, selectionStart, selectionEnd, octaveType);

            // Replace entire text
            musicInput.value = modifiedText;

            // Restore focus and selection (adjust for potential line additions)
            musicInput.focus();
            musicInput.setSelectionRange(selectionStart, selectionStart + (modifiedText.length - fullText.length) + (selectionEnd - selectionStart));

            // Trigger re-parsing
            this.handleInput({ target: musicInput });

            UI.setStatus(`Applied ${octaveType} octave adjustment`, 'success');

        } catch (error) {
            console.error('Octave adjustment error:', error);
            UI.setStatus('Failed to apply octave adjustment', 'error');
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

        // Find which lines and columns contain the selected notes
        const selectedNotePositions = this.findSelectedNotePositions(fullText, selectionStart, selectionEnd);

        if (selectedNotePositions.length === 0) {
            return fullText; // No notes found in selection
        }

        // Group note positions by line
        const notesByLine = new Map();
        for (const pos of selectedNotePositions) {
            if (!notesByLine.has(pos.lineIndex)) {
                notesByLine.set(pos.lineIndex, []);
            }
            notesByLine.get(pos.lineIndex).push(pos.column);
        }

        // Process each line that has selected notes
        for (const [lineIndex, columns] of notesByLine) {
            if (this.isUpperOctave(octaveType)) {
                this.addToUpperLineWithColumns(lines, lineIndex, columns, marker);
            } else {
                this.addToLowerLineWithColumns(lines, lineIndex, columns, marker);
            }
        }

        return lines.join('\n');
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

        for (const column of columns) {
            // Ensure the line is long enough
            while (result.length <= column) {
                result += ' ';
            }
            // Add marker at the specified column
            result = result.substring(0, column) + marker + result.substring(column + 1);
        }

        return result;
    }

    // Create new line with markers at specific columns
    createLineWithMarkersAtColumns(columns, marker) {
        if (columns.length === 0) {
            return '';
        }

        const maxColumn = Math.max(...columns);
        let line = ' '.repeat(maxColumn + 1);

        for (const column of columns) {
            line = line.substring(0, column) + marker + line.substring(column + 1);
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
            'lowest': ':',   // -2 octaves
            'lowish': '.',   // -1 octave (same as lower for now)
            'lower': '.',    // -1 octave
            'higher': '.',   // +1 octave
            'highish': '.',  // +1 octave (same as higher for now)
            'highest': ':'   // +3 octaves
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
window.MusicTextApp = app;

// Global functions for onclick handlers (to maintain compatibility with existing HTML)
window.parseMusic = () => app.parseMusic();
window.generateSVG = () => app.generateSVG();
window.clearAll = () => app.clearAll();
window.switchTab = (tabName, clickedTab) => UI.switchTab(tabName, clickedTab);
window.changeFontFamily = (fontClass) => FontManager.changeFont(fontClass);
window.toggleSlur = () => app.editorManager.toggleSlur();
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