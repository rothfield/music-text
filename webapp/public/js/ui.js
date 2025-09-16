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
        document.getElementById('parser-output').textContent = 'Enter music notation to see parser output';
        document.getElementById('rhythm-output').textContent = 'Enter music notation to see rhythm analyzer output';
        document.getElementById('spatial-output').textContent = 'Enter music notation to see spatial analysis output';
        document.getElementById('analyzer-output').textContent = 'Enter music notation to see analyzer output';
        document.getElementById('tokens-output').textContent = 'Enter music notation to see syntax tokens';
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
            
            // Parser Output - raw parser + spatial analysis output  
            document.getElementById('parser-output').textContent = 
                JSON.stringify(result.parsed_document || {}, null, 2);
            
            // Rhythm Output - document after rhythm analyzer
            document.getElementById('rhythm-output').textContent = 
                JSON.stringify(result.processed_staves || [], null, 2);
            
            // Spatial Analysis Output - document after spatial analysis (with positions)
            document.getElementById('spatial-output').textContent = 
                JSON.stringify(result.parsed_document || {}, null, 2);
            
            // Analyzer Output - rhythm analyzed document (final output from analyzer)
            document.getElementById('analyzer-output').textContent =
                JSON.stringify(result.rhythm_analyzed_document || {}, null, 2);
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

    updateXMLOutput(result) {
        const xmlOutput = document.getElementById('xml-output');
        
        if (result.success && result.xml_representation) {
            // Escape the XML for display
            const escapedXML = this.escapeHTML(result.xml_representation);
            
            // Display clean XML
            xmlOutput.innerHTML = `
                <pre style="background: #f6f8fa; padding: 12px; border-radius: 6px; overflow-x: auto; font-family: 'SF Mono', Monaco, Consolas, monospace; line-height: 1.4;">${escapedXML}</pre>
            `;
        } else if (result.success) {
            xmlOutput.textContent = 'Parse successful but no XML representation available';
        } else {
            xmlOutput.textContent = `Parse error: ${result.error}`;
        }
    },

    // Apply syntax highlighting to escaped XML using CSS classes
    highlightXML(escapedXML) {
        return escapedXML
            // Highlight note tags and content (working with escaped XML)
            .replace(/(&lt;note&gt;)([^&]+)(&lt;\/note&gt;)/g, '<span class="xml-note">$1</span><span class="xml-note-content">$2</span><span class="xml-note">$3</span>')
            
            // Highlight barline tags and content
            .replace(/(&lt;barline&gt;)([^&]+)(&lt;\/barline&gt;)/g, '<span class="xml-barline">$1</span><span class="xml-barline-content">$2</span><span class="xml-barline">$3</span>')
            
            // Highlight dash tags and content
            .replace(/(&lt;dash&gt;)([^&]+)(&lt;\/dash&gt;)/g, '<span class="xml-dash">$1</span><span class="xml-dash-content">$2</span><span class="xml-dash">$3</span>')
            
            // Highlight syllable tags and content
            .replace(/(&lt;syllable&gt;)([^&]+)(&lt;\/syllable&gt;)/g, '<span class="xml-syllable">$1</span><span class="xml-syllable-content">$2</span><span class="xml-syllable">$3</span>')
            
            // Highlight whitespace tags and content
            .replace(/(&lt;whitespace&gt;)([^&]+)(&lt;\/whitespace&gt;)/g, '<span class="xml-whitespace">$1</span><span class="xml-whitespace-content">$2</span><span class="xml-whitespace">$3</span>')
            
            // Highlight rest tags and content
            .replace(/(&lt;rest&gt;)([^&]+)(&lt;\/rest&gt;)/g, '<span class="xml-rest">$1</span><span class="xml-rest-content">$2</span><span class="xml-rest">$3</span>')
            
            // Highlight breath tags and content
            .replace(/(&lt;breath&gt;)([^&]+)(&lt;\/breath&gt;)/g, '<span class="xml-breath">$1</span><span class="xml-breath-content">$2</span><span class="xml-breath">$3</span>')
            
            // Highlight unknown tags and content
            .replace(/(&lt;unknown&gt;)([^&]+)(&lt;\/unknown&gt;)/g, '<span class="xml-unknown">$1</span><span class="xml-unknown-content">$2</span><span class="xml-unknown">$3</span>')
            
            // Highlight structural tags (music, stave, lyrics)
            .replace(/(&lt;\/?(?:music|stave|lyrics)&gt;)/g, '<span class="xml-structure">$1</span>');
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

    // Helper function to escape HTML for safe display
    escapeHTML(str) {
        return str
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;')
            .replace(/'/g, '&#39;');
    },

    displayXMLInEditor(base64XML) {
        const xmlContent = atob(base64XML);
        const originalContent = document.getElementById('musicInput').value;
        
        // Get the CodeMirror manager from the app instance
        if (window.MusicTextApp && window.MusicTextApp.codeMirrorManager) {
            window.MusicTextApp.codeMirrorManager.displayXML(xmlContent, originalContent);
        }
    },

    // Clear empty inputs
    clearEmptyInputs() {
        document.getElementById('vexflow-output').innerHTML = '';
        document.getElementById('lilypond-output').textContent = 'Enter music notation above to see LilyPond source';
        document.getElementById('xml-output').textContent = 'Enter music notation to see XML representation';
        document.getElementById('document-output').textContent = 'Enter music notation to see parsed document output';
        document.getElementById('parser-output').textContent = 'Enter music notation to see parser output';
        document.getElementById('rhythm-output').textContent = 'Enter music notation to see rhythm analyzer output';
        document.getElementById('spatial-output').textContent = 'Enter music notation to see spatial analysis output';
        document.getElementById('analyzer-output').textContent = 'Enter music notation to see analyzer output';
        document.getElementById('tokens-output').textContent = 'Enter music notation to see syntax tokens';
        document.getElementById('roundtrip-output').textContent = 'Enter music notation to test round-trip reconstruction';
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