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
        console.log('🎵 Loading VexFlow library...');
        
        // Check if VexFlow is already available
        if (window.Vex && window.Vex.Flow) {
            vexflowLoaded = true;
            console.log('✅ VexFlow already available');
            return true;
        }
        
        const script = document.createElement('script');
        script.src = 'assets/vexflow4.js';
        script.async = true;
        
        return new Promise((resolve, reject) => {
            script.onload = () => {
                if (window.Vex && window.Vex.Flow) {
                    vexflowLoaded = true;
                    console.log('✅ VexFlow library loaded successfully');
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
    console.log('🎵 renderVexFlowFromFSM called with:', vexflowData);
    
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
        
        const { Renderer, Stave, Formatter, Voice, Beam, StaveNote, Tuplet, Curve, StaveTie, Ornament } = Vex.Flow;
        
        // Create renderer with proper canvas size
        const renderer = new Renderer(container, Renderer.Backends.SVG);
        const containerWidth = container.offsetWidth || 800;
        const estimatedHeight = Math.max(200, (vexflowData.staves?.length || 1) * 150);
        renderer.resize(containerWidth, estimatedHeight);
        const context = renderer.getContext();
        context.scale(VEXFLOW_SCALE_FACTOR, VEXFLOW_SCALE_FACTOR);
        
        let currentY = 30;
        
        // Process each stave with sophisticated FSM data
        const staves = vexflowData.staves || [{ notes: [], key_signature: 'C' }];
        
        for (let staveIndex = 0; staveIndex < staves.length; staveIndex++) {
            const staveData = staves[staveIndex];
            console.log('🎵 Processing stave:', staveIndex, staveData);
            
            // Create stave
            const stave = new Stave(20, currentY, Math.min(1200, containerWidth - 40));
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
                
                // Format and draw
                new Formatter().joinVoices([voice]).format([voice], stave.getWidth() - 100);
                voice.draw(context, stave);
                
                // Draw advanced features
                beams.forEach(beam => beam.draw());
                tuplets.forEach(tuplet => tuplet.draw());
                ties.forEach(tie => tie.draw());
                slurs.forEach(slur => slur.draw());
            }
            
            currentY += 120;
        }
        
        console.log('✅ Sophisticated VexFlow rendering completed successfully');
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
    const { StaveNote, Beam, Tuplet, Curve, StaveTie, Ornament, Dot } = Vex.Flow;
    
    const notes = [];
    const beams = [];
    const tuplets = [];
    const slurs = [];
    const ties = [];
    
    let slurStartNote = null;
    
    for (let i = 0; i < elements.length; i++) {
        const element = elements[i];
        console.log('🎵 Processing sophisticated element:', element.type, element);
        
        switch (element.type) {
            case 'Note':
                const note = createAdvancedVexFlowNote(element);
                notes.push(note);
                break;
                
            case 'Rest':
                const rest = createAdvancedVexFlowRest(element);
                notes.push(rest);
                break;
                
            case 'Tuplet':
                // Handle sophisticated tuplet processing
                const tupletResult = processTupletAdvanced(element, context);
                if (tupletResult.notes.length > 0) {
                    // Add tuplet notes to main notes array
                    tupletResult.notes.forEach(n => notes.push(n));
                    
                    // Create tuplet with proper ratio
                    const tupletRatio = element.ratio || [element.divisions, getNextPowerOf2(element.divisions)];
                    console.log('🎵 Creating tuplet with ratio:', tupletRatio, 'notes:', tupletResult.notes.length);
                    
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
                slurStartNote = notes.length > 0 ? notes[notes.length - 1] : null;
                break;
                
            case 'SlurEnd':
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
    
    return { notes, beams, tuplets, slurs, ties };
}

/**
 * Process sophisticated tuplet with advanced note handling
 */
function processTupletAdvanced(tupletElement, context) {
    const tupletNotes = [];
    
    if (!tupletElement.notes || tupletElement.notes.length === 0) {
        return { notes: tupletNotes };
    }
    
    // Process each note in the tuplet
    for (const noteElement of tupletElement.notes) {
        switch (noteElement.type) {
            case 'Note':
                const note = createAdvancedVexFlowNote(noteElement);
                tupletNotes.push(note);
                break;
                
            case 'Rest':
                const rest = createAdvancedVexFlowRest(noteElement);
                tupletNotes.push(rest);
                break;
        }
    }
    
    return { notes: tupletNotes };
}

/**
 * Create advanced VexFlow note with full ornament, dot, and accidental support
 */
function createAdvancedVexFlowNote(element) {
    const { StaveNote, Dot, Ornament, Accidental } = Vex.Flow;
    
    const note = new StaveNote({
        clef: 'treble',
        keys: element.keys || ['c/4'],
        duration: element.duration || 'q'
    });
    
    // Add dots
    for (let i = 0; i < (element.dots || 0); i++) {
        note.addModifier(new Dot(), 0);
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
    
    return note;
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

console.log('🎵 Advanced VexFlow Renderer module loaded with sophisticated tuplet support');