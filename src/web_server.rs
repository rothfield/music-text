use axum::{
    extract::{Query, Json as ExtractJson},
    response::{IntoResponse, Json, Response},
    http::{HeaderMap, HeaderValue},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_http::{cors::CorsLayer, services::ServeDir};
use music_text::{parse_document, process_notation};
use music_text::parse::model::NotationSystem;
use music_text::renderers::render_lilypond;
use music_text::smoke_test;
use music_text::renderers::LilyPondGenerator;

/// CodeMirror empty line span structure
const NEWLINE_SPAN: &str = "<span role=\"presentation\" style=\"padding-right: 0.1px;\"><span cm-text=\"\">&ZeroWidthSpace;</span></span>";

/// Check for Unicode characters and convert musical symbols to ASCII
fn check_for_unicode_chars(input: &str) -> Result<String, String> {
    let mut converted = input.to_string();
    
    // Convert fancy musical symbols to ASCII
    converted = converted.replace('♯', "#");
    converted = converted.replace('♭', "b");
    
    let problematic_chars = [
        ('▬', '-'), // Black rectangle for dashes
        ('•', '.'), // Bullet for dots  
        ('┃', '|'), // Heavy vertical line for barlines
    ];
    
    for (unicode_char, standard_char) in &problematic_chars {
        if converted.contains(*unicode_char) {
            let char_positions: Vec<usize> = converted.char_indices()
                .filter(|(_, c)| *c == *unicode_char)
                .map(|(i, _)| i)
                .collect();
            
            return Err(format!(
                "UNICODE CHARACTER DETECTED: Found '{}' (should be '{}') at positions {:?}. Frontend conversion failed!", 
                unicode_char, standard_char, char_positions
            ));
        }
    }
    Ok(converted)
}

#[derive(Deserialize)]
struct ParseRequest {
    input: String,
    generate_svg: Option<bool>,
}

#[derive(Serialize)]
struct ParseResponse {
    success: bool,
    parsed_document: Option<serde_json::Value>,
    rhythm_analyzed_document: Option<serde_json::Value>,
    detected_notation_systems: Option<Vec<NotationSystem>>,
    lilypond: Option<String>,
    lilypond_svg: Option<String>,
    vexflow: Option<serde_json::Value>,
    vexflow_svg: Option<String>,
    roundtrip: Option<RoundtripResult>,
    xml_representation: Option<String>,
    syntax_tokens: Option<Vec<SyntaxToken>>,
    error: Option<String>,
}

#[derive(Serialize, Debug)]
struct SyntaxToken {
    token_type: String,
    start: usize,
    end: usize,
    content: String,
}

#[derive(Serialize)]
struct RoundtripResult {
    works: bool,
    reconstructed_text: String,
    where_it_failed: Option<String>,
    original_length: usize,
    reconstructed_length: usize,
}


#[derive(Serialize)]
struct ValidPitchesResponse {
    flat_patterns: Vec<String>,
    sharp_patterns: Vec<String>,
}

/// Test if we can reconstruct the original text from the parsed document
fn test_roundtrip(original_text: &str, document: &serde_json::Value) -> RoundtripResult {
    let reconstructed = reconstruct_text_from_document(document);
    let works = reconstructed == original_text;
    
    let where_it_failed = if !works {
        Some(find_first_difference(original_text, &reconstructed))
    } else {
        None
    };
    
    let reconstructed_length = reconstructed.len();
    
    RoundtripResult {
        works,
        reconstructed_text: reconstructed,
        where_it_failed,
        original_length: original_text.len(),
        reconstructed_length,
    }
}

/// Reconstruct original text from parsed document - simplified implementation
fn reconstruct_text_from_document(document: &serde_json::Value) -> String {
    let mut result = String::new();

    // Simple approach: extract content from staves
    if let Some(staves) = document.get("staves").and_then(|s| s.as_array()) {
        for stave in staves {
            if let Some(content_line) = stave.get("content_line").and_then(|cl| cl.as_array()) {
                for element in content_line {
                    // Handle each token type and extract its text value
                    if let Some(note) = element.get("Note") {
                        if let Some(value) = note.get("value").and_then(|v| v.as_str()) {
                            result.push_str(value);
                        }
                    } else if let Some(barline) = element.get("Barline") {
                        if let Some(style) = barline.get("style").and_then(|s| s.as_str()) {
                            result.push_str(style);
                        }
                    } else if let Some(whitespace) = element.get("Whitespace") {
                        if let Some(value) = whitespace.get("value").and_then(|v| v.as_str()) {
                            result.push_str(value);
                        }
                    } else if let Some(unknown) = element.get("Unknown") {
                        if let Some(value) = unknown.get("value").and_then(|v| v.as_str()) {
                            result.push_str(value);
                        }
                    } else if let Some(newline) = element.get("Newline") {
                        if let Some(value) = newline.get("value").and_then(|v| v.as_str()) {
                            result.push_str(value);
                        }
                    } else if let Some(_dash) = element.get("Dash") {
                        result.push_str("-");
                    }
                    // EndOfInput tokens represent EOF, not actual characters - skip them
                }
            }
        }
    }

    result
}

/// Recursively collect all Source fields from document JSON that still have content
fn collect_all_sources<'a>(value: &'a serde_json::Value, result: &mut Vec<&'a serde_json::Value>) {
    match value {
        serde_json::Value::Object(map) => {
            // Check if this is a Source object with unconsumed content
            if let (Some(_pos), Some(val)) = (map.get("position"), map.get("value")) {
                if !val.is_null() {
                    result.push(value);
                }
            }
            // Recurse into all object values
            for v in map.values() {
                collect_all_sources(v, result);
            }
        }
        serde_json::Value::Array(arr) => {
            // Recurse into all array elements
            for item in arr {
                collect_all_sources(item, result);
            }
        }
        _ => {} // Primitive values don't contain sources
    }
}

/// Reconstruct content line from parsed elements
fn reconstruct_content_line(elements: &serde_json::Value) -> String {
    let mut result = String::new();
    
    if let Some(element_array) = elements.as_array() {
        for element in element_array {
            if let Some(note) = element.get("Note") {
                if let Some(value) = note.get("value").and_then(|v| v.as_str()) {
                    result.push_str(value);
                }
            } else if let Some(barline) = element.get("Barline") {
                if let Some(style) = barline.get("style").and_then(|s| s.as_str()) {
                    result.push_str(style);
                }
            } else if let Some(whitespace) = element.get("Whitespace") {
                if let Some(value) = whitespace.get("value").and_then(|v| v.as_str()) {
                    result.push_str(value);
                }
            } else if let Some(breath) = element.get("Breath") {
                result.push('\'');
            } else if let Some(dash) = element.get("Dash") {
                result.push('-');
            } else if let Some(newline) = element.get("Newline") {
                if let Some(value) = newline.get("value").and_then(|v| v.as_str()) {
                    result.push_str(value);
                }
            } else if let Some(unknown) = element.get("Unknown") {
                if let Some(value) = unknown.get("value").and_then(|v| v.as_str()) {
                    result.push_str(value);
                }
            } else if let Some(_end_of_input) = element.get("EndOfInput") {
                // EndOfInput represents EOF, not actual text content - ignore during reconstruction
            }
            // Add more element types as needed
        }
    }
    
    result
}

/// Find the first character position where two strings differ
fn find_first_difference(original: &str, reconstructed: &str) -> String {
    let orig_chars: Vec<char> = original.chars().collect();
    let recon_chars: Vec<char> = reconstructed.chars().collect();
    
    for (i, (orig_char, recon_char)) in orig_chars.iter().zip(recon_chars.iter()).enumerate() {
        if orig_char != recon_char {
            return format!("Difference at position {}: original='{}' reconstructed='{}'", i, orig_char, recon_char);
        }
    }
    
    if orig_chars.len() != recon_chars.len() {
        return format!("Length difference: original={} chars, reconstructed={} chars", orig_chars.len(), recon_chars.len());
    }
    
    "No differences found".to_string()
}

/// Generate XML representation from parsed document - simplified implementation
fn generate_xml_representation(document: &serde_json::Value) -> String {
    let mut xml = String::new();

    // Simple approach: generate XML from staves
    if let Some(staves) = document.get("staves").and_then(|s| s.as_array()) {
        for stave in staves {
            xml.push_str("<pre class=\"CodeMirror-line\" role=\"presentation\">");

            if let Some(content_line) = stave.get("content_line").and_then(|cl| cl.as_array()) {
                for element in content_line {
                    // Handle each token type and generate appropriate XML
                    if let Some(note) = element.get("Note") {
                        if let Some(value) = note.get("value").and_then(|v| v.as_str()) {
                            xml.push_str(&format!("<note>{}</note>", escape_xml(value)));
                        }
                    } else if let Some(barline) = element.get("Barline") {
                        if let Some(style) = barline.get("style").and_then(|s| s.as_str()) {
                            xml.push_str(&format!("<barline>{}</barline>", escape_xml(style)));
                        }
                    } else if let Some(whitespace) = element.get("Whitespace") {
                        if let Some(value) = whitespace.get("value").and_then(|v| v.as_str()) {
                            xml.push_str(&format!("<whitespace>{}</whitespace>", escape_xml(value)));
                        }
                    } else if let Some(unknown) = element.get("Unknown") {
                        if let Some(value) = unknown.get("value").and_then(|v| v.as_str()) {
                            xml.push_str(&format!("<unknown>{}</unknown>", escape_xml(value)));
                        }
                    } else if let Some(newline) = element.get("Newline") {
                        xml.push_str("<newline>\\n</newline>");
                    } else if let Some(_dash) = element.get("Dash") {
                        xml.push_str("<dash>-</dash>");
                    }
                    // EndOfInput tokens are skipped in XML representation
                }
            }

            xml.push_str("</pre>");
        }
    }

    xml
}

/// Generate syntax tokens from parsed document for CodeMirror highlighting
fn generate_syntax_tokens(document: &serde_json::Value) -> Vec<SyntaxToken> {
    let mut tokens = Vec::new();
    let mut position = 0usize;
    
    // Process staves to extract tokens in order
    if let Some(staves) = document.get("staves").and_then(|s| s.as_array()) {
        for stave in staves {
            // Process content line elements
            if let Some(content_line) = stave.get("content_line").and_then(|cl| cl.as_array()) {
                for element in content_line {
                    if let Some(note) = element.get("Note") {
                        if let Some(value) = note.get("value").and_then(|v| v.as_str()) {
                            tokens.push(SyntaxToken {
                                token_type: "note".to_string(),
                                start: position,
                                end: position + value.len(),
                                content: value.to_string(),
                            });
                            position += value.len();
                        }
                    } else if let Some(whitespace) = element.get("Whitespace") {
                        if let Some(value) = whitespace.get("value").and_then(|v| v.as_str()) {
                            tokens.push(SyntaxToken {
                                token_type: "whitespace".to_string(),
                                start: position,
                                end: position + value.len(),
                                content: value.to_string(),
                            });
                            position += value.len();
                        }
                    } else if let Some(barline) = element.get("Barline") {
                        if let Some(style) = barline.get("style").and_then(|s| s.as_str()) {
                            tokens.push(SyntaxToken {
                                token_type: "barline".to_string(),
                                start: position,
                                end: position + style.len(),
                                content: style.to_string(),
                            });
                            position += style.len();
                        }
                    } else if let Some(_dash) = element.get("Dash") {
                        tokens.push(SyntaxToken {
                            token_type: "dash".to_string(),
                            start: position,
                            end: position + 1,
                            content: "-".to_string(),
                        });
                        position += 1;
                    } else if let Some(rest) = element.get("Rest") {
                        if let Some(value) = rest.get("value").and_then(|v| v.as_str()) {
                            tokens.push(SyntaxToken {
                                token_type: "rest".to_string(),
                                start: position,
                                end: position + value.len(),
                                content: value.to_string(),
                            });
                            position += value.len();
                        }
                    } else if let Some(_breath) = element.get("Breath") {
                        tokens.push(SyntaxToken {
                            token_type: "breath".to_string(),
                            start: position,
                            end: position + 1,
                            content: "'".to_string(),
                        });
                        position += 1;
                    } else if let Some(unknown) = element.get("Unknown") {
                        if let Some(value) = unknown.get("value").and_then(|v| v.as_str()) {
                            tokens.push(SyntaxToken {
                                token_type: "unknown".to_string(),
                                start: position,
                                end: position + value.len(),
                                content: value.to_string(),
                            });
                            position += value.len();
                        }
                    }
                }
            }
        }
    }
    
    tokens
}

/// Escape XML special characters
fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// Helper function to add cache control headers to JSON responses
fn json_with_no_cache<T>(data: T) -> Response 
where
    T: serde::Serialize,
{
    let mut headers = HeaderMap::new();
    headers.insert("Cache-Control", HeaderValue::from_static("no-cache, no-store, must-revalidate"));
    headers.insert("Pragma", HeaderValue::from_static("no-cache"));
    headers.insert("Expires", HeaderValue::from_static("0"));
    
    let json_response = Json(data);
    let mut response = json_response.into_response();
    response.headers_mut().extend(headers);
    response
}

// Core parsing logic extracted to shared function
async fn handle_parse_request(input: String, generate_svg: bool) -> Response {
    
    // Check for Unicode characters and convert musical symbols
    let converted_input = match check_for_unicode_chars(&input) {
        Ok(converted) => converted,
        Err(unicode_error) => {
            return json_with_no_cache(ParseResponse {
                success: false,
                parsed_document: None,
                rhythm_analyzed_document: None,
                detected_notation_systems: None,
                lilypond: None,
                lilypond_svg: None,
                vexflow: None,
                vexflow_svg: None,
                roundtrip: None,
                xml_representation: None,
                syntax_tokens: None,
                error: Some(format!("FRONTEND CONVERSION FAILURE: {}", unicode_error)),
            });
        }
    };
    
    if converted_input.trim().is_empty() {
        return json_with_no_cache(ParseResponse {
            success: true,
            parsed_document: None,
            rhythm_analyzed_document: None,
            detected_notation_systems: None,
            lilypond: None,
            lilypond_svg: None,
            vexflow: None,
            vexflow_svg: None,
            roundtrip: None,
            xml_representation: None,
            syntax_tokens: None,
            error: None,
        });
    }
    
    // Get parse output
    let _parse_result = match parse_document(&converted_input) {
        Ok(document) => {
            Some(serde_json::to_value(&document).unwrap())
        }
        Err(e) => {
            return json_with_no_cache(ParseResponse {
                success: false,
                parsed_document: None,
                rhythm_analyzed_document: None,
                detected_notation_systems: None,
                lilypond: None,
                lilypond_svg: None,
                vexflow: None,
                vexflow_svg: None,
                roundtrip: None,
                xml_representation: None,
                syntax_tokens: None,
                error: Some(format!("{}", e)),
            });
        }
    };
    
    // Get pipeline processing result
    let (parsed_doc, rhythm_analyzed_doc, detected_systems, lilypond, vexflow_data, vexflow_svg, lilypond_svg) = 
        match process_notation(&converted_input) {
            Ok(result) => {
                let detected_systems = result.parsed_document.get_detected_notation_systems();
                
                // Generate SVG if requested
                let lilypond_svg = if generate_svg {
                    let temp_dir = std::env::temp_dir().join("music-text-svg");
                    let generator = LilyPondGenerator::new(temp_dir.to_string_lossy().to_string());
                    
                    match generator.generate_svg(&result.lilypond).await {
                        svg_result if svg_result.success => {
                            eprintln!("✅ SVG generation successful");
                            svg_result.svg_content
                        },
                        svg_result => {
                            eprintln!("❌ SVG generation failed: {:?}", svg_result.error);
                            None
                        }
                    }
                } else {
                    None
                };
                
                (
                    Some(serde_json::to_value(&result.parsed_document).unwrap()),
                    Some(serde_json::to_value(&result.rhythm_analyzed_document).unwrap()),
                    Some(detected_systems),
                    Some(result.lilypond),
                    Some(result.vexflow_data),
                    Some(result.vexflow_svg),
                    lilypond_svg,
                )
            },
            Err(_e) => {
                (None, None, None, None, None, None, None)
            },
        };
    
    // Test roundtrip if we have parsed data
    let roundtrip = if let Some(ref parsed_doc) = parsed_doc {
        Some(test_roundtrip(&converted_input, parsed_doc))
    } else {
        None
    };

    // Generate XML representation if we have parsed data
    let xml_representation = if let Some(ref parsed_doc) = parsed_doc {
        Some(generate_xml_representation(parsed_doc))
    } else {
        None
    };

    // Generate syntax tokens if we have parsed data
    let syntax_tokens = if let Some(ref parsed_doc) = parsed_doc {
        Some(generate_syntax_tokens(parsed_doc))
    } else {
        None
    };

    json_with_no_cache(ParseResponse {
        success: true,
        parsed_document: parsed_doc,
        rhythm_analyzed_document: rhythm_analyzed_doc,
        detected_notation_systems: detected_systems,
        lilypond,
        lilypond_svg,
        vexflow: vexflow_data,
        vexflow_svg,
        roundtrip,
        xml_representation,
        syntax_tokens,
        error: None,
    })
}


async fn get_valid_pitches() -> Response {
    // Generate all valid pitch patterns that can have flats/sharps
    let flat_patterns = vec![
        // Number notation
        "1b".to_string(), "1bb".to_string(),
        "2b".to_string(), "2bb".to_string(),
        "3b".to_string(), "3bb".to_string(),
        "4b".to_string(), "4bb".to_string(),
        "5b".to_string(), "5bb".to_string(),
        "6b".to_string(), "6bb".to_string(),
        "7b".to_string(), "7bb".to_string(),
        // Western notation
        "Cb".to_string(), "Cbb".to_string(),
        "Db".to_string(), "Dbb".to_string(),
        "Eb".to_string(), "Ebb".to_string(),
        "Fb".to_string(), "Fbb".to_string(),
        "Gb".to_string(), "Gbb".to_string(),
        "Ab".to_string(), "Abb".to_string(),
        "Bb".to_string(), "Bbb".to_string(),
        // Sargam notation
        "Sb".to_string(), "Sbb".to_string(),
        "sb".to_string(), // lowercase s
        "Rb".to_string(), "Rbb".to_string(),
        "rb".to_string(), // lowercase r (komal Re)
        "Gb".to_string(), "Gbb".to_string(), // Ga
        "gb".to_string(), // lowercase g (komal Ga)
        "Mb".to_string(), "Mbb".to_string(), "mb".to_string(), "mbb".to_string(),
        "Pb".to_string(), "Pbb".to_string(),
        "pb".to_string(), // lowercase p
        "Db".to_string(), "Dbb".to_string(), // Dha
        "db".to_string(), // lowercase d (komal Dha)
        "Nb".to_string(), "Nbb".to_string(),
        "nb".to_string(), // lowercase n (komal Ni)
    ];
    
    let sharp_patterns = vec![
        // Number notation
        "1#".to_string(), "1##".to_string(),
        "2#".to_string(), "2##".to_string(),
        "3#".to_string(), "3##".to_string(),
        "4#".to_string(), "4##".to_string(),
        "5#".to_string(), "5##".to_string(),
        "6#".to_string(), "6##".to_string(),
        "7#".to_string(), "7##".to_string(),
        // Western notation
        "C#".to_string(), "C##".to_string(),
        "D#".to_string(), "D##".to_string(),
        "E#".to_string(), "E##".to_string(),
        "F#".to_string(), "F##".to_string(),
        "G#".to_string(), "G##".to_string(),
        "A#".to_string(), "A##".to_string(),
        "B#".to_string(), "B##".to_string(),
        // Sargam notation
        "S#".to_string(), "S##".to_string(),
        "R#".to_string(), "R##".to_string(),
        "G#".to_string(), "G##".to_string(), // Ga
        "M#".to_string(), "M##".to_string(), "m#".to_string(), "m##".to_string(),
        "P#".to_string(), "P##".to_string(),
        "D#".to_string(), "D##".to_string(), // Dha
        "N#".to_string(), "N##".to_string(),
    ];
    
    json_with_no_cache(ValidPitchesResponse {
        flat_patterns,
        sharp_patterns,
    })
}

pub fn start() {
    tokio::runtime::Runtime::new()
        .expect("Failed to create Tokio runtime")
        .block_on(async {
    
    // Run smoke tests on startup
    if let Err(_e) = smoke_test::run_smoke_tests() {
        // Continue running server
    }
    
    let app = Router::new()
        .route("/api/parse", get(parse_text))
        .route("/api/valid-pitches", get(get_valid_pitches))
        .nest_service("/", ServeDir::new("webapp/public"))
        .layer(CorsLayer::permissive());

    let listener = match tokio::net::TcpListener::bind("127.0.0.1:3000").await {
        Ok(listener) => {
            println!("Server running on http://127.0.0.1:3000");
            listener
        }
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            println!("Port 3000 already in use. Check if server is already running at http://127.0.0.1:3000");
            return;
        }
        Err(_e) => {
            return;
        }
    };
    
    axum::serve(listener, app).await.unwrap();
        });
}

// GET handler for query parameters
async fn parse_text(Query(params): Query<HashMap<String, String>>) -> Response {
    let input = params.get("input").cloned().unwrap_or_default();
    let generate_svg = params.get("generate_svg").map(|s| s == "true").unwrap_or(false);
    handle_parse_request(input, generate_svg).await
}