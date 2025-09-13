use axum::{
    extract::Query,
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
    error: Option<String>,
}


#[derive(Serialize)]
struct ValidPitchesResponse {
    flat_patterns: Vec<String>,
    sharp_patterns: Vec<String>,
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

async fn parse_text(Query(params): Query<HashMap<String, String>>) -> Response {
    let input = params.get("input").cloned().unwrap_or_default();
    let generate_svg = params.get("generate_svg").map(|s| s == "true").unwrap_or(false);
    
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
    
    json_with_no_cache(ParseResponse {
        success: true,
        parsed_document: parsed_doc,
        rhythm_analyzed_document: rhythm_analyzed_doc,
        detected_notation_systems: detected_systems,
        lilypond,
        lilypond_svg,
        vexflow: vexflow_data,
        vexflow_svg,
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
        .nest_service("/", ServeDir::new("public"))
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