// Web server for live notation parsing
use axum::{
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::{cors::CorsLayer, services::ServeDir};
use pest::Parser;

#[derive(Debug, Deserialize)]
pub struct ParseRequest {
    input: String,
    system: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ParseResponse {
    success: bool,
    ast: Option<crate::ast::Document>,
    error: Option<String>,
    yaml: Option<String>,
    spatial: Option<String>,
    fsm: Option<String>,
    vexflow: Option<serde_json::Value>, // Simplified for now
    lilypond: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PestDebugRequest {
    input: String,
}

#[derive(Debug, Serialize)]
pub struct PestDebugResponse {
    success: bool,
    parse_tree: Option<String>,
    stave_count: Option<usize>,
    error: Option<String>,
}

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/api/parse", post(parse_endpoint))
        .route("/api/pest/debug", post(pest_debug_endpoint))
        .route("/health", get(health_endpoint))
        .route("/api/lilypond/minimal", post(lilypond_minimal_endpoint))
        .route("/api/lilypond/svg", post(lilypond_svg_endpoint))
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

async fn parse_endpoint(Json(request): Json<ParseRequest>) -> impl IntoResponse {
    let result = crate::parse(&request.input, request.system.as_deref());
    
    Json(ParseResponse {
        success: result.success,
        ast: result.document,
        error: result.error_message,
        yaml: result.yaml,
        spatial: result.spatial,
        fsm: result.fsm,
        vexflow: None, // VexFlow temporarily disabled
        lilypond: result.lilypond,
    })
}

async fn health_endpoint() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

#[derive(Debug, Deserialize)]
pub struct LilypondRequest {
    input: String,
    notation: String,
}

#[derive(Debug, Deserialize)]
pub struct LilypondSvgRequest {
    lilypond_source: String,
}

#[derive(Debug, Serialize)]
pub struct LilypondResponse {
    success: bool,
    lilypond_source: Option<String>,
    svg: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SvgResponse {
    success: bool,
    svg: Option<String>,
    error: Option<String>,
}

async fn lilypond_minimal_endpoint(Json(request): Json<LilypondRequest>) -> impl IntoResponse {
    // Parse the input to get LilyPond source
    let result = crate::parse(&request.input, Some(&request.notation));
    
    // Use structure-preserving FSM approach for minimal LilyPond
    let processed_doc = crate::structure_preserving_fsm::ProcessedDocument::from_document(result.document.as_ref().unwrap_or(&crate::ast::Document::new()));
    
    // Generate minimal LilyPond using the same renderer but with minimal template
    let minimal_lilypond = {
        let metadata = crate::models::Metadata {
            title: None,
            directives: Vec::new(),
            detected_system: None,
            attributes: std::collections::HashMap::new(),
        };
        crate::renderers::lilypond::renderer::convert_processed_document_to_lilypond_minimal(&processed_doc, &metadata, Some(&request.input))
            .ok()
    };
    
    if let Some(minimal_source) = minimal_lilypond {
        // Generate SVG from LilyPond source
        let svg_content = if minimal_source.trim().is_empty() {
            Some("<svg><text>No musical content to render</text></svg>".to_string())
        } else {
            // Use the LilyPond generator to create SVG
            let generator = crate::renderers::lilypond::generator::LilyPondGenerator::new("webapp/public".to_string());
            match generator.generate_svg(&minimal_source).await {
                result if result.success => {
                    if let Some(svg_url) = result.svg_url {
                        // Read the generated SVG file and return its content
                        match std::fs::read_to_string(format!("webapp/public{}", svg_url)) {
                            Ok(svg_content) => Some(svg_content),
                            Err(_) => Some("<svg><text>Failed to read generated SVG</text></svg>".to_string()),
                        }
                    } else {
                        Some("<svg><text>SVG generation failed</text></svg>".to_string())
                    }
                },
                result => Some(format!("<svg><text>LilyPond error: {}</text></svg>", 
                    result.error.unwrap_or_else(|| "Unknown error".to_string())))
            }
        };

        Json(LilypondResponse {
            success: true,
            lilypond_source: Some(minimal_source),
            svg: svg_content,
            error: None,
        })
    } else {
        Json(LilypondResponse {
            success: false,
            lilypond_source: None,
            svg: None,
            error: Some("Failed to generate minimal LilyPond source".to_string()),
        })
    }
}

async fn lilypond_svg_endpoint(Json(request): Json<LilypondSvgRequest>) -> impl IntoResponse {
    // Use the provided LilyPond source directly (no re-parsing needed)
    if request.lilypond_source.trim().is_empty() {
        return Json(SvgResponse {
            success: false,
            svg: None,
            error: Some("Empty LilyPond source provided".to_string()),
        });
    }
    
    // Use the existing LilyPond generator to create SVG
    let generator = crate::renderers::lilypond::generator::LilyPondGenerator::new("webapp/public".to_string());
    let generation_result = generator.generate_svg(&request.lilypond_source).await;
    
    if generation_result.success {
        if let Some(svg_url) = generation_result.svg_url {
            // Read the generated SVG file and return its content
            match std::fs::read_to_string(format!("webapp/public{}", svg_url)) {
                Ok(svg_content) => Json(SvgResponse {
                    success: true,
                    svg: Some(svg_content),
                    error: None,
                }),
                Err(_) => Json(SvgResponse {
                    success: false,
                    svg: None,
                    error: Some("Failed to read generated SVG file".to_string()),
                }),
            }
        } else {
            Json(SvgResponse {
                success: false,
                svg: None,
                error: Some("SVG generation succeeded but no file path returned".to_string()),
            })
        }
    } else {
        Json(SvgResponse {
            success: false,
            svg: None,
            error: generation_result.error,
        })
    }
}

async fn pest_debug_endpoint(Json(request): Json<PestDebugRequest>) -> impl IntoResponse {
    match crate::parser::MusicTextParser::parse(crate::parser::Rule::document, &request.input) {
        Ok(pairs) => {
            let mut parse_tree = String::new();
            let mut stave_count = 0;
            
            for pair in pairs {
                count_staves_in_pair(&pair, &mut stave_count);
                format_parse_tree(&pair, 0, &mut parse_tree);
            }
            
            Json(PestDebugResponse {
                success: true,
                parse_tree: Some(parse_tree),
                stave_count: Some(stave_count),
                error: None,
            })
        }
        Err(e) => {
            Json(PestDebugResponse {
                success: false,
                parse_tree: None,
                stave_count: None,
                error: Some(e.to_string()),
            })
        }
    }
}

fn count_staves_in_pair(pair: &pest::iterators::Pair<crate::parser::Rule>, stave_count: &mut usize) {
    match pair.as_rule() {
        crate::parser::Rule::stave => {
            *stave_count += 1;
        }
        _ => {
            for inner in pair.clone().into_inner() {
                count_staves_in_pair(&inner, stave_count);
            }
        }
    }
}

fn format_parse_tree(pair: &pest::iterators::Pair<crate::parser::Rule>, depth: usize, output: &mut String) {
    use std::fmt::Write;
    
    let indent = "  ".repeat(depth);
    writeln!(output, "{}Rule::{:?} -> {:?}", indent, pair.as_rule(), pair.as_str()).unwrap();
    
    for inner in pair.clone().into_inner() {
        format_parse_tree(&inner, depth + 1, output);
    }
}

