use axum::{
    extract::Json,
    http::{HeaderValue, Method, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
};
use serde_json;

use crate::renderers::svg::{Document, SvgRenderer, SvgRendererConfig};

/// HTTP API for SVG rendering
pub fn create_svg_router() -> Router {
    Router::new()
        .route("/render/svg", post(render_svg))
        .route("/svg/health", get(health_check))
        .nest_service("/fonts", ServeDir::new("assets/fonts"))
        .layer(
            CorsLayer::new()
                .allow_origin("*".parse::<HeaderValue>().unwrap())
                .allow_headers([axum::http::header::CONTENT_TYPE])
                .allow_methods([Method::GET, Method::POST])
        )
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    "SVG Renderer API is running"
}

/// SVG rendering endpoint
async fn render_svg(Json(doc): Json<Document>) -> impl IntoResponse {
    match render_document_to_svg(doc) {
        Ok(svg) => Html(svg).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, err).into_response(),
    }
}

/// Core rendering function
fn render_document_to_svg(doc: Document) -> Result<String, String> {
    let config = SvgRendererConfig::default();
    let mut renderer = SvgRenderer::new(config);
    renderer.render(&doc)
}

/// Example usage and test helper
pub fn create_test_document() -> Document {
    use crate::renderers::svg::{Element, Ornament, OrnamentNote};

    Document {
        title: Some("Test Piece".to_string()),
        composer: Some("Test Composer".to_string()),
        notation_type: "number".to_string(),
        font_size: 14.0,
        supports_utf8: true,
        elements: vec![
            Element::Pitch {
                value: "1".to_string(),
                octave: 1,
                accidental: Some("sharp".to_string()),
                ornaments: vec![
                    Ornament::OnTopGraceNotes {
                        notes: vec![
                            OrnamentNote { value: "N".to_string(), octave: 0, accidental: None },
                            OrnamentNote { value: "R".to_string(), octave: 0, accidental: None },
                            OrnamentNote { value: "S".to_string(), octave: 0, accidental: None },
                        ],
                    },
                ],
                lyrics: vec!["La".to_string()],
            },
            Element::Dash { is_rest: false },
            Element::Pitch {
                value: "2".to_string(),
                octave: 0,
                accidental: None,
                ornaments: vec![
                    Ornament::SymbolicOrnament {
                        symbol: "mordent".to_string(),
                    },
                ],
                lyrics: vec!["la".to_string()],
            },
            Element::Barline {
                style: "single".to_string(),
            },
        ],
    }
}

/// Generate test SVG for development
pub fn generate_test_svg() -> Result<String, String> {
    let doc = create_test_document();
    render_document_to_svg(doc)
}