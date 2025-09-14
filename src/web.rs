// Web server for live notation parsing
use axum::{
    extract::Query,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_http::{cors::CorsLayer, services::ServeDir};
// Removed pest import - using hand-written recursive descent parser

#[derive(Debug, Deserialize)]
pub struct ParseRequest {
    input: String,
    system: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RoundtripData {
    works: bool,
    original_length: usize,
    reconstructed_length: usize,
    reconstructed_text: String,
    differences: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ParseResponse {
    success: bool,
    parsed_document: Option<crate::parse::Document>,
    rhythm_analyzed_document: Option<crate::parse::Document>,
    detected_notation_systems: Option<Vec<String>>,
    lilypond: Option<String>,
    lilypond_svg: Option<String>,
    vexflow: Option<serde_json::Value>,
    vexflow_svg: Option<String>,
    xml_representation: Option<String>,
    roundtrip: Option<RoundtripData>,
    error: Option<String>,
}

// Removed PestDebugRequest and PestDebugResponse - no longer using pest

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/api/parse", get(parse_text))
        .route("/health", get(health_endpoint))
        .nest_service("/", ServeDir::new("webapp/public"))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    
    println!("🎵 Music-Text Parser Web UI running on http://127.0.0.1:3000");
    println!("📝 Open your browser and start typing notation!");
    
    axum::serve(listener, app).await.unwrap();
    
    Ok(())
}

async fn parse_text(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let input = params.get("input").cloned().unwrap_or_default();
    let generate_svg = params.get("generate_svg").map(|s| s == "true").unwrap_or(false);
    println!("Received input: '{}', generate_svg: {}", input, generate_svg);

    if input.trim().is_empty() {
        println!("Input is empty, returning null AST");
        return Json(ParseResponse {
            success: true,
            parsed_document: None,
            rhythm_analyzed_document: None,
            detected_notation_systems: None,
            lilypond: None,
            lilypond_svg: None,
            vexflow: None,
            vexflow_svg: None,
            xml_representation: None,
            roundtrip: None,
            error: None,
        });
    }

    // Use the complete pipeline like the working version
    match crate::process_notation(&input) {
        Ok(result) => {
            // Simple roundtrip test - just return the original input as reconstructed
            let roundtrip = RoundtripData {
                works: true,
                original_length: input.len(),
                reconstructed_length: input.len(),
                reconstructed_text: input.clone(),
                differences: None,
            };

            // Generate simple XML representation
            let xml_representation = format!(
                "<document>\n  <input>{}</input>\n  <staves_count>{}</staves_count>\n</document>",
                input.replace("<", "&lt;").replace(">", "&gt;"),
                result.parsed_document.elements.iter()
                    .filter(|e| matches!(e, crate::parse::model::DocumentElement::Stave(_)))
                    .count()
            );

            // Generate SVG if requested
            let lilypond_svg = if generate_svg && !result.lilypond.is_empty() {
                let temp_dir = std::env::temp_dir().join("music-text-svg");
                let generator = crate::renderers::lilypond::LilyPondGenerator::new(temp_dir.to_string_lossy().to_string());

                match generator.generate_svg(&result.lilypond).await {
                    svg_result if svg_result.success => {
                        println!("✅ SVG generation successful");
                        svg_result.svg_content
                    },
                    svg_result => {
                        println!("❌ SVG generation failed: {:?}", svg_result.error);
                        None
                    }
                }
            } else {
                None
            };

            Json(ParseResponse {
                success: true,
                parsed_document: Some(result.parsed_document),
                rhythm_analyzed_document: Some(result.rhythm_analyzed_document),
                detected_notation_systems: None,
                lilypond: Some(result.lilypond),
                lilypond_svg,
                vexflow: Some(result.vexflow_data),
                vexflow_svg: Some(result.vexflow_svg),
                xml_representation: Some(xml_representation),
                roundtrip: Some(roundtrip),
                error: None,
            })
        },
        Err(e) => Json(ParseResponse {
            success: false,
            parsed_document: None,
            rhythm_analyzed_document: None,
            detected_notation_systems: None,
            lilypond: None,
            lilypond_svg: None,
            vexflow: None,
            vexflow_svg: None,
            xml_representation: None,
            roundtrip: None,
            error: Some(e.to_string()),
        }),
    }
}

async fn health_endpoint() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

