// Version check for main.js
if (confirm('main.js version: STRUCTURE_PRESERVING_FSM_v2.0 - Continue?')) {
    console.log('Loading main.js STRUCTURE_PRESERVING_FSM_v2.0');
}

import { showStatus, formatJsonAsYaml } from './utils.js';
import { parseNotationApi, generateLilypondSvgApi, checkServerHealth } from './api.js';
import { renderLiveVexFlowPreview } from './vexflow-renderer.js';

let wasm;
let wasmLoaded = false;
let lilypondGenerationTimestamps = [];
let staffPreviewEnabled = true;
let isLiveVexflowEnabled = true;
let isCurrentlyParsing = false;
let parseTimeout;
let liveVexflowTimeout;
let serverStatus = { online: null, lastCheck: 0 };
let serverHealthCheckInterval;

const elements = {
    notationInput: document.getElementById('notation-input'),
    statusContainer: document.getElementById('status-container'),
    generateStaffBtn: document.getElementById('generate-staff-btn'),
    staffNotationSection: document.getElementById('staff-notation-section'),
    staffNotationImage: document.getElementById('staff-notation-image'),
    staffNotationPlaceholder: document.getElementById('staff-notation-placeholder'),
    lilypondSourceSection: document.getElementById('lilypond-source-section'),
    lilypondSourceElement: document.getElementById('lilypond-source'),
    showOutlineBtn: document.getElementById('show-outline-btn'),
    outlineOutputSection: document.getElementById('outline-output-section'),
    outlineOutput: document.getElementById('outline-output'),
    showYamlBtn: document.getElementById('show-yaml-btn'),
    yamlOutputSection: document.getElementById('yaml-output-section'),
    yamlOutput: document.getElementById('yaml-output'),
    showFsmBtn: document.getElementById('show-fsm-btn'),
    showAllDebugBtn: document.getElementById('show-all-debug-btn'),
    fsmDebugSection: document.getElementById('fsm-debug-section'),
    tokenizedDisplay: document.getElementById('tokenized-display'),
    attachedDisplay: document.getElementById('attached-display'),
    fsmBeatsDisplay: document.getElementById('fsm-beats-display'),
    debugLilypondSource: document.getElementById('debug-lilypond-source'),
    fsmYamlDisplay: document.getElementById('fsm-yaml-display'),
    upperOctaveBtn: document.getElementById('upper-octave-btn'),
    middleOctaveBtn: document.getElementById('middle-octave-btn'),
    lowerOctaveBtn: document.getElementById('lower-octave-btn'),
    modeSelect: document.getElementById('mode-select'),
    applyModeBtn: document.getElementById('apply-mode-btn'),
    customModeContainer: document.getElementById('custom-mode-container'),
    customModeInput: document.getElementById('custom-mode-input'),
    constrainModeCheckbox: document.getElementById('constrain-mode-checkbox'),
    livePreviewCheckbox: document.getElementById('live-preview-checkbox'),
    liveVexflowSection: document.getElementById('live-vexflow-section'),
    liveVexflowContainer: document.getElementById('live-vexflow-container'),
    liveVexflowPlaceholder: document.getElementById('live-vexflow-placeholder'),
    liveVexflowNotation: document.getElementById('live-vexflow-notation'),
    staffPreviewToggle: document.getElementById('staff-preview-toggle'),
    versionDisplay: document.getElementById('version-display'),
    timestampDisplay: document.getElementById('timestamp-display'),
    detectedSystemDisplay: document.getElementById('detected-system-display'),
    serverStatusIndicator: document.getElementById('server-status-indicator'),
    serverStatusText: document.getElementById('server-status-text'),
    serverStatusDetails: document.getElementById('server-status-details'),
};

async function loadWasm() {
    // WASM module disabled - using pest parser API instead
    console.log('WASM module disabled - using pest parser API on port 3000');
    wasmLoaded = false;
    window.wasmLoaded = false;
    
    // Set version display for API mode
    if (elements.versionDisplay) {
        elements.versionDisplay.textContent = 'API Mode';
    }
    if (elements.timestampDisplay) {
        elements.timestampDisplay.textContent = '(using pest parser API)';
    }
    
    showStatus(elements.statusContainer, `✅ Using pest parser API mode`, 'success');
    
    // Original WASM loading code commented out since we're using the pest parser API
    /*
    try {
        const loadStartTime = performance.now();
        const unique_version = new Date().getTime();
        const wasmModule = await import(`../../pkg/music_text_parser.js?v=${unique_version}`);
        await wasmModule.default();
        wasm = wasmModule;
        const loadEndTime = performance.now();
        const loadTimeMs = Math.round(loadEndTime - loadStartTime);
        wasmLoaded = true;
        
        // Expose WASM module globally for VexFlow renderer
        window.wasm = wasm;
        window.wasmLoaded = wasmLoaded;
        console.log(`WASM module loaded successfully in ${loadTimeMs}ms`);

        const version = wasm.get_version();
        if (elements.versionDisplay) {
            elements.versionDisplay.textContent = 'v' + version;
        }

        const timestamp = wasm.get_build_timestamp();
        if (elements.timestampDisplay) {
            elements.timestampDisplay.textContent = `(built ${timestamp}, loaded in ${loadTimeMs}ms)`;
        }
        
        showStatus(elements.statusContainer, `✅ WASM module loaded successfully in ${loadTimeMs}ms!`, 'success');
    } catch (error) {
        console.error('Failed to load WASM module:', error);
        showStatus(elements.statusContainer, `❌ Failed to load WASM module: ${error.message}`, 'error');
    }
    */
}

// WebSocket connection for WASM auto-reload during development
// Automatically reloads WASM module when files change on server
function setupAutoReload() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}`;
    
    let ws;
    let reconnectAttempts = 0;
    const maxReconnectAttempts = 5;
    
    function connect() {
        console.log('🔌 Connecting to auto-reload WebSocket...');
        ws = new WebSocket(wsUrl);
        
        ws.onopen = () => {
            console.log('✅ Auto-reload WebSocket connected');
            reconnectAttempts = 0;
            showStatus(elements.statusContainer, '🔄 Auto-reload enabled', 'success');
        };
        
        ws.onmessage = async (event) => {
            try {
                const message = JSON.parse(event.data);
                
                if (message.type === 'wasm-reload') {
                    console.log(`🔄 WASM file changed: ${message.file}, reloading...`);
                    showStatus(elements.statusContainer, `🔄 WASM updated (${message.file}), reloading...`, 'info');
                    
                    // Clear old WASM references
                    wasm = null;
                    wasmLoaded = false;
                    window.wasm = null;
                    window.wasmLoaded = false;
                    
                    // Wait a brief moment for file system to stabilize
                    await new Promise(resolve => setTimeout(resolve, 500));
                    
                    // Reload WASM
                    await loadWasm();
                    
                    // Show success message
                    showStatus(elements.statusContainer, `✅ WASM auto-reloaded successfully!`, 'success');
                }
            } catch (error) {
                console.error('Error handling WebSocket message:', error);
            }
        };
        
        ws.onclose = () => {
            console.log('🔌 Auto-reload WebSocket closed');
            
            if (reconnectAttempts < maxReconnectAttempts) {
                reconnectAttempts++;
                const delay = Math.pow(2, reconnectAttempts) * 1000; // Exponential backoff
                console.log(`🔄 Attempting to reconnect in ${delay}ms (attempt ${reconnectAttempts}/${maxReconnectAttempts})`);
                setTimeout(connect, delay);
            } else {
                console.log('❌ Max reconnection attempts reached, auto-reload disabled');
                showStatus(elements.statusContainer, '❌ Auto-reload disconnected', 'warning');
            }
        };
        
        ws.onerror = (error) => {
            console.error('WebSocket error:', error);
        };
    }
    
    connect();
}

async function updateServerStatus() {
    elements.serverStatusIndicator.className = 'server-status-indicator checking';
    elements.serverStatusText.textContent = 'Checking server...';
    elements.serverStatusDetails.textContent = '';
    
    const health = await checkServerHealth();
    serverStatus.online = health.online;
    serverStatus.lastCheck = Date.now();
    
    if (health.online) {
        elements.serverStatusIndicator.className = 'server-status-indicator online';
        elements.serverStatusText.textContent = '🟢 ' + health.status;
        elements.serverStatusDetails.textContent = health.details;
    } else {
        elements.serverStatusIndicator.className = 'server-status-indicator offline';
        elements.serverStatusText.textContent = '🔴 ' + health.status;
        elements.serverStatusDetails.textContent = health.details;
    }
}

function startServerHealthChecks() {
    // Initial check
    updateServerStatus();
    
    // Check every 30 seconds
    serverHealthCheckInterval = setInterval(updateServerStatus, 30000);
}

function stopServerHealthChecks() {
    if (serverHealthCheckInterval) {
        clearInterval(serverHealthCheckInterval);
        serverHealthCheckInterval = null;
    }
}

function debouncedLiveVexFlow(notation) {
    if (!isLiveVexflowEnabled) return;
    
    clearTimeout(liveVexflowTimeout);
    liveVexflowTimeout = setTimeout(() => {
        const state = { staffPreviewEnabled, isLiveVexflowEnabled };
        const vexflowElements = { 
            liveVexflowPlaceholder: elements.liveVexflowPlaceholder, 
            liveVexflowNotation: elements.liveVexflowNotation 
        };
        renderLiveVexFlowPreview(notation, vexflowElements, state);
    }, 300);
}

async function parseNotation(notation, showMessages = true) {
    const trimmedNotation = notation.trim();
    if (!trimmedNotation) {
        elements.detectedSystemDisplay.textContent = '???';
        return;
    }

    if (isCurrentlyParsing) return;
    isCurrentlyParsing = true;
    
    if (showMessages) {
        showStatus(elements.statusContainer, 'Processing your notation...', 'info');
    }

    try {
        const success = wasm.parse_notation(trimmedNotation);
        
        if (success) {
            const detectedSystem = wasm.get_detected_system();
            elements.detectedSystemDisplay.textContent = detectedSystem;
            
            debouncedLiveVexFlow(trimmedNotation);
            
            if (showMessages) {
                showStatus(elements.statusContainer, '✅ Successfully processed with WASM!', 'success');
            }
        } else {
            const errorMsg = wasm.get_error_message();
            elements.detectedSystemDisplay.textContent = '???';
        }
    } catch (error) {
        console.error('WASM parsing error:', error);
        if (showMessages) {
            showStatus(elements.statusContainer, `❌ Error: ${error.message}`, 'error');
        }
    } finally {
        isCurrentlyParsing = false;
    }
}

function getCanvasEditorContent() {
    return elements.notationInput.value;
}

async function generateStaffNotation() {
    const now = Date.now();
    const oneMinuteAgo = now - 60000;

    lilypondGenerationTimestamps = lilypondGenerationTimestamps.filter(ts => ts > oneMinuteAgo);

    if (lilypondGenerationTimestamps.length >= 3) {
        showStatus(elements.statusContainer, 'Rate limit exceeded (3 times per minute). Please wait.', 'error');
        return;
    }

    elements.staffNotationSection.style.display = 'block';
    elements.generateStaffBtn.disabled = true;
    elements.generateStaffBtn.textContent = 'Processing...';
    
    try {
        const notation = getCanvasEditorContent();
        if (!notation.trim()) {
            elements.staffNotationImage.style.display = 'none';
            elements.staffNotationPlaceholder.style.display = 'block';
            elements.staffNotationPlaceholder.innerHTML = 'No notation to generate staff notation';
            return;
        }
        
        lilypondGenerationTimestamps.push(now);

        // Use pest parser API to generate LilyPond SVG
        const svgResult = await generateLilypondSvgApi(notation, 'auto');
        
        if (svgResult.success && svgResult.svg_url) {
            elements.staffNotationImage.src = svgResult.svg_url;
            elements.staffNotationImage.style.display = 'block';
            elements.staffNotationPlaceholder.style.display = 'none';
            showStatus(elements.statusContainer, 'Staff notation generated successfully!', 'success');
            
            // Show LilyPond source if available
            if (svgResult.lilypond_source) {
                elements.lilypondSourceSection.style.display = 'block';
                elements.lilypondSourceElement.textContent = svgResult.lilypond_source;
            }
        } else {
            throw new Error(svgResult.error || 'Failed to generate staff notation');
        }
    } catch (error) {
        console.error('Staff notation generation failed:', error);
        elements.staffNotationImage.style.display = 'none';
        elements.staffNotationPlaceholder.style.display = 'block';
        
        // Check if this is a server connectivity issue
        const isServerError = error.message.includes('fetch') || 
                             error.message.includes('500') || 
                             error.message.includes('502') || 
                             error.message.includes('503') ||
                             error.message.includes('timeout');
                             
        if (isServerError) {
            elements.staffNotationPlaceholder.innerHTML = `
                <div style="text-align: center; color: #721c24;">
                    <div style="font-size: 18px; margin-bottom: 10px;">⚠️ Server Connection Issue</div>
                    <div style="margin-bottom: 15px;">
                        Unable to generate LilyPond staff notation due to server problems.
                    </div>
                    <div style="background-color: #d1ecf1; color: #0c5460; padding: 15px; border-radius: 4px; margin: 10px 0;">
                        <strong>✅ VexFlow preview still works!</strong><br>
                        The live notation preview above uses client-side rendering and works offline.
                    </div>
                    <div style="font-size: 14px; color: #666; margin-top: 10px;">
                        LilyPond requires server-side processing for high-quality output.<br>
                        Please check your internet connection or try again later.
                    </div>
                </div>
            `;
            showStatus(elements.statusContainer, `🔴 Server unavailable for LilyPond generation`, 'error');
            
            // Trigger a server status update
            updateServerStatus();
        } else {
            elements.staffNotationPlaceholder.innerHTML = `
                <div style="text-align: center; color: #721c24;">
                    <div style="font-size: 18px; margin-bottom: 10px;">❌ Generation Failed</div>
                    <div style="margin-bottom: 15px;">
                        ${error.message}
                    </div>
                    <div style="background-color: #d1ecf1; color: #0c5460; padding: 15px; border-radius: 4px; margin: 10px 0;">
                        <strong>✅ VexFlow preview still works!</strong><br>
                        Check the live notation preview above for instant feedback.
                    </div>
                </div>
            `;
            showStatus(elements.statusContainer, `❌ Error: ${error.message}`, 'error');
        }
    } finally {
        elements.generateStaffBtn.disabled = false;
        elements.generateStaffBtn.textContent = 'Generate LilyPond Staff Notation';
    }
}

async function showOutline() {
    const notation = elements.notationInput.value.trim();
    if (!notation) {
        elements.outlineOutputSection.style.display = 'none';
        return;
    }

    if (!wasmLoaded) {
        showStatus(elements.statusContainer, 'WASM not loaded, please wait.', 'error');
        return;
    }

    try {
        if (wasm.parse_notation(notation)) {
            const outlineCode = wasm.get_outline_output();
            elements.outlineOutput.innerHTML = (outlineCode && outlineCode.trim()) ? outlineCode : 'No outline generated.';
        } else {
            throw new Error(wasm.get_error_message() || 'Failed to parse notation');
        }
    } catch (error) {
        elements.outlineOutput.textContent = `Error: ${error.message}`;
    } finally {
        elements.outlineOutputSection.style.display = 'block';
    }
}

async function showYaml() {
    const notation = elements.notationInput.value.trim();
    if (!notation) {
        elements.yamlOutputSection.style.display = 'none';
        return;
    }

    if (!wasmLoaded) {
        showStatus(elements.statusContainer, 'WASM not loaded, please wait.', 'error');
        return;
    }

    try {
        if (wasm.parse_notation(notation)) {
            const yamlCode = wasm.get_yaml_output();
            elements.yamlOutput.textContent = (yamlCode && yamlCode.trim()) ? yamlCode : 'No YAML generated.';
        } else {
            throw new Error(wasm.get_error_message() || 'Failed to parse notation');
        }
    } catch (error) {
        elements.yamlOutput.textContent = `Error: ${error.message}`;
    } finally {
        elements.yamlOutputSection.style.display = 'block';
    }
}

async function showFsmDebug() {
    const notation = elements.notationInput.value.trim();
    if (!notation) {
        elements.fsmDebugSection.style.display = 'none';
        localStorage.setItem('fsmDebugVisible', 'false');
        return;
    }

    try {
        const result = await parseNotationApi(notation);
        
        if (result.success && result.vexflowFsm) {
            const yamlOutput = formatJsonAsYaml(result.vexflowFsm);
            const fsmBeatsOutput = result.documentOutline ? result.documentOutline.trim() : 'No FSM document structure available.';
            const tokenizedOutput = result.tokenizedData ? formatJsonAsYaml(result.tokenizedData) : 'No tokenized data available.';
            const attachedOutput = result.attachedItemsData ? result.attachedItemsData.trim() : 'No attached items data available.';
            // Use minimal LilyPond source from API result
            let lilypondSourceText = 'No LilyPond output available';
            if (result.lilypond) {
                // Extract minimal source from full LilyPond output
                lilypondSourceText = extractMinimalLilyPond(result.lilypond);
            }
            
            elements.tokenizedDisplay.textContent = tokenizedOutput;
            elements.attachedDisplay.textContent = attachedOutput;
            elements.fsmBeatsDisplay.textContent = fsmBeatsOutput;
            elements.debugLilypondSource.textContent = lilypondSourceText;
            elements.fsmYamlDisplay.textContent = yamlOutput || 'No VexFlow FSM data available.';
            elements.fsmDebugSection.style.display = 'block';
            elements.showFsmBtn.textContent = 'Hide VexFlow JSON';
            localStorage.setItem('fsmDebugVisible', 'true');
        } else {
            throw new Error(result.error || 'Failed to get VexFlow FSM data');
        }
    } catch (error) {
        elements.fsmYamlDisplay.textContent = `Error: ${error.message}`;
        elements.fsmDebugSection.style.display = 'block';
    }
}

function toggleFsmDebug() {
    const isVisible = elements.fsmDebugSection.style.display !== 'none';
    if (isVisible) {
        elements.fsmDebugSection.style.display = 'none';
        elements.showFsmBtn.textContent = 'Show Debug Panels';
        localStorage.setItem('fsmDebugVisible', 'false');
    } else {
        showFsmDebug();
    }
}

function toggleAllDebugPanels() {
    const debugSections = [elements.fsmDebugSection, elements.outlineOutputSection, elements.yamlOutputSection];
    const isAnyVisible = debugSections.some(section => section.style.display !== 'none');
    
    const showAll = !isAnyVisible;
    debugSections.forEach(section => section.style.display = showAll ? 'block' : 'none');
    
    localStorage.setItem('allDebugVisible', showAll);
    localStorage.setItem('fsmDebugVisible', showAll);
    localStorage.setItem('outlineVisible', showAll);
    localStorage.setItem('yamlVisible', showAll);

    if (showAll && elements.notationInput.value.trim()) {
        showFsmDebug();
        showOutline();
        showYaml();
    }
}

function debouncedParse() {
    clearTimeout(parseTimeout);
    parseTimeout = setTimeout(() => {
        const notation = elements.notationInput.value;
        parseNotation(notation, false);
        saveToLocalStorage(notation);
    }, 500);
}

function saveToLocalStorage(notation) {
    try {
        const settings = {
            notation: notation,
            mode: elements.modeSelect.value,
            customMode: elements.customModeInput.value,
            constrain: elements.constrainModeCheckbox.checked,
            livePreview: elements.livePreviewCheckbox.checked,
            staffPreview: staffPreviewEnabled
        };
        localStorage.setItem('notationSettings', JSON.stringify(settings));
    } catch (e) {
        console.error("Failed to save to local storage:", e);
    }
}

function loadFromLocalStorage() {
    try {
        const savedSettings = localStorage.getItem('notationSettings');
        if (savedSettings) {
            const settings = JSON.parse(savedSettings);
            elements.notationInput.value = settings.notation || '';
            elements.modeSelect.value = settings.mode || 'ionian';
            elements.customModeInput.value = settings.customMode || '';
            elements.constrainModeCheckbox.checked = settings.constrain || false;
            elements.livePreviewCheckbox.checked = settings.livePreview !== undefined ? settings.livePreview : true;
            staffPreviewEnabled = settings.staffPreview !== undefined ? settings.staffPreview : true;
            elements.staffPreviewToggle.checked = staffPreviewEnabled;
            elements.modeSelect.dispatchEvent(new Event('change'));
        } else {
            elements.livePreviewCheckbox.checked = true;
            staffPreviewEnabled = true;
            elements.staffPreviewToggle.checked = true;
        }
        restoreDebugPanelState();
    } catch (e) {
        console.error("Failed to load from local storage:", e);
        elements.livePreviewCheckbox.checked = true;
        restoreDebugPanelState();
    }
}

function restoreDebugPanelState() {
    const fsmDebugVisible = true;
    const outlineVisible = true;
    const yamlVisible = true;
    
    elements.fsmDebugSection.style.display = fsmDebugVisible ? 'block' : 'none';
    elements.outlineOutputSection.style.display = outlineVisible ? 'block' : 'none';
    elements.yamlOutputSection.style.display = yamlVisible ? 'block' : 'none';
    
    localStorage.setItem('fsmDebugVisible', 'true');
    localStorage.setItem('outlineVisible', 'true');
    localStorage.setItem('yamlVisible', 'true');
    localStorage.setItem('allDebugVisible', 'true');
}

function handleOctaveChange(octaveType) {
    console.log('Octave change not yet implemented:', octaveType);
}

function applyMode() {
    console.log('Mode application not yet implemented');
}

function setupEventListeners() {
    elements.notationInput.addEventListener('input', debouncedParse);
    elements.generateStaffBtn.addEventListener('click', generateStaffNotation);
    elements.showOutlineBtn.addEventListener('click', showOutline);
    elements.showYamlBtn.addEventListener('click', showYaml);
    elements.showFsmBtn.addEventListener('click', toggleFsmDebug);
    elements.showAllDebugBtn.addEventListener('click', toggleAllDebugPanels);

    elements.staffPreviewToggle.addEventListener('change', function() {
        staffPreviewEnabled = this.checked;
        elements.liveVexflowSection.style.display = staffPreviewEnabled ? 'block' : 'none';
        if (staffPreviewEnabled) {
            const notation = getCanvasEditorContent();
            if (notation) renderLiveVexFlowPreview(notation, { liveVexflowPlaceholder: elements.liveVexflowPlaceholder, liveVexflowNotation: elements.liveVexflowNotation }, { staffPreviewEnabled, isLiveVexflowEnabled });
        } else {
            clearTimeout(liveVexflowTimeout);
        }
        saveToLocalStorage(getCanvasEditorContent());
    });

    elements.livePreviewCheckbox.addEventListener('change', function() {
        isLiveVexflowEnabled = this.checked;
        elements.liveVexflowSection.style.display = isLiveVexflowEnabled ? 'block' : 'none';
        if (isLiveVexflowEnabled) {
            const notation = getCanvasEditorContent();
            if (notation) renderLiveVexFlowPreview(notation, { liveVexflowPlaceholder: elements.liveVexflowPlaceholder, liveVexflowNotation: elements.liveVexflowNotation }, { staffPreviewEnabled, isLiveVexflowEnabled });
        } else {
            clearTimeout(liveVexflowTimeout);
        }
        saveToLocalStorage(getCanvasEditorContent());
    });

    elements.upperOctaveBtn.addEventListener('click', () => handleOctaveChange('upper'));
    elements.middleOctaveBtn.addEventListener('click', () => handleOctaveChange('middle'));
    elements.lowerOctaveBtn.addEventListener('click', () => handleOctaveChange('lower'));
    elements.applyModeBtn.addEventListener('click', applyMode);

    elements.modeSelect.addEventListener('change', () => {
        elements.customModeContainer.style.display = elements.modeSelect.value === 'custom' ? 'flex' : 'none';
    });
}

async function main() {
    loadFromLocalStorage();
    startServerHealthChecks(); // Start monitoring server status
    await loadWasm();
    const initialNotation = getCanvasEditorContent();
    if (initialNotation) {
        await parseNotation(initialNotation, false);
    }
    setupEventListeners();
    setupAutoReload(); // Enable WASM auto-reload during development
}

// Extract minimal LilyPond content (same logic as server-side)
function extractMinimalLilyPond(fullSource) {
    const startMarker = "\\relative c' {";
    const startIndex = fullSource.indexOf(startMarker);
    
    if (startIndex === -1) {
        return "% Unable to extract musical content";
    }
    
    const contentStart = startIndex + startMarker.length;
    const endIndex = fullSource.indexOf("    }", contentStart);
    
    if (endIndex === -1) {
        return "% Unable to extract musical content";
    }
    
    const musicalContent = fullSource.substring(contentStart, endIndex);
    
    // Clean up the content - remove settings and empty lines
    const lines = musicalContent
        .split('\n')
        .map(line => line.trim())
        .filter(line => 
            line.length > 0 && 
            !line.startsWith('\\key') && 
            !line.startsWith('\\time') &&
            !line.startsWith('\\autoBeamOff') &&
            !line.startsWith('\\set')
        );
    
    if (lines.length === 0) {
        return "% No musical content";
    }
    
    return lines.join(' ');
}

main();
