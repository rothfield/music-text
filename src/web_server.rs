// Web server for the pest-based music-text parser
// Provides REST API endpoints for parsing and conversion

use axum::{
    extract::Query,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use crate::{parse_notation_full, renderers::lilypond::LilyPondGenerator};

#[derive(Debug, Deserialize)]
pub struct ParseRequest {
    pub input: String,  // The notation text to parse
    pub notation: Option<String>,  // The notation system (auto, sargam, number, western, etc.)
    pub output: Option<Vec<String>>, // ["ast", "vexflow", "lilypond", "yaml"]
}

#[derive(Debug, Serialize)]
pub struct ParseResponse {
    pub success: bool,
    pub error: Option<String>,
    pub ast: Option<String>,
    pub spatial: Option<String>,
    pub fsm: Option<String>,
    pub vexflow: Option<serde_json::Value>,
    pub lilypond: Option<String>,
    pub yaml: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct HealthQuery {
    pub detailed: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct LilyPondSvgRequest {
    pub input: String,  // The notation text to parse
    pub notation: Option<String>,  // The notation system (auto, sargam, number, western, etc.)
}

#[derive(Debug, Serialize)]
pub struct LilyPondSvgResponse {
    pub success: bool,
    pub error: Option<String>,
    pub svg_url: Option<String>,
    pub lilypond_source: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub parser: String,
    pub version: String,
    pub endpoints: Vec<String>,
}

// Health check endpoint
async fn health_check(Query(params): Query<HealthQuery>) -> Json<HealthResponse> {
    let detailed = params.detailed.unwrap_or(false);
    
    let mut endpoints = vec![
        "/health".to_string(),
        "/api/parse".to_string(),
    ];
    
    if detailed {
        endpoints.extend([
            "/api/parse/ast".to_string(),
            "/api/parse/full".to_string(),
        ]);
    }
    
    Json(HealthResponse {
        status: "healthy".to_string(),
        parser: "pest-grammar".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        endpoints,
    })
}

// Main parsing endpoint
async fn parse_notation_endpoint(Json(payload): Json<ParseRequest>) -> Result<Json<ParseResponse>, StatusCode> {
    let system = payload.notation.as_deref().unwrap_or("auto");
    let outputs = payload.output.unwrap_or_else(|| vec!["ast".to_string()]);
    
    // Don't remove empty lines - they're semantically important for separating staves
    // Just use the notation as-is
    let notation_to_parse = &payload.input;
    
    // Parse with structure-preserving FSM
    let full_result = parse_notation_full(notation_to_parse, Some(system));
    if full_result.success {
        let mut response = ParseResponse {
            success: true,
            error: None,
            ast: None,
            spatial: None,
            fsm: None,
            vexflow: None,
            lilypond: None,
            yaml: None,
        };
        
        // Add requested outputs
        for output_type in outputs {
            match output_type.as_str() {
                "ast" => {
                    response.ast = full_result.ast.clone();
                }
                "yaml" => {
                    response.yaml = full_result.yaml.clone();
                }
                "full" => {
                    // Use the full parse result data we already have
                    response.ast = full_result.ast.clone();
                    response.spatial = full_result.spatial.clone();
                    response.fsm = full_result.fsm.clone();
                    response.yaml = full_result.yaml.clone();
                    response.lilypond = full_result.lilypond.clone();
                    if let Some(vf) = &full_result.vexflow {
                        response.vexflow = Some(serde_json::to_value(vf).unwrap_or(serde_json::Value::Null));
                    }
                }
                "lilypond" => {
                    // Use LilyPond output from full result we already have
                    response.lilypond = full_result.lilypond.clone();
                }
                    "vexflow" => {
                        // VexFlow output temporarily disabled due to compilation issues
                        eprintln!("Warning: VexFlow output temporarily disabled");
                    }
                    _ => {
                        eprintln!("Warning: Unknown output type '{}'", output_type);
                    }
                }
            }
            
            Ok(Json(response))
    } else {
        Ok(Json(ParseResponse {
            success: false,
            error: Some(full_result.error_message.unwrap_or("Unknown parse error".to_string())),
            ast: None,
            spatial: None,
            fsm: None,
            vexflow: None,
            lilypond: None,
            yaml: None,
        }))
    }
}

// Simple AST-only endpoint
async fn parse_ast_only(Json(payload): Json<ParseRequest>) -> Result<Json<serde_json::Value>, StatusCode> {
    let system = payload.notation.as_deref().unwrap_or("auto");
    
    // Remove empty lines and lines containing only whitespace
    let cleaned_notation = payload.input
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    
    let full_result = parse_notation_full(&cleaned_notation, Some(system));
    if full_result.success {
        if let Some(document) = full_result.document {
            Ok(Json(serde_json::to_value(&document).unwrap_or(serde_json::Value::Null)))
        } else {
            let error_response = serde_json::json!({
                "error": "No document generated",
                "success": false
            });
            Ok(Json(error_response))
        }
    } else {
        let error_response = serde_json::json!({
            "error": full_result.error_message.unwrap_or("Unknown parse error".to_string()),
            "success": false
        });
        Ok(Json(error_response))
    }
}

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 Starting Unified Music-Text Parser Server (Development Mode)");
    
    // Update cache-busting versions for web assets
    println!("🔄 Updating cache-busting versions...");
    if let Err(e) = update_cache_bust().await {
        eprintln!("⚠️ Warning: Failed to update cache-busting versions: {}", e);
    }
    
    println!("📍 Web UI: http://127.0.0.1:3000");
    println!("📍 API: http://127.0.0.1:3000/api/parse");
    println!("🛑 Press Ctrl+C to stop");
    println!();

    // Build the router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/parse", post(parse_notation_endpoint))
        .route("/api/parse/ast", post(parse_ast_only))
        .route("/api/parse/full", post(parse_notation_endpoint))
        .route("/api/lilypond/svg", post(generate_lilypond_svg))
        .route("/api/lilypond/minimal", post(generate_minimal_lilypond))
        // Serve static files from webapp/public
        .nest_service("/", ServeDir::new("webapp/public"))
        .layer(CorsLayer::permissive()); // Enable CORS for frontend requests

    // Start the server
    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    println!("🚀 Unified server running on http://127.0.0.1:3000");
    println!("📋 Available:");
    println!("  🌐 Web UI: http://127.0.0.1:3000/");
    println!("  ⚕️  Health: GET /health");
    println!("  🎼 Parse: POST /api/parse");
    println!("  🎯 AST Only: POST /api/parse/ast");
    println!("  📊 Full Parse: POST /api/parse/full");
    println!();

    axum::serve(listener, app).await?;
    
    Ok(())
}

// LilyPond SVG generation endpoint
async fn generate_lilypond_svg(Json(payload): Json<LilyPondSvgRequest>) -> Result<Json<LilyPondSvgResponse>, StatusCode> {
    let system = payload.notation.as_deref().unwrap_or("auto");
    
    // First parse to get LilyPond source
    match parse_notation_full(&payload.input, Some(system)) {
        result if result.success => {
            if let Some(full_lilypond_source) = &result.lilypond {
                // Generate SVG using dedicated generator
                let generator = LilyPondGenerator::new("webapp/public".to_string());
                let generation_result = generator.generate_svg(full_lilypond_source).await;
                
                // Extract minimal LilyPond source for API response (just the musical content)
                let minimal_source = extract_minimal_lilypond(&full_lilypond_source);
                
                Ok(Json(LilyPondSvgResponse {
                    success: generation_result.success,
                    error: generation_result.error,
                    svg_url: generation_result.svg_url,
                    lilypond_source: Some(minimal_source),
                }))
            } else {
                Ok(Json(LilyPondSvgResponse {
                    success: false,
                    error: Some("LilyPond source generation failed".to_string()),
                    svg_url: None,
                    lilypond_source: None,
                }))
            }
        },
        result => Ok(Json(LilyPondSvgResponse {
            success: false,
            error: result.error_message,
            svg_url: None,
            lilypond_source: None,
        }))
    }
}

// Minimal LilyPond source endpoint
async fn generate_minimal_lilypond(Json(payload): Json<LilyPondSvgRequest>) -> Result<Json<serde_json::Value>, StatusCode> {
    let system = payload.notation.as_deref().unwrap_or("auto");
    
    match parse_notation_full(&payload.input, Some(system)) {
        result if result.success => {
            if let Some(full_source) = &result.lilypond {
                let minimal_source = extract_minimal_lilypond(full_source);
                Ok(Json(serde_json::json!({
                    "success": true,
                    "lilypond_source": minimal_source
                })))
            } else {
                Ok(Json(serde_json::json!({
                    "success": false,
                    "error": "LilyPond source generation failed"
                })))
            }
        },
        result => Ok(Json(serde_json::json!({
            "success": false,
            "error": result.error_message.unwrap_or("Parse error".to_string())
        })))
    }
}

// Extract the complete score section from full LilyPond source
fn extract_minimal_lilypond(full_source: &str) -> String {
    // Find the \score { ... } block
    if let Some(score_start) = full_source.find("\\score {") {
        // Find the matching closing brace for the score
        let mut brace_count = 0;
        let mut score_end = score_start + "\\score {".len();
        let chars: Vec<char> = full_source.chars().collect();
        
        // Start with 1 because we already have the opening brace
        brace_count = 1;
        
        for i in (score_start + "\\score {".len())..chars.len() {
            match chars[i] {
                '{' => brace_count += 1,
                '}' => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        score_end = i + 1;
                        break;
                    }
                },
                _ => {}
            }
        }
        
        if brace_count == 0 {
            let score_section = &full_source[score_start..score_end];
            return score_section.to_string();
        }
    }
    
    // Fallback - try to extract just the musical content
    if let Some(start) = full_source.find("\\relative c' {") {
        let content_start = start + "\\relative c' {".len();
        if let Some(mut end) = full_source[content_start..].find("    }") {
            end += content_start;
            let musical_content = &full_source[content_start..end];
            
            // Clean up the content - remove leading/trailing whitespace and settings
            let lines: Vec<&str> = musical_content
                .lines()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty() && 
                              !line.starts_with("\\key") && 
                              !line.starts_with("\\time") &&
                              !line.starts_with("\\autoBeamOff") &&
                              !line.starts_with("\\set"))
                .collect();
            
            if !lines.is_empty() {
                return lines.join(" ");
            }
        }
    }
    
    // Final fallback - return just a comment
    "% Unable to extract musical content".to_string()
}

// Update cache-busting versions by running the Node.js script
async fn update_cache_bust() -> Result<(), Box<dyn std::error::Error>> {
    use tokio::process::Command;
    use std::env;
    
    // Get the current working directory and ensure we're in the right place
    let current_dir = env::current_dir()?;
    let webapp_dir = current_dir.join("webapp");
    
    if !webapp_dir.exists() {
        return Err("webapp directory not found".into());
    }
    
    let output = Command::new("node")
        .arg("update-cache-bust.js")
        .current_dir(&webapp_dir)
        .output()
        .await?;
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Cache-bust script failed: {}", error).into());
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.trim().is_empty() {
        println!("{}", stdout.trim());
    }
    
    Ok(())
}

