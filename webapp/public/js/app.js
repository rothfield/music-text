/**
 * Music Text Web Interface - Main Application Module
 * Orchestrates the entire web application
 */

import { LocalStorage } from './localStorage.js';
import { UI } from './ui.js';
import { API } from './api.js';
import { FontManager } from './fontManager.js';
import { EditorManager } from './editorManager.js';

class MusicTextApp {
    constructor() {
        this.currentParseResult = null;
        this.inputTimer = null;
        this.editorManager = new EditorManager();
    }

    // Initialize the application
    async init() {
        try {
            await this.setupUI();
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

    // Setup event listeners
    setupEventListeners() {
        const musicInput = document.getElementById('musicInput');
        if (!musicInput) {
            throw new Error('Music input element not found');
        }

        // Input event listener for real-time updates
        musicInput.addEventListener('input', (e) => this.handleInput(e));
        
        // Cursor position saving events
        const saveCursor = () => this.saveCursorPosition();
        musicInput.addEventListener('keyup', saveCursor);
        musicInput.addEventListener('mouseup', saveCursor);
        musicInput.addEventListener('click', saveCursor);
    }

    // Handle input events
    handleInput(event) {
        
        const textarea = event.target;
        const originalValue = textarea.value;
        
        // Convert music notation symbols
        const convertedValue = UI.convertMusicNotation(originalValue);
        
        if (convertedValue !== originalValue) {
            // Save cursor position before changing text
            const start = textarea.selectionStart;
            const end = textarea.selectionEnd;
            
            // Update text with converted symbols
            textarea.value = convertedValue;
            
            // Restore cursor position (accounting for potential symbol changes)
            const cursorOffset = convertedValue.length - originalValue.length;
            textarea.setSelectionRange(start + cursorOffset, end + cursorOffset);
        }
        
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

            // Apply character styles using server-generated character styles (preferred)
            if (result.success && result.character_styles) {
                this.editorManager.applyCharacterStyles(result.character_styles);
            }
            // Fallback to token-based highlighting if character styles not available
            else if (result.success && result.syntax_tokens) {
                this.editorManager.applySyntaxTokens(result.syntax_tokens);
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
window.UI = UI;

// Initialize when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    app.init();
});

// Export for potential use in other modules
export { app as MusicTextApp };