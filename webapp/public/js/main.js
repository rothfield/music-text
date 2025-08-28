import { showStatus, formatJsonAsYaml } from './utils.js';
import { parseNotationApi, generateLilypondPngApi } from './api.js';
import { renderLiveVexFlowPreview } from './vexflow-renderer.js';

let wasm;
let wasmLoaded = false;
let lilypondGenerationTimestamps = [];
let staffPreviewEnabled = true;
let isLiveVexflowEnabled = true;
let isCurrentlyParsing = false;
let parseTimeout;
let liveVexflowTimeout;

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
};

async function loadWasm() {
    try {
        const loadStartTime = performance.now();
        const unique_version = new Date().getTime();
        const wasmModule = await import(`../../pkg/notation_parser.js?v=${unique_version}`);
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
    if (!notation.trim()) {
        elements.detectedSystemDisplay.textContent = '???';
        return;
    }

    if (isCurrentlyParsing) return;
    isCurrentlyParsing = true;
    
    if (showMessages) {
        showStatus(elements.statusContainer, 'Processing your notation...', 'info');
    }

    try {
        const success = wasm.parse_notation(notation);
        
        if (success) {
            const detectedSystem = wasm.get_detected_system();
            elements.detectedSystemDisplay.textContent = detectedSystem;
            
            debouncedLiveVexFlow(notation);
            
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
        
        if (!wasmLoaded) return;
        
        // Use WASM ParseResult to get LilyPond output (same as VexFlow approach)
        const result = wasm.parse_notation(notation);
        if (!result.success) {
            elements.staffNotationImage.style.display = 'none';
            elements.staffNotationPlaceholder.style.display = 'block';
            elements.staffNotationPlaceholder.innerHTML = 'Failed to parse notation';
            return;
        }
        const lilypondCode = result.lilypond_output;
        
        if (!lilypondCode || typeof lilypondCode !== 'string' || !lilypondCode.trim()) {
            elements.staffNotationImage.style.display = 'none';
            elements.staffNotationPlaceholder.style.display = 'block';
            elements.staffNotationPlaceholder.innerHTML = 'No LilyPond output to generate staff notation';
            return;
        }
        
        elements.lilypondSourceSection.style.display = 'block';
        elements.lilypondSourceElement.textContent = lilypondCode;

        lilypondGenerationTimestamps.push(now);

        const pngResult = await generateLilypondPngApi(lilypondCode);
        
        if (pngResult.success && pngResult.imageUrl) {
            elements.staffNotationImage.src = pngResult.imageUrl;
            elements.staffNotationImage.style.display = 'block';
            elements.staffNotationPlaceholder.style.display = 'none';
            showStatus(elements.statusContainer, 'Staff notation generated successfully!', 'success');
            elements.lilypondSourceSection.style.display = 'block';
        } else {
            throw new Error(pngResult.error || 'Failed to generate staff notation');
        }
    } catch (error) {
        console.error('Staff notation generation failed:', error);
        elements.staffNotationImage.style.display = 'none';
        elements.staffNotationPlaceholder.style.display = 'block';
        elements.staffNotationPlaceholder.innerHTML = 'Staff notation generation failed';
        showStatus(elements.statusContainer, `Error: ${error.message}`, 'error');
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
            let lilypondSourceText = 'No LilyPond output available';
            if (wasmLoaded) {
                try {
                    lilypondSourceText = wasm.get_lilypond_output() || lilypondSourceText;
                } catch (e) {
                    lilypondSourceText = 'Error getting LilyPond output';
                }
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
    await loadWasm();
    const initialNotation = getCanvasEditorContent();
    if (initialNotation) {
        await parseNotation(initialNotation, false);
    }
    setupEventListeners();
}

main();
