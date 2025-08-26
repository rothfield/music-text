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
    const { Renderer, Stave, Formatter, Voice, Beam, Tuplet, StaveTie, Curve, Accidental, BarNote } = Vex.Flow;
    
    liveVexflowNotation.innerHTML = '';
    liveVexflowPlaceholder.style.display = 'none';
    liveVexflowNotation.style.display = 'block';
    
    const renderer = new Renderer(liveVexflowNotation, Renderer.Backends.SVG);
    const estimatedHeight = Math.max(150, staves.length * 100);
    renderer.resize(800, estimatedHeight);
    const context = renderer.getContext();
    context.scale(0.7, 0.7);
    
    let currentY = 20;
    
    staves.forEach((staveData, staveIndex) => {
        if (!staveData.notes || staveData.notes.length === 0) return;
        
        const stave = new Stave(20, currentY, 1000);
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
                if (element.type === 'Tuplet') {
                    const tupletNotes = element.notes.map(noteData => {
                        const isRest = noteData.type === 'Rest' || (noteData.keys && noteData.keys.length > 0 && noteData.keys[0].startsWith('-'));
                        let note;
                        if (isRest) {
                            note = new Vex.Flow.StaveNote({ clef: 'treble', keys: ['b/4'], duration: noteData.duration + 'r' + (noteData.dots > 0 ? 'd'.repeat(noteData.dots) : '') });
                        } else {
                            note = new Vex.Flow.StaveNote({ clef: 'treble', keys: noteData.keys, duration: noteData.duration + (noteData.dots > 0 ? 'd'.repeat(noteData.dots) : '') });
                        }
                        if (!isRest && noteData.accidentals) {
                            noteData.accidentals.forEach(accData => {
                                if (accData.accidental && accData.accidental !== 'n') {
                                    note.addModifier(new Accidental(accData.accidental), accData.index);
                                }
                            });
                        }
                        if (noteData.dots > 0) {
                            for (let i = 0; i < noteData.dots; i++) {
                                note.addDot(0);
                            }
                        }
                        return note;
                    });
                    vexNotes.push(...tupletNotes);
                    noteOnlyArray.push(...tupletNotes);
                    if (tupletNotes.length >= 2) {
                        const hasRests = tupletNotes.some(note => note.getDuration().includes('r'));
                        const firstNoteDuration = tupletNotes[0].getDuration();
                        if (!hasRests && (firstNoteDuration === '8' || firstNoteDuration === '16' || firstNoteDuration === '32')) {
                            beamGroups.push(tupletNotes);
                        }
                    }
                    const noteCount = tupletNotes.length;
                    let totalDurationEighths = 0;
                    tupletNotes.forEach(note => {
                        const duration = note.getDuration();
                        if (duration === 'w' || duration === '1') totalDurationEighths += 8;
                        else if (duration === 'h' || duration === '2') totalDurationEighths += 4;
                        else if (duration === 'q' || duration === '4') totalDurationEighths += 2;
                        else if (duration === '8') totalDurationEighths += 1;
                        else if (duration === '16') totalDurationEighths += 0.5;
                        else if (duration === '32') totalDurationEighths += 0.25;
                        else totalDurationEighths += 2;
                    });
                    let tupletOptions = null;
                    if (totalDurationEighths === 3 && noteCount === 2) tupletOptions = { notes_occupied: 2, num_notes: 3 };
                    else if (noteCount === 3 && totalDurationEighths === 3) tupletOptions = { notes_occupied: 2, num_notes: 3 };
                    else if (noteCount === 3 && totalDurationEighths === 6) tupletOptions = { notes_occupied: 2, num_notes: 3 };
                    else if (noteCount === 5) tupletOptions = { notes_occupied: 4, num_notes: 5 };
                    else if (noteCount === 6) tupletOptions = { notes_occupied: 4, num_notes: 6 };
                    else if (noteCount === 7) tupletOptions = { notes_occupied: 4, num_notes: 7 };
                    else {
                        const isPowerOfTwo = (noteCount & (noteCount - 1)) === 0;
                        if (!isPowerOfTwo && noteCount > 2) {
                            tupletOptions = { notes_occupied: Math.max(2, noteCount - 1), num_notes: noteCount };
                        }
                    }
                    tupletGroups.push({ notes: tupletNotes, options: tupletOptions });
                } else if (element.type === 'Note') {
                    const note = new Vex.Flow.StaveNote({ clef: 'treble', keys: element.keys, duration: element.duration + (element.dots > 0 ? 'd'.repeat(element.dots) : '') });
                    if (element.accidentals) {
                        element.accidentals.forEach(acc => {
                            if (acc.accidental && acc.accidental !== 'n') {
                                note.addModifier(new Accidental(acc.accidental), acc.index);
                            }
                        });
                    }
                    for (let i = 0; i < element.dots; i++) note.addDot(0);
                    vexNotes.push(note);
                    noteOnlyArray.push(note);
                    if (!element.type || element.type !== 'Rest') {
                        if (element.beam_start || element.beam_end || (element.duration === '8' || element.duration === '16')) {
                            if (element.beam_start) currentBeamGroup = [note];
                            else if (element.beam_end && currentBeamGroup.length > 0) {
                                currentBeamGroup.push(note);
                                if (currentBeamGroup.length >= 2) beamGroups.push([...currentBeamGroup]);
                                currentBeamGroup = [];
                            } else if (currentBeamGroup.length > 0) currentBeamGroup.push(note);
                        }
                    }
                } else if (element.type === 'Rest') {
                    const rest = new Vex.Flow.StaveNote({ clef: 'treble', keys: ['b/4'], duration: element.duration + 'r' + (element.dots > 0 ? 'd'.repeat(element.dots) : '') });
                    for (let i = 0; i < element.dots; i++) rest.addDot(0);
                    vexNotes.push(rest);
                    if (currentBeamGroup.length >= 2) beamGroups.push([...currentBeamGroup]);
                    currentBeamGroup = [];
                } else if (element.type === 'BarLine') {
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
        
        new Formatter().joinVoices([voice]).format([voice], 225);
        voice.draw(context, stave);
        beams.forEach(beam => beam.draw());
        tuplets.forEach(tuplet => tuplet.draw());
        ties.forEach(tie => tie.draw());
        curves.forEach(curve => curve.draw());
        
        currentY += 100;
    });
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
        const result = await parseNotationApi(notation);
        
        if (!result.success || !result.vexflowFsm || !Array.isArray(result.vexflowFsm)) {
            throw new Error('Invalid API response');
        }
        
        renderVexFlowFromFSM(result.vexflowFsm, { liveVexflowNotation, liveVexflowPlaceholder });
        
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
