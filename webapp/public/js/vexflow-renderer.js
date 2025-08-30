import { parseNotationApi } from './api.js';

let vexflowLoaded = false;

async function loadVexFlow() {
    if (vexflowLoaded) return true;
    
    try {
        console.log('Loading VexFlow library...');
        const script = document.createElement('script');
        script.src = 'lib/vexflow.js';
        script.async = true;
        
        return new Promise((resolve, reject) => {
            script.onload = () => {
                vexflowLoaded = true;
                console.log('✅ VexFlow library loaded successfully');
                resolve(true);
            };
            script.onerror = () => {
                console.error('❌ Failed to load VexFlow library');
                reject(new Error('Failed to load VexFlow'));
            };
            document.head.appendChild(script);
        });
    } catch (error) {
        console.error('VexFlow loading error:', error);
        return false;
    }
}

function renderVexFlowFromFSM(staves, { liveVexflowNotation, liveVexflowPlaceholder }) {
    console.log('🚀 renderVexFlowFromFSM called with 2000px width configuration');
    const { Renderer, Stave, Formatter, Voice, Beam, Tuplet, StaveTie, Curve, Accidental, BarNote } = Vex.Flow;
    
    liveVexflowNotation.innerHTML = '';
    liveVexflowPlaceholder.style.display = 'none';
    liveVexflowNotation.style.display = 'block';
    
    const renderer = new Renderer(liveVexflowNotation, Renderer.Backends.SVG);
    // Since we're scaling by 0.5, we need to double the logical height
    const estimatedHeight = Math.max(300, staves.length * 200);
    
    // Set fixed width to 2000px
    const renderWidth = 2000;
    
    renderer.resize(renderWidth, estimatedHeight);
    const context = renderer.getContext();
    
    // Scale everything to half size
    context.scale(0.5, 0.5);
    
    let currentY = 20;
    
    staves.forEach((staveData, staveIndex) => {
        console.log(`Processing stave ${staveIndex}:`, staveData);
        console.log(`Notes array:`, staveData.notes);
        if (!staveData.notes || staveData.notes.length === 0) {
            console.log(`Skipping stave ${staveIndex} - no notes`);
            return;
        }
        
        // Since we're scaling by 0.5, we need to double the logical dimensions
        const staveWidth = (renderWidth - 40) * 2; // Double the width to account for 0.5 scale
        const stave = new Stave(40, currentY * 2, staveWidth); // Double positions for 0.5 scale
        if (staveIndex === 0) {
            stave.addClef('treble');
            
            if (staveData.key_signature) {
                const keyMap = {
                    'C': 'C', 'G': 'G', 'D': 'D', 'A': 'A', 'E': 'E', 'B': 'B', 'F#': 'F#', 'C#': 'C#',
                    'F': 'F', 'Bb': 'Bb', 'Eb': 'Eb', 'Ab': 'Ab', 'Db': 'Db', 'Gb': 'Gb', 'Cb': 'Cb',
                    'c': 'C', 'g': 'G', 'd': 'D', 'a': 'A', 'e': 'E', 'b': 'B', 'f#': 'F#', 'c#': 'C#',
                    'f': 'F', 'bb': 'Bb', 'eb': 'Eb', 'ab': 'Ab', 'db': 'Db', 'gb': 'Gb'
                };
                const vexflowKey = keyMap[staveData.key_signature] || 'C';
                stave.addKeySignature(vexflowKey);
            }
        }
        stave.setContext(context).draw();
        
        const vexNotes = [];
        const noteOnlyArray = [];
        const beamGroups = [];
        const tupletGroups = [];
        let currentBeamGroup = [];
        
        staveData.notes.forEach((element) => {
            try {
                console.log('Processing element:', element);
                
                // Handle Rust enum JSON format: {Tuplet: {...}} vs {type: 'Tuplet', ...}
                const elementType = element.type || (element.Tuplet ? 'Tuplet' : element.Note ? 'Note' : 'Unknown');
                const elementData = element.type ? element : (element.Tuplet || element.Note || element);
                
                console.log('Element type:', elementType);
                console.log('Element data:', elementData);
                
                if (elementType === 'Tuplet') {
                    console.log('Processing tuplet with notes:', elementData.notes);
                    const tupletNotes = elementData.notes.map((noteData, index) => {
                        console.log(`Tuplet note ${index}:`, noteData);
                        
                        // Handle Rust enum format for tuplet notes too
                        const noteType = noteData.type || (noteData.Note ? 'Note' : noteData.Rest ? 'Rest' : 'Unknown');
                        const actualNoteData = noteData.type ? noteData : (noteData.Note || noteData.Rest || noteData);
                        
                        console.log(`Tuplet note type: ${noteType}, data:`, actualNoteData);
                        
                        const isRest = noteType === 'Rest' || (actualNoteData.keys && actualNoteData.keys.length > 0 && actualNoteData.keys[0].startsWith('-'));
                        let note;
                        if (isRest) {
                            note = new Vex.Flow.StaveNote({ clef: 'treble', keys: ['b/4'], duration: actualNoteData.duration + 'r' });
                        } else {
                            note = new Vex.Flow.StaveNote({ clef: 'treble', keys: actualNoteData.keys, duration: actualNoteData.duration });
                        }
                        if (!isRest && actualNoteData.accidentals) {
                            actualNoteData.accidentals.forEach(accData => {
                                if (accData.accidental && accData.accidental !== 'n') {
                                    note.addModifier(new Accidental(accData.accidental), accData.index);
                                }
                            });
                        }
                        // Add dots using Dot class if present
                        if (actualNoteData.dots && actualNoteData.dots > 0) {
                            const Dot = Vex.Flow.Dot;
                            for (let i = 0; i < actualNoteData.dots; i++) {
                                note.addModifier(new Dot(), 0);
                            }
                        }
                        return note;
                    });
                    vexNotes.push(...tupletNotes);
                    noteOnlyArray.push(...tupletNotes);
                    // Auto-beam tuplet notes if they are beamable durations
                    if (tupletNotes.length >= 2) {
                        const hasRests = tupletNotes.some(note => note.getDuration().includes('r'));
                        const beamableDurations = tupletNotes.every(note => {
                            const duration = note.getDuration().replace('r', '');
                            return ['8', '16', '32', '64'].includes(duration);
                        });
                        if (!hasRests && beamableDurations) {
                            beamGroups.push(tupletNotes);
                        }
                    }
                    // Trust the FSM - it should always provide correct tuplet ratios
                    if (!elementData.ratio || elementData.ratio.length !== 2) {
                        console.error('FSM tuplet missing or invalid ratio field:', elementData);
                        return; // Skip malformed tuplets
                    }
                    
                    const [num_notes, notes_occupied] = elementData.ratio;
                    const tupletOptions = { notes_occupied, num_notes };
                    console.log(`Using FSM ratio [${num_notes}, ${notes_occupied}] for tuplet options:`, tupletOptions);
                    tupletGroups.push({ notes: tupletNotes, options: tupletOptions });
                } else if (elementType === 'Note') {
                    // Don't include dots in duration string for VexFlow 4.2.2
                    const note = new Vex.Flow.StaveNote({ clef: 'treble', keys: elementData.keys, duration: elementData.duration });
                    if (elementData.accidentals) {
                        elementData.accidentals.forEach(acc => {
                            if (acc.accidental && acc.accidental !== 'n') {
                                note.addModifier(new Accidental(acc.accidental), acc.index);
                            }
                        });
                    }
                    // Add dots using Dot class if present
                    if (elementData.dots && elementData.dots > 0) {
                        const Dot = Vex.Flow.Dot;
                        for (let i = 0; i < elementData.dots; i++) {
                            note.addModifier(new Dot(), 0);
                        }
                    }
                    vexNotes.push(note);
                    noteOnlyArray.push(note);
                    if (elementType !== 'Rest') {
                        // Handle explicit beam markers first
                        if (elementData.beam_start) {
                            // Finalize any previous beam group
                            if (currentBeamGroup.length >= 2) beamGroups.push([...currentBeamGroup]);
                            currentBeamGroup = [note];
                        } else if (elementData.beam_end) {
                            if (currentBeamGroup.length > 0) {
                                currentBeamGroup.push(note);
                                if (currentBeamGroup.length >= 2) beamGroups.push([...currentBeamGroup]);
                            }
                            currentBeamGroup = [];
                        } else {
                            // Auto-beam logic for beamable durations
                            const isBeamable = ['8', '16', '32', '64'].includes(elementData.duration);
                            if (isBeamable) {
                                currentBeamGroup.push(note);
                            } else {
                                // Non-beamable note, finalize any current beam group
                                if (currentBeamGroup.length >= 2) beamGroups.push([...currentBeamGroup]);
                                currentBeamGroup = [];
                            }
                        }
                    }
                } else if (elementType === 'Rest') {
                    const rest = new Vex.Flow.StaveNote({ clef: 'treble', keys: ['b/4'], duration: elementData.duration + 'r' });
                    // Add dots using Dot class if present
                    if (elementData.dots && elementData.dots > 0) {
                        const Dot = Vex.Flow.Dot;
                        for (let i = 0; i < elementData.dots; i++) {
                            rest.addModifier(new Dot(), 0);
                        }
                    }
                    vexNotes.push(rest);
                    if (currentBeamGroup.length >= 2) beamGroups.push([...currentBeamGroup]);
                    currentBeamGroup = [];
                } else if (elementType === 'BarLine') {
                    if (currentBeamGroup.length >= 2) beamGroups.push([...currentBeamGroup]);
                    currentBeamGroup = [];
                    vexNotes.push(new BarNote());
                }
            } catch (error) {
                console.error('VexFlow element processing error:', error, 'Problem element:', element);
            }
        });
        
        if (currentBeamGroup.length >= 2) beamGroups.push(currentBeamGroup);
        if (vexNotes.length === 0) return;
        
        const voice = new Voice({ num_beats: 4, beat_value: 4, resolution: Vex.Flow.RESOLUTION }).setStrict(false);
        voice.addTickables(vexNotes);
        
        const beams = beamGroups.filter(group => group.length >= 2).map(group => new Beam(group).setContext(context));
        const tuplets = tupletGroups.filter(group => group.options !== null).map(group => new Tuplet(group.notes, group.options).setContext(context));
        
        const ties = [];
        let vexNoteIndex = 0;
        staveData.notes.forEach((element, elementIndex) => {
            if (element.type === 'Note') {
                if (element.tied) {
                    let nextNoteIndex = vexNoteIndex + 1;
                    let searchIndex = elementIndex + 1;
                    while (searchIndex < staveData.notes.length) {
                        const nextElement = staveData.notes[searchIndex];
                        if (nextElement.type === 'Note') {
                            ties.push(new StaveTie({ first_note: vexNotes[vexNoteIndex], last_note: vexNotes[nextNoteIndex], first_indices: [0], last_indices: [0] }).setContext(context));
                            break;
                        } else if (nextElement.type === 'Tuplet') {
                            if (nextElement.notes.length > 0 && nextElement.notes[0].type === 'Note') {
                                ties.push(new StaveTie({ first_note: vexNotes[vexNoteIndex], last_note: vexNotes[nextNoteIndex], first_indices: [0], last_indices: [0] }).setContext(context));
                                break;
                            }
                            nextNoteIndex += nextElement.notes.filter(n => n.type === 'Note').length;
                        } else if (nextElement.type === 'Rest') {
                            nextNoteIndex++;
                        }
                        searchIndex++;
                    }
                }
                vexNoteIndex++;
            } else if (element.type === 'Tuplet') {
                element.notes.forEach((note, noteIndex) => {
                    if (note.tied) {
                        let nextNoteIndex = vexNoteIndex + 1;
                        if (noteIndex < element.notes.length - 1) {
                            ties.push(new StaveTie({ first_note: vexNotes[vexNoteIndex], last_note: vexNotes[nextNoteIndex], first_indices: [0], last_indices: [0] }).setContext(context));
                        } else {
                            let searchIndex = elementIndex + 1;
                            while (searchIndex < staveData.notes.length) {
                                const nextElement = staveData.notes[searchIndex];
                                if (nextElement.type === 'Note' || (nextElement.type === 'Tuplet' && nextElement.notes.length > 0)) {
                                    ties.push(new StaveTie({ first_note: vexNotes[vexNoteIndex], last_note: vexNotes[nextNoteIndex], first_indices: [0], last_indices: [0] }).setContext(context));
                                    break;
                                }
                                searchIndex++;
                            }
                        }
                    }
                    vexNoteIndex++;
                });
            } else if (element.type === 'Rest') {
                vexNoteIndex++;
            }
        });

        const curves = [];
        let slurStartNoteIndex = -1;
        let noteOnlyIndex = 0;
        staveData.notes.forEach((element) => {
            if (element.type === 'SlurStart') {
                slurStartNoteIndex = noteOnlyIndex;
            } else if (element.type === 'SlurEnd' && slurStartNoteIndex >= 0) {
                const endNoteIndex = noteOnlyIndex - 1;
                if (endNoteIndex > slurStartNoteIndex && endNoteIndex < noteOnlyArray.length) {
                    const startNote = noteOnlyArray[slurStartNoteIndex];
                    const endNote = noteOnlyArray[endNoteIndex];
                    let canCreateCurve = true;
                    try { startNote.getStem(); } catch (e) { canCreateCurve = false; }
                    try { if (!endNote.getStem()) { if (typeof endNote.buildStem === 'function') endNote.buildStem(); if (!endNote.getStem()) canCreateCurve = false; } } catch (e) { canCreateCurve = false; }
                    if (canCreateCurve) {
                        curves.push(new Curve(startNote, endNote, { cps: [{ x: 0, y: 10 }, { x: 0, y: 10 }] }).setContext(context));
                    }
                }
                slurStartNoteIndex = -1;
            } else if (element.type === 'Note') {
                noteOnlyIndex++;
            } else if (element.type === 'Tuplet') {
                noteOnlyIndex += element.notes.filter(n => n.type === 'Note').length;
            }
        });
        
        const formatterWidth = staveWidth - 200; // Double margins for 0.5 scale
        new Formatter().joinVoices([voice]).format([voice], formatterWidth);
        voice.draw(context, stave);
        beams.forEach(beam => beam.draw());
        tuplets.forEach(tuplet => tuplet.draw());
        ties.forEach(tie => tie.draw());
        curves.forEach(curve => curve.draw());
        
        currentY += 200; // Double the increment for 0.5 scale
    });
    
    // After all rendering is complete, force SVG to be 2000px wide
    setTimeout(() => {
        const svg = liveVexflowNotation.querySelector('svg');
        if (svg) {
            svg.setAttribute('width', '2000');
            svg.setAttribute('height', estimatedHeight.toString());
            svg.setAttribute('viewBox', `0 0 2000 ${estimatedHeight}`);
            console.log(`🔧 Final SVG dimensions: width=${svg.getAttribute('width')}, viewBox=${svg.getAttribute('viewBox')}`);
        }
    }, 50);
}

export async function renderLiveVexFlowPreview(notation, { liveVexflowPlaceholder, liveVexflowNotation }, { staffPreviewEnabled, isLiveVexflowEnabled }) {
    console.log('🎼 Starting VexFlow render');
    
    if (!staffPreviewEnabled) {
        liveVexflowPlaceholder.innerHTML = `
            <div style="text-align: center; padding: 20px; color: #666;">
                🎼 Staff notation preview is disabled<br>
                <small>Enable it with the toggle above for faster startup</small>
            </div>
        `;
        liveVexflowPlaceholder.style.display = 'block';
        liveVexflowNotation.style.display = 'none';
        return;
    }
    
    if (!vexflowLoaded) {
        liveVexflowPlaceholder.innerHTML = `
            <div style="text-align: center; padding: 20px;">
                <div>📦 Loading VexFlow library...</div>
            </div>
        `;
        liveVexflowPlaceholder.style.display = 'block';
        liveVexflowNotation.style.display = 'none';
        
        try {
            await loadVexFlow();
        } catch (error) {
            liveVexflowPlaceholder.innerHTML = `
                <div style="text-align: center; padding: 20px; color: #ff6b6b;">
                    ❌ Failed to load VexFlow: ${error.message}
                </div>
            `;
            return;
        }
    }
    
    if (!isLiveVexflowEnabled || !notation.trim() || !window.Vex) {
        liveVexflowPlaceholder.style.display = 'block';
        liveVexflowNotation.style.display = 'none';
        return;
    }

    liveVexflowPlaceholder.innerHTML = `
        <div style="text-align: center; padding: 20px;">
            <div>🎼 Rendering VexFlow...</div>
            <div style="font-size: 0.8em; color: #666; margin-top: 5px;">
                Build: Aug 2 19:20 (triplet detection + proper beaming)
            </div>
        </div>
    `;
    liveVexflowPlaceholder.style.display = 'block';
    liveVexflowNotation.style.display = 'none';

    try {
        // Use WASM module directly instead of server API
        if (!window.wasm || !window.wasmLoaded) {
            throw new Error('WASM module not loaded');
        }
        
        const result = window.wasm.parse_notation(notation);
        
        if (!result.success) {
            throw new Error(result.error_message || 'Parsing failed');
        }
        
        const vexflowJsonStr = result.vexflow_output;
        if (!vexflowJsonStr || vexflowJsonStr === '[]') {
            throw new Error('No VexFlow output generated');
        }
        
        const vexflowData = JSON.parse(vexflowJsonStr);
        console.log('VexFlow JSON data:', vexflowData);
        
        if (!Array.isArray(vexflowData)) {
            throw new Error('Invalid VexFlow data format');
        }
        
        console.log('VexFlow data length:', vexflowData.length);
        if (vexflowData.length > 0) {
            console.log('First stave:', vexflowData[0]);
            if (vexflowData[0].notes && vexflowData[0].notes.length > 0) {
                console.log('First note:', vexflowData[0].notes[0]);
                console.log('All notes:', vexflowData[0].notes);
            }
        }
        
        renderVexFlowFromFSM(vexflowData, { liveVexflowNotation, liveVexflowPlaceholder });
        
    } catch (error) {
        console.error('VexFlow preview error:', error);
        liveVexflowPlaceholder.innerHTML = `
            <div style="text-align: center; padding: 20px; color: #ff6b6b;">
                <div>⚠️ VexFlow render failed: ${error.message}</div>
                <div style="font-size: 0.8em; color: #666; margin-top: 5px;">
                    Build: Aug 2 19:20 (triplet detection + proper beaming)
                </div>
            </div>
        `;
        liveVexflowPlaceholder.style.display = 'block';
        liveVexflowNotation.style.display = 'none';
    }
}
