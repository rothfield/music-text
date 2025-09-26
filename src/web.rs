// Web server for live notation parsing
use axum::{
    extract::{Query, Multipart, Form, Path},
    response::{IntoResponse, Html, Response},
    routing::{get, post},
    Json, Router,
    http::{StatusCode, header},
    body::Body,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use chrono;
use tower_http::{cors::CorsLayer, services::ServeDir};
use tokio::fs;
use tokio::io::AsyncReadExt;
// Removed pest import - using hand-written recursive descent parser
use crate::pipeline;
use crate::renderers::lilypond::renderer;
use crate::renderers::editor::svg;

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
    plain_text: Option<String>,
    document: Option<crate::parse::Document>,
    detected_notation_systems: Option<Vec<String>>,
    lilypond: Option<String>,
    lilypond_minimal: Option<String>,
    lilypond_svg: Option<String>,
    vexflow: Option<serde_json::Value>,
    vexflow_svg: Option<String>,
    canvas_svg: Option<String>,  // Canvas WYSIWYG SVG
    error: Option<String>,
}

// Document-first API structures
#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub music_text: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct CreateDocumentResponse {
    pub document_id: String,
    pub document: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct DocumentModel {
    pub version: String,
    pub timestamp: String,
    pub elements: serde_json::Value,
    pub content: Vec<String>,
    pub metadata: serde_json::Value,
    pub ui_state: serde_json::Value,
}

// Document transformation structures
#[derive(Debug, Deserialize)]
pub struct TransformDocumentRequest {
    pub document: serde_json::Value,
    pub command_type: String, // "apply_slur", "set_octave", etc.
    pub target_uuids: Vec<String>,
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct TransformDocumentResponse {
    pub success: bool,
    pub document: serde_json::Value,
    pub updated_elements: Vec<String>,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExportDocumentRequest {
    pub document: serde_json::Value,
    pub format: String, // "lilypond", "svg", "midi"
    pub options: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ExportDocumentResponse {
    pub success: bool,
    pub format: String,
    pub content: String,
    pub message: Option<String>,
}

// Document storage - on-disk for durability
type DocumentStore = std::path::PathBuf;

// UUID-based API structures
#[derive(Debug, Deserialize)]
pub struct TransformDocumentByIdRequest {
    pub command_type: String,
    pub target_uuids: Vec<String>,
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ExportDocumentByIdRequest {
    pub format: String,
    pub options: Option<serde_json::Value>,
}

// Helper functions for document storage
fn get_documents_dir() -> std::path::PathBuf {
    std::path::Path::new("./documents").to_path_buf()
}

fn get_document_path(document_id: &str) -> std::path::PathBuf {
    get_documents_dir().join(format!("{}.json", document_id))
}

async fn save_document(document_id: &str, document: &serde_json::Value) -> Result<(), std::io::Error> {
    let docs_dir = get_documents_dir();
    tokio::fs::create_dir_all(&docs_dir).await?;

    let doc_path = get_document_path(document_id);
    let content = serde_json::to_string_pretty(document)?;
    tokio::fs::write(&doc_path, content).await?;
    Ok(())
}

async fn load_document(document_id: &str) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let doc_path = get_document_path(document_id);
    let content = tokio::fs::read_to_string(&doc_path).await?;
    let document: serde_json::Value = serde_json::from_str(&content)?;
    Ok(document)
}

// Removed PestDebugRequest and PestDebugResponse - no longer using pest

#[derive(Debug, Deserialize)]
pub struct RetroParseRequest {
    input: String,
    action: String,
}

#[derive(Debug, Deserialize)]
pub struct SvgPocRequest {
    input: String,
    notation_type: String,
}

#[derive(Debug, Deserialize)]
pub struct CanvasSvgRequest {
    input_text: String,
    notation_type: String,
    cursor_position: Option<usize>,
    selection_start: Option<usize>,
    selection_end: Option<usize>,
}

use crate::parse::Document;
use crate::parse::actions::{TransformRequest, TransformResponse, apply_octave_transform, apply_slur_transform};
// Template rendering helper
async fn render_retro_template(
    input: &str,
    svg_content: Option<&str>,
    lilypond_content: Option<&str>,
    error_message: Option<&str>,
    success_message: Option<&str>,
) -> String {
    let template = match fs::read_to_string("webapp/public/retro-template.html").await {
        Ok(content) => content,
        Err(_) => {
            // Fallback template if file can't be read
            r#"<!DOCTYPE html>
<html><head><title>Music Text - Retro Mode</title></head>
<body>
<h1>Music Text - Retro Mode</h1>
<form method="POST" action="/retro/parse">
<textarea name="input">{{preserved_input}}</textarea><br>
<button type="submit" name="action" value="preview">Preview</button>
</form>
{{#if_error}}<div style="color:red">{{error_message}}</div>{{/if_error}}
{{#if_success}}<div style="color:green">{{success_message}}</div>{{/if_success}}
</body></html>"#.to_string()
        }
    };

    template
        .replace("{{preserved_input}}", input)
        .replace("{{#if_results}}", if svg_content.is_some() || lilypond_content.is_some() || error_message.is_some() || success_message.is_some() { "" } else { "<!--" })
        .replace("{{/if_results}}", if svg_content.is_some() || lilypond_content.is_some() || error_message.is_some() || success_message.is_some() { "" } else { "-->" })
        .replace("{{#if_svg}}", if svg_content.is_some() { "" } else { "<!--" })
        .replace("{{/if_svg}}", if svg_content.is_some() { "" } else { "-->" })
        .replace("{{svg_content}}", svg_content.unwrap_or(""))
        .replace("{{#if_lilypond}}", if lilypond_content.is_some() { "" } else { "<!--" })
        .replace("{{/if_lilypond}}", if lilypond_content.is_some() { "" } else { "-->" })
        .replace("{{lilypond_content}}", lilypond_content.unwrap_or(""))
        .replace("{{#if_error}}", if error_message.is_some() { "" } else { "<!--" })
        .replace("{{/if_error}}", if error_message.is_some() { "" } else { "-->" })
        .replace("{{error_message}}", error_message.unwrap_or(""))
        .replace("{{#if_success}}", if success_message.is_some() { "" } else { "<!--" })
        .replace("{{/if_success}}", if success_message.is_some() { "" } else { "-->" })
        .replace("{{success_message}}", success_message.unwrap_or(""))
}

#[derive(Debug, Deserialize)]
pub struct RenderFromModelRequest {
    document: Document,
    notation_type: String,
    cursor_position: Option<usize>,
    selection_start: Option<usize>,
    selection_end: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct SplitLineRequest {
    text: String,
    cursor_position: usize,
}

#[derive(Debug, Serialize)]
pub struct SplitLineResponse {
    new_text: String,
    new_cursor_position: usize,
}

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    // Preload and validate CSS file on server startup
    match std::fs::read_to_string("assets/svg-styles.css") {
        Ok(css_content) => {
            println!("✅ Successfully loaded CSS file: {} characters", css_content.len());
            if css_content.contains(".lower-octave") {
                println!("✅ CSS contains expected lower-octave styles");
            } else {
                println!("⚠️  Warning: CSS doesn't contain expected lower-octave styles");
            }
        }
        Err(e) => {
            println!("⚠️  Warning: Could not load assets/svg-styles.css: {}", e);
            println!("   SVG rendering will use fallback CSS");
        }
    }

    // SVG router removed - using canvas SVG instead

    let app = Router::new()
        .route("/api/parse", get(parse_text))
        .route("/api/render-svg-poc", post(render_svg_poc))
        .route("/api/canvas-svg", post(render_canvas_svg))
        .route("/api/render-from-model", post(render_from_model))
        .route("/api/split-line", post(split_line_handler))
        .route("/api/transform/octave", post(transform_octave_handler))
        .route("/api/transform/slur", post(transform_slur_handler))
        .route("/api/documents", post(create_document_handler))
        .route("/api/documents/transform", post(transform_document_handler))
        .route("/api/documents/export", post(export_document_handler))
        .route("/api/documents/:document_id/transform", post(transform_document_by_id_handler))
        .route("/api/documents/:document_id/export", post(export_document_by_id_handler))
        .route("/retro/load", post(retro_load))
        .route("/retro", post(retro_parse).get(serve_retro_page))
        .route("/retro/", get(serve_retro_page))
        .route("/health", get(health_endpoint))
        .nest_service("/assets", ServeDir::new("assets"))
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

async fn render_from_model(Json(request): Json<RenderFromModelRequest>) -> impl IntoResponse {
    // In this new flow, the document *is* the source of truth.
    // The original `input_text` is less relevant but the renderer uses it for placeholders.
    // We can pass an empty string as a stand-in.
    let placeholder_text = "";

    let svg_content = match crate::renderers::editor::render_canvas_svg(
        &request.document,
        &request.notation_type,
        placeholder_text,
        request.cursor_position,
        request.selection_start,
        request.selection_end,
    ) {
        Ok(svg) => svg,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("SVG generation failed: {}", err),
            )
                .into_response()
        }
    };
    Html(svg_content).into_response()
}

async fn split_line_handler(Json(request): Json<SplitLineRequest>) -> impl IntoResponse {
    let (new_text, new_cursor_position) =
        crate::parse::actions::split_line_at_cursor(&request.text, request.cursor_position);

    let response = SplitLineResponse {
        new_text,
        new_cursor_position,
    };

    Json(response)
}

async fn transform_octave_handler(Json(request): Json<TransformRequest>) -> impl IntoResponse {
    let octave_type = request.octave_type.as_deref().unwrap_or("higher");

    let response = apply_octave_transform(
        &request.text,
        request.selection_start,
        request.selection_end,
        octave_type,
    );

    Json(response)
}

async fn transform_slur_handler(Json(request): Json<TransformRequest>) -> impl IntoResponse {
    let response = apply_slur_transform(
        &request.text,
        request.selection_start,
        request.selection_end,
    );

    Json(response)
}

// Document creation endpoint
async fn create_document_handler(Json(request): Json<CreateDocumentRequest>) -> impl IntoResponse {
    let document_id = Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();

    // Create empty document model
    let mut document = serde_json::json!({
        "version": "1.0.0",
        "timestamp": timestamp,
        "elements": {},
        "content": [],
        "metadata": request.metadata.unwrap_or(serde_json::json!({})),
        "ui_state": {
            "selection": {
                "selected_uuids": [],
                "cursor_uuid": null,
                "cursor_position": 0
            },
            "viewport": {
                "scroll_x": 0,
                "scroll_y": 0,
                "zoom_level": 1.0
            },
            "editor_mode": "text",
            "active_tab": "vexflow"
        },
        "format_cache": {
            "music_text": request.music_text,
            "lilypond": null,
            "svg": null,
            "midi": null
        }
    });

    // If music_text was provided, we could parse it here to populate elements
    // For now, just return the empty document structure

    // Save document to disk
    if let Err(e) = save_document(&document_id, &document).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Failed to save document: {}", e)}))
        ).into_response();
    }

    let response = CreateDocumentResponse {
        document_id: document_id.clone(),
        document: document,
    };

    Json(response).into_response()
}

// Document transformation endpoint
async fn transform_document_handler(Json(request): Json<TransformDocumentRequest>) -> impl IntoResponse {
    // For now, implement basic transformations
    // In a full implementation, this would parse the document JSON back to the Document model,
    // apply the transformation, and return the updated document

    let mut updated_document = request.document.clone();
    let updated_elements = request.target_uuids.clone();

    match request.command_type.as_str() {
        "apply_slur" => {
            // For demonstration, just add slur metadata to the document
            if let Some(metadata) = updated_document.get_mut("metadata") {
                if let Some(meta_obj) = metadata.as_object_mut() {
                    let slur_count = meta_obj.get("slur_count").and_then(|v| v.as_u64()).unwrap_or(0);
                    meta_obj.insert("slur_count".to_string(), serde_json::Value::Number(serde_json::Number::from(slur_count + 1)));
                }
            }
        }
        "set_octave" => {
            // For demonstration, add octave metadata
            if let Some(metadata) = updated_document.get_mut("metadata") {
                if let Some(meta_obj) = metadata.as_object_mut() {
                    if let Some(params) = &request.parameters {
                        if let Some(octave_type) = params.get("octave_type") {
                            meta_obj.insert("last_octave_change".to_string(), octave_type.clone());
                        }
                    }
                }
            }
        }
        _ => {
            return Json(TransformDocumentResponse {
                success: false,
                document: request.document,
                updated_elements: vec![],
                message: Some(format!("Unknown command type: {}", request.command_type)),
            });
        }
    }

    // Update timestamp
    if let Some(timestamp_field) = updated_document.get_mut("timestamp") {
        *timestamp_field = serde_json::Value::String(chrono::Utc::now().to_rfc3339());
    }

    Json(TransformDocumentResponse {
        success: true,
        document: updated_document,
        updated_elements,
        message: Some(format!("Applied {} to {} elements", request.command_type, request.target_uuids.len())),
    })
}

// Document export endpoint
async fn export_document_handler(Json(request): Json<ExportDocumentRequest>) -> impl IntoResponse {
    match request.format.as_str() {
        "lilypond" => {
            // Extract music_text from document format_cache and convert to LilyPond
            if let Some(format_cache) = request.document.get("format_cache") {
                if let Some(music_text) = format_cache.get("music_text") {
                    if let Some(text_str) = music_text.as_str() {
                        if !text_str.is_empty() {
                            match pipeline::process_notation(text_str) {
                                Ok(result) => {
                                    let minimal_lilypond = match renderer::convert_processed_document_to_minimal_lilypond_src(&result.document, Some(text_str)) {
                                        Ok(lily) => lily,
                                        Err(e) => return Json(ExportDocumentResponse {
                                            success: false,
                                            format: request.format,
                                            content: "".to_string(),
                                            message: Some(format!("LilyPond conversion failed: {}", e)),
                                        }),
                                    };

                                    return Json(ExportDocumentResponse {
                                        success: true,
                                        format: request.format,
                                        content: minimal_lilypond,
                                        message: Some("Exported to LilyPond successfully".to_string()),
                                    });
                                }
                                Err(e) => return Json(ExportDocumentResponse {
                                    success: false,
                                    format: request.format,
                                    content: "".to_string(),
                                    message: Some(format!("Parse failed: {}", e)),
                                }),
                            }
                        }
                    }
                }
            }

            Json(ExportDocumentResponse {
                success: false,
                format: request.format,
                content: "".to_string(),
                message: Some("No music_text found in document format_cache".to_string()),
            })
        }
        "svg" => {
            // Similar to LilyPond but export to SVG
            if let Some(format_cache) = request.document.get("format_cache") {
                if let Some(music_text) = format_cache.get("music_text") {
                    if let Some(text_str) = music_text.as_str() {
                        if !text_str.is_empty() {
                            match pipeline::process_notation(text_str) {
                                Ok(result) => {
                                    match svg::render_canvas_svg(
                                        &result.document,
                                        "number", // notation type - could be extracted from document
                                        text_str,
                                        None, None, None // cursor/selection - could be extracted from ui_state
                                    ) {
                                        Ok(svg) => return Json(ExportDocumentResponse {
                                            success: true,
                                            format: request.format,
                                            content: svg,
                                            message: Some("Exported to SVG successfully".to_string()),
                                        }),
                                        Err(e) => return Json(ExportDocumentResponse {
                                            success: false,
                                            format: request.format,
                                            content: "".to_string(),
                                            message: Some(format!("SVG render failed: {}", e)),
                                        }),
                                    }
                                }
                                Err(e) => return Json(ExportDocumentResponse {
                                    success: false,
                                    format: request.format,
                                    content: "".to_string(),
                                    message: Some(format!("Parse failed: {}", e)),
                                }),
                            }
                        }
                    }
                }
            }

            Json(ExportDocumentResponse {
                success: false,
                format: request.format,
                content: "".to_string(),
                message: Some("No music_text found in document format_cache".to_string()),
            })
        }
        _ => {
            let format_str = request.format.clone();
            Json(ExportDocumentResponse {
                success: false,
                format: request.format,
                content: "".to_string(),
                message: Some(format!("Unsupported export format: {}", format_str)),
            })
        }
    }
}

// UUID-based transform handler
async fn transform_document_by_id_handler(
    Path(document_id): Path<String>,
    Json(request): Json<TransformDocumentByIdRequest>
) -> impl IntoResponse {
    // Load document from disk
    let mut document = match load_document(&document_id).await {
        Ok(doc) => doc,
        Err(e) => return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("Document not found: {}", e)}))
        ).into_response()
    };

    // Apply transformation (same logic as before, but simpler since we have the document)
    let timestamp = chrono::Utc::now().to_rfc3339();
    document["timestamp"] = serde_json::Value::String(timestamp);

    // Update metadata based on command type
    if let Some(metadata) = document.get_mut("metadata") {
        match request.command_type.as_str() {
            "apply_slur" => {
                let slur_count = metadata.get("slur_count").and_then(|v| v.as_u64()).unwrap_or(0);
                metadata.as_object_mut().unwrap().insert("slur_count".to_string(), serde_json::Value::Number(serde_json::Number::from(slur_count + 1)));
            }
            "set_octave" => {
                if let Some(params) = &request.parameters {
                    if let Some(octave) = params.get("octave") {
                        metadata.as_object_mut().unwrap().insert("default_octave".to_string(), octave.clone());
                    }
                }
            }
            _ => {}
        }
    }

    // Save updated document back to disk
    if let Err(e) = save_document(&document_id, &document).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Failed to save document: {}", e)}))
        ).into_response();
    }

    Json(TransformDocumentResponse {
        success: true,
        document: document,
        message: Some(format!("Applied {} to {} elements", request.command_type, request.target_uuids.len())),
        updated_elements: Vec::new(), // Would contain actual updated element UUIDs in full implementation
    }).into_response()
}

// UUID-based export handler
async fn export_document_by_id_handler(
    Path(document_id): Path<String>,
    Json(request): Json<ExportDocumentByIdRequest>
) -> impl IntoResponse {
    // Load document from disk
    let document = match load_document(&document_id).await {
        Ok(doc) => doc,
        Err(e) => return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("Document not found: {}", e)}))
        ).into_response()
    };

    // Use existing export logic
    let export_request = ExportDocumentRequest {
        document: document,
        format: request.format,
        options: request.options,
    };

    export_document_handler(Json(export_request)).await.into_response()
}

async fn parse_text(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let input = params.get("input").cloned().unwrap_or_default();
    let generate_svg = params.get("generate_svg").map(|s| s == "true").unwrap_or(false);
    let notation_type = params.get("notation_type").cloned().unwrap_or_else(|| "number".to_string());
    println!("Received input: '{}', generate_svg: {}, notation_type: {}", input, generate_svg, notation_type);

    if input.trim().is_empty() {
        println!("Input is empty, returning null AST");
        return Json(ParseResponse {
            success: true,
            plain_text: Some(input.clone()),
            document: None,
            detected_notation_systems: None,
            lilypond: None,
            lilypond_minimal: None,
            lilypond_svg: None,
            vexflow: None,
            vexflow_svg: None,
            canvas_svg: None,
            error: None,
        });
    }

    // Use the complete pipeline like the working version
    match crate::process_notation(&input) {
        Ok(result) => {



            // Extract beats from all ContentLine elements in staves

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


            // Generate canvas SVG for WYSIWYG editor using editor SVG renderer
            let canvas_svg = crate::renderers::editor::render_canvas_svg(&result.document, &notation_type, &input, None, None, None).ok();

            // Generate minimal lilypond before moving document
            let lilypond_minimal = crate::renderers::lilypond::renderer::convert_processed_document_to_minimal_lilypond_src(&result.document, Some(&input)).ok();

            Json(ParseResponse {
                success: true,
                plain_text: Some(input.clone()),
                document: Some(result.document),
                detected_notation_systems: None,
                lilypond: Some(result.lilypond.clone()),
                lilypond_minimal,
                lilypond_svg,
                vexflow: Some(result.vexflow_data),
                vexflow_svg: Some(result.vexflow_svg),
                canvas_svg,
                error: None,
            })
        },
        Err(e) => Json(ParseResponse {
            success: false,
            plain_text: Some(input.clone()),
            document: None,
            detected_notation_systems: None,
            lilypond: None,
            lilypond_minimal: None,
            lilypond_svg: None,
            vexflow: None,
            vexflow_svg: None,
            canvas_svg: None,
            error: Some(e.to_string()),
        }),
    }
}

async fn health_endpoint() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

/// SVG POC rendering endpoint - converts text to SVG via document conversion
async fn render_svg_poc(Json(request): Json<SvgPocRequest>) -> impl IntoResponse {
    println!("SVG POC request: input_len={}, notation_type={}",
             request.input.len(), request.notation_type);

    // Step 1: Parse the input text to get a document
    match crate::pipeline::process_notation(&request.input) {
        Ok(result) => {
            // Step 2: Use the SVG renderer module to generate SVG
            let svg_content = match crate::renderers::editor::render_canvas_svg(&result.document, &request.notation_type, &request.input, None, None, None) {
                Ok(svg) => svg,
                Err(err) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("SVG generation failed: {}", err)).into_response()
            };

            println!("✅ SVG POC generation successful, length: {}", svg_content.len());
            Html(svg_content).into_response()
        }
        Err(err) => {
            println!("❌ Parsing failed: {}", err);
            (StatusCode::BAD_REQUEST, format!("Parsing failed: {}", err)).into_response()
        }
    }
}

// New API endpoint for canvas SVG with cursor position support
async fn render_canvas_svg(Json(request): Json<CanvasSvgRequest>) -> impl IntoResponse {
    println!("Canvas SVG request: input_len={}, notation_type={}, cursor_position={:?}",
             request.input_text.len(), request.notation_type, request.cursor_position);

    // Parse the input text to get a document
    match crate::pipeline::process_notation(&request.input_text) {
        Ok(result) => {
            // Use the SVG renderer with cursor position and selection
            let svg_content = match crate::renderers::editor::render_canvas_svg(
                &result.document,
                &request.notation_type,
                &request.input_text,
                request.cursor_position,
                request.selection_start,
                request.selection_end
            ) {
                Ok(svg) => svg,
                Err(err) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Canvas SVG generation failed: {}", err)).into_response()
            };
            println!("✅ Canvas SVG generation successful, length: {}", svg_content.len());
            Html(svg_content).into_response()
        }
        Err(err) => {
            println!("❌ Parsing failed: {}", err);
            (StatusCode::BAD_REQUEST, format!("Parsing failed: {}", err)).into_response()
        }
    }
}

// Serve the retro page at /retro
async fn serve_retro_page() -> impl IntoResponse {
    match fs::read_to_string("webapp/public/retro.html").await {
        Ok(content) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html")
            .body(Body::from(content))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header(header::CONTENT_TYPE, "text/plain")
            .body(Body::from("Retro page not found"))
            .unwrap()
    }
}

// Retro mode form handling
async fn retro_parse(Form(form_data): Form<RetroParseRequest>) -> impl IntoResponse {
    let input = form_data.input;
    let action = form_data.action;

    println!("Retro parse request: action={}, input_len={}", action, input.len());

    match action.as_str() {
        "preview" => {
            // Generate preview with SVG
            match crate::process_notation(&input) {
                Ok(result) => {
                    let lilypond_svg = if !result.lilypond.is_empty() {
                        let temp_dir = std::env::temp_dir().join("music-text-svg");
                        let generator = crate::renderers::lilypond::LilyPondGenerator::new(temp_dir.to_string_lossy().to_string());

                        match generator.generate_svg(&result.lilypond).await {
                            svg_result if svg_result.success => svg_result.svg_content,
                            _ => None
                        }
                    } else {
                        None
                    };

                    let html = render_retro_template(
                        &input,
                        lilypond_svg.as_deref(),
                        Some(&result.lilypond),
                        None,
                        Some("Notation parsed successfully!")
                    ).await;
                    Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/html")
                        .body(Body::from(html))
                        .unwrap()
                }
                Err(e) => {
                    let html = render_retro_template(
                        &input,
                        None,
                        None,
                        Some(&e.to_string()),
                        None
                    ).await;
                    Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/html")
                        .body(Body::from(html))
                        .unwrap()
                }
            }
        }
        "save_pdf" => {
            // Generate PDF and return as download
            match crate::process_notation(&input) {
                Ok(result) if !result.lilypond.is_empty() => {
                    let temp_dir = std::env::temp_dir().join("music-text-pdf");
                    let generator = crate::renderers::lilypond::LilyPondGenerator::new(temp_dir.to_string_lossy().to_string());

                    match generator.generate_pdf(&result.lilypond).await {
                        pdf_result if pdf_result.success => {
                            if let Some(pdf_data) = pdf_result.pdf_data {
                                return Response::builder()
                                    .status(StatusCode::OK)
                                    .header(header::CONTENT_TYPE, "application/pdf")
                                    .header(header::CONTENT_DISPOSITION, "attachment; filename=\"score.pdf\"")
                                    .body(Body::from(pdf_data))
                                    .unwrap();
                            }
                        }
                        _ => {}
                    }

                    // If PDF generation failed, show error
                    let html = render_retro_template(
                        &input,
                        None,
                        None,
                        Some("PDF generation failed. Please ensure LilyPond is installed and accessible."),
                        None
                    ).await;
                    Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/html")
                        .body(Body::from(html))
                        .unwrap()
                }
                Ok(_) => {
                    let html = render_retro_template(
                        &input,
                        None,
                        None,
                        Some("No valid notation found for PDF generation."),
                        None
                    ).await;
                    Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/html")
                        .body(Body::from(html))
                        .unwrap()
                }
                Err(e) => {
                    let html = render_retro_template(
                        &input,
                        None,
                        None,
                        Some(&e.to_string()),
                        None
                    ).await;
                    Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/html")
                        .body(Body::from(html))
                        .unwrap()
                }
            }
        }
        "save_lily" => {
            // Return LilyPond source as download
            match crate::process_notation(&input) {
                Ok(result) if !result.lilypond.is_empty() => {
                    Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/plain")
                        .header(header::CONTENT_DISPOSITION, "attachment; filename=\"score.ly\"")
                        .body(Body::from(result.lilypond))
                        .unwrap()
                }
                Ok(_) => {
                    let html = render_retro_template(
                        &input,
                        None,
                        None,
                        Some("No valid notation found for LilyPond generation."),
                        None
                    ).await;
                    Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/html")
                        .body(Body::from(html))
                        .unwrap()
                }
                Err(e) => {
                    let html = render_retro_template(
                        &input,
                        None,
                        None,
                        Some(&e.to_string()),
                        None
                    ).await;
                    Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/html")
                        .body(Body::from(html))
                        .unwrap()
                }
            }
        }
        "save_mt" => {
            // Return original input as .mt file
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/plain")
                .header(header::CONTENT_DISPOSITION, "attachment; filename=\"notation.mt\"")
                .body(Body::from(input))
                .unwrap()
        }
        _ => {
            let html = render_retro_template(
                &input,
                None,
                None,
                Some("Unknown action requested."),
                None
            ).await;
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html")
                .body(Body::from(html))
                .unwrap()
        }
    }
}

// Retro mode file upload handling
async fn retro_load(mut multipart: Multipart) -> impl IntoResponse {
    println!("🔧 File upload request received");

    match multipart.next_field().await {
        Ok(Some(field)) => {
            let name = field.name().unwrap_or("").to_string();
            println!("🔧 Field name: {}", name);

            if name == "musicfile" {
                let filename = field.file_name().unwrap_or("unknown").to_string();
                println!("🔧 Filename: {}", filename);

                match field.bytes().await {
                    Ok(data) => {
                        let content = String::from_utf8_lossy(&data);
                        println!("🔧 Loaded file: {} ({} bytes)", filename, data.len());

                        let html = render_retro_template(
                            &content,
                            None,
                            None,
                            None,
                            Some(&format!("File '{}' loaded successfully! ({} characters)", filename, content.len()))
                        ).await;

                        return Response::builder()
                            .status(StatusCode::OK)
                            .header(header::CONTENT_TYPE, "text/html")
                            .body(Body::from(html))
                            .unwrap();
                    },
                    Err(e) => {
                        println!("🔧 Error reading file data: {}", e);
                        let html = render_retro_template(
                            "",
                            None,
                            None,
                            Some(&format!("Error reading file: {}", e)),
                            None
                        ).await;
                        return Response::builder()
                            .status(StatusCode::OK)
                            .header(header::CONTENT_TYPE, "text/html")
                            .body(Body::from(html))
                            .unwrap();
                    }
                }
            } else {
                println!("🔧 Wrong field name, expected 'musicfile', got '{}'", name);
            }
        },
        Ok(None) => {
            println!("🔧 No fields in multipart");
        },
        Err(e) => {
            println!("🔧 Error processing multipart: {}", e);
            let html = render_retro_template(
                "",
                None,
                None,
                Some(&format!("Error processing upload: {}", e)),
                None
            ).await;
            return Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html")
                .body(Body::from(html))
                .unwrap();
        }
    }

    println!("🔧 No file found in upload");
    let html = render_retro_template(
        "",
        None,
        None,
        Some("No file uploaded or file could not be read."),
        None
    ).await;
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html")
        .body(Body::from(html))
        .unwrap()
}

