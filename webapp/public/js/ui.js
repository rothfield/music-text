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
        document.getElementById('document-output').textContent = 'Enter music notation to see document structure';
        document.getElementById('parser-output').textContent = 'Enter music notation to see parser output';
        document.getElementById('rhythm-output').textContent = 'Enter music notation to see rhythm analyzer output';
        document.getElementById('spatial-output').textContent = 'Enter music notation to see spatial analysis output';
        document.getElementById('analyzer-output').textContent = 'Enter music notation to see analyzer output';
        document.getElementById('status').textContent = '';
    },

    // Update pipeline data outputs
    updatePipelineData(result) {
        if (result.success) {
            // Document Output - text-lines, paragraphs, and content lines structure
            const documentStructure = {
                source: result.parsed_document?.source || null,
                directives: result.parsed_document?.directives || [],
                staves: result.parsed_document?.staves?.map(stave => ({
                    text_lines_before: stave.text_lines_before || [],
                    content_line: stave.content_line || [],
                    text_lines_after: stave.text_lines_after || [],
                    upper_lines: stave.upper_lines || [],
                    lower_lines: stave.lower_lines || [],
                    lyrics_lines: stave.lyrics_lines || [],
                    source: stave.source
                })) || []
            };
            document.getElementById('document-output').textContent = 
                JSON.stringify(documentStructure, null, 2);
            
            // Parser Output - raw parser + spatial analysis output  
            document.getElementById('parser-output').textContent = 
                JSON.stringify(result.parsed_document || {}, null, 2);
            
            // Rhythm Output - document after rhythm analyzer
            document.getElementById('rhythm-output').textContent = 
                JSON.stringify(result.processed_staves || [], null, 2);
            
            // Spatial Analysis Output - document after spatial analysis (with positions)
            document.getElementById('spatial-output').textContent = 
                JSON.stringify(result.parsed_document || {}, null, 2);
            
            // Analyzer Output - VexFlow data, notation systems, etc.
            const analyzerData = {
                vexflow: result.vexflow,
                detected_notation_systems: result.detected_notation_systems,
                error: result.error
            };
            document.getElementById('analyzer-output').textContent = 
                JSON.stringify(analyzerData, null, 2);
        } else {
            // Show error in all sections
            const errorMsg = `Parse error: ${result.error}`;
            document.getElementById('document-output').textContent = errorMsg;
            document.getElementById('parser-output').textContent = errorMsg;
            document.getElementById('rhythm-output').textContent = errorMsg;
            document.getElementById('spatial-output').textContent = errorMsg;
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

    // Clear empty inputs
    clearEmptyInputs() {
        document.getElementById('vexflow-output').innerHTML = '';
        document.getElementById('lilypond-output').textContent = 'Enter music notation above to see LilyPond source';
        document.getElementById('document-output').textContent = 'Enter music notation to see document structure';
        document.getElementById('parser-output').textContent = 'Enter music notation to see parser output';
        document.getElementById('rhythm-output').textContent = 'Enter music notation to see rhythm analyzer output';
        document.getElementById('spatial-output').textContent = 'Enter music notation to see spatial analysis output';
        document.getElementById('analyzer-output').textContent = 'Enter music notation to see analyzer output';
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