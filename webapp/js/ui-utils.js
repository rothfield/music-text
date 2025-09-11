// UI Utilities Module
// Common UI helper functions and utilities

import { saveActiveTab, loadActiveTab } from './storage.js';

// JSON syntax highlighting for output display
export function syntaxHighlight(json) {
    json = json.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
    return json.replace(/(\"(\\u[a-fA-F0-9]{4}|\\[^u]|[^\\\"])*\"(\s*:)?|\b(true|false|null)\b|-?\d+(?:\.\d*)?(?:[eE][+\-]?\d+)?)/g, function (match) {
        let cls = 'number';
        if (/^\"/.test(match)) {
            if (/:$/.test(match)) {
                cls = 'key';
            } else {
                cls = 'string';
            }
        } else if (/true|false/.test(match)) {
            cls = 'boolean';
        } else if (/null/.test(match)) {
            cls = 'null';
        }
        return '<span class="' + cls + '">' + match + '</span>';
    });
}

// Auto-expand textarea based on content
export function autoExpandTextarea(textarea) {
    // Reset height to auto to get the correct scrollHeight
    textarea.style.height = 'auto';
    
    // Calculate the new height based on scrollHeight
    const newHeight = Math.max(60, textarea.scrollHeight); // Min height of 60px (about 3 rows)
    
    // Set the new height
    textarea.style.height = newHeight + 'px';
}

// Tab management utilities
export class TabManager {
    constructor() {
        this.setupBootstrapTabs();
    }

    setupBootstrapTabs() {
        // Bootstrap handles tab switching automatically, but we still need to save active tab
        const tabButtons = document.querySelectorAll('.nav-link');
        tabButtons.forEach(button => {
            button.addEventListener('shown.bs.tab', this.saveActiveTabFromBootstrap);
            
            // Auto-generate SVG when LilyPond SVG tab is clicked
            if (button.id === 'svg-tab-btn') {
                button.addEventListener('shown.bs.tab', () => {
                    // Trigger SVG generation if the global function exists
                    if (typeof window.generateSvgFromLilypond === 'function') {
                        window.generateSvgFromLilypond();
                    }
                });
            }
        });
    }

    saveActiveTabFromBootstrap() {
        const activeTab = document.querySelector('.nav-link.active');
        if (activeTab) {
            const tabName = activeTab.id.replace('-tab-btn', '');
            saveActiveTab(tabName);
        }
    }

    restoreActiveTab() {
        const savedTab = loadActiveTab();
        
        // Find the saved tab button and activate it using Bootstrap
        const tabButton = document.getElementById(savedTab + '-tab-btn');
        
        if (tabButton) {
            // Use Bootstrap's Tab API to activate the tab
            const tab = new bootstrap.Tab(tabButton);
            tab.show();
        } else {
            // Fallback to first tab if saved tab doesn't exist
            const firstButton = document.querySelector('.nav-link');
            if (firstButton) {
                const tab = new bootstrap.Tab(firstButton);
                tab.show();
            }
        }
    }

    isSvgTabActive() {
        const svgTabButton = document.getElementById('svg-tab-btn');
        return svgTabButton && svgTabButton.classList.contains('active');
    }
}

// Loading state management
export class LoadingStateManager {
    static setLoadingState(element, message = 'Loading...') {
        if (!element) return;
        element.innerHTML = message;
        element.className = element.className.includes('json-output') 
            ? 'json-output p-3 loading' 
            : 'p-3 loading';
    }

    static setErrorState(element, errorMessage) {
        if (!element) return;
        const errorMsg = `<div class="error">Error: ${errorMessage}</div>`;
        element.innerHTML = errorMsg;
        element.className = element.className.includes('json-output')
            ? 'json-output p-3'
            : 'p-3';
    }

    static setContentState(element, content, isJson = false) {
        if (!element) return;
        
        if (isJson) {
            const jsonString = JSON.stringify(content, null, 2);
            element.innerHTML = '<div class="syntax-highlight">' + syntaxHighlight(jsonString) + '</div>';
            element.className = 'json-output p-3';
        } else if (typeof content === 'string' && content.includes('<')) {
            // HTML content (like SVG)
            element.innerHTML = content;
            element.className = 'p-3';
        } else {
            // Plain text content (like LilyPond)
            element.innerHTML = '<pre style="white-space: pre-wrap;">' + content + '</pre>';
            element.className = 'json-output p-3';
        }
    }

    static setEmptyState(element, message = 'No content available') {
        if (!element) return;
        element.innerHTML = message;
        element.className = element.className.includes('json-output')
            ? 'json-output p-3 loading'
            : 'p-3 loading';
    }
}

// Output display manager
export class OutputDisplayManager {
    constructor() {
        this.outputs = {
            document: document.querySelector('#document-tab .json-output'),
            processed: document.querySelector('#processed-tab .json-output'),
            minimalLily: document.querySelector('#minimal-lily-tab .json-output'),
            fullLily: document.querySelector('#full-lily-tab .json-output'),
            svg: document.getElementById('svg-content'),
            vexflowOutput: document.getElementById('vexflow-output'),
            detectedSystems: document.getElementById('detected-systems')
        };
    }

    updateDetectedSystems(systems) {
        if (!this.outputs.detectedSystems) return;
        
        if (systems && systems.length > 0) {
            this.outputs.detectedSystems.textContent = systems.join(', ');
            console.log('🎵 Updated notation systems display:', systems);
        } else {
            this.outputs.detectedSystems.textContent = 'None detected';
            console.log('⚠️ No notation systems detected');
        }
    }

    updateOutputs(data) {
        if (data.success) {
            this.updateDetectedSystems(data.detected_notation_systems);
            
            // Update each output section
            this.updateDocumentOutput(data.parsed_document);
            this.updateProcessedOutput(data.processed_staves);
            this.updateMinimalLilyOutput(data.minimal_lilypond);
            this.updateFullLilyOutput(data.full_lilypond);
            this.updateSvgOutput(data.lilypond_svg);
            
        } else {
            this.showError(data.error);
        }
    }


    updateDocumentOutput(documentOutput) {
        if (documentOutput) {
            LoadingStateManager.setContentState(this.outputs.document, documentOutput, true);
        } else {
            LoadingStateManager.setEmptyState(this.outputs.document, 'No document structure available');
        }
    }

    updateProcessedOutput(processedOutput) {
        if (processedOutput) {
            LoadingStateManager.setContentState(this.outputs.processed, processedOutput, true);
        } else {
            LoadingStateManager.setEmptyState(this.outputs.processed, 'No processed staves available');
        }
    }

    updateMinimalLilyOutput(lilyOutput) {
        if (lilyOutput) {
            LoadingStateManager.setContentState(this.outputs.minimalLily, lilyOutput);
        } else {
            LoadingStateManager.setEmptyState(this.outputs.minimalLily, 'No minimal LilyPond available');
        }
    }

    updateFullLilyOutput(lilyOutput) {
        if (lilyOutput) {
            LoadingStateManager.setContentState(this.outputs.fullLily, lilyOutput);
        } else {
            LoadingStateManager.setEmptyState(this.outputs.fullLily, 'No full LilyPond available');
        }
    }

    updateSvgOutput(svgOutput) {
        if (svgOutput) {
            LoadingStateManager.setContentState(this.outputs.svg, svgOutput);
        } else {
            LoadingStateManager.setEmptyState(this.outputs.svg, 'No SVG available');
        }
    }

    showError(error) {
        if (error === 'Parse error') {
            this.outputs.detectedSystems.textContent = 'Parse error';
        } else if (error === 'Network error') {
            this.outputs.detectedSystems.textContent = 'Network error';
        }

        // Show error in all output sections
        Object.values(this.outputs).forEach(output => {
            if (output && output !== this.outputs.detectedSystems) {
                LoadingStateManager.setErrorState(output, error);
            }
        });
    }

    showLoading() {
        // Set all outputs to loading state
        LoadingStateManager.setLoadingState(this.outputs.document, 'Parsing...');
        LoadingStateManager.setLoadingState(this.outputs.processed, 'Processing...');
        LoadingStateManager.setLoadingState(this.outputs.minimalLily, 'Generating...');
        LoadingStateManager.setLoadingState(this.outputs.fullLily, 'Generating...');
        LoadingStateManager.setLoadingState(this.outputs.svg, 'Rendering...');
    }

    showEmpty() {
        this.outputs.detectedSystems.textContent = 'Enter some music to see detected systems...';
        
        LoadingStateManager.setEmptyState(this.outputs.document, 'Parsed document structure will appear here...');
        LoadingStateManager.setEmptyState(this.outputs.processed, 'Processed staves will appear here...');
        LoadingStateManager.setEmptyState(this.outputs.minimalLily, 'Minimal LilyPond notation will appear here...');
        LoadingStateManager.setEmptyState(this.outputs.fullLily, 'Full LilyPond score will appear here...');
        LoadingStateManager.setEmptyState(this.outputs.svg, 'LilyPond SVG rendering will appear here...');
    }
}