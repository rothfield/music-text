use crate::parse::model::{Document, DocumentElement};
use crate::models::pitch::Degree;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VexFlowOutput {
    pub staves: Vec<VexFlowStave>,
    pub key_signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VexFlowStave {
    pub notes: Vec<serde_json::Value>,
}

#[derive(Debug, Clone)]
struct BeamingInfo {
    should_beam: bool,
    beamable_notes: Vec<usize>,
}

/// VexFlow renderer that works directly with Documents using rhythm analysis
pub struct VexFlowRenderer;

impl VexFlowRenderer {
    pub fn new() -> Self {
        Self
    }

    /// Render VexFlow data directly from Document structure
    pub fn render_data_from_document(&self, document: &Document) -> serde_json::Value {
        let mut staves_data = Vec::new();

        // Convert each stave using rhythm analysis results
        for element in &document.elements {
            if let DocumentElement::Stave(stave) = element {
                let notes = if let Some(rhythm_items) = &stave.rhythm_items {
                    process_items(rhythm_items)
                } else {
                    Vec::new() // No rhythm items available
                };

                staves_data.push(serde_json::json!({
                    "notes": notes,
                    "key_signature": "C"
                }));
            }
        }

        serde_json::json!({
            "staves": staves_data,
            "title": document.directives.iter().find_map(|d| {
                if d.key == "title" { Some(&d.value) } else { None }
            }),
            "author": document.directives.iter().find_map(|d| {
                if d.key == "author" { Some(&d.value) } else { None }
            }),
            "time_signature": "4/4",
            "clef": "treble",
            "key_signature": "C"
        })
    }
}

pub fn convert_to_vexflow(document: &Document) -> Result<VexFlowOutput, String> {
    let renderer = VexFlowRenderer::new();
    let data = renderer.render_data_from_document(document);

    let mut staves = Vec::new();
    
    for stave in &document.staves {
        eprintln!("DEBUG: Processing stave with {} measures", stave.content_line.measures.len());
        let mut vex_notes = Vec::new();
        
        for measure in &stave.content_line.measures {
            // Add start barline if present
            if let Some(barline) = &measure.start_barline {
                vex_notes.push(VexFlowElement::BarLine {
                    bar_type: barline_to_vexflow(barline),
                });
            }
            
            // Process beats
            for beat in &measure.beats {
                match beat {
                    Beat::Delimited { elements, .. } | Beat::Undelimited { elements, .. } => {
                        process_beat_elements(elements, &mut vex_notes, &document.notation_system)?;
                    }
                }
            }
            
            // Add end barline if present
            if let Some(barline) = &measure.end_barline {
                vex_notes.push(VexFlowElement::BarLine {
                    bar_type: barline_to_vexflow(barline),
                });
            }
        }
        
        staves.push(VexFlowStave { notes: vex_notes });
    }
    
    Ok(VexFlowOutput {
        staves,
        key_signature: document.attributes.get("key").cloned(),
    })
}

fn process_beat_elements(
    elements: &[BeatElement], 
    vex_notes: &mut Vec<VexFlowElement>,
    notation_system: &NotationSystem
) -> Result<(), String> {
    for element in elements {
        match element {
            BeatElement::Pitch { value, accidental, octave, .. } => {
                let pitch_name = convert_pitch_to_vexflow(value, accidental, *octave, notation_system)?;
                vex_notes.push(VexFlowElement::Note {
                    keys: vec![pitch_name],
                    duration: "q".to_string(), // Default to quarter note
                });
            },
            BeatElement::Dash => {
                // For now, treat dashes as quarter rests
                vex_notes.push(VexFlowElement::Rest {
                    duration: "q".to_string(),
                });
            },
            BeatElement::Space => {
                // Spaces don't generate VexFlow elements
            },
            _ => {
                // Handle other elements as needed
            }
        }
    }
    Ok(())
}

fn convert_pitch_to_vexflow(value: &str, accidental: &Option<String>, octave: i8, notation_system: &NotationSystem) -> Result<String, String> {
    let base_note = match notation_system {
        NotationSystem::Sargam => {
            match value.trim().to_uppercase().as_str() {
                "S" => "C",
                "R" => "D", 
                "G" => "E",
                "M" => "F",
                "P" => "G",
                "D" => "A",
                "N" => "B",
                _ => return Err(format!("Unknown sargam note: {}", value)),
            }
        },
        NotationSystem::Number => {
            match value.trim() {
                "1" => "C",
                "2" => "D",
                "3" => "E", 
                "4" => "F",
                "5" => "G",
                "6" => "A",
                "7" => "B",
                _ => return Err(format!("Unknown number note: {}", value)),
            }
        },
        NotationSystem::Western => {
            let upper = value.trim().to_uppercase();
            match upper.as_str() {
                "C" => "C", "D" => "D", "E" => "E", "F" => "F",
                "G" => "G", "A" => "A", "B" => "B",
                _ => return Err(format!("Unknown western note: {}", value)),
            }
        },
        _ => {
            let upper = value.trim().to_uppercase();
            match upper.as_str() {
                "C" => "C", "D" => "D", "E" => "E", "F" => "F",
                "G" => "G", "A" => "A", "B" => "B",
                _ => return Err(format!("Unknown note: {}", value)),
            }
        },
    };
    
    let vexflow_octave = 4 + octave; // Default to 4th octave, adjust by octave marker
    let mut pitch = format!("{}/{}", base_note, vexflow_octave);
    
    if let Some(acc) = accidental {
        match acc.as_str() {
            "#" | "s" => pitch = format!("{}#/{}", base_note, vexflow_octave),
            "b" => pitch = format!("{}b/{}", base_note, vexflow_octave),
            _ => {}
        }
    }
    
    Ok(pitch)
}

fn barline_to_vexflow(barline: &Barline) -> String {
    match barline {
        Barline::Single => "single".to_string(),
        Barline::Double => "double".to_string(),
        Barline::Final => "end".to_string(),
        Barline::ReverseFinal => "start".to_string(),
        Barline::LeftRepeat => "repeat-begin".to_string(),
        Barline::RightRepeat => "repeat-end".to_string(),
    }
}

pub fn generate_vexflow_html(vexflow_output: &VexFlowOutput) -> String {
    let json_data = serde_json::to_string_pretty(vexflow_output).unwrap_or_else(|_| "{}".to_string());
    
    format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>VexFlow Notation</title>
    <script src="https://unpkg.com/vexflow@4/build/cjs/vexflow.js"></script>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        #notation {{ margin: 20px 0; }}
        .controls {{ margin-bottom: 20px; }}
        textarea {{ width: 100%; height: 100px; font-family: monospace; }}
    </style>
</head>
<body>
    <div id="notation"></div>
    <div class="controls">
        <textarea id="input" placeholder="Type notation here..."></textarea>
        <button onclick="updateNotation()">Update</button>
    </div>
    
    <script>
        const VF = Vex.Flow;
        let renderer, context, stave;
        
        const vexflowData = {json_data};
        
        function initVexFlow() {{
            const div = document.getElementById('notation');
            div.innerHTML = '';
            
            renderer = new VF.Renderer(div, VF.Renderer.Backends.SVG);
            renderer.resize(800, 200);
            context = renderer.getContext();
            
            renderNotation(vexflowData);
        }}
        
        function renderNotation(data) {{
            context.clear();
            
            stave = new VF.Stave(10, 40, 700);
            stave.addClef('treble');
            
            if (data.key_signature) {{
                stave.addKeySignature(data.key_signature);
            }}
            
            stave.setContext(context).draw();
            
            if (data.staves && data.staves[0] && data.staves[0].notes) {{
                const notes = [];
                
                for (const element of data.staves[0].notes) {{
                    if (element.type === 'Note') {{
                        const note = new VF.StaveNote({{
                            clef: 'treble',
                            keys: element.keys,
                            duration: element.duration
                        }});
                        notes.push(note);
                    }} else if (element.type === 'Rest') {{
                        const rest = new VF.StaveNote({{
                            clef: 'treble',
                            keys: ['d/5'],
                            duration: element.duration + 'r'
                        }});
                        notes.push(rest);
                    }}
                }}
                
                if (notes.length > 0) {{
                    const voice = new VF.Voice({{num_beats: 4, beat_value: 4}});
                    voice.addTickables(notes);
                    
                    const formatter = new VF.Formatter().joinVoices([voice]).format([voice], 600);
                    voice.draw(context, stave);
                }}
            }}
        }}
        
        function updateNotation() {{
            const input = document.getElementById('input').value;
            
            fetch('/api/parse', {{
                method: 'POST',
                headers: {{ 'Content-Type': 'application/json' }},
                body: JSON.stringify({{ input: input }})
            }})
            .then(response => response.json())
            .then(data => {{
                if (data.success && data.vexflow) {{
                    renderNotation(data.vexflow);
                }} else {{
                    console.error('Parse error:', data.error);
                }}
            }})
            .catch(error => console.error('Error:', error));
        }}
        
        // Auto-update on input change
        document.getElementById('input').addEventListener('input', function() {{
            clearTimeout(this.updateTimeout);
            this.updateTimeout = setTimeout(updateNotation, 300);
        }});
        
        // Initialize on page load
        window.onload = initVexFlow;
    </script>
</body>
</html>
    "#, json_data = json_data)
}