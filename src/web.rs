// Web server for live notation parsing
use axum::{
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::{cors::CorsLayer, services::ServeDir};

use crate::{ast::{Document, Beat}, parser, renderers::vexflow, renderers::lilypond, ast_to_parsed};

#[derive(Debug, Deserialize)]
pub struct ParseRequest {
    input: String,
    system: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ParseResponse {
    success: bool,
    result: Option<Document>,
    error: Option<String>,
    yaml: Option<String>,
    vexflow: Option<vexflow::EnhancedVexFlowOutput>,
    lilypond: Option<String>,
}

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/", get(serve_index))
        .route("/api/parse", post(parse_endpoint))
        .nest_service("/static", ServeDir::new("static"))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    
    println!("🎵 Music-Text Parser Web UI running on http://127.0.0.1:3000");
    println!("📝 Open your browser and start typing notation!");
    
    axum::serve(listener, app).await.unwrap();
    
    Ok(())
}

async fn serve_index() -> impl IntoResponse {
    Html(INDEX_HTML)
}


async fn parse_endpoint(Json(request): Json<ParseRequest>) -> impl IntoResponse {
    let system = request.system.as_deref().unwrap_or("auto");
    
    match parser::parse_notation(&request.input, system) {
        Ok(document) => {
            // Create YAML with 4-space indentation
            let yaml = {
                let mut yaml_str = String::new();
                yaml_str.push_str("---\n");
                if let Ok(serialized) = serde_yaml::to_string(&document) {
                    // Re-indent the YAML with 4 spaces
                    for line in serialized.lines() {
                        if line.starts_with("---") {
                            continue; // Skip the document separator
                        }
                        if !line.is_empty() {
                            // Count leading spaces
                            let indent_level = line.len() - line.trim_start().len();
                            let spaces_per_indent = 2; // serde_yaml default
                            let new_indent_level = (indent_level / spaces_per_indent) * 4;
                            yaml_str.push_str(&" ".repeat(new_indent_level));
                            yaml_str.push_str(line.trim_start());
                        }
                        yaml_str.push('\n');
                    }
                    Some(yaml_str)
                } else {
                    Some(format_document_as_outline(&document))
                }
            };
            
            // Convert AST to parsed elements for unified processing  
            let _parsed_elements = ast_to_parsed::convert_ast_to_parsed_elements(&document);
            
            // Use FSM-based renderers (both consume same FSM output)
            let vexflow_output = vexflow::convert_document_to_enhanced_vexflow(&document).ok();
            let lilypond_output = lilypond::convert_document_to_enhanced_lilypond(&document).ok().map(|l| l.source);
            
            Json(ParseResponse {
                success: true,
                result: Some(document),
                error: None,
                yaml,
                vexflow: vexflow_output,
                lilypond: lilypond_output,
            })
        }
        Err(e) => {
            Json(ParseResponse {
                success: false,
                result: None,
                error: Some(e.to_string()),
                yaml: None,
                vexflow: None,
                lilypond: None,
            })
        }
    }
}

const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>🎵 Music-Text Parser</title>
    <script src="https://unpkg.com/vexflow@4/build/cjs/vexflow.js"></script>
    <meta http-equiv="Cache-Control" content="no-cache, no-store, must-revalidate">
    <meta http-equiv="Pragma" content="no-cache">
    <meta http-equiv="Expires" content="0">
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
            text-decoration: none !important;
        }

        *::before,
        *::after {
            text-decoration: none !important;
        }

        div, span, p, pre, code {
            text-decoration: none !important;
            text-decoration-line: none !important;
        }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
            background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
            color: #333;
            min-height: 100vh;
            line-height: 1.6;
        }

        .header {
            background: white;
            box-shadow: 0 4px 12px rgba(0,0,0,0.15);
            padding: 2rem;
            text-align: center;
            margin-bottom: 2rem;
        }

        .header h1 {
            color: #2c3e50;
            font-size: 2.5rem;
            font-weight: 300;
            margin-bottom: 0.5rem;
        }

        .header p {
            color: #7f8c8d;
            font-size: 1.1rem;
        }

        .container {
            max-width: 1600px;
            margin: 0 auto;
            padding: 0 2rem;
            display: flex;
            gap: 2rem;
            flex-wrap: wrap;
        }
        
        .input-row {
            display: flex;
            gap: 2rem;
            flex-wrap: wrap;
        }
        
        .input-box, .lilypond-box {
            flex: 1;
            min-width: 400px;
        }

        .panel {
            background: white;
            border-radius: 12px;
            box-shadow: 0 8px 32px rgba(0,0,0,0.1);
            overflow: hidden;
            flex: 1;
            min-width: 400px;
            display: flex;
            flex-direction: column;
        }

        .panel-header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 1rem 1.5rem;
            font-weight: 600;
            font-size: 1.1rem;
        }

        .panel-content {
            padding: 1.5rem;
            flex: 1;
            display: flex;
            flex-direction: column;
        }

        .controls {
            display: flex;
            gap: 1rem;
            margin-bottom: 1.5rem;
            align-items: center;
            flex-wrap: wrap;
        }

        .system-selector {
            padding: 0.75rem 1rem;
            border: 2px solid #e1e8ed;
            border-radius: 8px;
            font-size: 0.95rem;
            background: white;
            color: #2c3e50;
            cursor: pointer;
            transition: all 0.3s ease;
        }

        .system-selector:focus {
            outline: none;
            border-color: #667eea;
            box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
        }

        .status {
            padding: 0.5rem 1rem;
            border-radius: 20px;
            font-size: 0.9rem;
            font-weight: 500;
            transition: all 0.3s ease;
        }

        .status.success {
            background: #d4edda;
            color: #155724;
            border: 1px solid #c3e6cb;
        }

        .status.error {
            background: #f8d7da;
            color: #721c24;
            border: 1px solid #f5c6cb;
        }

        .input-container {
            flex: 1;
            display: flex;
            flex-direction: column;
        }

        #input, #lilypond-output {
            width: 100%;
            min-height: 80px;
            padding: 1rem;
            border: 2px solid #e1e8ed;
            border-radius: 8px;
            font-family: 'Monaco', 'Menlo', 'SF Mono', 'Consolas', monospace;
            font-size: 14px;
            line-height: 1.6;
            resize: vertical;
            transition: all 0.3s ease;
            background: #fafbfc;
            flex: 1;
        }

        #input:focus, #lilypond-output:focus {
            outline: none;
            border-color: #667eea;
            box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
            background: white;
        }
        
        #lilypond-output {
            background: #f8f9fa;
            color: #495057;
        }

        .output-container {
            flex: 1;
            display: flex;
            flex-direction: column;
        }

        #notation-container {
            border: 2px solid #e1e8ed;
            border-radius: 8px;
            background: #ffffff;
            min-height: 200px;
            display: flex;
            align-items: center;
            justify-content: center;
            margin-bottom: 1rem;
        }

        #notation {
            text-align: center;
            color: #667eea;
            font-style: italic;
        }

        #output {
            flex: 1;
            min-height: 200px;
            padding: 1.5rem;
            background: #ffffff;
            border: 2px solid #e1e8ed;
            border-radius: 8px;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
            font-size: 14px;
            line-height: 1.6;
            overflow-y: auto;
            white-space: pre-wrap;
            color: #2c3e50;
            text-decoration: none;
            text-decoration-line: none;
            text-decoration-style: none;
            text-decoration-color: transparent;
        }

        #output::before,
        #output::after,
        #output *::before,
        #output *::after {
            text-decoration: none !important;
            text-decoration-line: none !important;
        }

        #output pre,
        #output code,
        #output span,
        #output div {
            text-decoration: none !important;
            text-decoration-line: none !important;
            border: none !important;
            outline: none !important;
        }

        .examples {
            margin-top: 1.5rem;
            padding: 1.5rem;
            background: #f8f9fa;
            border-radius: 8px;
            border: 1px solid #e9ecef;
        }

        .examples h3 {
            color: #495057;
            margin-bottom: 1rem;
            font-size: 1rem;
            font-weight: 600;
        }

        .example {
            display: inline-block;
            margin: 0.5rem 0.75rem 0.5rem 0;
            padding: 0.5rem 1rem;
            background: white;
            border: 1px solid #dee2e6;
            border-radius: 20px;
            cursor: pointer;
            font-size: 0.9rem;
            transition: all 0.2s ease;
            color: #495057;
            font-family: 'Monaco', 'Menlo', 'SF Mono', 'Consolas', monospace;
        }

        .example:hover {
            background: #667eea;
            color: white;
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
        }

        .typing-indicator {
            color: #667eea;
            font-style: italic;
            margin: 0.5rem 0;
            font-size: 0.9rem;
        }

        .explanation {
            margin-top: 1rem;
            padding: 1rem;
            background: #e3f2fd;
            border-left: 4px solid #2196f3;
            border-radius: 0 8px 8px 0;
            font-size: 0.9rem;
            color: #1976d2;
        }

        .explanation h4 {
            margin-bottom: 0.5rem;
            color: #1565c0;
        }

        .explanation ul {
            margin-left: 1.5rem;
            margin-top: 0.5rem;
        }

        .explanation li {
            margin-bottom: 0.25rem;
        }

        @media (max-width: 768px) {
            .container {
                flex-direction: column;
                padding: 0 1rem;
            }
            
            .panel {
                min-width: unset;
            }
            
            .header h1 {
                font-size: 2rem;
            }
        }
    </style>
</head>
<body>

    <div class="container">
        <div class="input-row">
            <div class="panel input-box">
                <div class="panel-header">✏️ Input Notation</div>
                <div class="panel-content">
                    <div class="controls">
                        <select id="system" class="system-selector">
                            <option value="auto">Auto Detect</option>
                            <option value="sargam">Sargam (S R G M P D N)</option>
                            <option value="number">Number (1 2 3 4 5 6 7)</option>
                            <option value="western">Western (C D E F G A B)</option>
                            <option value="abc">ABC Notation</option>
                            <option value="doremi">Doremi (d r m f s l t)</option>
                        </select>
                        <div id="status" class="status"></div>
                    </div>
                    
                    <div class="input-container">
                        <textarea id="input" placeholder="Type musical notation here...

Try: S R G M
Or: &lt;S R G&gt;
Or: | S R G M |"></textarea>
                    </div>
                </div>
            </div>

            <div class="panel lilypond-box">
                <div class="panel-header">🎼 LilyPond Source</div>
                <div class="panel-content">
                    <div class="input-container">
                        <textarea id="lilypond-output" readonly placeholder="LilyPond source will appear here..."></textarea>
                    </div>
                </div>
            </div>
        </div>

        <div class="panel">
            <div class="panel-header">🎼 Musical Staff</div>
            <div class="panel-content">
                <div id="notation-container">
                    <div id="notation">Start typing to see musical notation...</div>
                </div>
            </div>
        </div>

        <div class="panel">
            <div class="panel-header">🔍 Parse Result</div>
            <div class="panel-content">
                <div id="typing" class="typing-indicator" style="display: none;">Analyzing notation...</div>
                
                <div class="output-container">
                    <div id="output">Start typing to see how your notation gets parsed...</div>
                </div>
                
                <div class="explanation">
                    <h4>Understanding the Output:</h4>
                    <ul>
                        <li><strong>attributes:</strong> Metadata like key signature, time signature</li>
                        <li><strong>staves:</strong> Musical lines/staves in your notation</li>
                        <li><strong>content_line:</strong> The main musical notes</li>
                        <li><strong>beats:</strong> How notes are grouped rhythmically</li>
                        <li><strong>Delimited:</strong> Notes grouped with &lt; &gt; brackets</li>
                        <li><strong>Undelimited:</strong> Notes separated by spaces</li>
                        <li><strong>Pitch:</strong> Individual musical notes (S, R, G, 1, 2, 3, etc.)</li>
                    </ul>
                </div>
            </div>
        </div>
    </div>

    <script>
        const VF = Vex.Flow;
        let renderer, context, currentStave;
        
        const input = document.getElementById('input');
        const output = document.getElementById('output');
        const status = document.getElementById('status');
        const system = document.getElementById('system');
        const typingIndicator = document.getElementById('typing');
        const notationDiv = document.getElementById('notation');
        const lilypondOutput = document.getElementById('lilypond-output');

        let parseTimeout;
        let lastInput = '';
        
        function initVexFlow() {
            notationDiv.innerHTML = '';
            
            renderer = new VF.Renderer(notationDiv, VF.Renderer.Backends.SVG);
            renderer.resize(800, 200);
            context = renderer.getContext();
            
            // Draw empty staff
            currentStave = new VF.Stave(10, 40, 700);
            currentStave.addClef('treble').setContext(context).draw();
        }
        
        function renderVexFlow(vexflowData) {
            if (!vexflowData || !vexflowData.staves || !vexflowData.staves[0] || !vexflowData.staves[0].notes || vexflowData.staves[0].notes.length === 0) {
                initVexFlow();
                return;
            }
            
            notationDiv.innerHTML = '';
            
            renderer = new VF.Renderer(notationDiv, VF.Renderer.Backends.SVG);
            renderer.resize(800, 200);
            context = renderer.getContext();
            
            currentStave = new VF.Stave(10, 40, 700);
            currentStave.addClef('treble');
            
            if (vexflowData.key_signature) {
                try {
                    currentStave.addKeySignature(vexflowData.key_signature);
                } catch (e) {
                    console.warn('Invalid key signature:', vexflowData.key_signature);
                }
            }
            
            currentStave.setContext(context).draw();
            
            const notes = [];
            const noteElements = vexflowData.staves[0].notes || [];
            
            for (const element of noteElements) {
                try {
                    if (element.type === 'Note' && element.keys && element.keys.length > 0) {
                        const note = new VF.StaveNote({
                            clef: 'treble',
                            keys: element.keys,
                            duration: element.duration || 'q'
                        });
                        notes.push(note);
                    } else if (element.type === 'Rest') {
                        const rest = new VF.StaveNote({
                            clef: 'treble',
                            keys: ['d/5'],
                            duration: (element.duration || 'q') + 'r'
                        });
                        notes.push(rest);
                    }
                } catch (e) {
                    console.warn('Error creating note:', element, e);
                }
            }
            
            if (notes.length > 0) {
                try {
                    const voice = new VF.Voice({num_beats: 4, beat_value: 4});
                    voice.addTickables(notes);
                    
                    const formatter = new VF.Formatter().joinVoices([voice]);
                    formatter.format([voice], 650);
                    
                    voice.draw(context, currentStave);
                } catch (e) {
                    console.error('Error rendering voice:', e);
                    initVexFlow();
                }
            }
        }

        // Auto-resize textarea
        function autoResize(textarea) {
            textarea.style.height = 'auto';
            textarea.style.height = Math.max(80, textarea.scrollHeight) + 'px';
        }
        
        function autoResizeLilyPond() {
            lilypondOutput.style.height = 'auto';
            lilypondOutput.style.height = Math.max(80, lilypondOutput.scrollHeight) + 'px';
        }

        input.addEventListener('input', () => {
            autoResize(input);
            // Save to localStorage
            localStorage.setItem('notationParser_input', input.value);
            localStorage.setItem('notationParser_system', system.value);
            
            clearTimeout(parseTimeout);
            parseTimeout = setTimeout(parseNotation, 300);
        });

        // Auto-resize output
        function autoResizeOutput() {
            const outputDiv = document.getElementById('output');
            outputDiv.style.height = 'auto';
            outputDiv.style.height = Math.max(200, outputDiv.scrollHeight) + 'px';
        }

        // Parse function with debouncing
        async function parseNotation() {
            const inputText = input.value.trim();
            const systemValue = system.value;
            
            if (inputText === lastInput) return;
            lastInput = inputText;

            if (!inputText) {
                output.textContent = 'Start typing to see how your notation gets parsed...';
                lilypondOutput.value = '';
                autoResizeLilyPond();
                status.textContent = '';
                status.className = 'status';
                initVexFlow();
                return;
            }

            // Show typing indicator
            typingIndicator.style.display = 'block';

            try {
                const response = await fetch('/api/parse', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({
                        input: inputText,
                        system: systemValue === 'auto' ? null : systemValue
                    })
                });

                const result = await response.json();
                typingIndicator.style.display = 'none';

                if (result.success) {
                    // Display the raw YAML structure
                    output.textContent = result.yaml || 'No YAML available';
                    
                    // Display LilyPond source
                    lilypondOutput.value = result.lilypond || 'No LilyPond source available';
                    autoResizeLilyPond();
                    
                    status.textContent = `✅ Parsed as ${result.result?.notation_system || 'unknown'} notation`;
                    status.className = 'status success';
                    
                    // Render VexFlow notation
                    if (result.vexflow) {
                        renderVexFlow(result.vexflow);
                    } else {
                        initVexFlow();
                    }
                    
                    setTimeout(autoResizeOutput, 100);
                } else {
                    output.textContent = `❌ Parse Error:\n${result.error}`;
                    lilypondOutput.value = '';
                autoResizeLilyPond();
                    status.textContent = '❌ Could not parse this notation';
                    status.className = 'status error';
                    initVexFlow();
                }
            } catch (error) {
                typingIndicator.style.display = 'none';
                output.textContent = `❌ Network Error:\n${error.message}`;
                lilypondOutput.value = '';
                autoResizeLilyPond();
                status.textContent = '❌ Connection problem';
                status.className = 'status error';
                initVexFlow();
            }
        }

        // Format parse result in a more readable way
        function formatParseResult(result) {
            if (!result) return 'No result';
            
            let formatted = `📋 Notation System: ${result.notation_system}\n\n`;
            
            if (result.attributes && Object.keys(result.attributes).length > 0) {
                formatted += `🎼 Attributes:\n`;
                for (const [key, value] of Object.entries(result.attributes)) {
                    formatted += `  ${key}: ${value}\n`;
                }
                formatted += '\n';
            }
            
            if (result.staves && result.staves.length > 0) {
                formatted += `🎵 Musical Content:\n`;
                result.staves.forEach((stave, i) => {
                    formatted += `\n  Stave:\n`;
                    
                    // Show upper annotation lines (octave markers, etc.)
                    if (stave.upper_lines && stave.upper_lines.length > 0) {
                        stave.upper_lines.forEach((line, lineIdx) => {
                            const annotations = line.items?.map(item => {
                                if (item.UpperOctaveMarker) return item.UpperOctaveMarker;
                                if (item.LowerOctaveMarker) return item.LowerOctaveMarker;
                                if (item.Tala) return item.Tala;
                                if (item.Chord) return `[${item.Chord}]`;
                                if (item.Ornament) return `<${item.Ornament.join(' ')}>`;
                                if (item.Mordent) return '~';
                                if (item.Symbol) return item.Symbol;
                                if (item.Space) return ' '.repeat(item.Space);
                                return JSON.stringify(item);
                            }).join('') || '';
                            if (annotations.trim()) {
                                formatted += `    Upper line ${lineIdx + 1}: ${annotations}\n`;
                            }
                        });
                    }
                    
                    if (stave.content_line && stave.content_line.measures) {
                        stave.content_line.measures.forEach((measure, j) => {
                            // Show start barline
                            let measureLine = `    Measure ${j + 1}:`;
                            if (measure.start_barline) {
                                measureLine += ` [${measure.start_barline}]`;
                            }
                            formatted += measureLine + `\n`;
                            
                            measure.beats?.forEach((beat, k) => {
                                if (beat.Delimited) {
                                    const notes = beat.Delimited.elements.map(e => 
                                        e.Pitch ? (e.Pitch.syllable ? `${e.Pitch.value.trim()}(${e.Pitch.syllable})` : e.Pitch.value.trim()) : e
                                    ).join(' ');
                                    formatted += `      [${notes}] (grouped together)\n`;
                                } else if (beat.Undelimited) {
                                    const notes = beat.Undelimited.elements.map(e => 
                                        e.Pitch ? (e.Pitch.syllable ? `${e.Pitch.value.trim()}(${e.Pitch.syllable})` : e.Pitch.value.trim()) : e
                                    ).join(' ');
                                    formatted += `      ${notes}\n`;
                                }
                            });
                            
                            // Show end barline
                            if (measure.end_barline) {
                                formatted += `      → ${measure.end_barline} barline\n`;
                            }
                        });
                    }
                    
                    // Show lower annotation lines
                    if (stave.lower_lines && stave.lower_lines.length > 0) {
                        stave.lower_lines.forEach((line, lineIdx) => {
                            const annotations = line.items?.map(item => {
                                if (item.UpperOctaveMarker) return item.UpperOctaveMarker;
                                if (item.LowerOctaveMarker) return item.LowerOctaveMarker;
                                if (item.Symbol) return item.Symbol;
                                if (item.Space) return ' '.repeat(item.Space);
                                return JSON.stringify(item);
                            }).join('') || '';
                            if (annotations.trim()) {
                                formatted += `    Lower line ${lineIdx + 1}: ${annotations}\n`;
                            }
                        });
                    }
                    
                    if (stave.lyrics_lines && stave.lyrics_lines.length > 0) {
                        formatted += `    Extra syllables: ${stave.lyrics_lines.map(l => l.syllables?.join(' ') || '').join(' | ')}\n`;
                    }
                });
            }
            
            return formatted;
        }

        // System change handler
        system.addEventListener('change', () => {
            localStorage.setItem('notationParser_system', system.value);
            parseNotation();
        });


        // Initialize - restore from localStorage
        const savedInput = localStorage.getItem('notationParser_input');
        const savedSystem = localStorage.getItem('notationParser_system');
        
        if (savedInput) {
            input.value = savedInput;
            autoResize(input);
            parseNotation();
        }
        
        if (savedSystem) {
            system.value = savedSystem;
        }
        
        input.focus();
        initVexFlow();
    </script>
</body>
</html>"#;

fn format_document_as_outline(document: &Document) -> String {
    let mut output = String::new();
    
    output.push_str("Document:\n");
    output.push_str("  Notation System:\n");
    output.push_str(&format!("    {}\n", format_notation_system(&document.notation_system)));
    
    if !document.attributes.is_empty() {
        output.push_str("  Attributes:\n");
        for (key, value) in &document.attributes {
            output.push_str(&format!("    {}:\n", key));
            output.push_str(&format!("      {}\n", value));
        }
    }
    
    if !document.staves.is_empty() {
        for stave in &document.staves {
            output.push_str("  Stave:\n");
            
            // Upper lines
            if !stave.upper_lines.is_empty() {
                for line in &stave.upper_lines {
                    output.push_str("    Upper Line:\n");
                    for item in &line.items {
                        let item_str = format_annotation_item(item);
                        if !item_str.trim().is_empty() {
                            output.push_str(&format!("      {}\n", item_str));
                        }
                    }
                }
            }
            
            // Content line
            output.push_str("    Content Line:\n");
            output.push_str("      Content:\n");
            for measure in &stave.content_line.measures {
                
                if let Some(barline) = &measure.start_barline {
                    output.push_str(&format!("        {}\n", format_barline(barline)));
                }
                
                for beat in &measure.beats {
                    match beat {
                        Beat::Delimited { elements, .. } => {
                            output.push_str("        Beat:\n");
                            output.push_str("          Type: Delimited\n");
                            for element in elements {
                                format_beat_element_outline(element, &mut output, "          ");
                            }
                        }
                        Beat::Undelimited { elements, .. } => {
                            output.push_str("        Beat:\n");
                            output.push_str("          Type: Undelimited\n");
                            for element in elements {
                                format_beat_element_outline(element, &mut output, "          ");
                            }
                        }
                    }
                }
                
                if let Some(barline) = &measure.end_barline {
                    output.push_str(&format!("        {}\n", format_barline(barline)));
                }
            }
            
            // Lower lines
            if !stave.lower_lines.is_empty() {
                for line in &stave.lower_lines {
                    output.push_str("    Lower Line:\n");
                    for item in &line.items {
                        let item_str = format_annotation_item(item);
                        if !item_str.trim().is_empty() {
                            output.push_str(&format!("      {}\n", item_str));
                        }
                    }
                }
            }
            
            // Lyrics lines
            if !stave.lyrics_lines.is_empty() {
                for line in &stave.lyrics_lines {
                    output.push_str("    Lyrics Line:\n");
                    for syllable in &line.syllables {
                        output.push_str(&format!("      {}\n", syllable));
                    }
                }
            }
        }
    }
    
    output
}

fn format_notation_system(system: &crate::ast::NotationSystem) -> &'static str {
    match system {
        crate::ast::NotationSystem::Sargam => "sargam",
        crate::ast::NotationSystem::Number => "number", 
        crate::ast::NotationSystem::Western => "western",
        crate::ast::NotationSystem::Abc => "abc",
        crate::ast::NotationSystem::Doremi => "doremi",
        crate::ast::NotationSystem::Mixed => "mixed",
    }
}

fn format_barline(barline: &crate::ast::Barline) -> &'static str {
    match barline {
        crate::ast::Barline::Single => "|",
        crate::ast::Barline::Double => "||",
        crate::ast::Barline::Final => "|]",
        crate::ast::Barline::ReverseFinal => "[|",
        crate::ast::Barline::LeftRepeat => "|:",
        crate::ast::Barline::RightRepeat => ":|",
    }
}

fn format_beat_element_simple(element: &crate::ast::BeatElement) -> String {
    match element {
        crate::ast::BeatElement::Pitch { value, accidental, syllable: _, slur_type, octave: _, .. } => {
            let mut result = value.clone();
            if let Some(acc) = accidental {
                result.push_str(acc);
            }
            if let Some(slur) = slur_type {
                match slur {
                    crate::ast::SlurType::BeginSlur => result.push_str("["),
                    crate::ast::SlurType::InSlur => result.push_str("~"),
                }
            }
            result
        }
        crate::ast::BeatElement::Dash => "-".to_string(),
        crate::ast::BeatElement::Rest { .. } => "r".to_string(),
        crate::ast::BeatElement::SlurStart => "(".to_string(),
        crate::ast::BeatElement::SlurEnd => ")".to_string(),
        crate::ast::BeatElement::Space => " ".to_string(),
    }
}

fn format_beat_element(element: &crate::ast::BeatElement) -> String {
    match element {
        crate::ast::BeatElement::Pitch { value, accidental, syllable, slur_type: _, octave: _, .. } => {
            let mut result = value.clone();
            if let Some(acc) = accidental {
                result.push_str(acc);
            }
            if let Some(syl) = syllable {
                result = format!("{}({})", result, syl);
            }
            result
        }
        crate::ast::BeatElement::Dash => "-".to_string(),
        crate::ast::BeatElement::Rest { .. } => "r".to_string(),
        crate::ast::BeatElement::SlurStart => "(".to_string(),
        crate::ast::BeatElement::SlurEnd => ")".to_string(),
        crate::ast::BeatElement::Space => " ".to_string(),
    }
}

fn format_beat_element_outline(element: &crate::ast::BeatElement, output: &mut String, indent: &str) {
    match element {
        crate::ast::BeatElement::Pitch { value, accidental, syllable, slur_type, octave, .. } => {
            output.push_str(&format!("{}Pitch:\n", indent));
            let mut pitch_val = value.clone();
            if let Some(acc) = accidental {
                pitch_val.push_str(acc);
            }
            output.push_str(&format!("{}  {}\n", indent, pitch_val));
            if let Some(syl) = syllable {
                output.push_str(&format!("{}Syllable:\n", indent));
                output.push_str(&format!("{}  {}\n", indent, syl));
            }
            if let Some(slur) = slur_type {
                output.push_str(&format!("{}Slur:\n", indent));
                let slur_str = match slur {
                    crate::ast::SlurType::BeginSlur => "begin_slur",
                    crate::ast::SlurType::InSlur => "in_slur",
                };
                output.push_str(&format!("{}  {}\n", indent, slur_str));
            }
            output.push_str(&format!("{}Octave:\n", indent));
            let octave_str = &octave.to_string();
            output.push_str(&format!("{}  {}\n", indent, octave_str));
        }
        crate::ast::BeatElement::Dash => {
            output.push_str(&format!("{}Dash:\n", indent));
            output.push_str(&format!("{}  -\n", indent));
        }
        crate::ast::BeatElement::SlurStart => {
            output.push_str(&format!("{}Slur Start:\n", indent));
            output.push_str(&format!("{}  (\n", indent));
        }
        crate::ast::BeatElement::SlurEnd => {
            output.push_str(&format!("{}Slur End:\n", indent));
            output.push_str(&format!("{}  )\n", indent));
        }
        crate::ast::BeatElement::Space => {
            output.push_str(&format!("{}Space\n", indent));
        }
    }
}

fn format_annotation_item(item: &crate::ast::AnnotationItem) -> String {
    match item {
        crate::ast::AnnotationItem::UpperOctaveMarker(s) => s.clone(),
        crate::ast::AnnotationItem::LowerOctaveMarker(s) => s.clone(),
        crate::ast::AnnotationItem::Tala(s) => s.clone(),
        crate::ast::AnnotationItem::Ornament(pitches) => format!("<{}>", pitches.join(" ")),
        crate::ast::AnnotationItem::Chord(s) => format!("[{}]", s),
        crate::ast::AnnotationItem::Mordent => "~".to_string(),
        crate::ast::AnnotationItem::Symbol(s) => s.clone(),
        crate::ast::AnnotationItem::Ending(s) => s.clone(),
        crate::ast::AnnotationItem::Slur(s) => format!("slur({})", s),
        crate::ast::AnnotationItem::BeatGrouping(s) => format!("beat_group({})", s),
        crate::ast::AnnotationItem::Space(n) => " ".repeat(*n),
    }
}

