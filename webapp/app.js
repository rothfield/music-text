let debounceTimer;
const STORAGE_KEYS = {
    INPUT_TEXT: 'music-text-parser-input',
    ACTIVE_TAB: 'music-text-parser-active-tab'
};

function switchTab(tabName) {
    // Remove active class from all tabs
    document.querySelectorAll('.tab-button').forEach(button => {
        button.classList.remove('active');
    });
    document.querySelectorAll('.tab-content').forEach(content => {
        content.classList.remove('active');
    });
    
    // Add active class to clicked tab
    event.target.classList.add('active');
    document.getElementById(tabName + '-tab').classList.add('active');
    
    // Save active tab to localStorage
    saveActiveTab(tabName);
}

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
    
    // Remove active class from all tabs
    document.querySelectorAll('.tab-button').forEach(button => {
        button.classList.remove('active');
    });
    document.querySelectorAll('.tab-content').forEach(content => {
        content.classList.remove('active');
    });
    
    // Find and activate the saved tab
    const tabButton = document.querySelector(`[onclick="switchTab('${savedTab}')"]`);
    const tabContent = document.getElementById(savedTab + '-tab');
    
    if (tabButton && tabContent) {
        tabButton.classList.add('active');
        tabContent.classList.add('active');
    } else {
        // Fallback to first tab if saved tab doesn't exist
        const firstButton = document.querySelector('.tab-button');
        const firstContent = document.querySelector('.tab-content');
        if (firstButton && firstContent) {
            firstButton.classList.add('active');
            firstContent.classList.add('active');
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
    const pestOutput = document.getElementById('pest-output');
    const documentOutput = document.getElementById('document-output');
    const processedOutput = document.getElementById('processed-output');
    const minimalLilyOutput = document.getElementById('minimal-lily-output');
    const fullLilyOutput = document.getElementById('full-lily-output');
    const svgOutput = document.getElementById('svg-output');
    const vexflowCanvas = document.getElementById('vexflow-canvas');
    const vexflowData = document.getElementById('vexflow-data');
    
    if (!input.trim()) {
        pestOutput.innerHTML = '<span class="loading">Type in the textarea above to see the raw PEST parse tree...</span>';
        documentOutput.innerHTML = '<span class="loading">Parsed document structure will appear here...</span>';
        processedOutput.innerHTML = '<span class="loading">Processed staves will appear here...</span>';
        minimalLilyOutput.innerHTML = '<span class="loading">Minimal LilyPond notation will appear here...</span>';
        fullLilyOutput.innerHTML = '<span class="loading">Full LilyPond score will appear here...</span>';
        svgOutput.innerHTML = '<span class="loading">LilyPond SVG rendering will appear here...</span>';
        vexflowData.innerHTML = '<span class="loading">VexFlow notation data will appear here...</span>';
        const ctx = vexflowCanvas.getContext('2d');
        ctx.clearRect(0, 0, vexflowCanvas.width, vexflowCanvas.height);
        ctx.fillStyle = '#fafafa';
        ctx.fillRect(0, 0, vexflowCanvas.width, vexflowCanvas.height);
        ctx.fillStyle = '#666';
        ctx.font = '14px Arial';
        ctx.fillText('VexFlow canvas will render here...', 20, 100);
        return;
    }
    
    try {
        // Set all outputs to loading
        pestOutput.innerHTML = '<span class="loading">Parsing...</span>';
        documentOutput.innerHTML = '<span class="loading">Parsing...</span>';
        processedOutput.innerHTML = '<span class="loading">Processing...</span>';
        minimalLilyOutput.innerHTML = '<span class="loading">Generating...</span>';
        fullLilyOutput.innerHTML = '<span class="loading">Generating...</span>';
        svgOutput.innerHTML = '<span class="loading">Rendering...</span>';
        vexflowData.innerHTML = '<span class="loading">Converting...</span>';
        const ctx = vexflowCanvas.getContext('2d');
        ctx.clearRect(0, 0, vexflowCanvas.width, vexflowCanvas.height);
        ctx.fillStyle = '#fafafa';
        ctx.fillRect(0, 0, vexflowCanvas.width, vexflowCanvas.height);
        ctx.fillStyle = '#666';
        ctx.font = '14px Arial';
        ctx.fillText('Rendering VexFlow...', 20, 100);
        
        // Fetch all outputs from unified endpoint
        const response = await fetch(`/api/parse?input=${encodeURIComponent(input)}`);
        const data = await response.json();
        
        if (data.success) {
            // PEST Output
            if (data.pest_output) {
                const jsonString = JSON.stringify(data.pest_output, null, 2);
                pestOutput.innerHTML = '<div class="syntax-highlight">' + syntaxHighlight(jsonString) + '</div>';
            } else {
                pestOutput.innerHTML = '<span class="loading">No PEST output available</span>';
            }
            
            // Document Structure
            if (data.parsed_document) {
                const docJsonString = JSON.stringify(data.parsed_document, null, 2);
                documentOutput.innerHTML = '<div class="syntax-highlight">' + syntaxHighlight(docJsonString) + '</div>';
            } else {
                documentOutput.innerHTML = '<span class="loading">No document structure available</span>';
            }
            
            // Processed Staves
            if (data.processed_staves) {
                const processedJsonString = JSON.stringify(data.processed_staves, null, 2);
                processedOutput.innerHTML = '<div class="syntax-highlight">' + syntaxHighlight(processedJsonString) + '</div>';
            } else {
                processedOutput.innerHTML = '<span class="loading">No processed staves available</span>';
            }
            
            // Minimal LilyPond
            if (data.minimal_lilypond) {
                minimalLilyOutput.innerHTML = '<pre style="white-space: pre-wrap;">' + data.minimal_lilypond + '</pre>';
            } else {
                minimalLilyOutput.innerHTML = '<span class="loading">No minimal LilyPond available</span>';
            }
            
            // Full LilyPond
            if (data.full_lilypond) {
                fullLilyOutput.innerHTML = '<pre style="white-space: pre-wrap;">' + data.full_lilypond + '</pre>';
            } else {
                fullLilyOutput.innerHTML = '<span class="loading">No full LilyPond available</span>';
            }
            
            // SVG Output
            if (data.lilypond_svg) {
                svgOutput.innerHTML = data.lilypond_svg;
            } else {
                svgOutput.innerHTML = '<span class="loading">No SVG available</span>';
            }
            
            // VexFlow
            if (data.vexflow) {
                // Display the data
                const vexJsonString = JSON.stringify(data.vexflow, null, 2);
                vexflowData.innerHTML = '<div class="syntax-highlight">' + syntaxHighlight(vexJsonString) + '</div>';
                
                // Draw dummy VexFlow rendering on canvas
                drawVexFlowPlaceholder(vexflowCanvas, input);
            } else {
                vexflowData.innerHTML = '<span class="loading">No VexFlow data available</span>';
            }
            
        } else {
            const errorMsg = '<div class="error">Parse Error: ' + (data.error || 'Unknown error') + '</div>';
            pestOutput.innerHTML = errorMsg;
            documentOutput.innerHTML = errorMsg;
            processedOutput.innerHTML = errorMsg;
            minimalLilyOutput.innerHTML = errorMsg;
            fullLilyOutput.innerHTML = errorMsg;
            svgOutput.innerHTML = errorMsg;
            vexflowData.innerHTML = errorMsg;
        }
        
    } catch (error) {
        const errorMsg = '<div class="error">Network Error: ' + error.message + '</div>';
        pestOutput.innerHTML = errorMsg;
        documentOutput.innerHTML = errorMsg;
        processedOutput.innerHTML = errorMsg;
        minimalLilyOutput.innerHTML = errorMsg;
        fullLilyOutput.innerHTML = errorMsg;
        svgOutput.innerHTML = errorMsg;
        vexflowData.innerHTML = errorMsg;
    }
}

document.getElementById('input-text').addEventListener('input', function(e) {
    const inputValue = e.target.value;
    
    // Save input text to localStorage
    saveInputText(inputValue);
    
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
        parseInput(inputValue);
    }, 300);
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

// Initialize the application on page load
function initializeApp() {
    // Restore saved input text
    const savedInput = loadInputText();
    const inputElement = document.getElementById('input-text');
    if (savedInput && inputElement) {
        inputElement.value = savedInput;
    }
    
    // Restore active tab
    restoreActiveTab();
    
    // Parse the restored input (or empty string if no saved input)
    parseInput(savedInput);
}

// Initialize when DOM is loaded
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initializeApp);
} else {
    // DOM is already loaded
    initializeApp();
}