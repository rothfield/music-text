// Web server for live notation parsing
use axum::{
    extract::{Query, Multipart, Form, Path},
    response::{IntoResponse, Html, Response},
    routing::{get, post, put},
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
    editor_svg: Option<String>,  // Canvas WYSIWYG SVG
    error: Option<String>,
}

// Document-first API structures
#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct CreateDocumentResponse {
    pub documentUUID: String,
    pub document: serde_json::Value,
    pub formats: DocumentFormats,
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

// Document update structure for PUT requests
#[derive(Debug, Deserialize)]
pub struct UpdateDocumentRequest {
    pub document: serde_json::Value,
    pub edit_command: Option<EditCommand>,
    pub notation_type: Option<String>,
}

// Edit command structure
#[derive(Debug, Deserialize)]
pub struct EditCommand {
    #[serde(rename = "type")]
    pub command_type: String,
    pub position: usize,
    pub text: Option<String>,
    pub direction: Option<String>,
    pub selection_start: Option<usize>,
    pub selection_end: Option<usize>,
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
    pub svg: Option<String>,
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

// Semantic command structures (for fallback)
#[derive(Debug, Deserialize)]
pub struct SemanticCommandRequest {
    pub command_type: String,
    pub target_uuids: Vec<String>,
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct SemanticCommandResponse {
    pub success: bool,
    pub message: Option<String>,
}

// Helper functions for document storage
fn get_documents_dir() -> std::path::PathBuf {
    std::path::Path::new("./documents").to_path_buf()
}

fn get_document_path(documentUUID: &str) -> std::path::PathBuf {
    get_documents_dir().join(format!("{}.json", documentUUID))
}

async fn save_document(documentUUID: &str, document: &serde_json::Value) -> Result<(), std::io::Error> {
    let docs_dir = get_documents_dir();
    tokio::fs::create_dir_all(&docs_dir).await?;

    let doc_path = get_document_path(documentUUID);
    let content = serde_json::to_string_pretty(document)?;
    tokio::fs::write(&doc_path, content).await?;
    Ok(())
}

async fn load_document(documentUUID: &str) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let doc_path = get_document_path(documentUUID);
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
    // Request now only needs document/model context, not raw input text.
    // Keeping selection fields for potential server-side highlighting.
    cursor_position: Option<usize>,
    selection_start: Option<usize>,
    selection_end: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct CanvasSvgResponse {
    svg: String,
    document: Document,
}

#[derive(Debug, Serialize)]
pub struct DocumentWithFormatsResponse {
    document: serde_json::Value,  // The document JSON
    formats: DocumentFormats,     // All rendered formats
}

#[derive(Debug, Serialize)]
pub struct DocumentFormats {
    vexflow_svg: Option<String>,
    editor_svg: Option<String>,
    lilypond_svg: Option<String>,
    lilypond_src: Option<String>,
    midi: Option<String>,
}

use crate::parse::Document;
use crate::parse::actions::{TransformRequest, TransformResponse, apply_octave_transform, apply_slur_transform};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

// Generate PNG from LilyPond source
async fn generate_lilypond_png(lilypond_src: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use tokio::process::Command;
    use std::io::Write;

    // Create a temporary directory
    let temp_dir = tempfile::tempdir()?;
    let lily_file = temp_dir.path().join("music.ly");
    let png_file = temp_dir.path().join("music.png");

    // Write LilyPond source to file
    let mut file = std::fs::File::create(&lily_file)?;
    file.write_all(lilypond_src.as_bytes())?;

    // Run LilyPond to generate PNG
    let output = Command::new("lilypond")
        .arg("--png")
        .arg("-dno-gs-load-fonts")
        .arg("-dinclude-eps-fonts")
        .arg("-dbackend=eps")
        .arg("-dresolution=150")
        .arg("-o")
        .arg(temp_dir.path().join("music"))
        .arg(&lily_file)
        .output()
        .await?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("LilyPond failed: {}", error).into());
    }

    // Read the generated PNG
    let png_data = tokio::fs::read(&png_file).await?;

    // Encode as base64
    let base64_data = BASE64.encode(&png_data);

    // Return as data URL
    Ok(format!("data:image/png;base64,{}", base64_data))
}

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

use crate::document::edit::structural::Clipboard;

// App state for managing shared resources like the clipboard
struct AppState {
    clipboard: Arc<Mutex<Option<Clipboard>>>,
}

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    // Preload CSS file on server startup
    match std::fs::read_to_string("assets/svg-styles.css") {
        Ok(_css_content) => {
            // CSS loaded successfully
        }
        Err(_e) => {
            // CSS not found, will use fallback
        }
    }

    // Shared state for the application
    let shared_state = Arc::new(AppState {
        clipboard: Arc::new(Mutex::new(None)),
    });

    let app = Router::new()
        // RESTful Document API endpoints
        .route("/api/documents", post(create_document_handler))
        .route("/api/documents/from-text", post(create_document_from_text_handler))
        .route("/api/documents/:documentUUID", get(get_document_by_id_handler))
        .route("/api/documents/:documentUUID", put(update_document_handler))
        .route("/api/documents/transform", post(transform_document_handler))
        .route("/api/documents/:documentUUID/transform", post(transform_document_by_id_handler))
        .route("/api/documents/export", post(export_document_handler))
        .route("/api/documents/:documentUUID/export", post(export_document_by_id_handler))
        .route("/health", get(health_endpoint))
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service("/", ServeDir::new("webapp/public"))
        .layer(CorsLayer::permissive())
        .with_state(shared_state);

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

    let svg_content = match crate::renderers::editor::svg::render_editor_svg(
        &request.document,
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

// Duplicate removed; defined earlier

#[derive(Debug, Deserialize)]
struct CreateDocumentFromTextRequest {
    music_text: String,
    metadata: Option<serde_json::Value>,
}

// Document creation endpoint
async fn create_document_handler(Json(request): Json<CreateDocumentRequest>) -> impl IntoResponse {
    let documentUUID = Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();

    // Parse initial notation to get document with full parse tree structure
    // Using "|SRG" creates a stave with a barline and some notes
    let parsed_empty = match crate::pipeline::process_notation("|SRG") {
        Ok(r) => r,
        Err(_) => {
            // Fallback to minimal structure if parsing fails
            crate::pipeline::ProcessingResult {
                original_input: String::new(),
                document: crate::parse::Document {
                    value: None,
                    char_index: 0,
                    title: None,
                    author: None,
                    directives: std::collections::HashMap::new(),
                    elements: vec![],
                },
                lilypond: String::new(),
                vexflow_svg: String::new(),
                vexflow_data: serde_json::Value::Null,
            }
        }
    };

    // Create document model from parsed empty document
    let mut document_value = serde_json::to_value(&parsed_empty.document).unwrap_or_else(|_| {
        serde_json::json!({
            "elements": [],
            "content": []
        })
    });

    // Add document metadata
    if let serde_json::Value::Object(ref mut map) = document_value {
        map.insert("documentUUID".to_string(), serde_json::Value::String(documentUUID.clone()));
        map.insert("version".to_string(), serde_json::Value::String("1.0.0".to_string()));
        map.insert("timestamp".to_string(), serde_json::Value::String(timestamp.clone()));
        map.insert("metadata".to_string(), request.metadata.unwrap_or(serde_json::json!({})));
        map.insert("ui_state".to_string(), serde_json::json!({
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
        }));
    }

    let document = document_value;

    // Save document to disk
    if let Err(e) = save_document(&documentUUID, &document).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Failed to save document: {}", e)}))
        ).into_response();
    }

    // Generate all formats from the parsed document
    let editor_svg = crate::renderers::editor::svg::render_editor_svg(&parsed_empty.document, None, None, None).ok();
    let lilypond_src = if !parsed_empty.lilypond.is_empty() {
        Some(parsed_empty.lilypond.clone())
    } else {
        None
    };
    // VexFlow data contains the actual JS code in the "vexflow_js" field
    let vexflow_svg = if let Some(vexflow_js) = parsed_empty.vexflow_data.get("vexflow_js") {
        vexflow_js.as_str().map(|s| s.to_string())
    } else if !parsed_empty.vexflow_svg.is_empty() {
        Some(parsed_empty.vexflow_svg.clone())
    } else {
        None
    };

    let formats = DocumentFormats {
        vexflow_svg,
        editor_svg,
        lilypond_svg: None,  // Would need to run lilypond command
        lilypond_src,
        midi: None,
    };

    let response = CreateDocumentResponse {
        documentUUID: documentUUID.clone(),
        document: document,
        formats,
    };

    Json(response).into_response()
}

// Create a new document from raw textual music notation (parse → save → render)
async fn create_document_from_text_handler(Json(request): Json<CreateDocumentFromTextRequest>) -> impl IntoResponse {
    // Parse textual notation using same pipeline as CLI
    let parse_result = match crate::pipeline::process_notation(&request.music_text) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Parsing failed: {}", e),
                }))
            ).into_response();
        }
    };

    // Build persisted document JSON mirroring existing schema
    let documentUUID = Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();

    let mut metadata = request.metadata.unwrap_or(serde_json::json!({}));
    if let Some(obj) = metadata.as_object_mut() {
        obj.entry("created_at").or_insert(serde_json::Value::String(chrono::Utc::now().to_rfc3339()));
        obj.entry("created_by").or_insert(serde_json::Value::String("Web Interface".to_string()));
    }

    let document = serde_json::json!({
        "documentUUID": documentUUID,
        "version": "1.0.0",
        "timestamp": timestamp,
        "elements": [],
        "content": [],
        "metadata": metadata,
        "ui_state": {
            "selection": {
                "selected_uuids": [],
                "cursor_uuid": null,
                "cursor_position": 0
            },
            "viewport": { "scroll_x": 0, "scroll_y": 0, "zoom_level": 1.0 },
            "editor_mode": "text",
            "active_tab": "vexflow"
        }
    });

    // Save document to disk
    if let Err(e) = save_document(&documentUUID, &document).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Failed to save document: {}", e)}))
        ).into_response();
    }

    // Generate all formats from the parsed document
    let editor_svg = crate::renderers::editor::svg::render_editor_svg(&parse_result.document, None, None, None).ok();

    // Use the already generated formats from the pipeline
    let lilypond_src = if !parse_result.lilypond.is_empty() {
        Some(parse_result.lilypond.clone())
    } else {
        None
    };

    // Extract VexFlow JavaScript from the pipeline result
    let vexflow_svg = parse_result.vexflow_data.get("vexflow_js")
        .and_then(|js| js.as_str())
        .map(|s| s.to_string());

    let formats = DocumentFormats {
        vexflow_svg,  // Self-executing JavaScript
        editor_svg,
        lilypond_svg: None,  // Would need to run lilypond command
        lilypond_src,
        midi: None,
    };

    let response = CreateDocumentResponse {
        documentUUID: documentUUID.clone(),
        document,
        formats,
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
    let mut svg_content: Option<String> = None;

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
            // FIXED DOCUMENT SERIALIZATION PATTERN:
            // Work directly with JSON to preserve UUIDs

            let octave_type = request.parameters
                .as_ref()
                .and_then(|p| p.get("octave_type"))
                .and_then(|v| v.as_str())
                .unwrap_or("higher");

            let octave_value = match octave_type {
                "lowest" => -2,
                "lower" => -1,
                "middle" => 0,
                "higher" => 1,
                "highest" => 2,
                _ => 1,
            };

            // Find and modify notes in JSON document directly
            let mut modified_count = 0;
            // Debug: Transform request for {} UUIDs"

            if let Some(elements) = updated_document.get_mut("elements") {
                if let Some(elements_array) = elements.as_array_mut() {
                    for element in elements_array {
                        if let Some(stave) = element.get_mut("Stave") {
                            if let Some(lines) = stave.get_mut("lines") {
                                if let Some(lines_array) = lines.as_array_mut() {
                                    for line in lines_array {
                                        if let Some(content_line) = line.get_mut("ContentLine") {
                                            if let Some(line_elements) = content_line.get_mut("elements") {
                                                if let Some(line_elements_array) = line_elements.as_array_mut() {
                                                    for line_element in line_elements_array {
                                                        if let Some(beat) = line_element.get_mut("Beat") {
                                                            if let Some(beat_id) = beat.get("id").and_then(|v| v.as_str()) {
                                                                // Check Beat UUID
                                                                // Check if this Beat UUID is targeted
                                                                if request.target_uuids.contains(&beat_id.to_string()) {
                                                                    // Beat UUID matches - apply octave
                                                                    // Apply octave to all notes in this beat
                                                                    if let Some(beat_elements) = beat.get_mut("elements") {
                                                                        if let Some(beat_elements_array) = beat_elements.as_array_mut() {
                                                                            for beat_element in beat_elements_array {
                                                                                if let Some(note) = beat_element.get_mut("Note") {
                                                                                    note.as_object_mut().unwrap().insert("octave".to_string(), serde_json::Value::Number(serde_json::Number::from(octave_value)));
                                                                                    modified_count += 1;
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                } else {
                                                                    // Check individual Note UUIDs within the beat
                                                                    if let Some(beat_elements) = beat.get_mut("elements") {
                                                                        if let Some(beat_elements_array) = beat_elements.as_array_mut() {
                                                                            for beat_element in beat_elements_array {
                                                                                if let Some(note) = beat_element.get_mut("Note") {
                                                                                    if let Some(note_id) = note.get("id").and_then(|v| v.as_str()) {
                                                                                        // Check Note UUID
                                                                                        if request.target_uuids.contains(&note_id.to_string()) {
                                                                                            // Note UUID matches
                                                                                            note.as_object_mut().unwrap().insert("octave".to_string(), serde_json::Value::Number(serde_json::Number::from(octave_value)));
                                                                                            modified_count += 1;
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if modified_count == 0 {
                return Json(TransformDocumentResponse {
                    success: false,
                    document: request.document,
                    updated_elements: vec![],
                    message: Some(format!("No notes found with the provided UUIDs. Searched for {} UUIDs in JSON document.", request.target_uuids.len())),
                    svg: None,
                });
            }

            // Debug: println!("🔧 JSON-based octave edit: Modified {} notes", modified_count);

            // For SVG regeneration, we need to deserialize to Document struct just for rendering
            if let Ok(document) = serde_json::from_value::<crate::parse::Document>(updated_document.clone()) {
                svg_content = crate::renderers::editor::svg::render_editor_svg(
                    &document,
                    None,
                    None,
                    None,
                ).ok();
            }

            // Update metadata to track the operation
            if let Some(metadata) = updated_document.get_mut("metadata") {
                if let Some(meta_obj) = metadata.as_object_mut() {
                    meta_obj.insert("last_octave_change".to_string(), serde_json::Value::String(octave_type.to_string()));
                }
            }
        }
        _ => {
            return Json(TransformDocumentResponse {
                success: false,
                document: request.document,
                updated_elements: vec![],
                message: Some(format!("Unknown command type: {}", request.command_type)),
                svg: None,
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
        svg: svg_content,
    })
}

// Document export endpoint
async fn export_document_handler(Json(request): Json<ExportDocumentRequest>) -> impl IntoResponse {
    match request.format.as_str() {
        "lilypond-png" => {
            // Generate LilyPond PNG from document
            if let Ok(doc) = serde_json::from_value::<Document>(request.document.clone()) {
                // Generate LilyPond source
                match crate::renderers::lilypond::renderer::convert_processed_document_to_lilypond_src(&doc, None) {
                    Ok(lilypond_src) => {
                        // Run LilyPond to generate PNG
                        match generate_lilypond_png(&lilypond_src).await {
                            Ok(png_base64) => {
                                Json(ExportDocumentResponse {
                                    success: true,
                                    format: request.format,
                                    content: png_base64,
                                    message: Some("LilyPond PNG generated successfully".to_string()),
                                })
                            }
                            Err(e) => {
                                Json(ExportDocumentResponse {
                                    success: false,
                                    format: request.format,
                                    content: String::new(),
                                    message: Some(format!("Failed to generate PNG: {}", e)),
                                })
                            }
                        }
                    }
                    Err(e) => {
                        Json(ExportDocumentResponse {
                            success: false,
                            format: request.format,
                            content: String::new(),
                            message: Some(format!("Failed to generate LilyPond source: {}", e)),
                        })
                    }
                }
            } else {
                Json(ExportDocumentResponse {
                    success: false,
                    format: request.format,
                    content: String::new(),
                    message: Some("Invalid document format".to_string()),
                })
            }
        }
        "lilypond" => {
            // Generate LilyPond source
            if let Ok(doc) = serde_json::from_value::<Document>(request.document.clone()) {
                match crate::renderers::lilypond::renderer::convert_processed_document_to_lilypond_src(&doc, None) {
                    Ok(lilypond_src) => {
                        Json(ExportDocumentResponse {
                            success: true,
                            format: request.format,
                            content: lilypond_src,
                            message: Some("LilyPond source generated successfully".to_string()),
                        })
                    }
                    Err(e) => {
                        Json(ExportDocumentResponse {
                            success: false,
                            format: request.format,
                            content: String::new(),
                            message: Some(format!("Failed to generate LilyPond source: {}", e)),
                        })
                    }
                }
            } else {
                Json(ExportDocumentResponse {
                    success: false,
                    format: request.format,
                    content: String::new(),
                    message: Some("Invalid document format".to_string()),
                })
            }
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

use axum::extract::State;

// PUT update document handler
async fn update_document_handler(
    Path(documentUUID): Path<String>,
    Json(request): Json<UpdateDocumentRequest>
) -> impl IntoResponse {
    // Deserialize the incoming JSON to our Document model.
    let mut doc: Document = match serde_json::from_value(request.document.clone()) {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Failed to deserialize document: {}", e) }))
            ).into_response();
        }
    };

    let mut new_cursor_pos = doc.ui_state.selection.cursor_position;

    // If there's an edit command, apply it directly to the Document model.
    if let Some(edit_command) = request.edit_command {
        let result = match edit_command.command_type.as_str() {
            "insert_text" => {
                // Standard editor behavior: if there's a selection, delete it before inserting.
                if let (Some(start), Some(end)) = (edit_command.selection_start, edit_command.selection_end) {
                    if start < end {
                        new_cursor_pos = crate::document::edit::structural::delete_selection(&mut doc, start, end)?;
                    }
                }
                let text = edit_command.text.unwrap_or_default();
                if text == "\n" {
                    crate::document::edit::structural::insert_newline(&mut doc, new_cursor_pos)
                } else if let Some(char_to_insert) = text.chars().next() {
                    crate::document::edit::structural::insert_char(&mut doc, new_cursor_pos, char_to_insert)
                        .map(|_| new_cursor_pos + 1)
                } else {
                    Ok(new_cursor_pos) // No text to insert
                }
            }
            "delete_text" => {
                if let (Some(start), Some(end)) = (edit_command.selection_start, edit_command.selection_end) {
                    if start < end {
                        crate::document::edit::structural::delete_selection(&mut doc, start, end)
                    } else if let Some(direction) = edit_command.direction {
                         if direction == "backward" {
                            crate::document::edit::structural::delete_char_left(&mut doc, edit_command.position)
                        } else {
                            crate::document::edit::structural::delete_char_right(&mut doc, edit_command.position)
                        }
                    } else {
                        Ok(edit_command.position)
                    }
                } else if let Some(direction) = edit_command.direction {
                    if direction == "backward" {
                        crate::document::edit::structural::delete_char_left(&mut doc, edit_command.position)
                    } else {
                        crate::document::edit::structural::delete_char_right(&mut doc, edit_command.position)
                    }
                } else {
                    Ok(edit_command.position)
                }
            }
            "copy_selection" => {
                if let (Some(start), Some(end)) = (edit_command.selection_start, edit_command.selection_end) {
                    if start < end {
                        match crate::document::edit::structural::copy_selection(&doc, start, end) {
                            Ok(clipboard_content) => {
                                doc.ui_state.clipboard_content = Some(clipboard_content.content);
                            }
                            Err(e) => return Err(e),
                        }
                    }
                }
                Ok(new_cursor_pos) // Copy doesn't move the cursor
            }
            "paste" => {
                if let Some(clipboard_content) = &doc.ui_state.clipboard_content {
                    // First, delete any existing selection
                    if let (Some(start), Some(end)) = (edit_command.selection_start, edit_command.selection_end) {
                        if start < end {
                           new_cursor_pos = crate::document::edit::structural::delete_selection(&mut doc, start, end)?;
                        }
                    }
                    // Then, paste
                    let clipboard = crate::document::edit::structural::Clipboard { content: clipboard_content.clone() };
                    crate::document::edit::structural::paste(&mut doc, new_cursor_pos, &clipboard)
                } else {
                    Ok(new_cursor_pos) // Nothing to paste
                }
            }
            _ => Err(format!("Unknown edit command type: {}", edit_command.command_type)),
        };

        match result {
            Ok(cursor) => new_cursor_pos = cursor,
            Err(e) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))).into_response(),
        }
    }
    
    // Update UI state and timestamp in the model
    doc.ui_state.selection.cursor_position = new_cursor_pos;
    doc.timestamp = chrono::Utc::now().to_rfc3339();

    // Convert the modified Document model back to JSON.
    let document_json = match serde_json::to_value(&doc) {
        Ok(json) => json,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": format!("Failed to re-serialize document: {}", e)}))).into_response(),
    };

    // Save the updated document.
    if let Err(e) = save_document(&documentUUID, &document_json).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": format!("Failed to save document: {}", e)}))).into_response();
    }

    // Generate all formats from the modified document.
    let formats = {
        let editor_svg = crate::renderers::editor::svg::render_editor_svg(&doc, None, None, None).ok();
        let lilypond_src = crate::renderers::lilypond::renderer::convert_processed_document_to_lilypond_src(&doc, None).ok();
        let vexflow_renderer = crate::renderers::vexflow::VexFlowRenderer::new();
        let vexflow_data = vexflow_renderer.render_data_from_document(&doc);
        let vexflow_svg = vexflow_data.get("vexflow_js").and_then(|js| js.as_str()).map(|s| s.to_string());
        DocumentFormats { vexflow_svg, editor_svg, lilypond_svg: None, lilypond_src, midi: None }
    };

    Json(serde_json::json!({
        "success": true,
        "document": document_json,
        "formats": formats,
        "cursor_position": new_cursor_pos
    })).into_response()
}



// GET document by UUID handler
async fn get_document_by_id_handler(
    Path(documentUUID): Path<String>
) -> impl IntoResponse {
    // Load document from disk
    match load_document(&documentUUID).await {
        Ok(mut document_json) => {
            // Ensure the document includes its own UUID
            if !document_json.get("documentUUID").is_some() {
                document_json["documentUUID"] = serde_json::Value::String(documentUUID.clone());
            }
            // Normalize JSON for older docs: ensure arrays are arrays
            if let Some(elements_val) = document_json.get_mut("elements") {
                if !elements_val.is_array() {
                    *elements_val = serde_json::Value::Array(vec![]);
                }
            }

    // If empty document, parse minimal notation to get proper document structure
    let is_empty = document_json.get("elements").and_then(|v| v.as_array()).map(|a| a.is_empty()).unwrap_or(true);
    let editor_svg = if is_empty {
        // Parse initial notation through music-text parser to get proper document with structure
        match crate::pipeline::process_notation("|SRG") {
            Ok(parse_result) => {
                crate::renderers::editor::svg::render_editor_svg(&parse_result.document, None, None, None).ok()
            },
            Err(_) => {
                // Fallback if parsing empty string fails
                match serde_json::from_value::<Document>(document_json.clone()) {
                    Ok(doc) => crate::renderers::editor::svg::render_editor_svg(&doc, None, None, None).ok(),
                    Err(_) => None
                }
            }
        }
    } else {
        match serde_json::from_value::<Document>(document_json.clone()) {
            Ok(doc) => crate::renderers::editor::svg::render_editor_svg(&doc, None, None, None).ok(),
            Err(_) => {
                // Try parsing initial notation as fallback
                match crate::pipeline::process_notation("|SRG") {
                    Ok(parse_result) => {
                        crate::renderers::editor::svg::render_editor_svg(&parse_result.document, None, None, None).ok()
                    },
                    Err(_) => None
                }
            }
        }
    };

            // Generate all formats from the document
            let formats = if let Ok(doc) = serde_json::from_value::<Document>(document_json.clone()) {
                // Generate LilyPond source
                let lilypond_src = crate::renderers::lilypond::renderer::convert_processed_document_to_lilypond_src(&doc, None).ok();

                // Generate VexFlow JavaScript (self-executing)
                let vexflow_renderer = crate::renderers::vexflow::VexFlowRenderer::new();
                let vexflow_data = vexflow_renderer.render_data_from_document(&doc);
                let vexflow_svg = vexflow_data.get("vexflow_js")
                    .and_then(|js| js.as_str())
                    .map(|s| s.to_string());

                DocumentFormats {
                    vexflow_svg,  // Contains self-executing JavaScript
                    editor_svg,
                    lilypond_svg: None,  // Would need to run lilypond command
                    lilypond_src,
                    midi: None,
                }
            } else {
                // Fallback with just editor SVG
                DocumentFormats {
                    vexflow_svg: None,
                    editor_svg,
                    lilypond_svg: None,
                    lilypond_src: None,
                    midi: None,
                }
            };

            let response = DocumentWithFormatsResponse {
                document: document_json,
                formats,
            };

            Json(response).into_response()
        },
        Err(e) => {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": format!("Document not found: {}", e)}))
            ).into_response()
        }
    }
}

// UUID-based transform handler
async fn transform_document_by_id_handler(
    Path(documentUUID): Path<String>,
    Json(request): Json<TransformDocumentByIdRequest>
) -> impl IntoResponse {
    // Load document from disk
    let mut document = match load_document(&documentUUID).await {
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
    if let Err(e) = save_document(&documentUUID, &document).await {
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
        svg: None, // This endpoint doesn't generate SVG
    }).into_response()
}

// UUID-based export handler
async fn export_document_by_id_handler(
    Path(documentUUID): Path<String>,
    Json(request): Json<ExportDocumentByIdRequest>
) -> impl IntoResponse {
    // Load document from disk
    let document = match load_document(&documentUUID).await {
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
            let svg_content = match crate::renderers::editor::svg::render_editor_svg(&result.document, None, None, None) {
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
async fn render_editor_svg(Json(request): Json<CanvasSvgRequest>) -> impl IntoResponse {
    // This endpoint now assumes the client provides or the server derives the Document elsewhere;
    // for compatibility, keep it simple: return 400 since raw input_text is removed.
    (StatusCode::BAD_REQUEST, "render_editor_svg requires a Document; raw input_text is not supported").into_response()
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
    

    match multipart.next_field().await {
        Ok(Some(field)) => {
            let name = field.name().unwrap_or("").to_string();
            

            if name == "musicfile" {
                let filename = field.file_name().unwrap_or("unknown").to_string();
                

                match field.bytes().await {
                    Ok(data) => {
                        let content = String::from_utf8_lossy(&data);
                        

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
                
            }
        },
        Ok(None) => {
            
        },
        Err(e) => {
            
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

// Semantic command handler - fallback for simple transformations without full document
async fn semantic_command_handler(Json(request): Json<SemanticCommandRequest>) -> impl IntoResponse {
    // For now, this is a simple fallback that just responds success
    // In the future, this could handle lightweight operations without requiring full document parsing

    println!("Semantic command: {} with {} target UUIDs", request.command_type, request.target_uuids.len());

    match request.command_type.as_str() {
        "set_octave" | "apply_slur" => {
            Json(SemanticCommandResponse {
                success: true,
                message: Some(format!("Semantic command '{}' applied to {} elements", request.command_type, request.target_uuids.len())),
            })
        }
        _ => {
            Json(SemanticCommandResponse {
                success: false,
                message: Some(format!("Unknown semantic command: {}", request.command_type)),
            })
        }
    }
}
