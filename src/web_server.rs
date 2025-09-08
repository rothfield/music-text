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
use music_text::renderers::render_web_fast_lilypond;
use log::{info, warn, error};
use crate::lilypond_generator::LilyPondGenerator;


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
    
    // Reduced logging to prevent flashing in terminal
    
    if input.trim().is_empty() {
        // Empty input - no logging needed
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
            error!("Document parsing failed: {}", e);
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
                warn!("Processing pipeline failed: {}", e);
                (None, None, None, None, None, None, None)
            },
        };
    
    let lilypond_svg = None;
    
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
    info!("LilyPond SVG generation request received");
    
    if request.notation.trim().is_empty() {
        return Json(SvgGenerateResponse {
            success: false,
            svg_content: None,
            error: Some("Empty notation provided".to_string()),
        });
    }
    
    // Parse notation and generate optimized LilyPond source
    let lilypond_source = match process_notation(&request.notation) {
        Ok(result) => {
            render_web_fast_lilypond(&result.processed_staves)
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

pub fn start() {
    tokio::runtime::Runtime::new()
        .expect("Failed to create Tokio runtime")
        .block_on(async {
    // Initialize logger to write to stdout and development.log
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
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