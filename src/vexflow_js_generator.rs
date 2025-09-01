// VexFlow JavaScript Generator
// Generates VexFlow-compatible JavaScript from FSM Item output

use crate::models::Metadata;
use crate::horizontal_parser::{Item, BeatElement};
use crate::pitch::Degree;
use crate::rhythm::RhythmConverter;
use serde::{Serialize, Deserialize};

/// VexFlow slur structure for JavaScript generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VexFlowSlur {
    pub from_note: usize,
    pub to_note: usize,
}

/// VexFlow beam structure for JavaScript generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VexFlowBeam {
    pub note_indices: Vec<usize>,
}

/// VexFlow tuplet structure for JavaScript generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VexFlowTuplet {
    pub note_indices: Vec<usize>,
    pub num_notes: usize,        // e.g., 3 for triplet
    pub notes_occupied: usize,   // e.g., 2 for triplet (3 in space of 2)
}

/// VexFlow barline structure for JavaScript generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VexFlowBarline {
    pub barline_type: crate::models::BarlineType,
    pub position: BarlinePosition,
    pub tala: Option<u8>, // Tala marker (0-6) to display above barline
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BarlinePosition {
    Beginning,  // First barline - use setBegBarType()
    End,        // Last barline - use setEndBarType()
    Middle(usize), // Mid-measure barline after note index
}

/// VexFlow note structure for JavaScript generation  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VexFlowNote {
    pub keys: Vec<String>,
    pub duration: String,
    pub dots: u8,
    pub tied: bool,
    pub beat_index: usize, // Track which beat this note belongs to
    pub ornaments: Vec<crate::parsed_models::OrnamentType>,
    pub syl: Option<String>, // Syllable/lyric text
}

/// VexFlow measure structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VexFlowMeasure {
    pub notes: Vec<VexFlowNote>,
    pub slurs: Vec<VexFlowSlur>,
    pub beams: Vec<VexFlowBeam>,
    pub tuplets: Vec<VexFlowTuplet>,
    pub barlines: Vec<VexFlowBarline>,
}

#[derive(Debug, Clone)]
enum SlurMarkerType {
    Start,
    End,
}

#[derive(Debug, Clone)]
struct SlurMarker {
    marker_type: SlurMarkerType,
    position: usize, // Position in FSM element sequence
}

/// Generate VexFlow JavaScript code from FSM Item output using 2-pass architecture
pub fn generate_vexflow_js(
    elements: &Vec<Item>,
    _metadata: &Metadata
) -> Result<String, String> {
    eprintln!("VEXFLOW DEBUG: Starting 2-pass processing of {} elements", elements.len());
    
    // Debug: Print all input elements
    for (i, element) in elements.iter().enumerate() {
        eprintln!("VEXFLOW DEBUG: Input Element {}: {:?}", i, element);
    }
    
    // PASS 1: Extract notes and slur markers into normalized form
    let (notes, slur_markers) = extract_notes_and_slur_markers(elements)?;
    
    eprintln!("VEXFLOW DEBUG: Pass 1 complete - {} notes, {} slur markers", notes.len(), slur_markers.len());
    
    // PASS 2: Create slurs from markers and normalized notes
    let slurs = create_slurs_from_markers(&notes, &slur_markers);
    
    eprintln!("VEXFLOW DEBUG: Pass 2 complete - {} slurs created", slurs.len());
    for (i, slur) in slurs.iter().enumerate() {
        eprintln!("VEXFLOW DEBUG: Slur {}: from note {} to note {}", i, slur.from_note, slur.to_note);
    }

    // PASS 3: Create beams from consecutive beamable notes
    let beams = create_beams_from_notes(&notes);
    
    eprintln!("VEXFLOW DEBUG: Pass 3 complete - {} beams created", beams.len());
    for (i, beam) in beams.iter().enumerate() {
        eprintln!("VEXFLOW DEBUG: Beam {}: notes {:?}", i, beam.note_indices);
    }

    // PASS 4: Create tuplets from tuplet beats
    let tuplets = create_tuplets_from_beats(elements, &notes)?;
    
    eprintln!("VEXFLOW DEBUG: Pass 4 complete - {} tuplets created", tuplets.len());
    for (i, tuplet) in tuplets.iter().enumerate() {
        eprintln!("VEXFLOW DEBUG: Tuplet {}: {} notes in space of {} -> {:?}", i, tuplet.num_notes, tuplet.notes_occupied, tuplet.note_indices);
    }

    // PASS 5: Create barlines from FSM barline items
    let barlines = create_barlines_from_elements(elements, &notes);
    
    eprintln!("VEXFLOW DEBUG: Pass 5 complete - {} barlines created", barlines.len());
    for (i, barline) in barlines.iter().enumerate() {
        eprintln!("VEXFLOW DEBUG: Barline {}: type '{:?}' at {:?}", i, barline.barline_type, barline.position);
    }

    // Store lengths before moving into measure
    let _notes_len = notes.len();
    let _slurs_len = slurs.len();
    let _beams_len = beams.len();
    let _tuplets_len = tuplets.len();
    let _barlines_len = barlines.len();
    
    let measure = VexFlowMeasure { notes, slurs, beams, tuplets, barlines };
    
    // Generate executable VexFlow JavaScript code  
    let js_code = format!(
        r#"(function() {{ // IIFE Start
// VexFlow rendering with auto-sizing

// Clear the canvas
const canvas = document.getElementById('vexflow-canvas');
if (!canvas) {{ console.error('VexFlow canvas not found'); return; }}
canvas.innerHTML = '';
canvas.classList.add('has-content');

// Initialize VexFlow renderer
const VF = Vex.Flow;
const renderer = new VF.Renderer(canvas, VF.Renderer.Backends.SVG);
const context = renderer.getContext();

// Create notes
const notes = {{}};
const vexNotes = [];

// Add notes to array
{}

// Create voice and add notes
// Set up voice with standard 4/4 time (4 beats, quarter note value)
const voice = new VF.Voice({{ num_beats: 4, beat_value: 4 }});
voice.setMode(VF.Voice.Mode.SOFT);  // Allow incomplete measures
voice.addTickables(vexNotes);

// Create beams (BEFORE formatting - this is crucial!)
{}

// Create tuplets (BEFORE formatting - also crucial!)
{}

// Use VexFlow's automatic width calculation for consistent spacing
const formatter = new VF.Formatter().joinVoices([voice]);
let minWidth = formatter.preCalculateMinTotalWidth([voice]);
console.log('VexFlow Debug: Raw minWidth=', minWidth);

// Safety check: if minWidth is NaN (VexFlow bug), use fallback calculation
if (isNaN(minWidth) || minWidth <= 0) {{
    console.log('VexFlow Debug: minWidth is invalid, using fallback calculation');
    // Estimate width: ~40px per quarter note + margins
    const noteCount = vexNotes.length;
    minWidth = noteCount * 50 + 100; // Conservative estimate
    console.log('VexFlow Debug: Fallback minWidth=', minWidth);
}}

const padding = 80; // Minimal margins for full width
// Use full viewport width for canvas  
const viewportWidth = window.innerWidth || document.documentElement.clientWidth || 800;
const canvasWidth = Math.max(viewportWidth - 20, minWidth + padding); // Nearly full width with 20px margin
console.log('VexFlow Debug: Final minWidth=', minWidth, 'viewportWidth=', viewportWidth, 'canvasWidth=', canvasWidth);
const canvasHeight = 200; // Compact height for better layout

// Set canvas and stave size based on calculated width
renderer.resize(canvasWidth, canvasHeight);
// Reduce scaling to allow more horizontal space
context.scale(0.9, 0.9);

// Create stave with full available width (minimal margins)
const stave = new VF.Stave(10, 40, canvasWidth - 20);
stave.addClef('treble').setContext(context);

// Set beginning and end barlines on the stave
{}

stave.draw();

// Format with calculated width for consistent spacing
try {{
    formatter.format([voice], minWidth);
    console.log('VexFlow Debug: Formatting successful');
}} catch (error) {{
    console.log('VexFlow Debug: Formatting failed, using basic formatting:', error.message);
    // Fallback: try formatting without explicit width
    try {{
        formatter.format([voice]);
    }} catch (fallbackError) {{
        console.log('VexFlow Debug: Basic formatting also failed:', fallbackError.message);
    }}
}}
voice.draw(context, stave);

// Draw beams (after voice is drawn)
beams.forEach(beam => beam.draw());

// Draw tuplets (after voice is drawn)
tuplets.forEach(tuplet => tuplet.draw());

// Create slurs
{}

// Draw syllables at consistent Y position below staff
if (window.vexflowSyllables && window.vexflowSyllables.length > 0) {{
    // Calculate maximum Y extent of all rendered elements
    let maxY = stave.getYForLine(4) + 10; // Start with staff bottom + small margin
    
    // Check note extents (stems, beams, etc.)
    vexNotes.forEach((note, index) => {{
        if (note.getBoundingBox) {{
            const bbox = note.getBoundingBox();
            maxY = Math.max(maxY, bbox.y + bbox.h + 5);
        }}
    }});
    
    // Add extra space for syllables
    const syllableY = maxY + 20;
    
    // Draw each syllable at the calculated Y position
    window.vexflowSyllables.forEach(syl => {{
        const note = vexNotes[syl.noteIndex];
        if (note && note.getAbsoluteX) {{
            const noteX = note.getAbsoluteX();
            context.save();
            context.font = 'italic 0.8em Arial';
            context.textAlign = 'center';
            context.fillStyle = '#000';
            context.fillText(syl.text, noteX, syllableY);
            context.restore();
            console.log('🎵 Drew syllable "' + syl.text + '" at x=' + noteX + ', y=' + syllableY);
        }}
    }});
    
    // Clear syllables for next render
    window.vexflowSyllables = [];
}}

// Render tala markers (after all VexFlow formatting is complete)
{}
}})(); // IIFE End"#,
        generate_notes_and_barlines_js(&measure.notes, &measure.barlines)?,
        generate_beam_creation_js(&measure.beams)?,
        generate_tuplet_creation_js(&measure.tuplets)?,
        generate_stave_barlines_js(&measure.barlines)?,
        generate_slur_creation_js(&measure.slurs)?,
        generate_tala_rendering_js(&measure.barlines)?
    );
    
    Ok(js_code)
}

/// Convert BeatElement to VexFlow note
fn convert_beat_element_to_vexflow_note(beat_element: &BeatElement) -> Result<VexFlowNote, String> {
    if !beat_element.is_note() {
        return Err("BeatElement is not a note".to_string());
    }

    let (degree, octave, _, _) = beat_element.as_note().ok_or("BeatElement is not a note")?;
    
    // Convert degree to VexFlow key
    let key = degree_to_vexflow_key(*degree, octave);
    
    // Convert duration using the proper fraction
    // Use tuplet_display_duration if it exists (for tuplets), otherwise use tuplet_duration
    let duration_to_use = beat_element.tuplet_display_duration.unwrap_or(beat_element.tuplet_duration);
    let vexflow_durations = RhythmConverter::fraction_to_vexflow(duration_to_use);
    
    // For now, just use the first duration (we may need to handle ties later for complex durations)
    let (duration, dots) = if !vexflow_durations.is_empty() {
        vexflow_durations[0].clone()
    } else {
        ("16".to_string(), 0) // Fallback to 16th note
    };
    
    Ok(VexFlowNote {
        keys: vec![key],
        duration,
        dots,
        tied: false,
        beat_index: 0, // This will be set later in extract_notes_from_beat_elements
        ornaments: beat_element.ornaments(),
        syl: beat_element.syl(),
    })
}

/// Convert Degree and octave to VexFlow key format
fn degree_to_vexflow_key(degree: Degree, octave: i8) -> String {
    use Degree::*;
    
    let note_name = match degree {
        N1 => "c", N1s => "c#", N1ss => "c##", N1b => "cb", N1bb => "cbb",
        N2 => "d", N2s => "d#", N2ss => "d##", N2b => "db", N2bb => "dbb",
        N3 => "e", N3s => "e#", N3ss => "e##", N3b => "eb", N3bb => "ebb",
        N4 => "f", N4s => "f#", N4ss => "f##", N4b => "fb", N4bb => "fbb",
        N5 => "g", N5s => "g#", N5ss => "g##", N5b => "gb", N5bb => "gbb",
        N6 => "a", N6s => "a#", N6ss => "a##", N6b => "ab", N6bb => "abb",
        N7 => "b", N7s => "b#", N7ss => "b##", N7b => "bb", N7bb => "bbb",
    };
    
    // VexFlow octave: 4 = middle C
    let vf_octave = octave + 4;
    
    format!("{}/{}", note_name, vf_octave)
}

/// Extract accidentals from VexFlow keys and return base keys + accidentals
fn extract_accidentals_from_keys(keys: &[String]) -> (Vec<String>, Vec<(usize, String)>) {
    let mut base_keys = Vec::new();
    let mut accidentals = Vec::new();
    
    for (i, key) in keys.iter().enumerate() {
        if key.contains("##") {
            // Double sharp: c##/4 -> c/4 + double sharp accidental
            let base_key = key.replace("##", "");
            base_keys.push(base_key);
            accidentals.push((i, "##".to_string()));
        } else if key.contains('#') {
            // Single sharp: c#/4 -> c/4 + sharp accidental
            let base_key = key.replace('#', "");
            base_keys.push(base_key);
            accidentals.push((i, "#".to_string()));
        } else if key.contains("bb") && !key.starts_with("bb") {
            // Double flat: dbb/4 -> d/4 + double flat accidental
            let base_key = key.replace("bb", "");
            base_keys.push(base_key);
            accidentals.push((i, "bb".to_string()));
        } else if key.contains('b') && !key.starts_with('b') {
            // Single flat: db/4 -> d/4 + flat accidental (but not for 'b' note itself)
            let base_key = key.replace('b', "");
            base_keys.push(base_key);
            accidentals.push((i, "b".to_string()));
        } else {
            // Natural note
            base_keys.push(key.clone());
        }
    }
    
    (base_keys, accidentals)
}

/// Generate JavaScript code to create VexFlow notes with BarNote objects properly inserted
fn generate_notes_and_barlines_js(notes: &[VexFlowNote], barlines: &[VexFlowBarline]) -> Result<String, String> {
    let mut js_lines = Vec::new();
    let mut barline_counter = 0;
    
    // Create all notes first
    for (i, note) in notes.iter().enumerate() {
        let _keys_js = format!("[{}]", 
            note.keys.iter()
                .map(|k| format!("'{}'", k))
                .collect::<Vec<_>>()
                .join(", ")
        );
        
        // Extract base key and accidental for proper VexFlow rendering
        let (base_keys, accidentals) = extract_accidentals_from_keys(&note.keys);
        let base_keys_js = format!("[{}]", 
            base_keys.iter()
                .map(|k| format!("'{}'", k))
                .collect::<Vec<_>>()
                .join(", ")
        );
        
        js_lines.push(format!("console.log('Note keys: [{}]');", 
            note.keys.iter().map(|k| format!("\"{}\"", k)).collect::<Vec<_>>().join(", ")));
        js_lines.push(format!(
            "const note{} = new VF.StaveNote({{ clef: 'treble', keys: {}, duration: '{}' }});",
            i, base_keys_js, note.duration
        ));
        
        // Add accidentals to make sharp symbols visible
        for (key_index, accidental) in accidentals {
            js_lines.push(format!(
                "note{}.addAccidental({}, new VF.Accidental('{}'));",
                i, key_index, accidental
            ));
        }
        
        // Add dots if needed
        for _ in 0..note.dots {
            js_lines.push(format!("note{}.addDot(0);", i));
        }

        // Add ornaments
        for ornament in &note.ornaments {
            let ornament_str = match ornament {
                crate::parsed_models::OrnamentType::Mordent => "mordent",
                _ => "mordent", // Default for now
            };
            js_lines.push(format!(
                "note{}.addModifier(new VF.Ornament('{}'), 0);",
                i, ornament_str
            ));
        }
        
        // Store syllable data for later rendering at fixed Y position
        if let Some(syllable) = &note.syl {
            js_lines.push(format!("console.log('🎵 Storing lyric for note {}: {}');", i, syllable));
            js_lines.push(format!(
                "if (!window.vexflowSyllables) window.vexflowSyllables = [];",
            ));
            js_lines.push(format!(
                "window.vexflowSyllables.push({{ noteIndex: {}, text: '{}', note: null }});",
                i, syllable
            ));
        }
        
        js_lines.push(format!("vexNotes.push(note{});", i));
        
        // Check if there's a barline after this note
        for barline in barlines {
            let should_create_barnote = match &barline.position {
                BarlinePosition::Middle(note_idx) => *note_idx == i,
                BarlinePosition::End => {
                    // FIXED: Don't create BarNote for end barlines with tala
                    // VexFlow has width calculation issues with BarNote objects
                    // Tala rendering will be handled separately via canvas drawing
                    false
                },
                BarlinePosition::Beginning => false, // Beginning barlines don't get BarNotes
            };
            
            if should_create_barnote {
                let vf_barline_type = convert_barline_type_to_vexflow(&barline.barline_type);
                js_lines.push(format!(
                    "const barNote{} = new VF.BarNote({});",
                    barline_counter, vf_barline_type
                ));
                
                // Add tala to the BarNote if present
                if let Some(tala_num) = barline.tala {
                    js_lines.push(format!(
                        "barNote{}.tala = {};  // Store tala on BarNote",
                        barline_counter, tala_num
                    ));
                }
                
                js_lines.push(format!("vexNotes.push(barNote{});", barline_counter));
                js_lines.push(format!("console.log('Inserted BarNote {} {} note {} (tala: {:?})');", barline_counter, 
                    match &barline.position { 
                        BarlinePosition::Middle(_) => "after", 
                        BarlinePosition::End => "after", 
                        BarlinePosition::Beginning => "before" 
                    }, i, barline.tala));
                barline_counter += 1;
            }
        }
    }
    
    // Beginning and end barlines will be set after stave creation in the main template
    
    Ok(js_lines.join("\n"))
}

/// Generate JavaScript code to create VexFlow beams
fn generate_beam_creation_js(beams: &[VexFlowBeam]) -> Result<String, String> {
    let mut js_lines = Vec::new();
    
    // Start with beam array declaration
    js_lines.push("const beams = [];".to_string());
    
    for (i, beam) in beams.iter().enumerate() {
        if beam.note_indices.len() < 2 {
            continue; // Skip invalid beams
        }
        
        // Generate array of note references for this beam
        let note_refs = beam.note_indices.iter()
            .map(|&idx| format!("note{}", idx))
            .collect::<Vec<_>>()
            .join(", ");
        
        let note_indices_str = beam.note_indices.iter().map(|idx| idx.to_string()).collect::<Vec<_>>().join(", ");
        
        js_lines.push(format!(
            r#"console.log('VexFlow: Creating beam {} with notes: [{}]');
const beamNotes{} = [{}];
if (beamNotes{}.every(note => note !== undefined)) {{
    const beam{} = new VF.Beam(beamNotes{}); 
    beam{}.setContext(context);
    beams.push(beam{});
    console.log('VexFlow: Beam {} created and stored');
}} else {{
    console.log('ERROR: Beam {} skipped due to undefined notes');
}}
"#,
            i, note_indices_str,
            i, note_refs,
            i,
            i, i,
            i,
            i,
            i,
            i
        ));
    }
    
    Ok(js_lines.join("\n"))
}

/// Generate JavaScript code to create VexFlow tuplets
fn generate_tuplet_creation_js(tuplets: &[VexFlowTuplet]) -> Result<String, String> {
    let mut js_lines = Vec::new();
    
    // Start with tuplet array declaration
    js_lines.push("const tuplets = [];".to_string());
    
    for (i, tuplet) in tuplets.iter().enumerate() {
        if tuplet.note_indices.len() < 2 {
            continue; // Skip invalid tuplets (need at least 2 notes)
        }
        
        // Generate array of note references for this tuplet
        let note_refs = tuplet.note_indices.iter()
            .map(|&idx| format!("note{}", idx))
            .collect::<Vec<_>>()
            .join(", ");
        
        js_lines.push(format!(
            r#"console.log('VexFlow: Creating tuplet {} with {} notes in space of {}');
const tupletNotes{} = [{}];
if (tupletNotes{}.every(note => note !== undefined)) {{
    const tupletOptions{} = {{
        notes_occupied: {},
        num_notes: {},
        bracketed: true
    }};
    const tuplet{} = new VF.Tuplet(tupletNotes{}, tupletOptions{});
    tuplet{}.setContext(context);
    tuplets.push(tuplet{});
    console.log('VexFlow: Tuplet {} created and stored');
}} else {{
    console.log('ERROR: Tuplet {} skipped due to undefined notes');
}}
"#,
            i, tuplet.num_notes, tuplet.notes_occupied,
            i, note_refs,
            i,
            i,
            tuplet.notes_occupied,
            tuplet.num_notes,
            i, i, i,
            i,
            i,
            i,
            i
        ));
    }
    
    Ok(js_lines.join("\n"))
}

/// Generate JavaScript code to create VexFlow slurs
fn generate_slur_creation_js(slurs: &[VexFlowSlur]) -> Result<String, String> {
    let mut js_lines = Vec::new();
    
    for (i, slur) in slurs.iter().enumerate() {
        js_lines.push(format!(
            r#"console.log('VexFlow: Slur {} from note{} to note{}');
console.log('VexFlow: Checking note{} and note{} exist...');
if (typeof note{} === 'undefined') {{
    console.log('ERROR: note{} is undefined!');
}} else if (typeof note{} === 'undefined') {{
    console.log('ERROR: note{} is undefined!');  
}} else {{
    const curve{} = new VF.Curve(note{}, note{}, {{ cps: [{{ x: 0, y: 10 }}, {{ x: 0, y: 10 }}] }});
    curve{}.setContext(context).draw();
    console.log('VexFlow: Slur {} created successfully');
}}
"#,
            i, slur.from_note, slur.to_note,
            slur.from_note, slur.to_note,
            slur.from_note,
            slur.from_note,
            slur.to_note,
            slur.to_note,
            i, slur.from_note, slur.to_note,
            i,
            i
        ));
    }
    
    Ok(js_lines.join("\n"))
}

/// PASS 1: Extract ALL notes first, then analyze simple FSM slur patterns
fn extract_notes_and_slur_markers(elements: &Vec<Item>) -> Result<(Vec<VexFlowNote>, Vec<SlurMarker>), String> {
    let mut notes = Vec::new();
    let mut slur_markers = Vec::new();
    
    // STEP 1: Extract all notes from all beats (track beat indices for beaming)
    let mut beat_index = 0;
    for (pos, element) in elements.iter().enumerate() {
        eprintln!("VEXFLOW DEBUG: Pass 1 - Element {}: {:?}", pos, element);
        
        if let Item::Beat(beat) = element {
            if beat.is_tuplet {
                eprintln!("VEXFLOW DEBUG: Pass 1 - Processing tuplet beat {} with {} elements (ratio: {:?})", 
                         beat_index, beat.elements.len(), beat.tuplet_ratio);
            } else {
                eprintln!("VEXFLOW DEBUG: Pass 1 - Processing regular beat {} with {} elements", beat_index, beat.elements.len());
            }
            extract_notes_from_beat_elements(&beat.elements, &mut notes, beat_index)?;
            beat_index += 1;
        }
    }
    
    // STEP 2: Analyze FSM sequence to determine precise slur span
    let slur_start_pos: Option<usize> = None;
    let slur_end_pos: Option<usize> = None;
    let mut current_note_index = 0;
    
    // Count total notes for indexing
    for element in elements.iter() {
        match element {
            Item::Beat(beat) => {
                let notes_in_beat = beat.elements.iter().filter(|e| e.is_note()).count();
                eprintln!("VEXFLOW DEBUG: Pass 1 - Beat with {} notes, current_note_index: {}", notes_in_beat, current_note_index);
                current_note_index += notes_in_beat;
            },
            _ => {}
        }
    }
    
    // Create slur markers if both positions found
    if let (Some(start_idx), Some(end_idx)) = (slur_start_pos, slur_end_pos) {
        let final_end_idx = if end_idx == usize::MAX {
            // SlurEnd appeared before notes - slur spans all notes from start
            notes.len() - 1
        } else {
            end_idx
        };
        
        if start_idx <= final_end_idx && final_end_idx < notes.len() {
            slur_markers.push(SlurMarker {
                marker_type: SlurMarkerType::Start,
                position: start_idx,
            });
            slur_markers.push(SlurMarker {
                marker_type: SlurMarkerType::End,
                position: final_end_idx,
            });
            eprintln!("VEXFLOW DEBUG: Pass 1 - FSM sequence slur: from note {} to note {}", start_idx, final_end_idx);
        } else {
            eprintln!("VEXFLOW DEBUG: Pass 1 - Invalid slur range: {} to {} (total notes: {})", start_idx, final_end_idx, notes.len());
        }
    }
    
    eprintln!("VEXFLOW DEBUG: Pass 1 complete - {} notes, {} slur markers", notes.len(), slur_markers.len());
    Ok((notes, slur_markers))
}

/// Extract notes from beat elements (works for Beat, Tuplet, any rhythm structure)
fn extract_notes_from_beat_elements(elements: &[BeatElement], notes: &mut Vec<VexFlowNote>, beat_index: usize) -> Result<(), String> {
    for beat_element in elements {
        if beat_element.is_note() {
            let mut vf_note = convert_beat_element_to_vexflow_note(beat_element)?;
            vf_note.beat_index = beat_index; // Track which beat this note belongs to
            let note_index = notes.len();
            eprintln!("VEXFLOW DEBUG: Pass 1 - Added note {} at index {} (beat {}): {:?}", note_index, note_index, beat_index, vf_note);
            notes.push(vf_note);
        }
    }
    Ok(())
}

/// PASS 2: Create slurs from markers using direct position mapping
fn create_slurs_from_markers(notes: &[VexFlowNote], slur_markers: &[SlurMarker]) -> Vec<VexFlowSlur> {
    let mut slurs = Vec::new();
    
    // Sort markers by position (note index)
    let mut sorted_markers = slur_markers.to_vec();
    sorted_markers.sort_by_key(|m| m.position);
    
    eprintln!("VEXFLOW DEBUG: Pass 2 - Found {} slur markers", sorted_markers.len());
    
    // Simple pair matching: SlurStart followed by SlurEnd
    let mut pending_start: Option<usize> = None;
    
    for marker in &sorted_markers {
        eprintln!("VEXFLOW DEBUG: Pass 2 - Processing marker {:?} at note index {}", 
                  marker.marker_type, marker.position);
        
        match marker.marker_type {
            SlurMarkerType::Start => {
                pending_start = Some(marker.position);
                eprintln!("VEXFLOW DEBUG: Pass 2 - SlurStart at note {}", marker.position);
            },
            SlurMarkerType::End => {
                if let Some(start_idx) = pending_start {
                    let end_idx = marker.position; // Use the exact position from Pass 1
                    
                    if start_idx <= end_idx && end_idx < notes.len() {
                        let slur = VexFlowSlur {
                            from_note: start_idx,
                            to_note: end_idx,
                        };
                        eprintln!("VEXFLOW DEBUG: Pass 2 - Creating slur from note {} to note {}", 
                                  start_idx, end_idx);
                        slurs.push(slur);
                    } else {
                        eprintln!("VEXFLOW DEBUG: Pass 2 - Invalid slur range: {} to {} (total notes: {})", 
                                  start_idx, end_idx, notes.len());
                    }
                    pending_start = None;
                } else {
                    eprintln!("VEXFLOW DEBUG: Pass 2 - SlurEnd without matching SlurStart");
                }
            }
        }
    }
    
    eprintln!("VEXFLOW DEBUG: Pass 2 complete - {} slurs created", slurs.len());
    slurs
}

/// PASS 3: Create beams from consecutive beamable notes WITHIN each beat
fn create_beams_from_notes(notes: &[VexFlowNote]) -> Vec<VexFlowBeam> {
    let mut beams = Vec::new();
    let mut current_beam_group = Vec::new();
    let mut current_beat_index: Option<usize> = None;
    
    eprintln!("VEXFLOW DEBUG: Pass 3 - Analyzing {} notes for beaming with beat boundaries", notes.len());
    
    for (i, note) in notes.iter().enumerate() {
        let duration = &note.duration;
        let beat_index = note.beat_index;
        eprintln!("VEXFLOW DEBUG: Pass 3 - Note {}: duration = '{}', beat = {}", i, duration, beat_index);
        
        // Check if this note is beamable (8th, 16th, 32nd notes)
        let is_beamable = matches!(duration.as_str(), "8" | "16" | "32");
        
        // Check if we've moved to a different beat - this breaks beaming!
        let beat_boundary_crossed = current_beat_index.map_or(false, |prev| prev != beat_index);
        
        if beat_boundary_crossed {
            eprintln!("VEXFLOW DEBUG: Pass 3 - Beat boundary crossed from {} to {}, finishing current beam", current_beat_index.unwrap(), beat_index);
            // Finish current beam group due to beat boundary
            if current_beam_group.len() >= 2 {
                let beam = VexFlowBeam {
                    note_indices: current_beam_group.clone(),
                };
                eprintln!("VEXFLOW DEBUG: Pass 3 - Created beam at beat boundary with {} notes: {:?}", beam.note_indices.len(), beam.note_indices);
                beams.push(beam);
            } else if current_beam_group.len() == 1 {
                eprintln!("VEXFLOW DEBUG: Pass 3 - Skipped single-note beam at beat boundary, index {}", current_beam_group[0]);
            }
            current_beam_group.clear();
        }
        
        current_beat_index = Some(beat_index);
        
        if is_beamable {
            // Add to current beam group (within same beat)
            current_beam_group.push(i);
            eprintln!("VEXFLOW DEBUG: Pass 3 - Added note {} to beam group in beat {} (size: {})", i, beat_index, current_beam_group.len());
        } else {
            // Non-beamable note breaks the current beam
            if current_beam_group.len() >= 2 {
                let beam = VexFlowBeam {
                    note_indices: current_beam_group.clone(),
                };
                eprintln!("VEXFLOW DEBUG: Pass 3 - Created beam with {} notes: {:?}", beam.note_indices.len(), beam.note_indices);
                beams.push(beam);
            } else if current_beam_group.len() == 1 {
                eprintln!("VEXFLOW DEBUG: Pass 3 - Skipped single-note beam at index {}", current_beam_group[0]);
            }
            current_beam_group.clear();
        }
    }
    
    // Don't forget the last beam group
    if current_beam_group.len() >= 2 {
        let beam = VexFlowBeam {
            note_indices: current_beam_group,
        };
        eprintln!("VEXFLOW DEBUG: Pass 3 - Created final beam with {} notes: {:?}", beam.note_indices.len(), beam.note_indices);
        beams.push(beam);
    } else if current_beam_group.len() == 1 {
        eprintln!("VEXFLOW DEBUG: Pass 3 - Skipped final single-note beam at index {}", current_beam_group[0]);
    }
    
    eprintln!("VEXFLOW DEBUG: Pass 3 complete - {} beams created", beams.len());
    beams
}

/// PASS 4: Create tuplets from tuplet beats  
fn create_tuplets_from_beats(elements: &Vec<Item>, _notes: &[VexFlowNote]) -> Result<Vec<VexFlowTuplet>, String> {
    let mut tuplets = Vec::new();
    let mut note_index = 0;
    
    eprintln!("VEXFLOW DEBUG: Pass 4 - Analyzing beats for tuplets");
    
    for (beat_idx, element) in elements.iter().enumerate() {
        if let Item::Beat(beat) = element {
            let notes_in_beat = beat.elements.iter().filter(|e| e.is_note()).count();
            
            if beat.is_tuplet {
                eprintln!("VEXFLOW DEBUG: Pass 4 - Found tuplet beat {} with {} notes, divisions: {}", 
                         beat_idx, notes_in_beat, beat.divisions);
                
                // Calculate tuplet parameters using the old JS logic
                let divisions = beat.divisions;
                let notes_occupied = get_next_power_of_2(divisions);
                
                // Create tuplet object with note indices from this beat
                let tuplet_note_indices: Vec<usize> = (note_index..note_index + notes_in_beat).collect();
                
                let tuplet = VexFlowTuplet {
                    note_indices: tuplet_note_indices.clone(),
                    num_notes: divisions,
                    notes_occupied,
                };
                
                eprintln!("VEXFLOW DEBUG: Pass 4 - Created tuplet: {} notes in space of {} -> {:?}", 
                         divisions, notes_occupied, tuplet_note_indices);
                
                tuplets.push(tuplet);
            } else {
                eprintln!("VEXFLOW DEBUG: Pass 4 - Skipping regular beat {} with {} notes", beat_idx, notes_in_beat);
            }
            
            note_index += notes_in_beat;
        }
    }
    
    eprintln!("VEXFLOW DEBUG: Pass 4 complete - {} tuplets created", tuplets.len());
    Ok(tuplets)
}

/// Helper function to calculate next power of 2 (like the old JS code)
fn get_next_power_of_2(n: usize) -> usize {
    if n <= 1 { return 1; }
    let mut power = 1;
    while power < n {
        power *= 2;
    }
    power / 2 // We want the next power of 2 that's less than n (like the old JS)
}

/// PASS 5: Create barlines from FSM elements
fn create_barlines_from_elements(elements: &Vec<Item>, _notes: &[VexFlowNote]) -> Vec<VexFlowBarline> {
    let mut barlines = Vec::new();
    let mut note_index = 0;
    
    eprintln!("VEXFLOW DEBUG: Pass 5 - Analyzing {} elements for barlines", elements.len());
    
    for (element_idx, element) in elements.iter().enumerate() {
        match element {
            Item::Barline(barline_type, tala) => {
                eprintln!("VEXFLOW DEBUG: Pass 5 - Found barline '{:?}' with tala {:?} at element {}", barline_type, tala, element_idx);
                
                let position = if element_idx == 0 {
                    // Barline at very start of music
                    BarlinePosition::Beginning
                } else if element_idx == elements.len() - 1 {
                    // Barline at very end of music
                    BarlinePosition::End
                } else {
                    // Mid-measure barline - position after current note
                    BarlinePosition::Middle(if note_index > 0 { note_index - 1 } else { 0 })
                };
                
                barlines.push(VexFlowBarline {
                    barline_type: barline_type.clone(),
                    position,
                    tala: *tala,
                });
            },
            Item::Beat(beat) => {
                // Count notes in this beat
                let notes_in_beat = beat.elements.iter().filter(|e| e.is_note()).count();
                note_index += notes_in_beat;
            },
            _ => {} // Ignore other elements
        }
    }
    
    eprintln!("VEXFLOW DEBUG: Pass 5 complete - {} barlines processed", barlines.len());
    barlines
}

// /// Generate JavaScript code to insert BarNote objects into the notes array
// fn generate_barline_setup_js(_barlines: &[VexFlowBarline]) -> Result<String, String> {
//     let mut js_lines = Vec::new();
//     
//     // We'll generate the BarNote insertion logic here
//     js_lines.push("// BarNote objects will be inserted during note creation".to_string());
//     
//     Ok(js_lines.join("\n"))
// }

/// Generate JavaScript code for setting stave beginning and end barlines
fn generate_stave_barlines_js(barlines: &[VexFlowBarline]) -> Result<String, String> {
    let mut js_lines = Vec::new();
    
    for barline in barlines {
        match &barline.position {
            BarlinePosition::Beginning => {
                let vf_type = convert_barline_type_to_vexflow(&barline.barline_type);
                js_lines.push(format!("stave.setBegBarType({})", vf_type));
                js_lines.push(format!("console.log('Set beginning barline: {:?}');", barline.barline_type));
            },
            BarlinePosition::End => {
                let vf_type = convert_barline_type_to_vexflow(&barline.barline_type);
                js_lines.push(format!("stave.setEndBarType({})", vf_type));
                js_lines.push(format!("console.log('Set end barline: {:?}');", barline.barline_type));
            },
            _ => {} // Middle barlines handled via BarNote objects
        }
    }
    
    if js_lines.is_empty() {
        js_lines.push("// No stave barlines to set".to_string());
    }
    
    Ok(js_lines.join("\n"))
}

/// Convert barline type enum to VexFlow BarlineType constant
fn convert_barline_type_to_vexflow(barline_type: &crate::models::BarlineType) -> String {
    use crate::models::BarlineType;
    match barline_type {
        BarlineType::RepeatStart => "VF.BarlineType.REPEAT_BEGIN".to_string(),
        BarlineType::RepeatEnd => "VF.BarlineType.REPEAT_END".to_string(),
        BarlineType::Double => "VF.BarlineType.DOUBLE".to_string(),
        BarlineType::Final => "VF.BarlineType.END".to_string(),
        BarlineType::RepeatBoth => "VF.BarlineType.REPEAT_BOTH".to_string(),
        BarlineType::Single => "VF.BarlineType.SINGLE".to_string(),
    }
}

/// Generate JavaScript code for rendering tala markers above barlines
fn generate_tala_rendering_js(barlines: &[VexFlowBarline]) -> Result<String, String> {
    let mut js_lines = Vec::new();
    
    // Re-enabled: Tala rendering after all VexFlow elements are drawn
    js_lines.push("// Tala rendering after all VexFlow elements".to_string());

    // Only render tala markers if there are barlines with tala
    let has_talas = barlines.iter().any(|b| b.tala.is_some());
    if !has_talas {
        js_lines.push("// No tala markers to render".to_string());
        return Ok(js_lines.join("\n"));
    }
    
    js_lines.push("// Render tala markers above barlines".to_string());
    
    for (i, barline) in barlines.iter().enumerate() {
        if let Some(tala_num) = barline.tala {
            let barline_x = match &barline.position {
                BarlinePosition::Beginning => "stave.getX()".to_string(), // At start barline
                BarlinePosition::End => {
                    // For end barlines, get the position of the last note and add some spacing
                    "(vexNotes[vexNotes.length - 1].getAbsoluteX() + vexNotes[vexNotes.length - 1].getWidth() + 20)".to_string()
                },
                BarlinePosition::Middle(note_idx) => {
                    // For middle barlines, find the BarNote that has this tala
                    // The BarNote should have the tala stored on it
                    format!("(function() {{ 
                        // Find BarNote with tala {}
                        for (let i = 0; i < vexNotes.length; i++) {{
                            if (vexNotes[i].attrs && vexNotes[i].attrs.type === 'BarNote' && vexNotes[i].tala === {}) {{
                                return vexNotes[i].getAbsoluteX();
                            }}
                        }}
                        // Fallback: find any BarNote after note {}
                        for (let i = {}; i < vexNotes.length; i++) {{
                            if (vexNotes[i].attrs && vexNotes[i].attrs.type === 'BarNote') {{
                                return vexNotes[i].getAbsoluteX();
                            }}
                        }}
                        // Final fallback
                        return stave.getX() + stave.getWidth()/2;
                    }})()", tala_num, tala_num, note_idx, note_idx + 1)
                },
            };
            
            let tala_display = if tala_num == 255 { "+".to_string() } else { tala_num.to_string() };
            let tala_text = format!(
                r#"
// Draw tala marker {} at barline {}
console.log('Drawing tala {} at barline {}');
context.save();
context.font = 'bold 14px Arial';
context.textAlign = 'center';
context.fillStyle = 'black';
const talaX{} = {};
const talaY{} = stave.getYForLine(-1) - 10; // Above staff
console.log('Tala {} position: x=' + talaX{} + ', y=' + talaY{});
context.fillText('{}', talaX{}, talaY{});
context.restore();
console.log('Tala {} drawn successfully');"#,
                tala_display, i, tala_display, i, i, barline_x, i, tala_display, i, i, tala_display, i, i, tala_display
            );
            
            js_lines.push(tala_text);
        }
    }
    
    Ok(js_lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pitch::Degree;

    #[test]
    fn test_degree_to_vexflow_key() {
        assert_eq!(degree_to_vexflow_key(Degree::N1, 0), "c/4");
        assert_eq!(degree_to_vexflow_key(Degree::N2, 0), "d/4");
        assert_eq!(degree_to_vexflow_key(Degree::N1s, 0), "c#/4");
        assert_eq!(degree_to_vexflow_key(Degree::N7b, 0), "bb/4");
        assert_eq!(degree_to_vexflow_key(Degree::N1, 1), "c/5"); // Higher octave
    }
}
