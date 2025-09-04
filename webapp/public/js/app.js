// Local storage keys
const STORAGE_KEYS = {
    notation: 'notationParser.notation',
    system: 'notationParser.system',
    currentTab: 'notationParser.currentTab',
    lastParsed: 'notationParser.lastParsed'
};

// State
let parseTimeout;
let svgTimeout;
let currentTab = 'ast';
let lastResult = null;
let svgCache = null;
let lastInputForSvg = '';
let lastSystemForSvg = '';

// Initialize on page load
document.addEventListener('DOMContentLoaded', function() {
    // Debug: Log when app.js loads
    console.log('app.js STRUCTURE_PRESERVING_FSM_v2.0 loaded at', new Date().toISOString());
    
    loadFromStorage();
    setupEventListeners();
    checkServerStatus();
    
    // Check server status every 10 seconds
    setInterval(checkServerStatus, 10000);
    
    // Parse on startup if there's content
    const input = document.getElementById('input').value;
    if (input && input.trim()) {
        parse();
    }
});

// Setup event listeners
function setupEventListeners() {
    const inputEl = document.getElementById('input');
    const systemEl = document.getElementById('system');
    
    // Auto-save on input change with debounce
    inputEl.addEventListener('input', function() {
        saveToStorage();
        
        // Auto-parse with debounce
        clearTimeout(parseTimeout);
        parseTimeout = setTimeout(() => {
            if (this.value.trim()) {
                parse();
            }
        }, 500);
    });
    
    // Save system preference on change
    systemEl.addEventListener('change', function() {
        saveToStorage();
        parse();
    });
}

// Save to local storage
function saveToStorage() {
    const input = document.getElementById('input').value;
    const system = document.getElementById('system').value;
    
    localStorage.setItem(STORAGE_KEYS.notation, input);
    localStorage.setItem(STORAGE_KEYS.system, system);
    localStorage.setItem(STORAGE_KEYS.currentTab, currentTab);
    
    // Save timestamp
    localStorage.setItem(STORAGE_KEYS.lastParsed, new Date().toISOString());
}

// Load from local storage
function loadFromStorage() {
    const savedNotation = localStorage.getItem(STORAGE_KEYS.notation);
    const savedSystem = localStorage.getItem(STORAGE_KEYS.system);
    const savedTab = localStorage.getItem(STORAGE_KEYS.currentTab);
    
    if (savedNotation !== null) {
        document.getElementById('input').value = savedNotation;
    }
    
    if (savedSystem !== null) {
        document.getElementById('system').value = savedSystem;
    }
    
    if (savedTab !== null) {
        currentTab = savedTab;
        // Activate the saved tab after tabs are loaded
        setTimeout(() => {
            const tabButton = document.querySelector(`.tab[data-tab="${savedTab}"]`);
            if (tabButton) {
                tabButton.click();
            }
        }, 100);
    }
}

// Tab switching
function showTab(tabName) {
    currentTab = tabName;
    saveToStorage(); // Save tab state
    
    // Update tab buttons
    document.querySelectorAll('.tab').forEach(tab => {
        tab.classList.remove('active');
    });
    const activeTabButton = document.querySelector(`.tab[data-tab="${tabName}"]`);
    if (activeTabButton) {
        activeTabButton.classList.add('active');
    }
    
    // Update tab content
    document.querySelectorAll('.tab-content').forEach(content => {
        content.classList.remove('active');
    });
    const contentEl = document.getElementById(tabName + '-content');
    if (contentEl) {
        contentEl.classList.add('active');
    }
    
    // Handle LilyPond PNG/SVG tab - generate SVG with debounce
    if (tabName === 'lilypond-png') {
        debouncedSvgGeneration();
    }
}

// Debounced SVG generation (5 seconds)
function debouncedSvgGeneration() {
    const input = document.getElementById('input').value;
    const system = document.getElementById('system').value;
    
    // Clear existing timeout
    clearTimeout(svgTimeout);
    
    // Check if we need to generate new SVG
    if (input === lastInputForSvg && system === lastSystemForSvg && svgCache) {
        // Use cached result
        displaySvgResult(svgCache);
        return;
    }
    
    // Show loading state
    document.getElementById('lilypond-png-output').innerHTML = 
        '<div style="color: #666; text-align: center; padding: 40px;">⏳ Generating SVG in 5 seconds...</div>';
    
    // Set timeout for 5 seconds
    svgTimeout = setTimeout(async () => {
        await generateSvg(input, system);
    }, 5000);
}

// Generate SVG
async function generateSvg(input, system) {
    if (!input.trim()) {
        document.getElementById('lilypond-png-output').innerHTML = 
            '<div class="warning">Enter some notation to generate SVG</div>';
        return;
    }
    
    // Show generating state
    document.getElementById('lilypond-png-output').innerHTML = 
        '<div style="color: #666; text-align: center; padding: 40px;">🎼 Generating SVG...</div>';
    
    try {
        const response = await fetch('/api/lilypond/svg', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ input: input, notation: system })
        });
        
        const svgResult = await response.json();
        
        // Cache the result
        svgCache = svgResult;
        lastInputForSvg = input;
        lastSystemForSvg = system;
        
        displaySvgResult(svgResult);
        
    } catch (error) {
        console.error('SVG generation error:', error);
        document.getElementById('lilypond-png-output').innerHTML = 
            '<div class="warning">Error generating LilyPond SVG</div>';
    }
}

// Display SVG result
function displaySvgResult(svgResult) {
    if (svgResult.success && svgResult.svg_url) {
        document.getElementById('lilypond-png-output').innerHTML = 
            `<img src="${svgResult.svg_url}" alt="LilyPond SVG Output" style="max-width: 100%; border: 1px solid #ddd;">`;
    } else {
        document.getElementById('lilypond-png-output').innerHTML = 
            '<div class="warning">Failed to generate LilyPond SVG</div>';
    }
}

// Check server status
async function checkServerStatus() {
    const statusEl = document.getElementById('server-status');
    try {
        const response = await fetch('/health');
        if (response.ok) {
            statusEl.className = 'status online';
            statusEl.textContent = 'Online';
            return true;
        }
    } catch (error) {
        // Server offline
    }
    statusEl.className = 'status offline';
    statusEl.textContent = 'Offline';
    return false;
}

// Parse function
async function parse() {
    const input = document.getElementById('input').value;
    const system = document.getElementById('system').value;
    
    if (!input.trim()) {
        updateAllOutputs('Enter some notation to parse...');
        return;
    }
    
    // Show loading state
    updateAllOutputs('⏳ Processing...');
    
    try {
        const requestBody = { 
            input: input,
            notation: system,
            output: ['full']
        };
        console.log('Making API request with:', requestBody);
        
        const response = await fetch('/api/parse', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(requestBody)
        });
        
        console.log('Response status:', response.status);
        console.log('Response headers:', Object.fromEntries(response.headers.entries()));
        
        // Get raw text first to log it
        const rawText = await response.text();
        console.log('=== RAW SERVER RESPONSE ===');
        console.log(rawText);
        console.log('=== END RAW RESPONSE ===');
        
        // Parse the JSON from the raw text
        const result = JSON.parse(rawText);
        lastResult = result;
        
        // Debug logging
        console.log('Parse result:', result);
        console.log('Success:', result.success);
        console.log('Has lilypond:', !!result.lilypond);
        console.log('Lilypond preview:', result.lilypond ? result.lilypond.substring(0, 100) + '...' : 'null');
        
        // Log AST specifically
        if (result.ast) {
            console.log('=== AST FIELD (raw string) ===');
            console.log(result.ast);
            console.log('=== PARSED AST OBJECT ===');
            try {
                const astObj = JSON.parse(result.ast);
                console.log(astObj);
                console.log('AST upper_lines:', astObj.staves?.[0]?.upper_lines);
                console.log('AST content_line:', astObj.staves?.[0]?.content_line);
            } catch (e) {
                console.error('Failed to parse AST JSON:', e);
            }
        }
        
        if (result.success) {
            // Update AST tab
            if (result.ast) {
                const astNote = '✅ Raw AST Parsing Complete\n' +
                              '===================================\n\n' +
                              'Pest grammar parsed into:\n' +
                              '• Document structure with staves\n' +
                              '• Measures and beats\n' +
                              '• Raw pitch and dash elements\n\n' +
                              'Raw AST (JSON):\n\n';
                updateOutput('ast-output', astNote + result.ast);
            } else {
                updateOutput('ast-output', 'No AST generated', 'warning');
            }
            
            // Update Parser/YAML tab
            if (result.yaml) {
                updateOutput('parser-output', result.yaml);
            } else {
                updateOutput('parser-output', 'No YAML output available', 'warning');
            }
            
            // Update spatial tab
            if (result.spatial) {
                const spatialNote = 'Spatial-Enhanced AST (JSON):\n\n';
                updateOutput('spatial-output', spatialNote + result.spatial);
            } else {
                updateOutput('spatial-output', 'Spatial processing not available', 'warning');
            }
            
            // Update FSM tab
            if (result.fsm) {
                const fsmNote = '✅ FSM Rhythm Analysis Complete\n' +
                              '======================================\n\n' +
                              'Rhythm features detected:\n' +
                              '• Beat divisions and tuplet analysis\n' +
                              '• Note subdivisions and durations\n' +
                              '• Extension and tie processing\n\n' +
                              'FSM Output (JSON):\n\n';
                updateOutput('fsm-output', fsmNote + result.fsm);
            } else {
                updateOutput('fsm-output', 
                    'FSM processing not available', 'warning');
            }
            
            // Update LilyPond Minimal tab
            console.log('Checking LilyPond section: result.lilypond exists =', !!result.lilypond);
            if (result.lilypond) {
                console.log('Entering LilyPond section - should display LilyPond');
                // Call minimal LilyPond API endpoint
                fetch('/api/lilypond/minimal', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ input: input, notation: system })
                })
                .then(response => response.json())
                .then(minimalResult => {
                    if (minimalResult.success) {
                        updateOutput('lilypond-minimal-output', minimalResult.lilypond_source);
                    } else {
                        updateOutput('lilypond-minimal-output', 
                            'Failed to extract minimal LilyPond', 'warning');
                    }
                })
                .catch(() => {
                    updateOutput('lilypond-minimal-output', 
                        'Error fetching minimal LilyPond', 'warning');
                });
                        
                // Update LilyPond Full tab with complete source
                console.log('Updating lilypond-src-output with:', result.lilypond.substring(0, 50) + '...');
                updateOutput('lilypond-src-output', result.lilypond);
            } else {
                console.log('ELSE: No lilypond found, showing not implemented message');
                updateOutput('lilypond-minimal-output', 
                    'LilyPond converter not yet implemented', 'warning');
                updateOutput('lilypond-src-output', 
                    'LilyPond converter not yet implemented', 'warning');
            }
            
            // LilyPond PNG tab - handled by tab switching with debounce
            // SVG generation only occurs when tab is selected
            
            // Update VexFlow tab
            if (result.vexflow) {
                updateOutput('vexflow-output', 
                    JSON.stringify(result.vexflow, null, 2));
            } else {
                updateOutput('vexflow-output', 
                    'VexFlow converter not yet implemented', 'warning');
            }
            
            // Update Raw JSON tab
            updateOutput('raw-output', JSON.stringify(result, null, 2));
            
        } else {
            updateAllOutputs('Parse Error:\n' + result.error, 'error');
        }
    } catch (error) {
        console.error('Parse error:', error);
        updateAllOutputs('Network error: ' + error.message, 'error');
    }
}

// Update single output
function updateOutput(elementId, content, className = '') {
    const el = document.getElementById(elementId);
    if (el) {
        el.textContent = content;
        el.className = className;
    }
}

// Update all outputs
function updateAllOutputs(message, className = '') {
    const outputIds = ['ast-output', 'parser-output', 'spatial-output', 
                      'fsm-output', 'lilypond-minimal-output', 'lilypond-src-output', 'vexflow-output', 
                      'raw-output'];
    
    outputIds.forEach(id => {
        updateOutput(id, message, className);
    });
    
    // Handle PNG output separately
    const pngEl = document.getElementById('lilypond-png-output');
    if (pngEl) {
        pngEl.innerHTML = `<div class="${className}">${message}</div>`;
    }
}

// Copy to clipboard
async function copyToClipboard(elementId) {
    const outputEl = document.getElementById(elementId);
    if (!outputEl) return;
    
    const text = outputEl.textContent;
    try {
        await navigator.clipboard.writeText(text);
        
        // Show feedback - get the button that was clicked
        const btn = window.event?.target || document.activeElement;
        if (btn && btn.tagName === 'BUTTON') {
            const originalText = btn.textContent;
            btn.textContent = '✓ Copied!';
            setTimeout(() => {
                btn.textContent = originalText;
            }, 2000);
        }
    } catch (err) {
        console.error('Failed to copy:', err);
        alert('Failed to copy to clipboard');
    }
}

// Export functions for HTML onclick handlers
window.showTab = showTab;
window.parse = parse;
window.copyToClipboard = copyToClipboard;