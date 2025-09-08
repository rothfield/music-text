let debounceTimer;
let svgDebounceTimer;
const STORAGE_KEYS = {
    INPUT_TEXT: 'music-text-parser-input',
    ACTIVE_TAB: 'music-text-parser-active-tab'
};

// Bootstrap handles tab switching automatically, but we still need to save active tab
function saveActiveTabFromBootstrap() {
    const activeTab = document.querySelector('.nav-link.active');
    if (activeTab) {
        const tabName = activeTab.id.replace('-tab-btn', '');
        saveActiveTab(tabName);
    }
}

// Add event listeners to Bootstrap tab buttons
document.addEventListener('DOMContentLoaded', function() {
    const tabButtons = document.querySelectorAll('.nav-link');
    tabButtons.forEach(button => {
        button.addEventListener('shown.bs.tab', saveActiveTabFromBootstrap);
    });
});

function saveInputText(text) {
    try {
        localStorage.setItem(STORAGE_KEYS.INPUT_TEXT, text);
    } catch (e) {
        console.warn('Failed to save input text to localStorage:', e);
    }
}

function loadInputText() {
    try {
        return localStorage.getItem(STORAGE_KEYS.INPUT_TEXT) || '';
    } catch (e) {
        console.warn('Failed to load input text from localStorage:', e);
        return '';
    }
}

function saveActiveTab(tabName) {
    try {
        localStorage.setItem(STORAGE_KEYS.ACTIVE_TAB, tabName);
    } catch (e) {
        console.warn('Failed to save active tab to localStorage:', e);
    }
}

function loadActiveTab() {
    try {
        return localStorage.getItem(STORAGE_KEYS.ACTIVE_TAB) || 'pest';
    } catch (e) {
        console.warn('Failed to load active tab from localStorage:', e);
        return 'pest';
    }
}

function restoreActiveTab() {
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

function syntaxHighlight(json) {
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

async function parseInput(input) {
    console.log('🚀 parseInput() called:', {
        inputLength: input.length,
        isEmpty: !input.trim(),
        firstLine: input.split('\n')[0],
        totalLines: input.split('\n').length,
        timestamp: new Date().toISOString()
    });
    
    const pestOutput = document.querySelector('#pest-tab .json-output');
    const documentOutput = document.querySelector('#document-tab .json-output');
    const processedOutput = document.querySelector('#processed-tab .json-output');
    const minimalLilyOutput = document.querySelector('#minimal-lily-tab .json-output');
    const fullLilyOutput = document.querySelector('#full-lily-tab .json-output');
    const svgOutput = document.getElementById('svg-content');
    const vexflowCanvas = document.getElementById('vexflow-canvas');
    const vexflowData = document.getElementById('vexflow-data');
    
    if (!input.trim()) {
        // Reset notation systems display
        const detectedSystemsSpan = document.getElementById('detected-systems');
        detectedSystemsSpan.textContent = 'Enter some music to see detected systems...';
        
        pestOutput.innerHTML = 'Type in the textarea above to see the raw PEST parse tree...';
        pestOutput.className = 'json-output p-3 loading';
        documentOutput.innerHTML = 'Parsed document structure will appear here...';
        documentOutput.className = 'json-output p-3 loading';
        processedOutput.innerHTML = 'Processed staves will appear here...';
        processedOutput.className = 'json-output p-3 loading';
        minimalLilyOutput.innerHTML = 'Minimal LilyPond notation will appear here...';
        minimalLilyOutput.className = 'json-output p-3 loading';
        fullLilyOutput.innerHTML = 'Full LilyPond score will appear here...';
        fullLilyOutput.className = 'json-output p-3 loading';
        svgOutput.innerHTML = 'LilyPond SVG rendering will appear here...';
        svgOutput.className = 'p-3 loading';
        if (vexflowData) vexflowData.innerHTML = 'VexFlow notation data will appear here...';
        if (vexflowCanvas) {
            const ctx = vexflowCanvas.getContext('2d');
            ctx.clearRect(0, 0, vexflowCanvas.width, vexflowCanvas.height);
            ctx.fillStyle = '#fafafa';
            ctx.fillRect(0, 0, vexflowCanvas.width, vexflowCanvas.height);
            ctx.fillStyle = '#666';
            ctx.font = '14px Arial';
            ctx.fillText('VexFlow canvas will render here...', 20, 100);
        }
        return;
    }
    
    try {
        // Set all outputs to loading
        pestOutput.innerHTML = 'Parsing...';
        pestOutput.className = 'tab-content json-output loading';
        documentOutput.innerHTML = 'Parsing...';
        documentOutput.className = 'tab-content json-output loading';
        processedOutput.innerHTML = 'Processing...';
        processedOutput.className = 'tab-content json-output loading';
        minimalLilyOutput.innerHTML = 'Generating...';
        minimalLilyOutput.className = 'tab-content json-output loading';
        fullLilyOutput.innerHTML = 'Generating...';
        fullLilyOutput.className = 'tab-content json-output loading';
        svgOutput.innerHTML = 'Rendering...';
        svgOutput.className = 'tab-content loading';
        if (vexflowData) vexflowData.innerHTML = 'Converting...';
        if (vexflowCanvas) {
            const ctx = vexflowCanvas.getContext('2d');
            ctx.clearRect(0, 0, vexflowCanvas.width, vexflowCanvas.height);
            ctx.fillStyle = '#fafafa';
            ctx.fillRect(0, 0, vexflowCanvas.width, vexflowCanvas.height);
            ctx.fillStyle = '#666';
            ctx.font = '14px Arial';
            ctx.fillText('Rendering VexFlow...', 20, 100);
        }
        
        // Fetch all outputs from unified endpoint
        const apiUrl = `/api/parse?input=${encodeURIComponent(input)}`;
        console.log('🔄 Making API request:', { 
            input: input.slice(0, 100) + (input.length > 100 ? '...' : ''),
            url: apiUrl,
            timestamp: new Date().toISOString()
        });
        
        const response = await fetch(apiUrl);
        console.log('📡 API Response received:', {
            status: response.status,
            ok: response.ok,
            headers: Object.fromEntries(response.headers.entries())
        });
        
        const data = await response.json();
        console.log('📋 Parsed API data:', {
            success: data.success,
            hasError: !!data.error,
            error: data.error?.slice(0, 200),
            detectedSystems: data.detected_notation_systems,
            outputsGenerated: {
                pest: !!data.pest_output,
                document: !!data.parsed_document,
                lily: !!data.minimal_lilypond,
                vexflow: !!data.vexflow
            }
        });
        
        if (data.success) {
            console.log('✅ Processing successful API response');
            
            // Update detected notation systems display
            const detectedSystemsSpan = document.getElementById('detected-systems');
            if (data.detected_notation_systems && data.detected_notation_systems.length > 0) {
                detectedSystemsSpan.textContent = data.detected_notation_systems.join(', ');
                console.log('🎵 Updated notation systems display:', data.detected_notation_systems);
            } else {
                detectedSystemsSpan.textContent = 'None detected';
                console.log('⚠️ No notation systems detected');
            }
            
            // PEST Output
            if (data.pest_output) {
                const jsonString = JSON.stringify(data.pest_output, null, 2);
                pestOutput.innerHTML = '<div class="syntax-highlight">' + syntaxHighlight(jsonString) + '</div>';
                pestOutput.className = 'tab-content json-output active';
            } else {
                pestOutput.innerHTML = 'No PEST output available';
                pestOutput.className = 'tab-content json-output loading active';
            }
            
            // Document Structure
            if (data.parsed_document) {
                const docJsonString = JSON.stringify(data.parsed_document, null, 2);
                documentOutput.innerHTML = '<div class="syntax-highlight">' + syntaxHighlight(docJsonString) + '</div>';
                documentOutput.className = 'tab-content json-output';
            } else {
                documentOutput.innerHTML = 'No document structure available';
                documentOutput.className = 'tab-content json-output loading';
            }
            
            // Processed Staves
            if (data.processed_staves) {
                const processedJsonString = JSON.stringify(data.processed_staves, null, 2);
                processedOutput.innerHTML = '<div class="syntax-highlight">' + syntaxHighlight(processedJsonString) + '</div>';
                processedOutput.className = 'tab-content json-output';
            } else {
                processedOutput.innerHTML = 'No processed staves available';
                processedOutput.className = 'tab-content json-output loading';
            }
            
            // Minimal LilyPond
            if (data.minimal_lilypond) {
                minimalLilyOutput.innerHTML = '<pre style="white-space: pre-wrap;">' + data.minimal_lilypond + '</pre>';
                minimalLilyOutput.className = 'tab-content json-output';
            } else {
                minimalLilyOutput.innerHTML = 'No minimal LilyPond available';
                minimalLilyOutput.className = 'tab-content json-output loading';
            }
            
            // Full LilyPond
            if (data.full_lilypond) {
                fullLilyOutput.innerHTML = '<pre style="white-space: pre-wrap;">' + data.full_lilypond + '</pre>';
                fullLilyOutput.className = 'tab-content json-output';
            } else {
                fullLilyOutput.innerHTML = 'No full LilyPond available';
                fullLilyOutput.className = 'tab-content json-output loading';
            }
            
            // SVG Output
            if (data.lilypond_svg) {
                svgOutput.innerHTML = data.lilypond_svg;
                svgOutput.className = 'tab-content';
            } else {
                svgOutput.innerHTML = 'No SVG available';
                svgOutput.className = 'tab-content loading';
            }
            
            // VexFlow - Enhanced rendering with professional features
            if (data.vexflow) {
                console.log('🎼 Rendering enhanced VexFlow output:', {
                    hasVexflowData: !!data.vexflow,
                    staves: data.vexflow.staves?.length,
                    hasAdvancedFeatures: data.vexflow.staves?.some(s => s.notes?.some(n => n.type === 'Tuplet' || n.type === 'SlurStart'))
                });
                
                const vexflowOutput = document.getElementById('vexflow-output');
                
                // Create container for VexFlow rendering
                vexflowOutput.innerHTML = `
                    <div class="vexflow-professional">
                        <div class="text-muted mb-2">Professional VexFlow Rendering with Advanced Features</div>
                        <div id="vexflow-notation" style="width: 100%; min-height: 200px; border: 1px solid #ddd; background: #fafafa;"></div>
                        <div class="mt-2">
                            <button id="toggle-vexflow-data" class="btn btn-sm btn-outline-secondary">Show JSON Data</button>
                        </div>
                        <div id="vexflow-data" class="json-output mt-2" style="display: none; max-height: 300px; overflow-y: auto;"></div>
                    </div>
                `;
                
                // Render with enhanced VexFlow renderer
                if (window.VexFlowRenderer) {
                    window.VexFlowRenderer.renderVexFlowNotation(data.vexflow, 'vexflow-notation')
                        .then(success => {
                            if (success) {
                                console.log('✅ Enhanced VexFlow rendering completed');
                            } else {
                                console.warn('⚠️ VexFlow rendering had issues');
                            }
                        })
                        .catch(error => {
                            console.error('🚨 VexFlow rendering failed:', error);
                            document.getElementById('vexflow-notation').innerHTML = 
                                `<div class="alert alert-warning">VexFlow rendering failed: ${error.message}</div>`;
                        });
                } else {
                    console.warn('⚠️ VexFlowRenderer not available, loading...');
                    // Try to load the renderer and retry
                    loadVexFlowRenderer().then(() => {
                        if (window.VexFlowRenderer) {
                            window.VexFlowRenderer.renderVexFlowNotation(data.vexflow, 'vexflow-notation');
                        }
                    });
                }
                
                // Setup JSON data toggle
                const vexJsonString = JSON.stringify(data.vexflow, null, 2);
                const vexflowDataDiv = document.getElementById('vexflow-data');
                vexflowDataDiv.innerHTML = '<div class="syntax-highlight">' + syntaxHighlight(vexJsonString) + '</div>';
                
                document.getElementById('toggle-vexflow-data').addEventListener('click', function() {
                    const dataDiv = document.getElementById('vexflow-data');
                    const button = this;
                    if (dataDiv.style.display === 'none') {
                        dataDiv.style.display = 'block';
                        button.textContent = 'Hide JSON Data';
                    } else {
                        dataDiv.style.display = 'none';
                        button.textContent = 'Show JSON Data';
                    }
                });
                
            } else {
                console.log('⚠️ No VexFlow data available');
                const vexflowOutput = document.getElementById('vexflow-output');
                vexflowOutput.innerHTML = '<div class="text-muted">No VexFlow data available - check parser output</div>';
            }
            
        } else {
            console.log('❌ API returned error response:', {
                success: data.success,
                error: data.error,
                errorLength: data.error?.length,
                timestamp: new Date().toISOString()
            });
            
            // Show error in notation systems display
            const detectedSystemsSpan = document.getElementById('detected-systems');
            detectedSystemsSpan.textContent = 'Parse error';
            console.log('🔴 Updated notation systems display with: Parse error');
            
            const errorMsg = '<div class="error">Parse Error: ' + (data.error || 'Unknown error') + '</div>';
            pestOutput.innerHTML = errorMsg;
            pestOutput.className = 'tab-content json-output active';
            documentOutput.innerHTML = errorMsg;
            documentOutput.className = 'tab-content json-output';
            processedOutput.innerHTML = errorMsg;
            processedOutput.className = 'tab-content json-output';
            minimalLilyOutput.innerHTML = errorMsg;
            minimalLilyOutput.className = 'tab-content json-output';
            fullLilyOutput.innerHTML = errorMsg;
            fullLilyOutput.className = 'tab-content json-output';
            svgOutput.innerHTML = errorMsg;
            svgOutput.className = 'tab-content';
            if (vexflowData) vexflowData.innerHTML = errorMsg;
            console.log('📄 Updated all output tabs with error message');
        }
        
    } catch (error) {
        console.error('🚨 Network/JavaScript error caught:', {
            message: error.message,
            name: error.name,
            stack: error.stack,
            timestamp: new Date().toISOString()
        });
        
        // Show network error in notation systems display  
        const detectedSystemsSpan = document.getElementById('detected-systems');
        detectedSystemsSpan.textContent = 'Network error';
        console.log('🔴 Updated notation systems display with: Network error');
        
        const errorMsg = '<div class="error">Network Error: ' + error.message + '</div>';
        pestOutput.innerHTML = errorMsg;
        pestOutput.className = 'tab-content json-output active';
        documentOutput.innerHTML = errorMsg;
        documentOutput.className = 'tab-content json-output';
        processedOutput.innerHTML = errorMsg;
        processedOutput.className = 'tab-content json-output';
        minimalLilyOutput.innerHTML = errorMsg;
        minimalLilyOutput.className = 'tab-content json-output';
        fullLilyOutput.innerHTML = errorMsg;
        fullLilyOutput.className = 'tab-content json-output';
        svgOutput.innerHTML = errorMsg;
        svgOutput.className = 'tab-content';
        if (vexflowData) vexflowData.innerHTML = errorMsg;
        console.log('📄 Updated all output tabs with network error message');
    }
}

// Check if SVG tab is currently active
function isSvgTabActive() {
    const svgTabButton = document.getElementById('svg-tab-btn');
    return svgTabButton && svgTabButton.classList.contains('active');
}

// Auto-expand textarea based on content
function autoExpandTextarea(textarea) {
    // Reset height to auto to get the correct scrollHeight
    textarea.style.height = 'auto';
    
    // Calculate the new height based on scrollHeight
    const newHeight = Math.max(60, textarea.scrollHeight); // Min height of 60px (about 3 rows)
    
    // Set the new height
    textarea.style.height = newHeight + 'px';
}

document.getElementById('input-text').addEventListener('input', function(e) {
    const inputValue = e.target.value;
    console.log('⌨️ Input event triggered:', {
        inputLength: inputValue.length,
        firstChars: inputValue.slice(0, 20),
        timestamp: new Date().toISOString(),
        svgTabActive: isSvgTabActive()
    });
    
    // Auto-expand textarea
    autoExpandTextarea(e.target);
    
    // Save input text to localStorage
    saveInputText(inputValue);
    
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
        console.log('⏰ Debounce timer triggered, calling parseInput');
        parseInput(inputValue);
    }, 1000); // Increased debounce to reduce API calls
    
    // If SVG tab is active, also trigger SVG generation with 3-second debounce
    if (isSvgTabActive()) {
        clearTimeout(svgDebounceTimer);
        svgDebounceTimer = setTimeout(() => {
            console.log('🎵 SVG debounce timer triggered, generating SVG automatically');
            generateSvgFromLilypond();
        }, 3000); // 3-second debounce for SVG generation
    }
});

function drawVexFlowPlaceholder(canvas, input) {
    const ctx = canvas.getContext('2d');
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    
    // Background
    ctx.fillStyle = '#fafafa';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    
    // Staff lines
    ctx.strokeStyle = '#333';
    ctx.lineWidth = 1;
    const staffY = 100;
    const staffWidth = 500;
    const staffX = 50;
    
    for (let i = 0; i < 5; i++) {
        ctx.beginPath();
        ctx.moveTo(staffX, staffY + i * 10);
        ctx.lineTo(staffX + staffWidth, staffY + i * 10);
        ctx.stroke();
    }
    
    // Title
    ctx.fillStyle = '#333';
    ctx.font = 'bold 16px serif';
    ctx.fillText('VexFlow Notation (Demo)', 50, 30);
    
    // Input display
    ctx.fillStyle = '#666';
    ctx.font = '11px monospace';
    ctx.fillText('Input: ' + input.substring(0, 50), 50, 50);
    
    // Treble clef (simplified)
    ctx.fillStyle = '#333';
    ctx.font = '30px serif';
    ctx.fillText('𝄞', staffX + 10, staffY + 25);
    
    // Time signature
    ctx.font = '16px serif';
    ctx.fillText('4', staffX + 50, staffY + 10);
    ctx.fillText('4', staffX + 50, staffY + 25);
    
    // Notes based on input
    let noteX = staffX + 80;
    const notes = input.match(/[1-7A-G]/g) || ['1', '2', '3'];
    
    for (let i = 0; i < Math.min(notes.length, 8); i++) {
        const note = notes[i];
        let noteY = staffY + 20; // Default middle line (B)
        
        // Map notes to staff positions
        if (['1', 'C'].includes(note)) noteY = staffY + 50; // C below staff
        else if (['2', 'D'].includes(note)) noteY = staffY + 45; // D
        else if (['3', 'E'].includes(note)) noteY = staffY + 40; // E
        else if (['4', 'F'].includes(note)) noteY = staffY + 35; // F
        else if (['5', 'G'].includes(note)) noteY = staffY + 30; // G
        else if (['6', 'A'].includes(note)) noteY = staffY + 25; // A
        else if (['7', 'B'].includes(note)) noteY = staffY + 20; // B
        
        // Draw note head
        ctx.fillStyle = '#333';
        ctx.beginPath();
        ctx.ellipse(noteX, noteY, 6, 4, 0, 0, 2 * Math.PI);
        ctx.fill();
        
        // Draw stem
        ctx.beginPath();
        ctx.moveTo(noteX + 6, noteY);
        ctx.lineTo(noteX + 6, noteY - 25);
        ctx.lineWidth = 2;
        ctx.stroke();
        ctx.lineWidth = 1;
        
        // Ledger lines if needed
        if (noteY >= staffY + 45) {
            ctx.beginPath();
            ctx.moveTo(noteX - 8, staffY + 50);
            ctx.lineTo(noteX + 14, staffY + 50);
            ctx.stroke();
        }
        
        noteX += 50;
    }
    
    // Bar line
    if (input.includes('|')) {
        ctx.lineWidth = 2;
        ctx.beginPath();
        ctx.moveTo(noteX - 10, staffY);
        ctx.lineTo(noteX - 10, staffY + 40);
        ctx.stroke();
        ctx.lineWidth = 1;
    }
    
    // Footer
    ctx.fillStyle = '#999';
    ctx.font = '10px Arial';
    ctx.fillText('VexFlow-style rendering placeholder', 50, canvas.height - 20);
}

// Load VexFlow renderer dynamically
async function loadVexFlowRenderer() {
    if (window.VexFlowRenderer) return;
    
    try {
        const script = document.createElement('script');
        script.src = 'vexflow-renderer.js';
        script.async = true;
        
        return new Promise((resolve, reject) => {
            script.onload = () => {
                console.log('✅ VexFlow renderer loaded');
                resolve();
            };
            script.onerror = () => {
                console.error('❌ Failed to load VexFlow renderer');
                reject(new Error('Failed to load VexFlow renderer'));
            };
            document.head.appendChild(script);
        });
    } catch (error) {
        console.error('🚨 Error loading VexFlow renderer:', error);
    }
}

// Initialize the application on page load
function initializeApp() {
    // Restore saved input text
    const savedInput = loadInputText();
    const inputElement = document.getElementById('input-text');
    if (savedInput && inputElement) {
        // Set value without triggering input event
        inputElement.value = savedInput;
        // Auto-expand textarea to fit restored content
        autoExpandTextarea(inputElement);
    }
    
    // Restore active tab
    restoreActiveTab();
    
    // Load VexFlow renderer
    loadVexFlowRenderer();
    
    // Only parse if there's actually saved input to avoid unnecessary API calls
    if (savedInput && savedInput.trim()) {
        parseInput(savedInput);
    }
}

// Initialize when DOM is loaded
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initializeApp);
} else {
    // DOM is already loaded
    initializeApp();
}

// SVG Generation function
async function generateSvgFromLilypond() {
    console.log("🎵 generateSvgFromLilypond() called");
    
    // Get notation directly from input field
    const inputField = document.getElementById("input-text");
    if (!inputField || !inputField.value.trim()) {
        alert("Please enter music notation first.");
        return;
    }
    
    const notation = inputField.value.trim();
    
    // Update button state
    const button = document.getElementById("generate-svg-btn");
    const svgContent = document.getElementById("svg-content");
    
    button.disabled = true;
    button.textContent = "Generating...";
    svgContent.innerHTML = "<div class=\"text-muted\">Generating SVG from notation...</div>";
    
    try {
        const response = await fetch("/api/lilypond-svg", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                notation: notation
            })
        });
        
        const result = await response.json();
        
        if (result.success && result.svg_content) {
            svgContent.innerHTML = result.svg_content;
            console.log("✅ SVG generated successfully");
        } else {
            svgContent.innerHTML = `<div class="alert alert-danger">SVG Generation Error: ${result.error || "Unknown error"}</div>`;
            console.error("❌ SVG generation failed:", result.error);
        }
    } catch (error) {
        console.error("🚨 Network error during SVG generation:", error);
        svgContent.innerHTML = `<div class="alert alert-danger">Network Error: ${error.message}</div>`;
    } finally {
        button.disabled = false;
        button.textContent = "Generate SVG";
    }
}
