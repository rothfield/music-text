use axum::{
    extract::Query,
    response::Json,
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

/// Check for Unicode characters that should have been converted to standard ASCII
fn check_for_unicode_chars(input: &str) -> Result<(), String> {
    let problematic_chars = [
        ('▬', '-'), // Black rectangle for dashes
        ('•', '.'), // Bullet for dots  
        ('┃', '|'), // Heavy vertical line for barlines
    ];
    
    for (unicode_char, standard_char) in &problematic_chars {
        if input.contains(*unicode_char) {
            let char_positions: Vec<usize> = input.char_indices()
                .filter(|(_, c)| *c == *unicode_char)
                .map(|(i, _)| i)
                .collect();
            
            return Err(format!(
                "UNICODE CHARACTER DETECTED: Found '{}' (should be '{}') at positions {:?}. Frontend conversion failed!", 
                unicode_char, standard_char, char_positions
            ));
        }
    }
    Ok(())
}

#[derive(Serialize)]
struct ParseResponse {
    success: bool,
    parsed_document: Option<serde_json::Value>,
    processed_staves: Option<serde_json::Value>,
    detected_notation_systems: Option<Vec<NotationSystem>>,
    lilypond: Option<String>,
    lilypond_svg: Option<String>,
    vexflow: Option<serde_json::Value>,
    vexflow_svg: Option<String>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct SvgGenerateRequest {
    notation: String,
}

#[derive(Serialize)]
struct SvgGenerateResponse {
    success: bool,
    svg_content: Option<String>,
    error: Option<String>,
}

#[derive(Serialize)]
struct ValidPitchesResponse {
    flat_patterns: Vec<String>,
    sharp_patterns: Vec<String>,
}


async fn parse_text(Query(params): Query<HashMap<String, String>>) -> Json<ParseResponse> {
    let input = params.get("input").cloned().unwrap_or_default();
    
    // Check for Unicode characters first
    if let Err(unicode_error) = check_for_unicode_chars(&input) {
        return Json(ParseResponse {
            success: false,
            parsed_document: None,
            processed_staves: None,
            detected_notation_systems: None,
            lilypond: None,
            lilypond_svg: None,
            vexflow: None,
            vexflow_svg: None,
            error: Some(format!("FRONTEND CONVERSION FAILURE: {}", unicode_error)),
        });
    }
    
    if input.trim().is_empty() {
        return Json(ParseResponse {
            success: true,
            parsed_document: None,
            processed_staves: None,
            detected_notation_systems: None,
            lilypond: None,
            lilypond_svg: None,
            vexflow: None,
            vexflow_svg: None,
            error: None,
        });
    }
    
    // Get parse output
    let _parse_result = match parse_document(&input) {
        Ok(document) => {
            Some(serde_json::to_value(&document).unwrap())
        }
        Err(e) => {
            return Json(ParseResponse {
                success: false,
                    parsed_document: None,
                processed_staves: None,
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
    let (parsed_doc, processed_staves, detected_systems, lilypond, vexflow_data, vexflow_svg) = 
        match process_notation(&input) {
            Ok(result) => {
                let detected_systems = result.parsed_document.get_detected_notation_systems();
                (
                    Some(serde_json::to_value(&result.parsed_document).unwrap()),
                    Some(serde_json::to_value(&result.processed_staves).unwrap()),
                    Some(detected_systems),
                    Some(result.lilypond),
                    Some(result.vexflow_data),
                    Some(result.vexflow_svg),
                )
            },
            Err(_e) => {
                (None, None, None, None, None, None)
            },
        };
    
    let lilypond_svg = None;
    
    Json(ParseResponse {
        success: true,
        parsed_document: parsed_doc,
        processed_staves,
        detected_notation_systems: detected_systems,
        lilypond,
        lilypond_svg,
        vexflow: vexflow_data,
        vexflow_svg,
        error: None,
    })
}

async fn generate_lilypond_svg(Json(request): Json<SvgGenerateRequest>) -> Json<SvgGenerateResponse> {
    // Check for Unicode characters first
    if let Err(unicode_error) = check_for_unicode_chars(&request.notation) {
        return Json(SvgGenerateResponse {
            success: false,
            svg_content: None,
            error: Some(format!("FRONTEND CONVERSION FAILURE: {}", unicode_error)),
        });
    }
    
    if request.notation.trim().is_empty() {
        return Json(SvgGenerateResponse {
            success: false,
            svg_content: None,
            error: Some("Empty notation provided".to_string()),
        });
    }
    
    // Parse notation and generate full LilyPond source with all staves
    let lilypond_source = match process_notation(&request.notation) {
        Ok(result) => {
            render_lilypond(&result.processed_staves)
        },
        Err(e) => {
            return Json(SvgGenerateResponse {
                success: false,
                svg_content: None,
                error: Some(format!("Parse error: {}", e)),
            });
        }
    };
    
    // Generate SVG using optimized LilyPond source
    let temp_dir = std::env::temp_dir().join("music-text-svg");
    let generator = LilyPondGenerator::new(temp_dir.to_string_lossy().to_string());
    
    let result = generator.generate_svg(&lilypond_source).await;
    
    Json(SvgGenerateResponse {
        success: result.success,
        svg_content: result.svg_content,
        error: result.error,
    })
}

async fn get_valid_pitches() -> Json<ValidPitchesResponse> {
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
    
    Json(ValidPitchesResponse {
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
        .route("/api/lilypond-svg", post(generate_lilypond_svg))
        .route("/api/valid-pitches", get(get_valid_pitches))
        .nest_service("/", ServeDir::new("webapp"))
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