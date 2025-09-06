use axum::{
    extract::Query,
    response::Json,
    routing::get,
    Router,
};
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use tower_http::{cors::CorsLayer, services::ServeDir};
use music_text::{parse_notation, pest_pair_to_json, process_notation};
use music_text::document::model::NotationSystem;
use log::{info, warn, error};


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

// pest_pair_to_json is now imported from parser module

async fn parse_text(Query(params): Query<HashMap<String, String>>) -> Json<ParseResponse> {
    let input = params.get("input").cloned().unwrap_or_default();
    
    info!("API request received with input: '{}'", input.chars().take(50).collect::<String>());
    
    if input.trim().is_empty() {
        info!("Empty input received, returning empty response");
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
    
    // Get PEST output
    let pest_result = match parse_notation(&input) {
        Ok(pairs) => {
            let result: Vec<serde_json::Value> = pairs
                .map(|pair| pest_pair_to_json(&pair))
                .collect();
            Some(serde_json::Value::Array(result))
        }
        Err(e) => {
            error!("PEST parsing failed: {}", e);
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
                info!("Detected notation systems: {:?}", detected_systems);
                info!("Generated outputs - lily: {}, vexflow: {}", 
                      !result.minimal_lilypond.is_empty(), !result.vexflow_svg.is_empty());
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
    
    let lilypond_svg = Some(format!(
        "<svg width='500' height='200' xmlns='http://www.w3.org/2000/svg'>
        <rect width='500' height='200' fill='#fffffb' stroke='#333' stroke-width='1'/>
        
        <!-- Title -->
        <text x='20' y='25' font-family='serif' font-size='16' font-weight='bold' fill='#333'>Bach-style Musical Notation (Demo)</text>
        <text x='20' y='45' font-family='monospace' font-size='11' fill='#666'>Input: {}</text>
        
        <!-- Staff lines -->
        <line x1='40' y1='80' x2='460' y2='80' stroke='#333' stroke-width='1'/>
        <line x1='40' y1='90' x2='460' y2='90' stroke='#333' stroke-width='1'/>
        <line x1='40' y1='100' x2='460' y2='100' stroke='#333' stroke-width='1'/>
        <line x1='40' y1='110' x2='460' y2='110' stroke='#333' stroke-width='1'/>
        <line x1='40' y1='120' x2='460' y2='120' stroke='#333' stroke-width='1'/>
        
        <!-- Treble clef -->
        <text x='50' y='115' font-family='serif' font-size='40' fill='#333'>𝄞</text>
        
        <!-- Notes (Bach-inspired pattern) -->
        <circle cx='100' cy='90' r='4' fill='#333'/>
        <line x1='104' y1='90' x2='104' y2='60' stroke='#333' stroke-width='2'/>
        
        <circle cx='130' cy='95' r='4' fill='#333'/>
        <line x1='134' y1='95' x2='134' y2='65' stroke='#333' stroke-width='2'/>
        
        <circle cx='160' cy='85' r='4' fill='#333'/>
        <line x1='164' y1='85' x2='164' y2='55' stroke='#333' stroke-width='2'/>
        
        <circle cx='190' cy='90' r='4' fill='#333'/>
        <line x1='194' y1='90' x2='194' y2='60' stroke='#333' stroke-width='2'/>
        
        <!-- Bar line -->
        <line x1='220' y1='80' x2='220' y2='120' stroke='#333' stroke-width='2'/>
        
        <!-- More notes -->
        <circle cx='250' cy='100' r='4' fill='#333'/>
        <line x1='254' y1='100' x2='254' y2='70' stroke='#333' stroke-width='2'/>
        
        <circle cx='280' cy='105' r='4' fill='#333'/>
        <line x1='284' y1='105' x2='284' y2='75' stroke='#333' stroke-width='2'/>
        
        <circle cx='310' cy='95' r='4' fill='#333'/>
        <line x1='314' y1='95' x2='314' y2='65' stroke='#333' stroke-width='2'/>
        
        <!-- Time signature -->
        <text x='80' y='95' font-family='serif' font-size='18' fill='#333'>4</text>
        <text x='80' y='110' font-family='serif' font-size='18' fill='#333'>4</text>
        
        <!-- Footer -->
        <text x='20' y='160' font-family='serif' font-size='12' fill='#666'>Generated by Music Text Parser</text>
        <text x='20' y='180' font-family='serif' font-size='10' fill='#999'>In the style of J.S. Bach • Placeholder SVG</text>
        </svg>", 
        input.chars().take(40).collect::<String>()
    ));
    
    Json(ParseResponse {
        success: true,
        pest_output: pest_result,
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

// parse_document function removed - now integrated into parse_text

#[tokio::main]
async fn main() {
    // Create fresh development.log file and set up file logging
    let log_file_path = "development.log";
    
    // Initialize logger to write to stdout (env_logger doesn't directly support file output)
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .target(env_logger::Target::Stdout)
        .format(|buf, record| {
            use std::io::Write;
            let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");
            writeln!(buf, "[{}] {} - {}", timestamp, record.level(), record.args())?;
            
            // Also write to development.log file
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("development.log") {
                writeln!(file, "[{}] {} - {}", timestamp, record.level(), record.args()).ok();
            }
            Ok(())
        })
        .init();
    
    // Create fresh log file with header
    if let Ok(mut file) = File::create(log_file_path) {
        writeln!(file, "=== Music Text Parser Development Log - {} ===", 
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")).ok();
    }
    
    info!("Music Text Parser server starting up");
    
    let app = Router::new()
        .route("/api/parse", get(parse_text))
        .nest_service("/", ServeDir::new("webapp"))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
        .await
        .unwrap();
        
    info!("Server binding successful on 127.0.0.1:3001");
    println!("Server running on http://127.0.0.1:3001");
    
    axum::serve(listener, app).await.unwrap();
}