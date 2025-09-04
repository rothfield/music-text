/**
 * VexFlow Renderer Module
 * Handles VexFlow library loading and music notation rendering
 */

import { showVexFlowPlaceholder, showVexFlowNotation, clearVexFlowContainer } from './ui.js';

let vexflowLoaded = false;

// Global scaling factor for consistent VexFlow rendering
const VEXFLOW_SCALE_FACTOR = 0.7;

/**
 * Load VexFlow library dynamically
 */
export async function loadVexFlow() {
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

/**
 * Check if VexFlow is loaded and available
 */
export function isVexFlowLoaded() {
    return vexflowLoaded && window.Vex && window.Vex.Flow;
}

/**
 * Render VexFlow notation from FSM data
 */
export function renderVexFlowFromFSM(staves) {
    console.log('🎵 renderVexFlowFromFSM called with:', staves);
    if (!isVexFlowLoaded()) {
        throw new Error('VexFlow not loaded');
    }
    
    const { Renderer, Stave, Formatter, Voice, Beam } = Vex.Flow;
    
    // Clear and prepare container
    clearVexFlowContainer();
    showVexFlowNotation();
    
    const notation = document.getElementById('live-vexflow-notation');
    
    // Create renderer with proper canvas size
    const renderer = new Renderer(notation, Renderer.Backends.SVG);
    const estimatedHeight = Math.max(150, staves.length * 100);
    renderer.resize(2000, estimatedHeight);
    const context = renderer.getContext();
    context.scale(VEXFLOW_SCALE_FACTOR, VEXFLOW_SCALE_FACTOR);
    
    let currentY = 20;
    
    // Process each stave
    staves.forEach((staveData, staveIndex) => {
        if (!staveData.notes || staveData.notes.length === 0) return;
        
        // Create stave
        const stave = new Stave(20, currentY, 1000);
        if (staveIndex === 0) {
            stave.addClef('treble');
            
            // Add key signature if present
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
        
        // Check if the last element is a barline and set it as the end barline
        if (staveData.notes.length > 0) {
            const lastElement = staveData.notes[staveData.notes.length - 1];
            if (lastElement.type === 'BarLine') {
                let vexflowType;
                switch (lastElement.bar_type) {
                    case 'repeat-begin':
                        vexflowType = Vex.Flow.Barline.type.REPEAT_BEGIN;
                        break;
                    case 'repeat-end':
                        vexflowType = Vex.Flow.Barline.type.REPEAT_END;
                        break;
                    case 'double':
                        vexflowType = Vex.Flow.Barline.type.DOUBLE;
                        break;
                    case 'final':
                        vexflowType = Vex.Flow.Barline.type.END;
                        break;
                    case 'double-repeat':
                        vexflowType = Vex.Flow.Barline.type.REPEAT_BOTH;
                        break;
                    case 'single':
                    default:
                        vexflowType = Vex.Flow.Barline.type.SINGLE;
                        break;
                }
                
                stave.setEndBarType(vexflowType);
                console.log('🎵 Set stave end barline:', lastElement.bar_type, vexflowType);
                console.log('Stave end_barline after setting:', stave.end_barline);
            }
        }
        
        // Check if the first element is a barline and set it as the begin barline
        if (staveData.notes.length > 0) {
            const firstElement = staveData.notes[0];
            if (firstElement.type === 'BarLine') {
                let vexflowType;
                switch (firstElement.bar_type) {
                    case 'repeat-begin':
                        vexflowType = Vex.Flow.Barline.type.REPEAT_BEGIN;
                        break;
                    case 'repeat-end':
                        vexflowType = Vex.Flow.Barline.type.REPEAT_END;
                        break;
                    case 'double':
                        vexflowType = Vex.Flow.Barline.type.DOUBLE;
                        break;
                    case 'final':
                        vexflowType = Vex.Flow.Barline.type.END;
                        break;
                    case 'double-repeat':
                        vexflowType = Vex.Flow.Barline.type.REPEAT_BOTH;
                        break;
                    case 'single':
                    default:
                        vexflowType = Vex.Flow.Barline.type.SINGLE;
                        break;
                }
                
                stave.setBegBarType(vexflowType);
                console.log('🎵 Set stave begin barline:', firstElement.bar_type, vexflowType);
            }
        }
        
        stave.setContext(context);
        
        // Force redraw after setting barlines
        stave.draw();
        
        // If we have an end barline, draw it manually
        if (staveData.notes.length > 0) {
            const lastElement = staveData.notes[staveData.notes.length - 1];
            if (lastElement.type === 'BarLine') {
                console.log('🎵 Manual barline draw attempt');
                // Get precise staff line positions from VexFlow
                const x = stave.getX() + stave.getWidth() - 1; // Move slightly inside
                const topLine = stave.getTopLineTopY();
                const bottomLine = stave.getBottomLineBottomY();
                const staffHeight = bottomLine - topLine;
                
                // Draw professional-looking barlines
                const drawBarline = (x, topY, bottomY, type) => {
                    context.save();
                    context.strokeStyle = '#000000';
                    context.lineCap = 'butt';
                    
                    console.log('🎵 Drawing barline type:', type);
                    
                    switch (type) {
                        case 'repeat-end':
                            // Repeat end: :|
                            // Thin line
                            context.lineWidth = 1;
                            context.beginPath();
                            context.moveTo(x - 6, topY);
                            context.lineTo(x - 6, bottomY);
                            context.stroke();
                            
                            // Thick line
                            context.lineWidth = 3;
                            context.beginPath();
                            context.moveTo(x - 1, topY);
                            context.lineTo(x - 1, bottomY);
                            context.stroke();
                            
                            // Dots (positioned in staff spaces)
                            context.fillStyle = '#000000';
                            const staffSpacing = (bottomY - topY) / 4;
                            const dotSize = 1.5;
                            const dotX = x - 10;
                            // Position dots in 2nd and 3rd staff spaces (between lines)
                            context.beginPath();
                            context.arc(dotX, topY + staffSpacing * 1.5, dotSize, 0, 2 * Math.PI);
                            context.fill();
                            context.beginPath();
                            context.arc(dotX, topY + staffSpacing * 2.5, dotSize, 0, 2 * Math.PI);
                            context.fill();
                            break;
                            
                        case 'repeat-begin':
                            // Repeat begin: |:
                            // Thick line
                            context.lineWidth = 3;
                            context.beginPath();
                            context.moveTo(x + 1, topY);
                            context.lineTo(x + 1, bottomY);
                            context.stroke();
                            
                            // Thin line
                            context.lineWidth = 1;
                            context.beginPath();
                            context.moveTo(x + 6, topY);
                            context.lineTo(x + 6, bottomY);
                            context.stroke();
                            
                            // Dots
                            context.fillStyle = '#000000';
                            const staffSpacing2 = (bottomY - topY) / 4;
                            const dotSize2 = 1.5;
                            const dotX2 = x + 10;
                            context.beginPath();
                            context.arc(dotX2, topY + staffSpacing2 * 1.5, dotSize2, 0, 2 * Math.PI);
                            context.fill();
                            context.beginPath();
                            context.arc(dotX2, topY + staffSpacing2 * 2.5, dotSize2, 0, 2 * Math.PI);
                            context.fill();
                            break;
                            
                        case 'double':
                            // Double barline: ||
                            context.lineWidth = 1;
                            context.beginPath();
                            context.moveTo(x - 3, topY);
                            context.lineTo(x - 3, bottomY);
                            context.stroke();
                            
                            context.beginPath();
                            context.moveTo(x - 1, topY);
                            context.lineTo(x - 1, bottomY);
                            context.stroke();
                            break;
                            
                        case 'final':
                            // Final barline: |‖
                            context.lineWidth = 1;
                            context.beginPath();
                            context.moveTo(x - 5, topY);
                            context.lineTo(x - 5, bottomY);
                            context.stroke();
                            
                            context.lineWidth = 3;
                            context.beginPath();
                            context.moveTo(x - 1, topY);
                            context.lineTo(x - 1, bottomY);
                            context.stroke();
                            break;
                            
                        case 'single':
                        default:
                            // Single barline: |
                            context.lineWidth = 1;
                            context.beginPath();
                            context.moveTo(x, topY);
                            context.lineTo(x, bottomY);
                            context.stroke();
                            break;
                    }
                    
                    context.restore();
                };
                
                drawBarline(x, topLine, bottomLine, lastElement.bar_type);
                console.log('🎵 Drew professional barline:', lastElement.bar_type, 'at', x);
            }
        }
        
        // Convert FSM elements to VexFlow notes
        const vexNotes = [];
        const beamGroups = [];
        const tupletGroups = [];
        const slurGroups = [];
        const pendingMordents = []; // Track mordents to attach to notes
        const elementToVexNoteMap = new Map(); // Map FSM element index to VexFlow note indices
        let currentBeamGroup = [];
        let slurStartIndex = null;
        
        staveData.notes.forEach((element, index) => {
            console.log('🎵 Processing element:', element.type, element);
            try {
                if (element.type === 'Tuplet') {
                    const startVexIndex = vexNotes.length;
                    const { notes: tupletNotes, beamGroup, tupletOptions, slurInfo } = processTuplet(element);
                    vexNotes.push(...tupletNotes);
                    
                    // Map this FSM element index to the first VexFlow note it created
                    elementToVexNoteMap.set(index, startVexIndex);
                    
                    if (beamGroup && beamGroup.length >= 2) {
                        beamGroups.push(beamGroup);
                    }
                    
                    if (tupletOptions) {
                        console.log('🎵 Adding tuplet with options:', tupletOptions, 'notes:', tupletNotes.length);
                        tupletGroups.push({ notes: tupletNotes, options: tupletOptions });
                    } else {
                        console.log('⚠️ No tuplet options generated for divisions:', element.divisions, 'noteCount:', tupletNotes.length);
                    }
                    
                    // Handle slurs within tuplet
                    if (slurInfo) {
                        if (slurInfo.start !== null && slurInfo.end !== null) {
                            const slurStartNote = tupletNotes[slurInfo.start];
                            const slurEndNote = tupletNotes[slurInfo.end];
                            if (slurStartNote && slurEndNote) {
                                slurs.push({ from: slurStartNote, to: slurEndNote });
                                console.log('🎵 Added slur within tuplet from note', slurInfo.start, 'to note', slurInfo.end);
                            }
                        }
                    }
                    
                } else if (element.type === 'Note') {
                    const vexIndex = vexNotes.length;
                    const note = createVexFlowNote(element);
                    vexNotes.push(note);
                    
                    // Map this FSM element index to its VexFlow note index
                    elementToVexNoteMap.set(index, vexIndex);
                    
                    handleBeaming(element, note, currentBeamGroup, beamGroups);
                    
                } else if (element.type === 'Rest') {
                    const rest = createVexFlowRest(element);
                    vexNotes.push(rest);
                    // Rests break beam groups
                    if (currentBeamGroup.length >= 2) {
                        beamGroups.push([...currentBeamGroup]);
                    }
                    currentBeamGroup = [];
                    
                } else if (element.type === 'BarLine') {
                    // Handle barlines - break beaming
                    if (currentBeamGroup.length >= 2) {
                        beamGroups.push([...currentBeamGroup]);
                    }
                    currentBeamGroup = [];
                    
                    // Store mid-measure barlines for manual drawing later
                    if (!staveData.midBarlines) {
                        staveData.midBarlines = [];
                    }
                    staveData.midBarlines.push({
                        type: element.bar_type,
                        afterNoteIndex: vexNotes.length - 1 // Draw after the last note added
                    });
                    console.log('🎵 Stored mid-measure barline:', element.bar_type, 'after note index', vexNotes.length - 1);
                    
                } else if (element.type === 'SlurStart') {
                    // Mark the start of a slur at the next note index
                    slurStartIndex = vexNotes.length;
                    console.log('🎵 Slur start marked at note index:', slurStartIndex);
                    
                } else if (element.type === 'SlurEnd') {
                    // Create a slur from the marked start to the previous note
                    // Only create slur if there are at least 2 notes (meaningful slur)
                    if (slurStartIndex !== null && vexNotes.length > slurStartIndex + 1) {
                        const slurEndIndex = vexNotes.length - 1;
                        const noteCount = slurEndIndex - slurStartIndex + 1;
                        
                        console.log('🎵 Slur spans', noteCount, 'notes from index', slurStartIndex, 'to', slurEndIndex);
                        
                        slurGroups.push({
                            startIndex: slurStartIndex,
                            endIndex: slurEndIndex
                        });
                        console.log('🎵 Valid slur created from index', slurStartIndex, 'to', slurEndIndex);
                    } else if (slurStartIndex !== null) {
                        console.log('🎵 Skipping single-note slur at index', slurStartIndex, '- slurs need at least 2 notes');
                    }
                    slurStartIndex = null;
                    
                } else if (element.type === 'Mordent') {
                    // Store mordent to attach to the note at the specified index
                    console.log('🎵 Found mordent:', element.mordent_type, 'for note index', element.note_index);
                    pendingMordents.push(element);
                }
            } catch (error) {
                console.error('VexFlow element processing error:', error);
                console.error('Problem element:', element);
            }
        });
        
        // Don't forget the last beam group
        if (currentBeamGroup.length >= 2) {
            beamGroups.push(currentBeamGroup);
        }
        
        if (vexNotes.length === 0) return;
        
        // Apply mordents to their target notes using the element mapping
        pendingMordents.forEach(mordent => {
            // Find the VexFlow note index for this FSM element
            const vexNoteIndex = elementToVexNoteMap.get(mordent.note_index);
            const targetNote = vexNotes[vexNoteIndex];
            
            console.log('🎵 Mordent debug - FSM element:', mordent.note_index, '-> VexFlow index:', vexNoteIndex);
            console.log('🎵 Target note:', targetNote);
            console.log('🎵 Target note constructor:', targetNote?.constructor?.name);
            console.log('🎵 Target note type check:', typeof targetNote);
            console.log('🎵 Has addModifier method:', typeof targetNote?.addModifier);
            
            // Check if it's a note that can accept modifiers (more flexible check)
            if (targetNote && typeof targetNote.addModifier === 'function' && 
                !targetNote.getDuration().includes('r')) { // Not a rest
                
                console.log('🎵 Applying', mordent.mordent_type, 'mordent to FSM element', mordent.note_index, '-> VexFlow note', vexNoteIndex);
                
                try {
                    // Create VexFlow ornament
                    const ornamentType = mordent.mordent_type === 'upper' ? 'mordent' : 'mordent_inverted';
                    const ornament = new Vex.Flow.Ornament(ornamentType);
                    
                    // Check available methods on ornament
                    console.log('🎵 Available ornament methods:', Object.getOwnPropertyNames(ornament.__proto__));
                    
                    // Use VexFlow's built-in positioning and sizing
                    if (typeof ornament.setPosition === 'function') {
                        ornament.setPosition(Vex.Flow.Modifier.Position.ABOVE);
                    }
                    
                    targetNote.addModifier(ornament, 0);
                    console.log('🎵 Successfully applied mordent');
                } catch (error) {
                    console.error('🎵 Error applying mordent:', error);
                    console.log('🎵 Ornament object:', ornament);
                }
            } else {
                console.warn('🎵 Could not apply mordent - invalid target note. FSM element:', mordent.note_index, '-> VexFlow index:', vexNoteIndex, 'Note type:', targetNote?.constructor?.name, 'Has addModifier:', typeof targetNote?.addModifier);
            }
        });
        
        // Create voice and add notes
        const voice = new Voice({
            num_beats: 4,
            beat_value: 4,
            resolution: Vex.Flow.RESOLUTION
        }).setStrict(false);
        
        voice.addTickables(vexNotes);
        
        // Create beams
        const beams = beamGroups
            .filter(group => group.length >= 2)
            .map(group => {
                const beam = new Beam(group);
                beam.setContext(context);
                return beam;
            });
        
        // Create tuplets
        const tuplets = tupletGroups
            .filter(group => group.options !== null)
            .map(group => {
                console.log('🎵 Creating VexFlow tuplet:', group.options, 'with', group.notes.length, 'notes');
                const tuplet = new Vex.Flow.Tuplet(group.notes, group.options);
                tuplet.setContext(context);
                return tuplet;
            });
            
        console.log('🎵 Total tuplets to draw:', tuplets.length);
        
        // Create ties - check each note element for tied property
        const ties = [];
        
        for (let i = 0; i < vexNotes.length - 1; i++) {
            // Find the FSM element index that corresponds to this VexFlow note
            let fsmElementIndex = -1;
            let noteIndexWithinElement = 0;
            let vexNoteCounter = 0;
            
            for (let j = 0; j < staveData.notes.length; j++) {
                const element = staveData.notes[j];
                
                if (element.type === 'Note') {
                    if (vexNoteCounter === i) {
                        fsmElementIndex = j;
                        noteIndexWithinElement = 0;
                        break;
                    }
                    vexNoteCounter++;
                } else if (element.type === 'Rest') {
                    if (vexNoteCounter === i) {
                        fsmElementIndex = j;
                        noteIndexWithinElement = 0;
                        break;
                    }
                    vexNoteCounter++;
                } else if (element.type === 'Tuplet' && element.notes) {
                    const tupletNoteCount = element.notes.filter(n => n.type === 'Note' || n.type === 'Rest').length;
                    if (vexNoteCounter <= i && i < vexNoteCounter + tupletNoteCount) {
                        fsmElementIndex = j;
                        noteIndexWithinElement = i - vexNoteCounter;
                        break;
                    }
                    vexNoteCounter += tupletNoteCount;
                }
            }
            
            // Check if current note should be tied to next
            let shouldTie = false;
            
            if (fsmElementIndex >= 0) {
                const currentElement = staveData.notes[fsmElementIndex];
                
                if (currentElement.type === 'Note' && currentElement.tied) {
                    shouldTie = true;
                } else if (currentElement.type === 'Tuplet' && currentElement.notes) {
                    const noteElements = currentElement.notes.filter(n => n.type === 'Note' || n.type === 'Rest');
                    if (noteElements[noteIndexWithinElement] && noteElements[noteIndexWithinElement].tied) {
                        shouldTie = true;
                    }
                }
            }
            
            // Create the tie if needed
            if (shouldTie && i + 1 < vexNotes.length) {
                const firstNote = vexNotes[i];
                const secondNote = vexNotes[i + 1];
                
                if (firstNote && secondNote && 
                    !firstNote.getDuration().includes('r') && 
                    !secondNote.getDuration().includes('r')) {
                    console.log('🎵 Creating tie from vexNote', i, 'to', i + 1);
                    const tie = new Vex.Flow.StaveTie({
                        first_note: firstNote,
                        last_note: secondNote,
                        first_indices: [0],
                        last_indices: [0]
                    });
                    tie.setContext(context);
                    ties.push(tie);
                }
            }
        }
        
        // Create slurs - only for valid note pairs
        const slurs = slurGroups
            .filter(slurInfo => {
                const startNote = vexNotes[slurInfo.startIndex];
                const endNote = vexNotes[slurInfo.endIndex];
                
                // Use flexible note checking (same approach as mordents)
                const isValidStartNote = startNote && 
                    typeof startNote.addModifier === 'function' && 
                    !startNote.getDuration().includes('r'); // Not a rest
                    
                const isValidEndNote = endNote && 
                    typeof endNote.addModifier === 'function' && 
                    !endNote.getDuration().includes('r'); // Not a rest
                
                const isValidSlur = isValidStartNote && isValidEndNote && 
                                  slurInfo.startIndex !== slurInfo.endIndex;
                
                if (!isValidSlur) {
                    console.log('🎵 Filtering out invalid slur:', slurInfo, 
                               'startNote type:', startNote?.constructor?.name,
                               'endNote type:', endNote?.constructor?.name,
                               'startNote valid:', isValidStartNote,
                               'endNote valid:', isValidEndNote);
                }
                
                return isValidSlur;
            })
            .map(slurInfo => {
                const startNote = vexNotes[slurInfo.startIndex];
                const endNote = vexNotes[slurInfo.endIndex];
                
                console.log('🎵 Creating valid slur from note', slurInfo.startIndex, 'to', slurInfo.endIndex);
                
                const curve = new Vex.Flow.Curve(
                    startNote,    // First note
                    endNote,      // Last note
                    {
                        cps: [     // Control points for the curve
                            { x: 0, y: 10 },   // First control point (relative)
                            { x: 0, y: 10 }    // Second control point (relative)
                        ]
                    }
                );
                curve.setContext(context);
                return curve;
            });
        
        // Format and draw
        new Formatter().joinVoices([voice]).format([voice], 225);
        voice.draw(context, stave);
        beams.forEach(beam => beam.draw());
        tuplets.forEach(tuplet => tuplet.draw());
        ties.forEach(tie => tie.draw());
        slurs.forEach(slur => slur.draw());
        
        // Draw mid-measure barlines
        if (staveData.midBarlines && staveData.midBarlines.length > 0) {
            staveData.midBarlines.forEach(barlineInfo => {
                if (barlineInfo.afterNoteIndex >= 0 && barlineInfo.afterNoteIndex < vexNotes.length) {
                    const note = vexNotes[barlineInfo.afterNoteIndex];
                    if (note && note.getAbsoluteX) {
                        // Position barline after the current note with proper spacing
                        const currentNote = note;
                        const noteWidth = currentNote.getWidth();
                        // Move barlines further to the right with more spacing
                        const x = currentNote.getAbsoluteX() + noteWidth * 2.5;
                        const topY = stave.getTopLineTopY();
                        const bottomY = stave.getBottomLineBottomY();
                        
                        // Use the same professional barline drawing function for mid-measure barlines
                        const drawMidBarline = (x, topY, bottomY, type) => {
                            context.save();
                            context.strokeStyle = '#000000';
                            context.lineCap = 'butt';
                            
                            console.log('🎵 Drawing mid-measure barline type:', type);
                            
                            switch (type) {
                                case 'repeat-end':
                                    // Repeat end: :|
                                    // Thin line
                                    context.lineWidth = 1;
                                    context.beginPath();
                                    context.moveTo(x - 8, topY);
                                    context.lineTo(x - 8, bottomY);
                                    context.stroke();
                                    
                                    // Thick line
                                    context.lineWidth = 3;
                                    context.beginPath();
                                    context.moveTo(x - 2, topY);
                                    context.lineTo(x - 2, bottomY);
                                    context.stroke();
                                    
                                    // Dots (positioned in staff spaces)
                                    context.fillStyle = '#000000';
                                    const staffSpacing = (bottomY - topY) / 4;
                                    const dotSize = 1.5;
                                    const dotX = x - 13;
                                    // Position dots in 2nd and 3rd staff spaces (between lines)
                                    context.beginPath();
                                    context.arc(dotX, topY + staffSpacing * 1.5, dotSize, 0, 2 * Math.PI);
                                    context.fill();
                                    context.beginPath();
                                    context.arc(dotX, topY + staffSpacing * 2.5, dotSize, 0, 2 * Math.PI);
                                    context.fill();
                                    break;
                                    
                                case 'repeat-begin':
                                    // Repeat begin: |:
                                    // Thick line
                                    context.lineWidth = 3;
                                    context.beginPath();
                                    context.moveTo(x + 2, topY);
                                    context.lineTo(x + 2, bottomY);
                                    context.stroke();
                                    
                                    // Thin line
                                    context.lineWidth = 1;
                                    context.beginPath();
                                    context.moveTo(x + 8, topY);
                                    context.lineTo(x + 8, bottomY);
                                    context.stroke();
                                    
                                    // Dots
                                    context.fillStyle = '#000000';
                                    const staffSpacing2 = (bottomY - topY) / 4;
                                    const dotSize2 = 1.5;
                                    const dotX2 = x + 13;
                                    context.beginPath();
                                    context.arc(dotX2, topY + staffSpacing2 * 1.5, dotSize2, 0, 2 * Math.PI);
                                    context.fill();
                                    context.beginPath();
                                    context.arc(dotX2, topY + staffSpacing2 * 2.5, dotSize2, 0, 2 * Math.PI);
                                    context.fill();
                                    break;
                                    
                                case 'single':
                                default:
                                    // Single barline: |
                                    context.lineWidth = 1;
                                    context.beginPath();
                                    context.moveTo(x, topY);
                                    context.lineTo(x, bottomY);
                                    context.stroke();
                                    break;
                            }
                            
                            context.restore();
                        };
                        
                        drawMidBarline(x, topY, bottomY, barlineInfo.type);
                        console.log('🎵 Drew mid-measure barline at x:', x);
                    }
                }
            });
        }
        
        
        currentY += 100;
    });
}

/**
 * Create a VexFlow note from FSM element
 */
function createVexFlowNote(element) {
    console.log('🎵 Creating VexFlow note:', element);
    const note = new Vex.Flow.StaveNote({
        clef: 'treble',
        keys: element.keys,
        duration: element.duration
    });
    console.log('🎵 Created note, adding', element.dots, 'dots');
    
    // Add accidentals
    if (element.accidentals) {
        element.accidentals.forEach(acc => {
            if (acc.accidental && acc.accidental !== 'n') {
                note.addModifier(new Vex.Flow.Accidental(acc.accidental), acc.index);
            }
        });
    }
    
    // Add dots
    for (let i = 0; i < element.dots; i++) {
        note.addModifier(new Vex.Flow.Dot(), 0);
    }
    
    // Add lyrics/syllable with SOP styling (small, italic)
    if (element.syl) {
        console.log('🎵 Adding lyric:', element.syl);
        const annotation = new Vex.Flow.Annotation(element.syl);
        annotation.setVerticalJustification(Vex.Flow.Annotation.VerticalJustify.BOTTOM);
        try {
            annotation.setJustification(Vex.Flow.Annotation.Justify.CENTER);
        } catch (e) {
            console.warn('VexFlow CENTER justification not available:', e);
        }
        // Try simpler font setting approach
        try {
            annotation.setFontSize(10);
        } catch (e) {
            console.warn('VexFlow setFontSize not available:', e);
        }
        try {
            annotation.setFont('Arial', 10, 'italic');
        } catch (e) {
            console.warn('VexFlow setFont not available:', e);
        }
        note.addModifier(annotation, 0);
    }
    
    // Add ornaments (mordents, trills, etc.)
    console.log('🎵 ORNAMENT DEBUG: Processing element:', JSON.stringify(element, null, 2));
    console.log('🎵 ORNAMENT DEBUG: element.ornaments =', element.ornaments);
    if (element.ornaments && element.ornaments.length > 0) {
        console.log('🎵 ORNAMENT DEBUG: Found', element.ornaments.length, 'ornaments to add');
        element.ornaments.forEach(ornamentType => {
            console.log('🎵 Adding ornament:', ornamentType);
            
            try {
                // Convert ornament type to VexFlow ornament name
                let vexflowOrnamentType;
                switch (ornamentType) {
                    case 'Mordent':
                        vexflowOrnamentType = 'mordent';
                        break;
                    case 'Trill':
                        vexflowOrnamentType = 'trill';
                        break;
                    case 'Turn':
                        vexflowOrnamentType = 'turn';
                        break;
                    default:
                        console.warn('🎵 Unknown ornament type:', ornamentType);
                        return;
                }
                
                // Create VexFlow ornament
                const ornament = new Vex.Flow.Ornament(vexflowOrnamentType);
                
                // Use VexFlow's built-in positioning and explicit sizing
                if (typeof ornament.setPosition === 'function') {
                    ornament.setPosition(Vex.Flow.Modifier.Position.ABOVE);
                }
                
                // Scale ornament to match global scaling
                if (typeof ornament.setFontScale === 'function') {
                    ornament.setFontScale(VEXFLOW_SCALE_FACTOR);
                } else if (typeof ornament.font_scale !== 'undefined') {
                    ornament.font_scale = VEXFLOW_SCALE_FACTOR;
                }
                
                note.addModifier(ornament, 0);
                console.log('🎵 Successfully added', ornamentType, 'ornament');
                
            } catch (error) {
                console.error('🎵 Error adding ornament:', ornamentType, error);
            }
        });
    }
    
    return note;
}

/**
 * Create a VexFlow rest from FSM element
 */
function createVexFlowRest(element) {
    const rest = new Vex.Flow.StaveNote({
        clef: 'treble',
        keys: ['b/4'],
        duration: element.duration + 'r'
    });
    
    // Add dots to rest
    for (let i = 0; i < element.dots; i++) {
        rest.addModifier(new Vex.Flow.Dot(), 0);
    }
    
    return rest;
}

/**
 * Process tuplet element and return VexFlow notes
 */
function processTuplet(element) {
    // Filter out non-note elements (SlurStart, SlurEnd, etc.) and only process Note and Rest types
    const noteElements = element.notes.filter(noteData => 
        noteData.type === 'Note' || noteData.type === 'Rest'
    );
    
    const tupletNotes = noteElements.map(noteData => {
        const isRest = noteData.type === 'Rest' || 
                      (noteData.keys && noteData.keys.length > 0 && noteData.keys[0].startsWith('-'));
        
        let note;
        if (isRest) {
            note = new Vex.Flow.StaveNote({
                clef: 'treble',
                keys: ['b/4'],
                duration: noteData.duration + 'r'
            });
        } else {
            note = new Vex.Flow.StaveNote({
                clef: 'treble',
                keys: noteData.keys,
                duration: noteData.duration
            });
        }
        
        // Add accidentals (skip for rests)
        if (!isRest && noteData.accidentals) {
            noteData.accidentals.forEach(accData => {
                if (accData.accidental && accData.accidental !== 'n') {
                    note.addModifier(new Vex.Flow.Accidental(accData.accidental), accData.index);
                }
            });
        }
        
        // Add dots
        if (noteData.dots > 0) {
            for (let i = 0; i < noteData.dots; i++) {
                note.addModifier(new Vex.Flow.Dot(), 0);
            }
        }
        
        // Add lyrics/syllable (for non-rests) with SOP styling (small, italic)
        if (!isRest && noteData.syl) {
            console.log('🎵 Adding tuplet lyric:', noteData.syl);
            const annotation = new Vex.Flow.Annotation(noteData.syl);
            annotation.setVerticalJustification(Vex.Flow.Annotation.VerticalJustify.BOTTOM);
            try {
                annotation.setJustification(Vex.Flow.Annotation.Justify.CENTER);
            } catch (e) {
                console.warn('VexFlow CENTER justification not available:', e);
            }
            // Try simpler font setting approach
            try {
                annotation.setFontSize(10);
            } catch (e) {
                console.warn('VexFlow setFontSize not available:', e);
            }
            try {
                annotation.setFont('Arial', 10, 'italic');
            } catch (e) {
                console.warn('VexFlow setFont not available:', e);
            }
            note.addModifier(annotation, 0);
        }
        
        return note;
    });
    
    // Determine beaming
    let beamGroup = null;
    if (tupletNotes.length >= 2) {
        const hasRests = tupletNotes.some(note => note.getDuration().includes('r'));
        const firstNoteDuration = tupletNotes[0].getDuration();
        
        if (!hasRests && (firstNoteDuration === '8' || firstNoteDuration === '16' || firstNoteDuration === '32')) {
            beamGroup = tupletNotes;
        }
    }
    
    // Calculate tuplet parameters
    const noteCount = tupletNotes.length;
    let tupletOptions = null;
    
    // Helper function to get next power of 2
    function getNextPowerOf2(n) {
        return Math.pow(2, Math.ceil(Math.log2(n)));
    }
    
    // Use divisions from FSM data if available, otherwise fall back to note count
    if (element.divisions) {
        // FSM provides the actual tuplet ratio
        // num_notes = beat.divisions, notes_occupied = next power of 2
        const beatDivisions = element.divisions;
        const notesOccupied = getNextPowerOf2(beatDivisions);
        
        tupletOptions = {
            notes_occupied: notesOccupied,
            num_notes: beatDivisions,
            bracketed: true  // Mixed durations typically need brackets
        };
        
        console.log(`🎵 Beat tuplet: ${beatDivisions} divisions in space of ${notesOccupied}`);
    } else {
        // Fall back to simple note count detection
        if (noteCount === 3) {
            tupletOptions = { notes_occupied: 2, num_notes: 3 };
        } else if (noteCount === 5) {
            tupletOptions = { notes_occupied: 4, num_notes: 5 };
        } else if (noteCount === 6) {
            tupletOptions = { notes_occupied: 4, num_notes: 6 };
        } else if (noteCount === 7) {
            tupletOptions = { notes_occupied: 4, num_notes: 7 };
        }
    }
    
    // Check for slur information in the original notes array
    const hasSlurStart = element.notes.some(item => item.type === 'SlurStart');
    const hasSlurEnd = element.notes.some(item => item.type === 'SlurEnd');
    
    let slurInfo = null;
    if (hasSlurStart || hasSlurEnd) {
        slurInfo = {
            start: hasSlurStart ? 0 : null,  // Start slur on first note
            end: hasSlurEnd ? tupletNotes.length - 1 : null  // End slur on last note
        };
    }
    
    return { notes: tupletNotes, beamGroup, tupletOptions, slurInfo };
}

/**
 * Handle beaming for individual notes
 */
function handleBeaming(element, note, currentBeamGroup, beamGroups) {
    if (element.type === 'Rest') return;
    
    if (element.beam_start || element.beam_end || (element.duration === '8' || element.duration === '16')) {
        if (element.beam_start) {
            currentBeamGroup.length = 0; // Clear array
            currentBeamGroup.push(note);
        } else if (element.beam_end && currentBeamGroup.length > 0) {
            currentBeamGroup.push(note);
            if (currentBeamGroup.length >= 2) {
                beamGroups.push([...currentBeamGroup]);
            }
            currentBeamGroup.length = 0; // Clear array
        } else if (currentBeamGroup.length > 0) {
            currentBeamGroup.push(note);
        }
    }
}