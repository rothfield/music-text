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


    // Tab management
    switchTab(tabName, clickedTab) {

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

        // Initialize MIDI player on first access to MIDI tab
        if (tabName === 'midi' && window.musicApp && !window.musicApp.midiInitialized) {
            console.log('🎵 Initializing MIDI player on first access...');
            window.musicApp.setupMIDI().then(() => {
                window.musicApp.midiInitialized = true;
                console.log('✅ MIDI player initialized successfully');
            }).catch(error => {
                console.error('❌ Failed to initialize MIDI player:', error);
            });
        }

        // Save active tab to localStorage
        LocalStorage.saveActiveTab(tabName);

        // Focus the canvas editor
        if (window.canvasEditor) {
            window.canvasEditor.focus();
        }
    },

    // Clear all UI content
    clearAllContent() {
        document.getElementById('musicInput').value = '';
        document.getElementById('lilypond-output').textContent = 'Enter music notation above to see LilyPond source';
        document.getElementById('vexflow-output').innerHTML = '';
        document.getElementById('svg-output').innerHTML = 'Click "LilyPond" to generate SVG';
        document.getElementById('svgpoc-output').innerHTML = 'Enter music notation and click "Generate SVG POC" to test the SVG renderer';
        document.getElementById('document-output').textContent = 'Enter music notation to see parsed document output';
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
                // Give VexFlow a moment to initialize
                setTimeout(() => {
                    if (window.Vex && window.Vex.Flow) {
                        console.log('🎵 VexFlow library loaded successfully');
                        resolve(true);
                    } else {
                        console.error('❌ VexFlow loaded but Vex.Flow not accessible. Available:', {
                            hasVex: !!window.Vex,
                            hasFlow: !!(window.Vex && window.Vex.Flow),
                            vexKeys: window.Vex ? Object.keys(window.Vex) : []
                        });
                        reject(new Error('VexFlow loaded but not accessible'));
                    }
                }, 100);
            };

            script.onerror = (error) => {
                console.error('❌ Failed to load VexFlow script:', error);
                reject(new Error('Failed to load VexFlow library'));
            };

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
        document.getElementById('source-output').textContent = 'Plain text will appear here after parsing';
    },

    // Update SVG source output
    updateSVGSourceOutput(result) {
        const svgSourceOutput = document.getElementById('svg-source-output');

        if (result.success && result.canvas_svg) {
            svgSourceOutput.innerHTML = `<pre>${this.escapeHTML(result.canvas_svg)}</pre>`;
        } else {
            svgSourceOutput.innerHTML = '<p>No SVG source available</p>';
        }
    },

    // Update backing text display (read-only)
    updateBackingTextOutput(text) {
        const output = document.getElementById('backing-text-output');
        if (output) {
            output.value = text;
            output.readOnly = true; // Ensure read-only in document-first architecture
        }
    },

};