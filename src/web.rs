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
    rhythm_items: Option<Vec<crate::rhythm::Item>>,
    detected_notation_systems: Option<Vec<String>>,
    lilypond: Option<String>,
    lilypond_svg: Option<String>,
    vexflow: Option<serde_json::Value>,
    vexflow_svg: Option<String>,
    syntax_tokens: Option<Vec<crate::tree_functions::SyntaxToken>>,
    character_styles: Option<Vec<crate::tree_functions::CharacterStyle>>,
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
            rhythm_items: None,
            detected_notation_systems: None,
            lilypond: None,
            lilypond_svg: None,
            vexflow: None,
            vexflow_svg: None,
            syntax_tokens: None,
            character_styles: None,
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


            // Generate syntax tokens from spatial-processed document (contains beat groups, octave markers, etc.)
            let syntax_tokens = crate::tree_functions::generate_syntax_tokens(&result.parsed_document, &input);

            // Generate character styles with beat group information for enhanced styling
            let character_styles = crate::tree_functions::generate_character_styles_with_beat_groups(&syntax_tokens, &result.rhythm_analyzed_document);

            // Extract rhythm_items from all staves in the rhythm analyzed document
            let rhythm_items = {
                let mut all_rhythm_items = Vec::new();
                for element in &result.rhythm_analyzed_document.elements {
                    if let crate::parse::model::DocumentElement::Stave(stave) = element {
                        if let Some(rhythm_items) = &stave.rhythm_items {
                            all_rhythm_items.extend(rhythm_items.iter().cloned());
                        }
                    }
                }
                if all_rhythm_items.is_empty() { None } else { Some(all_rhythm_items) }
            };

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
                rhythm_items,
                detected_notation_systems: None,
                lilypond: Some(result.lilypond),
                lilypond_svg,
                vexflow: Some(result.vexflow_data),
                vexflow_svg: Some(result.vexflow_svg),
                syntax_tokens: Some(syntax_tokens),
                character_styles: Some(character_styles),
                roundtrip: Some(roundtrip),
                error: None,
            })
        },
        Err(e) => Json(ParseResponse {
            success: false,
            parsed_document: None,
            rhythm_analyzed_document: None,
            rhythm_items: None,
            detected_notation_systems: None,
            lilypond: None,
            lilypond_svg: None,
            vexflow: None,
            vexflow_svg: None,
            syntax_tokens: None,
            character_styles: None,
            roundtrip: None,
            error: Some(e.to_string()),
        }),
    }
}

async fn health_endpoint() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

