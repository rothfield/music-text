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
use music_text::document::model::NotationSystem;
use music_text::renderers::render_full_lilypond;
use music_text::smoke_test;
use log::{info, warn, error};
use crate::lilypond_generator::LilyPondGenerator;

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
            
            error!("🚨 UNICODE CHARACTER DETECTED IN BACKEND INPUT!");
            error!("🚨 Found '{}' (should be '{}') at positions: {:?}", unicode_char, standard_char, char_positions);
            error!("🚨 This indicates the frontend Unicode-to-standard conversion is NOT working!");
            error!("🚨 Input preview (first 200 chars): {}", &input[..input.len().min(200)]);
            
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
    pest_output: Option<serde_json::Value>,
    parsed_document: Option<serde_json::Value>,
    processed_staves: Option<serde_json::Value>,
    detected_notation_systems: Option<Vec<NotationSystem>>,
    minimal_lilypond: Option<String>,
    full_lilypond: Option<String>,
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

// Using hand-written parser instead of Pest

async fn parse_text(Query(params): Query<HashMap<String, String>>) -> Json<ParseResponse> {
    let input = params.get("input").cloned().unwrap_or_default();
    
    info!("📥 API /parse request - input length: {}, first 50 chars: {:?}", 
        input.len(), 
        input.chars().take(50).collect::<String>()
    );
    
    // Check for Unicode characters first - complain loudly if found
    if let Err(unicode_error) = check_for_unicode_chars(&input) {
        error!("🚨 PARSE ENDPOINT: {}", unicode_error);
        return Json(ParseResponse {
            success: false,
            pest_output: None,
            parsed_document: None,
            processed_staves: None,
            detected_notation_systems: None,
            minimal_lilypond: None,
            full_lilypond: None,
            lilypond_svg: None,
            vexflow: None,
            vexflow_svg: None,
            error: Some(format!("FRONTEND CONVERSION FAILURE: {}", unicode_error)),
        });
    }
    
    // Reduced logging to prevent flashing in terminal
    
    if input.trim().is_empty() {
        info!("📤 API /parse response - empty input, returning empty response");
        return Json(ParseResponse {
            success: true,
            pest_output: None,
            parsed_document: None,
            processed_staves: None,
            detected_notation_systems: None,
            minimal_lilypond: None,
            full_lilypond: None,
            lilypond_svg: None,
            vexflow: None,
            vexflow_svg: None,
            error: None,
        });
    }
    
    // Get parse output
    let parse_result = match parse_document(&input) {
        Ok(document) => {
            Some(serde_json::to_value(&document).unwrap())
        }
        Err(e) => {
            error!("📤 API /parse response - document parsing failed: {}", e);
            return Json(ParseResponse {
                success: false,
                pest_output: None,
                parsed_document: None,
                processed_staves: None,
                detected_notation_systems: None,
                minimal_lilypond: None,
                full_lilypond: None,
                lilypond_svg: None,
                vexflow: None,
                vexflow_svg: None,
                error: Some(format!("{}", e)),
            });
        }
    };
    
    // Get pipeline processing result
    let (parsed_doc, processed_staves, detected_systems, minimal_lilypond, full_lilypond, vexflow_data, vexflow_svg) = 
        match process_notation(&input) {
            Ok(result) => {
                let detected_systems = result.parsed_document.get_detected_notation_systems();
                // Reduced logging to prevent terminal flashing
                (
                    Some(serde_json::to_value(&result.parsed_document).unwrap()),
                    Some(serde_json::to_value(&result.processed_staves).unwrap()),
                    Some(detected_systems),
                    Some(result.minimal_lilypond),
                    Some(result.full_lilypond),
                    Some(result.vexflow_data),
                    Some(result.vexflow_svg),
                )
            },
            Err(e) => {
                warn!("📤 API /parse - processing pipeline failed: {}", e);
                (None, None, None, None, None, None, None)
            },
        };
    
    let lilypond_svg = None;
    
    info!("📤 API /parse response - success: true, outputs: doc={}, processed={}, lily={}, vex={}", 
        parsed_doc.is_some(), processed_staves.is_some(), minimal_lilypond.is_some(), vexflow_data.is_some());
    
    Json(ParseResponse {
        success: true,
        pest_output: parse_result,
        parsed_document: parsed_doc,
        processed_staves,
        detected_notation_systems: detected_systems,
        minimal_lilypond,
        full_lilypond,
        lilypond_svg,
        vexflow: vexflow_data,
        vexflow_svg,
        error: None,
    })
}

async fn generate_lilypond_svg(Json(request): Json<SvgGenerateRequest>) -> Json<SvgGenerateResponse> {
    info!("📥 API /lilypond-svg request - notation length: {}, first 50 chars: {:?}", 
        request.notation.len(), 
        request.notation.chars().take(50).collect::<String>()
    );
    
    // Check for Unicode characters first - complain loudly if found
    if let Err(unicode_error) = check_for_unicode_chars(&request.notation) {
        error!("🚨 SVG ENDPOINT: {}", unicode_error);
        return Json(SvgGenerateResponse {
            success: false,
            svg_content: None,
            error: Some(format!("FRONTEND CONVERSION FAILURE: {}", unicode_error)),
        });
    }
    
    if request.notation.trim().is_empty() {
        info!("📤 API /lilypond-svg response - empty notation, returning error");
        return Json(SvgGenerateResponse {
            success: false,
            svg_content: None,
            error: Some("Empty notation provided".to_string()),
        });
    }
    
    // Parse notation and generate full LilyPond source with all staves
    let lilypond_source = match process_notation(&request.notation) {
        Ok(result) => {
            render_full_lilypond(&result.processed_staves)
        },
        Err(e) => {
            error!("📤 API /lilypond-svg response - parse error: {}", e);
            return Json(SvgGenerateResponse {
                success: false,
                svg_content: None,
                error: Some(format!("Parse error: {}", e)),
            });
        }
    };
    
    info!("🎼 Generated LilyPond source ({} chars), calling SVG generator", lilypond_source.len());
    
    // Generate SVG using optimized LilyPond source
    let temp_dir = std::env::temp_dir().join("music-text-svg");
    let generator = LilyPondGenerator::new(temp_dir.to_string_lossy().to_string());
    
    let result = generator.generate_svg(&lilypond_source).await;
    
    info!("📤 API /lilypond-svg response - success: {}, svg_len: {}, error: {:?}", 
        result.success, 
        result.svg_content.as_ref().map(|s| s.len()).unwrap_or(0),
        result.error.as_ref().map(|e| &e[..e.len().min(100)])
    );
    
    Json(SvgGenerateResponse {
        success: result.success,
        svg_content: result.svg_content,
        error: result.error,
    })
}

pub fn start() {
    tokio::runtime::Runtime::new()
        .expect("Failed to create Tokio runtime")
        .block_on(async {
    // Initialize logger to write to stdout and development.log
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .target(env_logger::Target::Stdout)
        .format(|buf, record| {
            use std::io::Write;
            let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");
            writeln!(buf, "[{}] {} - {}", timestamp, record.level(), record.args())?;
            
            // Also append to development.log file (like Rails)
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("development.log") {
                writeln!(file, "[{}] {} - {}", timestamp, record.level(), record.args()).ok();
            }
            Ok(())
        })
        .init();
    
    info!("Music Text Parser server starting up");
    
    // Run comprehensive smoke tests on startup
    info!("🔥🔥🔥 RUNNING COMPREHENSIVE SMOKE TESTS 🔥🔥🔥");
    match smoke_test::run_smoke_tests() {
        Ok(_) => {
            info!("✅✅✅ ALL SMOKE TESTS PASSED - SERVER READY ✅✅✅");
        },
        Err(e) => {
            error!("🚨🚨🚨 SMOKE TESTS FAILED! 🚨🚨🚨");
            error!("🚨 ERROR: {}", e);
            error!("🚨 SERVER MAY NOT FUNCTION CORRECTLY!");
            error!("🚨 Please review the errors above and fix before deploying!");
            // Continue running server but with loud warnings
        }
    }
    
    let app = Router::new()
        .route("/api/parse", get(parse_text))
        .route("/api/lilypond-svg", post(generate_lilypond_svg))
        .nest_service("/", ServeDir::new("webapp"))
        .layer(CorsLayer::permissive());

    let listener = match tokio::net::TcpListener::bind("127.0.0.1:3000").await {
        Ok(listener) => {
            info!("Server binding successful on 127.0.0.1:3000");
            println!("Server running on http://127.0.0.1:3000");
            listener
        }
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            info!("Port 3000 already in use - server may already be running");
            println!("Port 3000 already in use. Check if server is already running at http://127.0.0.1:3000");
            return;
        }
        Err(e) => {
            error!("Failed to bind to 127.0.0.1:3000: {}", e);
            return;
        }
    };
    
    axum::serve(listener, app).await.unwrap();
        });
}