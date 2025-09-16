/**
 * UI Utilities Module
 * Handles DOM manipulation and UI state management
 */

import { LocalStorage } from './localStorage.js';

export const UI = {
    // Status management
    setStatus(message, type = 'loading') {
        const status = document.getElementById('status');
        status.className = `status ${type}`;
        status.textContent = message;
        if (type === 'success' || type === 'error') {
            setTimeout(() => status.textContent = '', 3000);
        }
    },

    // Cursor and focus management
    restoreFocusAndCursor() {
        const musicInput = document.getElementById('musicInput');
        
        // Always focus first
        musicInput.focus();
        
        // Multiple attempts to ensure cursor restoration works
        const restoreCursor = () => {
            const {start, end} = LocalStorage.loadCursorPosition();
            const textLength = musicInput.value.length;
            
            // Ensure cursor positions are valid for current text
            const safeStart = Math.min(Math.max(0, start), textLength);
            const safeEnd = Math.min(Math.max(0, end), textLength);
            
            console.log('🔄 Restoring cursor position:', {start, end, safeStart, safeEnd, textLength});
            
            // Force the selection range
            try {
                musicInput.setSelectionRange(safeStart, safeEnd);
                // Verify the position was actually set
                const actualStart = musicInput.selectionStart;
                const actualEnd = musicInput.selectionEnd;
                console.log('✅ Cursor restored - requested:', safeStart, safeEnd, 'actual:', actualStart, actualEnd);
            } catch (e) {
                console.warn('Failed to set cursor position:', e);
            }
        };
        
        // Try multiple times to ensure it works
        requestAnimationFrame(restoreCursor);
        setTimeout(restoreCursor, 10);
        setTimeout(restoreCursor, 50);
    },

    // Tab management
    switchTab(tabName, clickedTab) {
        // Save cursor position BEFORE switching tabs
        const musicInput = document.getElementById('musicInput');
        const currentStart = musicInput.selectionStart;
        const currentEnd = musicInput.selectionEnd;
        if (currentStart !== undefined && currentEnd !== undefined && currentStart >= 0 && currentEnd >= 0) {
            LocalStorage.saveCursorPosition(currentStart, currentEnd);
        }
        
        // Remove active class from all tabs and content
        document.querySelectorAll('.tab').forEach(tab => tab.classList.remove('active'));
        document.querySelectorAll('.tab-content').forEach(content => content.classList.remove('active'));
        
        // Add active class to clicked tab and corresponding content
        if (clickedTab) {
            clickedTab.classList.add('active');
        } else {
            document.querySelector(`[onclick*="${tabName}"]`)?.classList.add('active');
        }
        document.getElementById(`${tabName}-tab`)?.classList.add('active');
        
        // Save active tab to localStorage
        LocalStorage.saveActiveTab(tabName);
        
        // ALWAYS restore focus and cursor to textarea
        this.restoreFocusAndCursor();
    },

    // Clear all UI content
    clearAllContent() {
        document.getElementById('musicInput').value = '';
        document.getElementById('lilypond-output').textContent = 'Enter music notation above to see LilyPond source';
        document.getElementById('vexflow-output').innerHTML = '';
        document.getElementById('svg-output').innerHTML = 'Click "LilyPond" to generate SVG';
        document.getElementById('document-output').textContent = 'Enter music notation to see parsed document output';
        document.getElementById('analyzer-output').textContent = 'Enter music notation to see analyzer output';
        document.getElementById('tokens-output').textContent = 'Enter music notation to see syntax tokens';
        document.getElementById('source-output').textContent = 'Plain text will appear here after parsing';
        document.getElementById('status').textContent = '';
    },

    // Update pipeline data outputs
    updatePipelineData(result) {
        if (result.success) {
            // Document Output - structured document representation
            const documentStructure = {
                source: result.parsed_document?.source || null,
                directives: result.parsed_document?.directives || [],
                elements: result.parsed_document?.elements?.map(element => {
                    if (element.Stave) {
                        return {
                            type: "Stave",
                            lines: element.Stave.lines || [],
                            rhythm_items: element.Stave.rhythm_items || [],
                            notation_system: element.Stave.notation_system || null,
                            source: element.Stave.source || null
                        };
                    }
                    return element;
                }) || []
            };
            document.getElementById('document-output').textContent = 
                JSON.stringify(documentStructure, null, 2);
            
            
            
            
            // Analyzer Output - rhythm items from analyzer
            document.getElementById('analyzer-output').textContent =
                JSON.stringify(result.rhythm_items || [], null, 2);
        } else {
            // Show error in all sections
            const errorMsg = `Parse error: ${result.error}`;
            document.getElementById('document-output').textContent = errorMsg;
            document.getElementById('analyzer-output').textContent = errorMsg;
        }
    },

    // Update LilyPond output
    updateLilyPondOutput(result) {
        if (result.success && result.lilypond) {
            document.getElementById('lilypond-output').textContent = result.lilypond;
        } else if (result.success) {
            document.getElementById('lilypond-output').textContent = 'No LilyPond source available';
        } else {
            document.getElementById('lilypond-output').textContent = `Parse error: ${result.error}`;
        }
    },

    // Update VexFlow output
    async updateVexFlowOutput(result) {
        if (result.success && result.vexflow) {
            await this.renderVexFlow(result.vexflow);
        } else if (result.success) {
            document.getElementById('vexflow-output').innerHTML = '<p>Parsed successfully, but no VexFlow data available.</p>';
        } else {
            document.getElementById('vexflow-output').innerHTML = `<p>Parse error: ${result.error}</p>`;
        }
    },

    // Render VexFlow notation
    async renderVexFlow(vexflowData) {
        const output = document.getElementById('vexflow-output');
        output.innerHTML = ''; // Clear previous content
        
        try {
            if (window.VexFlowRenderer) {
                // Use the sophisticated VexFlow renderer
                const success = await window.VexFlowRenderer.renderVexFlowNotation(vexflowData, 'vexflow-output');
                if (!success) {
                    output.innerHTML = `<p>VexFlow rendering failed. Showing raw data:</p><pre>${JSON.stringify(vexflowData, null, 2)}</pre>`;
                }
            } else {
                output.innerHTML = '<p>VexFlow renderer not loaded. Showing raw data:</p><pre>' + JSON.stringify(vexflowData, null, 2) + '</pre>';
            }
        } catch (error) {
            output.innerHTML = `<p>VexFlow rendering error: ${error.message}</p><pre>${JSON.stringify(vexflowData, null, 2)}</pre>`;
        }
    },

    // Update SVG output
    updateSVGOutput(result) {
        if (result.success && result.lilypond_svg) {
            document.getElementById('svg-output').innerHTML = result.lilypond_svg;
            return true;
        } else if (result.success) {
            let errorMsg = 'SVG generation failed - no SVG content returned. Check server console for LilyPond errors.';
            if (!result.lilypond) {
                errorMsg = 'No LilyPond source available to generate SVG from.';
            }
            document.getElementById('svg-output').innerHTML = `<p>${errorMsg}</p><details><summary>Debug Info</summary><pre>${JSON.stringify(result, null, 2)}</pre></details>`;
            return false;
        } else {
            document.getElementById('svg-output').innerHTML = `<p>Parse error: ${result.error}</p>`;
            return false;
        }
    },

    // Update roundtrip output
    updateRoundtripOutput(result) {
        const roundtripOutput = document.getElementById('roundtrip-output');
        
        if (result.success && result.roundtrip) {
            const roundtrip = result.roundtrip;
            
            if (roundtrip.works) {
                roundtripOutput.innerHTML = `
                    <div style="color: green; font-weight: bold;">✅ ROUNDTRIP SUCCESS</div>
                    <div>Original length: ${roundtrip.original_length} characters</div>
                    <div>Reconstructed length: ${roundtrip.reconstructed_length} characters</div>
                    <div style="margin-top: 1em;"><strong>Reconstructed text:</strong></div>
                    <pre style="background: #f0f0f0; padding: 8px; border-radius: 4px;">${roundtrip.reconstructed_text}</pre>
                `;
            } else {
                roundtripOutput.innerHTML = `
                    <div style="color: red; font-weight: bold;">❌ ROUNDTRIP FAILURE</div>
                    <div>Original length: ${roundtrip.original_length} characters</div>
                    <div>Reconstructed length: ${roundtrip.reconstructed_length} characters</div>
                    <div style="margin-top: 1em;"><strong>Where it failed:</strong></div>
                    <div style="color: red;">${roundtrip.where_it_failed || 'Unknown failure'}</div>
                    <div style="margin-top: 1em;"><strong>Reconstructed text:</strong></div>
                    <pre style="background: #ffe6e6; padding: 8px; border-radius: 4px;">${roundtrip.reconstructed_text}</pre>
                `;
            }
        } else if (result.success) {
            roundtripOutput.textContent = 'Parse successful but no roundtrip data available';
        } else {
            roundtripOutput.textContent = `Parse error: ${result.error}`;
        }
    },


    // Update syntax tokens output
    updateTokensOutput(result) {
        const tokensOutput = document.getElementById('tokens-output');

        if (result.success && result.syntax_tokens) {
            tokensOutput.textContent = JSON.stringify(result.syntax_tokens, null, 2);
        } else if (result.success) {
            tokensOutput.textContent = 'Parse successful but no syntax tokens available';
        } else {
            tokensOutput.textContent = `Parse error: ${result.error}`;
        }
    },

    // Update source output
    updateSourceOutput(result) {
        const sourceOutput = document.getElementById('source-output');

        if (result.plain_text) {
            sourceOutput.innerHTML = `<pre>${this.escapeHTML(result.plain_text)}</pre>`;
        } else {
            sourceOutput.innerHTML = '<p>No plain text in response</p>';
        }
    },

    // Helper function to escape HTML for safe display
    escapeHTML(str) {
        return str
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;')
            .replace(/'/g, '&#39;');
    },


    // Clear empty inputs
    clearEmptyInputs() {
        document.getElementById('vexflow-output').innerHTML = '';
        document.getElementById('lilypond-output').textContent = 'Enter music notation above to see LilyPond source';
        document.getElementById('document-output').textContent = 'Enter music notation to see parsed document output';
        document.getElementById('analyzer-output').textContent = 'Enter music notation to see analyzer output';
        document.getElementById('tokens-output').textContent = 'Enter music notation to see syntax tokens';
        document.getElementById('source-output').textContent = 'Plain text will appear here after parsing';
    },

    // Music notation symbol conversion
    convertMusicNotation(text) {
        let convertedValue = text;
        
        // Convert # to ♯ when it follows a note (number, letter, or sargam)
        convertedValue = convertedValue.replace(/([1-7A-GSRGMPDNsrgmpdnrbdb])#/g, '$1♯');
        convertedValue = convertedValue.replace(/([1-7A-GSRGMPDNsrgmpdnrbdb])##/g, '$1♯♯');
        
        // Convert b to ♭ when it follows a note (number, letter, or sargam) 
        convertedValue = convertedValue.replace(/([1-7A-GSRGMPDNsrgmpdnrbdb])b/g, '$1♭');
        convertedValue = convertedValue.replace(/([1-7A-GSRGMPDNsrgmpdnrbdb])bb/g, '$1♭♭');
        
        return convertedValue;
    }
};