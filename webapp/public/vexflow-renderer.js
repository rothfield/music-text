/**
 * VexFlow Renderer Module from Old Music-Text Project
 * Handles VexFlow library loading and professional music notation rendering
 * with full tuplet, slur, tie, and advanced feature support
 */

let vexflowLoaded = false;

// Global scaling factor for consistent VexFlow rendering
const VEXFLOW_SCALE_FACTOR = 0.8;

/**
 * Load VexFlow library dynamically
 */
async function loadVexFlow() {
    if (vexflowLoaded) return true;
    
    try {
        // console.log('🎵 Loading VexFlow library...');
        
        // Check if VexFlow is already available
        if (window.Vex && window.Vex.Flow) {
            vexflowLoaded = true;
            // console.log('✅ VexFlow already available');
            return true;
        }
        
        const script = document.createElement('script');
        script.src = 'assets/vexflow4.js';
        script.async = true;
        
        return new Promise((resolve, reject) => {
            script.onload = () => {
                if (window.Vex && window.Vex.Flow) {
                    vexflowLoaded = true;
                    // console.log('✅ VexFlow library loaded successfully');
                    resolve(true);
                } else {
                    console.error('❌ VexFlow loaded but Vex.Flow not available');
                    reject(new Error('VexFlow loaded but not accessible'));
                }
            };
            script.onerror = () => {
                console.error('❌ Failed to load VexFlow library');
                reject(new Error('Failed to load VexFlow'));
            };
            document.head.appendChild(script);
        });
    } catch (error) {
        console.error('🚨 VexFlow loading error:', error);
        return false;
    }
}

/**
 * Check if VexFlow is loaded and available
 */
function isVexFlowLoaded() {
    return vexflowLoaded && window.Vex && window.Vex.Flow;
}

/**
 * Main rendering function for sophisticated VexFlow notation from FSM data
 */
async function renderVexFlowNotation(vexflowData, containerId = 'vexflow-output') {
    // console.log('🎵 renderVexFlowFromFSM called with:', vexflowData);
    
    // Ensure VexFlow is loaded
    if (!isVexFlowLoaded()) {
        const loaded = await loadVexFlow();
        if (!loaded) {
            console.error('❌ Failed to load VexFlow');
            return false;
        }
    }
    
    const container = document.getElementById(containerId);
    if (!container) {
        console.error('❌ VexFlow container not found:', containerId);
        return false;
    }
    
    try {
        // Clear container
        container.innerHTML = '';
        
        const { Renderer, Stave, Formatter, Voice, Beam, StaveNote, Tuplet, Curve, StaveTie, Ornament, Annotation } = Vex.Flow;
        
        // Create renderer using old VexFlow scaling approach
        const renderer = new Renderer(container, Renderer.Backends.SVG);
        
        // Calculate width using old code's approach
        let minWidth = 400; // Conservative base width
        const totalNotes = vexflowData.staves?.reduce((sum, stave) => 
            sum + (stave.notes?.length || 0), 0) || 0;
        minWidth = totalNotes * 50 + 100; // Like old code: ~50px per note + margins
        
        // Add extra width for syllables (like old code)
        const totalSyllables = vexflowData.staves?.reduce((sum, stave) => 
            sum + (stave.notes?.filter(n => n.syl)?.length || 0), 0) || 0;
        if (totalSyllables > 0) {
            const syllableExtraWidth = totalSyllables * 30; // 30px extra per syllable
            minWidth += syllableExtraWidth;
        }
        
        const padding = 80; // Minimal margins for full width
        // Use full viewport width for canvas (like old code)
        const viewportWidth = window.innerWidth || document.documentElement.clientWidth || 800;
        const canvasWidth = Math.max(viewportWidth - 20, minWidth + padding); // Nearly full width with 20px margin
        const canvasHeight = Math.max(200, (vexflowData.staves?.length || 1) * 150);
        
        // Set canvas size based on calculated width
        renderer.resize(canvasWidth, canvasHeight);
        const context = renderer.getContext();
        
        // Apply scaling like old code
        context.scale(0.9, 0.9);
        
        let currentY = 30;
        
        // Render title and author on same line if present
        if (vexflowData.title || vexflowData.author) {
            context.save();
            
            // Title centered
            if (vexflowData.title) {
                context.setFont('serif', 18, 'bold');
                context.setFillStyle('darkslategray');
                const titleWidth = context.measureText(vexflowData.title).width || vexflowData.title.length * 10;
                const titleX = (canvasWidth / 2) - (titleWidth / 2);
                context.fillText(vexflowData.title, titleX, currentY);
            }
            
            // Author on same line, right-aligned
            if (vexflowData.author) {
                context.setFont('serif', 14, 'normal');
                context.setFillStyle('dimgray');
                const authorWidth = context.measureText(vexflowData.author).width || vexflowData.author.length * 8;
                const rightMargin = 20;
                const authorX = canvasWidth - authorWidth - rightMargin;
                context.fillText(vexflowData.author, authorX, currentY);
            }
            
            context.restore();
            currentY += 35; // Space after title/author line
        }
        
        // Process each stave with sophisticated FSM data
        const staves = vexflowData.staves || [{ notes: [], key_signature: 'C' }];
        
        for (let staveIndex = 0; staveIndex < staves.length; staveIndex++) {
            const staveData = staves[staveIndex];
            // console.log('🎵 Processing stave:', staveIndex, staveData);
            
            // Create stave with full available width (like old code: minimal margins)
            const stave = new Stave(10, currentY, canvasWidth - 20);
            if (staveIndex === 0) {
                stave.addClef('treble');
                
                // Add key signature
                if (staveData.key_signature) {
                    stave.addKeySignature(staveData.key_signature);
                }
            }
            
            // Handle barlines
            processBarlines(stave, staveData.notes || []);
            
            stave.setContext(context);
            stave.draw();
            
            // Convert sophisticated VexFlow elements to renderable notes
            const renderingResult = processVexFlowElementsAdvanced(staveData.notes || [], context, stave);
            const { notes, beams, tuplets, slurs, ties } = renderingResult;
            
            if (notes.length > 0) {
                // Create voice
                const voice = new Voice({
                    num_beats: 4,
                    beat_value: 4,
                    resolution: Vex.Flow.RESOLUTION
                }).setStrict(false);
                
                voice.addTickables(notes);
                
                // Apply VexFlow's automatic accidental tracking system
                // This handles courtesy/cautionary accidentals within measures
                const keySignature = staveData.key_signature || 'C';
                try {
                    Vex.Flow.Accidental.applyAccidentals([voice], keySignature);
                    // console.log('✅ Applied VexFlow accidental tracking with key signature:', keySignature);
                } catch (error) {
                    console.warn('⚠️ VexFlow accidental tracking failed:', error);
                }
                
                // Format with minimal width for tighter spacing (like old code)
                const formatter = new Formatter().joinVoices([voice]);
                let formatterMinWidth = formatter.preCalculateMinTotalWidth([voice]);
                
                // Count dots and add extra space for each dot
                const totalDots = staveData.notes.reduce((sum, note) => sum + (note.dots || 0), 0);
                const dotExtraWidth = totalDots * 15; // 15px extra per dot
                
                // Safety check: if minWidth is NaN (VexFlow bug), use fallback calculation
                if (isNaN(formatterMinWidth) || formatterMinWidth <= 0) {
                    formatterMinWidth = notes.length * 50 + 100; // Conservative estimate like old code
                }
                
                // Add dot spacing to formatter width
                formatterMinWidth += dotExtraWidth;
                
                // Format with adjusted width for dotted notes
                formatter.format([voice], formatterMinWidth);
                voice.draw(context, stave);
                
                // Draw advanced features
                beams.forEach(beam => beam.draw());
                tuplets.forEach(tuplet => tuplet.draw());
                ties.forEach(tie => tie.draw());
                slurs.forEach(slur => slur.draw());
                
                // Draw syllables using old code approach - relative to stave bottom
                drawSyllablesRelativeToStave(context, stave, notes);
            }
            
            currentY += 120;
        }
        
        // console.log('✅ Sophisticated VexFlow rendering completed successfully');
        return true;
        
    } catch (error) {
        console.error('🚨 VexFlow rendering error:', error);
        container.innerHTML = `<div class="alert alert-danger">VexFlow rendering error: ${error.message}</div>`;
        return false;
    }
}

/**
 * Process barlines for sophisticated barline rendering
 */
function processBarlines(stave, elements) {
    if (!elements || elements.length === 0) return;
    
    // Check first element for begin barline
    const firstElement = elements[0];
    if (firstElement?.type === 'BarLine') {
        const beginBarType = mapBarlineType(firstElement.bar_type);
        if (beginBarType) {
            stave.setBegBarType(beginBarType);
        }
    }
    
    // Check last element for end barline
    const lastElement = elements[elements.length - 1];
    if (lastElement?.type === 'BarLine') {
        const endBarType = mapBarlineType(lastElement.bar_type);
        if (endBarType) {
            stave.setEndBarType(endBarType);
        }
    }
}

/**
 * Map barline type strings to VexFlow barline types
 */
function mapBarlineType(barType) {
    const { Barline } = Vex.Flow;
    switch (barType) {
        case 'repeat-begin':
            return Barline.type.REPEAT_BEGIN;
        case 'repeat-end':
            return Barline.type.REPEAT_END;
        case 'double':
            return Barline.type.DOUBLE;
        case 'final':
            return Barline.type.END;
        case 'double-repeat':
            return Barline.type.REPEAT_BOTH;
        case 'single':
        default:
            return Barline.type.SINGLE;
    }
}

/**
 * Process sophisticated VexFlow elements with full tuplet, slur, and advanced feature support
 */
function processVexFlowElementsAdvanced(elements, context, stave) {
    const { StaveNote, Beam, Tuplet, Curve, StaveTie, Ornament, Dot, Annotation } = Vex.Flow;
    
    const notes = [];
    const beams = [];
    const tuplets = [];
    const slurs = [];
    const ties = [];
    
    let slurStartNote = null;
    let pendingSlurStart = false;
    
    for (let i = 0; i < elements.length; i++) {
        const element = elements[i];
        // console.log('🎵 Processing sophisticated element:', element.type, element);
        
        switch (element.type) {
            case 'Note':
                const note = createAdvancedVexFlowNote(element);
                notes.push(note);
                
                // If we have a pending slur start, this note begins the slur
                if (pendingSlurStart) {
                    slurStartNote = note;
                    pendingSlurStart = false;
                }
                
                // Check for ties - if this note is tied, create StaveTie to previous note
                if (element.tied && notes.length >= 2) {
                    const prevNote = notes[notes.length - 2];
                    const currNote = notes[notes.length - 1];
                    
                    const tie = new StaveTie({
                        first_note: prevNote,
                        last_note: currNote,
                        first_indices: [0],
                        last_indices: [0]
                    });
                    tie.setContext(context);
                    ties.push(tie);
                }
                break;
                
            case 'Rest':
                const rest = createAdvancedVexFlowRest(element);
                notes.push(rest);
                break;
                
            case 'Tuplet':
                // Handle sophisticated tuplet processing
                const tupletResult = processTupletAdvanced(element, context);
                if (tupletResult.notes.length > 0) {
                    // Check for ties involving tuplet notes before adding them
                    const originalNotesLength = notes.length;
                    
                    // Add tuplet notes to main notes array
                    tupletResult.notes.forEach(n => notes.push(n));
                    
                    // Handle slurs within tuplets
                    if (tupletResult.slurInfo.hasSlurStart && tupletResult.slurInfo.hasSlurEnd) {
                        const startNote = notes[originalNotesLength + tupletResult.slurInfo.slurStartIndex];
                        const endNote = notes[originalNotesLength + tupletResult.slurInfo.slurEndIndex];
                        if (startNote && endNote && startNote !== endNote) {
                            const slur = new Curve(startNote, endNote, {
                                cps: [{ x: 0, y: 10 }, { x: 0, y: 10 }]
                            });
                            slur.setContext(context);
                            slurs.push(slur);
                        }
                    }
                    
                    // Check for ties involving the tuplet notes
                    for (let i = 0; i < element.notes.length; i++) {
                        const tupletElement = element.notes[i];
                        const noteInMainArray = notes[originalNotesLength + i];
                        
                        // Check if this tuplet note should be tied to previous note
                        if (tupletElement.tied && notes.length >= 2) {
                            const noteIndex = originalNotesLength + i;
                            if (noteIndex > 0) {
                                const prevNote = notes[noteIndex - 1];
                                const currNote = notes[noteIndex];
                                
                                const tie = new StaveTie({
                                    first_note: prevNote,
                                    last_note: currNote,
                                    first_indices: [0],
                                    last_indices: [0]
                                });
                                tie.setContext(context);
                                ties.push(tie);
                            }
                        }
                    }
                    
                    // Create tuplet with proper ratio
                    const tupletRatio = element.ratio || [element.divisions, getNextPowerOf2(element.divisions)];
                    // console.log('🎵 Creating tuplet with ratio:', tupletRatio, 'notes:', tupletResult.notes.length);
                    
                    const tuplet = new Tuplet(tupletResult.notes, {
                        notes_occupied: tupletRatio[1],  // denominator (space of)
                        num_notes: tupletRatio[0],       // numerator (actual notes)
                        bracketed: true
                    });
                    tuplet.setContext(context);
                    tuplets.push(tuplet);
                    
                    // Add beaming for tuplet if appropriate
                    if (shouldBeamTupletNotes(tupletResult.notes)) {
                        const beam = new Beam(tupletResult.notes);
                        beam.setContext(context);
                        beams.push(beam);
                    }
                }
                break;
                
            case 'SlurStart':
                // Mark that the NEXT note should start a slur
                pendingSlurStart = true;
                break;
                
            case 'SlurEnd':
                // Create slur from the marked start note to the PREVIOUS note
                if (slurStartNote && notes.length > 0) {
                    const endNote = notes[notes.length - 1];
                    if (slurStartNote !== endNote) {
                        const slur = new Curve(slurStartNote, endNote, {
                            cps: [{ x: 0, y: 10 }, { x: 0, y: 10 }]
                        });
                        slur.setContext(context);
                        slurs.push(slur);
                    }
                }
                slurStartNote = null;
                break;
                
            case 'BarLine':
                // Barlines are handled at the stave level
                break;
                
            case 'Breathmark':
                // Could add breath marks as annotations in the future
                break;
        }
    }
    
    // Add beaming for consecutive beamable notes
    addBeamsFromFlags(elements, notes, beams, context);
    
    return { notes, beams, tuplets, slurs, ties };
}

/**
 * Add beams based on beam_start and beam_end flags in the elements
 */
function addBeamsFromFlags(elements, notes, beams, context) {
    const { Beam } = Vex.Flow;
    
    let noteIndex = 0;
    let beamStart = -1;
    let beamNotes = [];
    
    for (let i = 0; i < elements.length; i++) {
        const element = elements[i];
        
        if (element.type === 'Note' || element.type === 'Rest') {
            if (element.type === 'Note') {
                // Check for beam start
                if (element.beam_start) {
                    beamStart = noteIndex;
                    beamNotes = [notes[noteIndex]];
                } else if (beamStart !== -1) {
                    // Continue beam
                    beamNotes.push(notes[noteIndex]);
                }
                
                // Check for beam end
                if (element.beam_end && beamStart !== -1) {
                    // Create beam if we have at least 2 notes
                    if (beamNotes.length >= 2) {
                        const beam = new Beam(beamNotes);
                        beam.setContext(context);
                        beams.push(beam);
                    }
                    
                    // Reset beam state
                    beamStart = -1;
                    beamNotes = [];
                }
            }
            noteIndex++;
        } else if (element.type === 'Tuplet') {
            // Skip tuplet notes - they handle beaming separately
            noteIndex += element.notes ? element.notes.length : 0;
        }
    }
}

/**
 * Process sophisticated tuplet with advanced note handling
 */
function processTupletAdvanced(tupletElement, context) {
    const tupletNotes = [];
    let slurInfo = { hasSlurStart: false, hasSlurEnd: false, slurStartIndex: -1, slurEndIndex: -1 };
    
    if (!tupletElement.notes || tupletElement.notes.length === 0) {
        return { notes: tupletNotes, slurInfo };
    }
    
    // Process each element in the tuplet
    let noteIndex = 0;
    for (let i = 0; i < tupletElement.notes.length; i++) {
        const noteElement = tupletElement.notes[i];
        switch (noteElement.type) {
            case 'Note':
                const note = createAdvancedVexFlowNote(noteElement);
                tupletNotes.push(note);
                
                // Check if SlurStart was right before this note
                if (i > 0 && tupletElement.notes[i-1].type === 'SlurStart') {
                    slurInfo.hasSlurStart = true;
                    slurInfo.slurStartIndex = noteIndex;
                }
                
                // Check if SlurEnd is right after this note
                if (i < tupletElement.notes.length - 1 && tupletElement.notes[i+1].type === 'SlurEnd') {
                    slurInfo.hasSlurEnd = true;
                    slurInfo.slurEndIndex = noteIndex;
                }
                
                noteIndex++;
                break;
                
            case 'Rest':
                const rest = createAdvancedVexFlowRest(noteElement);
                tupletNotes.push(rest);
                noteIndex++;
                break;
                
            case 'SlurStart':
            case 'SlurEnd':
                // These are handled in relation to notes above
                break;
        }
    }
    
    return { notes: tupletNotes, slurInfo };
}

/**
 * Create advanced VexFlow note with full ornament, dot, and accidental support
 */
function createAdvancedVexFlowNote(element) {
    const { StaveNote, Dot, Ornament, Accidental, Annotation } = Vex.Flow;
    
    // Debug: Log element with dots
    if (element.dots > 0) {
        console.log('🎵 Note with dots:', element.duration, 'dots:', element.dots, 'keys:', element.keys);
    }
    
    const note = new StaveNote({
        clef: 'treble',
        keys: element.keys || ['c/4'],
        duration: element.duration || 'q'
    });
    
    // Add dots - use addDot() method if available, otherwise use addModifier
    for (let i = 0; i < (element.dots || 0); i++) {
        if (note.addDot) {
            note.addDot();
        } else {
            note.addModifier(new Dot(), 0);
        }
    }
    
    // Add accidentals
    if (element.accidentals && element.accidentals.length > 0) {
        element.accidentals.forEach(acc => {
            if (acc.accidental) {
                note.addModifier(new Accidental(acc.accidental), acc.index);
            }
        });
    }
    
    // Add ornaments with sophisticated handling
    if (element.ornaments && element.ornaments.length > 0) {
        element.ornaments.forEach(ornamentType => {
            try {
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
                    case 'Grace':
                        // Grace notes need special handling
                        console.log('🎵 Grace note ornament detected - needs special handling');
                        return;
                    default:
                        console.warn('🎵 Unknown ornament type:', ornamentType);
                        return;
                }
                
                const ornament = new Ornament(vexflowOrnamentType);
                note.addModifier(ornament, 0);
                console.log('✅ Added advanced ornament:', ornamentType);
            } catch (error) {
                console.error('🚨 Error adding ornament:', ornamentType, error);
            }
        });
    }
    
    // Add syllable/lyric if present - store for later positioning relative to stave
    if (element.syl && element.syl.trim()) {
        // Store syllable for manual rendering like old code did
        note._syllable = element.syl;
    }
    
    return note;
}

/**
 * Draw syllables using the old VexFlow approach - relative to stave bottom
 */
function drawSyllablesRelativeToStave(context, stave, notes) {
    const notesWithSyllables = notes.filter(note => note._syllable);
    if (notesWithSyllables.length === 0) return;
    
    // Calculate syllable Y position relative to staff bottom (like old code)
    let maxY = stave.getYForLine(4) + 10; // Start with staff bottom + small margin
    
    // Check note extents (stems, beams, etc.) like old code did
    notes.forEach(note => {
        if (note.getBoundingBox) {
            const bbox = note.getBoundingBox();
            maxY = Math.max(maxY, bbox.y + bbox.h + 5);
        }
    });
    
    // Add extra space for syllables (like old code)
    const syllableY = maxY + 20;
    
    // Draw syllables positioned under their notes
    notesWithSyllables.forEach(note => {
        if (note.getAbsoluteX && note._syllable) {
            const noteX = note.getAbsoluteX();
            
            context.save();
            context.font = 'italic 0.8em Arial';  // Like old code
            context.textAlign = 'center';
            context.fillStyle = 'black';
            context.fillText(note._syllable, noteX, syllableY);
            context.restore();
        }
    });
}

/**
 * Create advanced VexFlow rest with dot support
 */
function createAdvancedVexFlowRest(element) {
    const { StaveNote, Dot } = Vex.Flow;
    
    const rest = new StaveNote({
        clef: 'treble',
        keys: ['d/5'],
        duration: (element.duration || 'q') + 'r'
    });
    
    // Add dots
    for (let i = 0; i < (element.dots || 0); i++) {
        rest.addModifier(new Dot(), 0);
    }
    
    return rest;
}

/**
 * Check if tuplet notes should be beamed (eighth notes and smaller)
 */
function shouldBeamTupletNotes(notes) {
    if (notes.length < 2) return false;
    
    return notes.every(note => {
        const duration = note.getDuration();
        return duration === '8' || duration === '16' || duration === '32' || duration === '64';
    });
}

/**
 * Get the next power of 2 for tuplet denominators
 */
function getNextPowerOf2(n) {
    if (n <= 1) return 1;
    
    let power = 1;
    while (power < n) {
        power *= 2;
    }
    
    // Return the largest power of 2 that is less than n
    return power === n ? power : power / 2;
}

// Export functions for global use
window.VexFlowRenderer = {
    loadVexFlow,
    isVexFlowLoaded,
    renderVexFlowNotation
};

// console.log('🎵 Advanced VexFlow Renderer module loaded with sophisticated tuplet support');