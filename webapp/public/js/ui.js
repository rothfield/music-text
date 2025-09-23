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
        document.getElementById('svgpoc-output').innerHTML = 'Enter music notation and click "Generate SVG POC" to test the SVG renderer';
        document.getElementById('document-output').textContent = 'Enter music notation to see parsed document output';
        document.getElementById('spans-output').textContent = 'Enter music notation to see syntax spans';
        document.getElementById('styles-output').textContent = 'Enter music notation to see character styles';
        document.getElementById('source-output').textContent = 'Plain text will appear here after parsing';
        document.getElementById('status').textContent = '';
    },

    // Update pipeline data outputs
    updatePipelineData(result) {
        if (result.success) {
            // Document Output - structured document representation
            document.getElementById('document-output').textContent =
                JSON.stringify(result.document || {}, null, 2);
            
            
            
            
        } else {
            // Show error in all sections
            const errorMsg = `Parse error: ${result.error}`;
            document.getElementById('document-output').textContent = errorMsg;
        }
    },

    // Update LilyPond output
    updateLilyPondOutput(result) {
        // Update minimal LilyPond output
        const minimalOutput = document.getElementById('lilypond-output');
        if (result.success && result.lilypond_minimal) {
            minimalOutput.innerHTML = `<pre class="lilypond-source">${this.escapeHTML(result.lilypond_minimal)}</pre>`;
        } else if (result.success) {
            minimalOutput.innerHTML = '<pre class="lilypond-source">No LilyPond source available</pre>';
        } else {
            minimalOutput.innerHTML = `<pre class="lilypond-source">Parse error: ${this.escapeHTML(result.error)}</pre>`;
        }

        // Update full LilyPond output
        const fullOutput = document.getElementById('lilypond-full-output');
        if (result.success && result.lilypond) {
            fullOutput.innerHTML = `<pre class="lilypond-source">${this.escapeHTML(result.lilypond)}</pre>`;
        } else if (result.success) {
            fullOutput.innerHTML = '<pre class="lilypond-source">No LilyPond source available</pre>';
        } else {
            fullOutput.innerHTML = `<pre class="lilypond-source">Parse error: ${this.escapeHTML(result.error)}</pre>`;
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

    // Update SVG POC output
    updateSvgPocOutput(result) {
        const output = document.getElementById('svgpoc-output');
        if (result.success && result.svg_poc) {
            output.innerHTML = result.svg_poc;
        } else if (result.success) {
            output.innerHTML = '<p>Parsed successfully, but no SVG POC data available.</p>';
        } else {
            output.innerHTML = `<p>Parse error: ${result.error}</p>`;
        }
    },

    // Render VexFlow notation - execute self-generated JavaScript
    async renderVexFlow(vexflowData) {
        const output = document.getElementById('vexflow-output');
        output.innerHTML = ''; // Clear previous content

        try {
            if (vexflowData.vexflow_js) {
                // Load VexFlow library if not already loaded
                await this.ensureVexFlowLoaded();

                // Execute the generated JavaScript
                console.log('🎵 Executing VexFlow JavaScript:', vexflowData.vexflow_js);
                eval(vexflowData.vexflow_js);
            } else {
                // Fallback: show the raw data
                const pre = document.createElement('pre');
                pre.style.backgroundColor = '#f8f9fa';
                pre.style.padding = '1rem';
                pre.style.border = '1px solid #dee2e6';
                pre.style.borderRadius = '0.375rem';
                pre.style.overflow = 'auto';
                pre.style.fontSize = '0.875rem';
                pre.textContent = JSON.stringify(vexflowData, null, 2);
                output.appendChild(pre);
            }
        } catch (error) {
            console.error('🎵 VexFlow execution error:', error);
            output.innerHTML = `<p style="color: red;">VexFlow rendering error: ${error.message}</p>
                               <pre style="font-size: 0.8em; background: #f8f9fa; padding: 1rem; margin-top: 1rem;">
${vexflowData.vexflow_js || JSON.stringify(vexflowData, null, 2)}</pre>`;
        }
    },

    // Ensure VexFlow library is loaded
    async ensureVexFlowLoaded() {
        if (window.Vex && window.Vex.Flow) {
            return true; // Already loaded
        }

        return new Promise((resolve, reject) => {
            const script = document.createElement('script');
            script.src = 'assets/vexflow4.js';
            script.async = true;

            script.onload = () => {
                if (window.Vex && window.Vex.Flow) {
                    console.log('🎵 VexFlow library loaded successfully');
                    resolve(true);
                } else {
                    reject(new Error('VexFlow loaded but not accessible'));
                }
            };

            script.onerror = () => reject(new Error('Failed to load VexFlow library'));
            document.head.appendChild(script);
        });
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


    // Update syntax spans output
    updateTokensOutput(result) {
        const tokensOutput = document.getElementById('spans-output');

        if (result.success && result.syntax_spans) {
            tokensOutput.textContent = JSON.stringify(result.syntax_spans, null, 2);
        } else if (result.success) {
            tokensOutput.textContent = 'Parse successful but no syntax spans available';
        } else {
            tokensOutput.textContent = `Parse error: ${result.error}`;
        }
    },

    // Update character styles output
    updateStylesOutput(result) {
        const stylesOutput = document.getElementById('styles-output');

        if (result.success && result.character_styles) {
            stylesOutput.textContent = JSON.stringify(result.character_styles, null, 2);
        } else if (result.success) {
            stylesOutput.textContent = 'Parse successful but no character styles available';
        } else {
            stylesOutput.textContent = `Parse error: ${result.error}`;
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
        document.getElementById('lilypond-output').innerHTML = '<pre class="lilypond-source">Enter music notation above to see minimal LilyPond source</pre>';
        document.getElementById('lilypond-full-output').innerHTML = '<pre class="lilypond-source">Enter music notation above to see full LilyPond source</pre>';
        document.getElementById('document-output').textContent = 'Enter music notation to see parsed document output';
        document.getElementById('spans-output').textContent = 'Enter music notation to see syntax spans';
        document.getElementById('styles-output').textContent = 'Enter music notation to see character styles';
        document.getElementById('source-output').textContent = 'Plain text will appear here after parsing';
    },

};