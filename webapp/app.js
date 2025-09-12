// Main App Module (Refactored)
// Orchestrates the music text webapp using modular components

import { DEBOUNCE_DELAYS } from './js/config.js';
import { saveInputText, loadInputText, saveUnicodePreference, loadUnicodePreference } from './js/storage.js';
import { setupUnicodeInput, convertUnicodeToStandard, applyUnicodeReplacements } from './js/unicode-processor.js';
import { FontManager } from './js/font-manager.js';
import { ApiClient } from './js/api-client.js';
import { TabManager, autoExpandTextarea, OutputDisplayManager } from './js/ui-utils.js';
import { VexFlowIntegration } from './js/vexflow-integration.js';

class MusicTextApp {
    constructor() {
        this.debounceTimer = null;
        this.svgDebounceTimer = null;
        this.useUnicode = { current: true }; // Mutable reference for event handlers
        
        // Initialize managers
        this.fontManager = null;
        this.tabManager = null;
        this.outputManager = null;
        this.vexflowIntegration = null;
        
        // DOM elements
        this.inputText = null;
        this.unicodeToggle = null;
    }

    async init() {
        this.bindElements();
        await this.initializeManagers();
        this.setupUnicodeToggle();
        this.setupInputHandling();
        this.setupSvgGeneration();
        this.restoreSavedState();
        
        console.log('✅ Music Text App initialized');
    }

    bindElements() {
        this.inputText = document.getElementById('input-text');
        this.unicodeToggle = document.getElementById('unicode-toggle');
        
        console.log('🔍 DOM elements found:', { 
            inputText: !!this.inputText, 
            unicodeToggle: !!this.unicodeToggle,
            unicodeToggleId: this.unicodeToggle?.id,
            unicodeToggleChecked: this.unicodeToggle?.checked 
        });
    }

    async initializeManagers() {
        // Initialize font manager
        if (this.inputText) {
            this.fontManager = new FontManager(this.inputText);
            this.fontManager.init();
        }

        // Initialize tab manager
        this.tabManager = new TabManager();

        // Initialize output display manager
        this.outputManager = new OutputDisplayManager();

        // Initialize VexFlow integration
        this.vexflowIntegration = new VexFlowIntegration();
        await this.vexflowIntegration.init();

        // Load valid pitches from server
        await ApiClient.loadValidPitches();
    }

    setupUnicodeToggle() {
        if (!this.unicodeToggle || !this.inputText) {
            console.error('❌ Unicode toggle setup failed:', { 
                unicodeToggle: !!this.unicodeToggle, 
                inputText: !!this.inputText 
            });
            return;
        }

        console.log('🎛️ Setting up Unicode toggle...');
        
        // Load saved Unicode preference
        try {
            this.useUnicode.current = loadUnicodePreference();
            this.unicodeToggle.checked = this.useUnicode.current;
            console.log('📂 Loaded Unicode preference:', this.useUnicode.current);
        } catch (e) {
            console.warn('Failed to load Unicode preference:', e);
        }
        
        // Setup Unicode toggle event listener
        console.log('🔗 Attaching event listener to Unicode toggle...');
        this.unicodeToggle.addEventListener('change', (e) => {
            console.log('🎯 Unicode toggle clicked! New state:', e.target.checked, 'Previous state:', this.useUnicode.current);
            console.log('🎯 Input text before toggle:', this.inputText.value.slice(0, 30));
            
            this.useUnicode.current = e.target.checked;
            
            // Save to localStorage
            saveUnicodePreference(this.useUnicode.current);
            console.log('💾 Saved Unicode preference:', this.useUnicode.current);
            
            // Refresh the display
            console.log('🔄 About to refresh text display...');
            if (this.fontManager) {
                this.fontManager.refreshTextDisplay(this.useUnicode.current);
            }
            console.log('✅ Text display refresh completed');
        });
        
        console.log('✅ Unicode toggle event listener attached successfully!');

        // Setup Unicode input handling
        // setupUnicodeInput(this.inputText, this.fontManager?.fontSelect, this.useUnicode);
    }

    setupInputHandling() {
        if (!this.inputText) return;

        this.inputText.addEventListener('input', (e) => {
            const inputValue = e.target.value;
            console.log('⌨️ Input event triggered:', {
                inputLength: inputValue.length,
                firstChars: inputValue.slice(0, 20),
                timestamp: new Date().toISOString(),
                svgTabActive: this.tabManager.isSvgTabActive()
            });
            
            // Auto-expand textarea
            autoExpandTextarea(e.target);
            
            // Save input text to localStorage
            saveInputText(inputValue);
            
            // Debounced parsing
            clearTimeout(this.debounceTimer);
            this.debounceTimer = setTimeout(() => {
                console.log('⏰ Debounce timer triggered, calling parseInput');
                this.parseInput(e.target.value);
            }, DEBOUNCE_DELAYS.PARSE);
            
            // SVG generation disabled - only generate on manual button click
            // if (this.tabManager.isSvgTabActive()) {
            //     clearTimeout(this.svgDebounceTimer);
            //     this.svgDebounceTimer = setTimeout(() => {
            //         console.log('🎵 SVG debounce timer triggered, generating SVG automatically');
            //         this.generateSvgFromLilypond();
            //     }, DEBOUNCE_DELAYS.SVG);
            // }
        });
    }

    setupSvgGeneration() {
        // Make generateSvgFromLilypond available globally for the button
        window.generateSvgFromLilypond = () => this.generateSvgFromLilypond();
    }

    restoreSavedState() {
        // Restore saved input text
        const savedInput = loadInputText();
        if (savedInput && this.inputText) {
            this.inputText.value = savedInput;
            autoExpandTextarea(this.inputText);
        }
        
        // Restore active tab
        this.tabManager.restoreActiveTab();
        
        // Parse if there's saved input
        if (savedInput && savedInput.trim()) {
            this.parseInput(savedInput);
        }
    }

    async parseInput(input) {
        if (!input.trim()) {
            this.outputManager.showEmpty();
            return;
        }

        this.outputManager.showLoading();

        try {
            const data = await ApiClient.parseInput(input);
            
            if (data.isEmpty) {
                this.outputManager.showEmpty();
                return;
            }

            if (data.success) {
                console.log('✅ Processing successful API response');
                this.outputManager.updateOutputs(data);
                
                // Handle VexFlow output
                if (data.vexflow) {
                    this.vexflowIntegration.handleVexFlowOutput(data.vexflow);
                } else {
                    this.vexflowIntegration.handleVexFlowOutput(null);
                }
                
            } else {
                console.log('❌ API returned error response:', {
                    success: data.success,
                    error: data.error,
                    errorLength: data.error?.length,
                    timestamp: new Date().toISOString()
                });
                
                this.outputManager.showError(data.error || 'Unknown error');
            }
            
        } catch (error) {
            console.error('🚨 Error in parseInput:', error);
            this.outputManager.showError(error.message);
        }
    }

    async generateSvgFromLilypond() {
        console.log("🎵 generateSvgFromLilypond() called");
        
        if (!this.inputText || !this.inputText.value.trim()) {
            alert("Please enter music notation first.");
            return;
        }
        
        const notation = this.inputText.value.trim();
        
        // Update button state
        const button = document.getElementById("generate-svg-btn");
        const svgContent = document.getElementById("svg-content");
        
        if (button) {
            button.disabled = true;
            button.textContent = "Generating...";
        }
        
        if (svgContent) {
            svgContent.innerHTML = '<div class="text-muted">Generating SVG from notation...</div>';
        }
        
        try {
            const result = await ApiClient.generateSvgFromLilypond(notation);
            
            if (result.success && result.svg_content) {
                if (svgContent) {
                    // Apply styles to float the SVG to the top
                    svgContent.style.margin = "0";
                    svgContent.style.padding = "0";
                    svgContent.innerHTML = result.svg_content;
                    // Find the SVG element and remove any margins
                    const svgElement = svgContent.querySelector('svg');
                    if (svgElement) {
                        svgElement.style.margin = "0";
                        svgElement.style.display = "block";
                    }
                }
                console.log("✅ SVG generated successfully");
            } else {
                if (svgContent) {
                    svgContent.innerHTML = `<div class="alert alert-danger">SVG Generation Error: ${result.error || "Unknown error"}</div>`;
                }
                console.error("❌ SVG generation failed:", result.error);
            }
        } catch (error) {
            console.error("🚨 Error during SVG generation:", error);
            if (svgContent) {
                svgContent.innerHTML = `<div class="alert alert-danger">Network Error: ${error.message}</div>`;
            }
        } finally {
            if (button) {
                button.disabled = false;
                button.textContent = "Generate SVG";
            }
        }
    }
}

// Initialize the application
document.addEventListener('DOMContentLoaded', async function() {
    const app = new MusicTextApp();
    await app.init();
});

// Also initialize if DOM is already loaded
if (document.readyState !== 'loading') {
    const app = new MusicTextApp();
    app.init();
}