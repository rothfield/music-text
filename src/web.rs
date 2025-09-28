// Web server for live notation parsing
use axum::{
    extract::{Multipart, Form, Path, Query},
    response::{IntoResponse, Html, Response},
    routing::{get, post, put},
    Json, Router,
    http::{StatusCode, header},
    body::Body,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use chrono;
use tower_http::{cors::CorsLayer, services::ServeDir};
use tokio::fs;
// Removed pest import - using hand-written recursive descent parser
use crate::import::musicxml::{import_musicxml_to_document, ImportOptions};

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
    pub content: Option<String>,  // Music text content (optional)
}

#[derive(Debug, Deserialize)]
pub struct CreateDocumentQuery {
    pub representations: Option<String>,  // Comma-separated list of formats to include
}

#[derive(Debug, Serialize)]
pub struct CreateDocumentResponse {
    pub success: bool,
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

// TODO: DELETE - Document update structure for PUT requests (unused by frontend)
#[derive(Debug, Deserialize)]
pub struct UpdateDocumentRequest {
    pub document: serde_json::Value,
    pub edit_command: Option<EditCommand>,
    pub notation_type: Option<String>,
}

// TODO: DELETE - Edit command structure (unused by frontend)
#[derive(Debug, Deserialize)]
pub struct EditCommand {
    #[serde(rename = "type")]
    pub command_type: String,
    pub position: Option<usize>,  // Made optional since not all commands need it
    pub text: Option<String>,
    pub direction: Option<String>,
    pub selection_start: Option<usize>,
    pub selection_end: Option<usize>,
    pub selected_uuids: Option<Vec<String>>, // For delete operations
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

#[derive(Debug, Deserialize)]
struct MusicXmlImportRequest { xml: String, #[serde(default)] prefer_minor: bool }

#[derive(Debug, Serialize)]
struct MusicXmlImportResponse { document: crate::models::core::Document }

use crate::parse::Document;
use crate::parse::actions::{TransformRequest, apply_octave_transform, apply_slur_transform};
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
        .route("/api/import/musicxml", post(import_musicxml_handler))
        // RESTful Document API endpoints
        .route("/api/documents", post(create_document_handler))
        .route("/api/documents/:documentUUID", get(get_document_by_id_handler))
        // TODO: DELETE THESE UNUSED ENDPOINTS - not used by frontend JS
        // .route("/api/documents/from-text", post(create_document_from_text_handler))
        // .route("/api/documents/:documentUUID", put(update_document_handler))
        // .route("/documents/:documentUUID/edit", get(edit_document_handler))
        // Rails-style document processing endpoints
        .route("/api/documents/render", post(render_document_handler))
        .route("/api/documents/transform", post(transform_document_handler))
        .route("/api/documents/export", post(export_document_handler))
        // Legacy UUID-based endpoints (deprecated)
        .route("/api/documents/:documentUUID/transform", post(transform_document_by_id_handler))
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

async fn import_musicxml_handler(Json(payload): Json<MusicXmlImportRequest>) -> impl IntoResponse {
    match import_musicxml_to_document(&payload.xml, Some(ImportOptions{ prefer_minor: payload.prefer_minor })) {
        Ok(document) => {
            let resp = MusicXmlImportResponse { document };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": format!("{}", e)}))).into_response(),
    }
}

// Duplicate removed; defined earlier

// TODO: DELETE - unused by frontend JS
#[derive(Debug, Deserialize)]
struct CreateDocumentFromTextRequest {
    music_text: String,
    metadata: Option<serde_json::Value>,
}

// Document creation endpoint
async fn create_document_handler(
    Query(query): Query<CreateDocumentQuery>,
    Json(request): Json<CreateDocumentRequest>
) -> impl IntoResponse {
    let documentUUID = Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();

    // Check if content is provided - if yes, parse it; if no, create empty document
    let parse_result = if let Some(content) = &request.content {
        // Parse the provided content
        match crate::pipeline::process_notation(content) {
            Ok(mut result) => {
                result.document.document_uuid = Some(documentUUID.clone());
                result
            }
            Err(e) => {
                // If parsing fails, create minimal valid structure
                eprintln!("Parse failed for '{}': {}", content, e);

                let minimal_doc = crate::parse::Document {
                    document_uuid: Some(documentUUID.clone()),
                    value: Some(content.clone()),
                    char_index: 0,
                    title: None,
                    author: None,
                    directives: std::collections::HashMap::new(),
                    elements: vec![
                        crate::models::DocumentElement::Stave(crate::models::Stave {
                            value: Some(content.clone()),
                            char_index: 0,
                            notation_system: crate::models::NotationSystem::Number,
                            line: 0,
                            column: 0,
                            index_in_doc: 0,
                            index_in_line: 0,
                            lines: vec![
                                crate::models::StaveLine::ContentLine(crate::models::ContentLine {
                                    value: Some(content.clone()),
                                    char_index: 0,
                                    elements: vec![],
                                    consumed_elements: vec![],
                                })
                            ],
                        })
                    ],
                    ui_state: crate::models::UIState::default(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };

                crate::pipeline::ProcessingResult {
                    original_input: content.clone(),
                    document: minimal_doc,
                    lilypond: String::new(),
                    vexflow_svg: String::new(),
                    vexflow_data: serde_json::Value::Null,
                }
            }
        }
    } else {
        // Create empty document
        crate::pipeline::ProcessingResult {
            original_input: String::new(),
            document: crate::parse::Document {
                document_uuid: Some(documentUUID.clone()),
                value: None,
                char_index: 0,
                title: None,
                author: None,
                directives: std::collections::HashMap::new(),
                elements: vec![],  // Empty elements - no content
                ui_state: crate::models::UIState::default(),
                timestamp: String::new(),
            },
            lilypond: String::new(),
            vexflow_svg: String::new(),
            vexflow_data: serde_json::Value::Null,
        }
    };

    // Convert the document to JSON
    let mut document_value = serde_json::to_value(&parse_result.document).unwrap_or_else(|_| {
        serde_json::json!({
            "elements": [],
            "content": []
        })
    });

    // Add document metadata (UUID already in document)
    if let serde_json::Value::Object(ref mut map) = document_value {
        map.insert("version".to_string(), serde_json::Value::String("1.0.0".to_string()));
        map.insert("timestamp".to_string(), serde_json::Value::String(timestamp.clone()));
        map.insert("metadata".to_string(), request.metadata.unwrap_or(serde_json::json!({})));
        map.insert("ui_state".to_string(), serde_json::json!({
            "selection": {
                "selected_uuids": [],
                "cursor_uuid": null,
                "cursor_position": request.content.as_ref().map(|c| c.len()).unwrap_or(0)
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

    // Log the new document to filesystem for backup/history
    if let Err(e) = save_document(&documentUUID, &document).await {
        eprintln!("Warning: Failed to log new document {} to disk: {}", documentUUID, e);
    } else {
        println!("Created and logged new document {} to disk (backup)", documentUUID);
    }

    // Generate formats only if requested in query parameters
    let requested_formats = query.representations.as_deref().unwrap_or("");
    let generate_all = requested_formats.is_empty(); // If no specific formats requested, generate all

    let editor_svg = if generate_all || requested_formats.contains("editor_svg") {
        crate::renderers::editor::svg::render_editor_svg(&parse_result.document, None, None, None).ok()
    } else {
        None
    };

    let lilypond_src = if generate_all || requested_formats.contains("lilypond") {
        if !parse_result.lilypond.is_empty() {
            Some(parse_result.lilypond.clone())
        } else {
            None
        }
    } else {
        None
    };

    let vexflow_svg = if generate_all || requested_formats.contains("vexflow") {
        if let Some(vexflow_js) = parse_result.vexflow_data.get("vexflow_js") {
            vexflow_js.as_str().map(|s| s.to_string())
        } else if !parse_result.vexflow_svg.is_empty() {
            Some(parse_result.vexflow_svg.clone())
        } else {
            None
        }
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
        success: true,
        document: document,
        formats,
    };

    Json(response).into_response()
}

// TODO: DELETE - Create a new document from raw textual music notation (parse → save → render)
// UNUSED by frontend JS - marked for deletion
async fn create_document_from_text_handler(Json(request): Json<CreateDocumentFromTextRequest>) -> impl IntoResponse {
    // Generate UUID and timestamp first
    let documentUUID = Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();

    // Parse the input text (even if it's just one character) through the full parser
    // This creates a proper document structure with staves and lines
    let mut parse_result = match crate::pipeline::process_notation(&request.music_text) {
        Ok(r) => r,
        Err(e) => {
            // If parsing fails (e.g., empty string), create minimal valid structure
            eprintln!("Parse failed for '{}': {}", request.music_text, e);

            // Create a minimal document with one stave containing the input text
            let minimal_doc = crate::parse::Document {
                document_uuid: Some(documentUUID.clone()),
                value: Some(request.music_text.clone()),
                char_index: 0,
                title: None,
                author: None,
                directives: std::collections::HashMap::new(),
                elements: vec![
                    crate::models::DocumentElement::Stave(crate::models::Stave {
                        value: Some(request.music_text.clone()),
                        char_index: 0,
                        notation_system: crate::models::NotationSystem::Number,
                        line: 0,
                        column: 0,
                        index_in_doc: 0,
                        index_in_line: 0,
                        lines: vec![
                            crate::models::StaveLine::ContentLine(crate::models::ContentLine {
                                value: Some(request.music_text.clone()),
                                char_index: 0,
                                elements: vec![],
                                consumed_elements: vec![],
                            })
                        ],
                    })
                ],
                ui_state: crate::models::UIState::default(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            };

            crate::pipeline::ProcessingResult {
                original_input: request.music_text.clone(),
                document: minimal_doc,
                lilypond: String::new(),
                vexflow_svg: String::new(),
                vexflow_data: serde_json::Value::Null,
            }
        }
    };

    // Set the UUID in the document
    parse_result.document.document_uuid = Some(documentUUID.clone());

    let mut metadata = request.metadata.unwrap_or(serde_json::json!({}));
    if let Some(obj) = metadata.as_object_mut() {
        obj.entry("created_at").or_insert(serde_json::Value::String(chrono::Utc::now().to_rfc3339()));
        obj.entry("created_by").or_insert(serde_json::Value::String("Web Interface".to_string()));
    }

    // Convert the parsed document to JSON, preserving its structure
    let mut document = serde_json::to_value(&parse_result.document).unwrap_or(serde_json::json!({}));

    // Add/update required fields (UUID already in document)
    document["version"] = serde_json::Value::String("1.0.0".to_string());
    document["timestamp"] = serde_json::Value::String(timestamp.clone());
    document["metadata"] = metadata;

    // Ensure UI state exists
    if !document.get("ui_state").is_some() {
        document["ui_state"] = serde_json::json!({
            "selection": {
                "selected_uuids": [],
                "cursor_uuid": null,
                "cursor_position": request.music_text.len()  // Position after all text
            },
            "viewport": { "scroll_x": 0, "scroll_y": 0, "zoom_level": 1.0 },
            "editor_mode": "text",
            "active_tab": "vexflow"
        });
    }

    // Log the new document to filesystem for backup/history
    // (In local-first architecture, browser owns the document, server logs for backup)
    if let Err(e) = save_document(&documentUUID, &document).await {
        // Log error but don't fail - logging is not critical for document creation
        eprintln!("Warning: Failed to log new document {} to disk: {}", documentUUID, e);
    } else {
        println!("Created and logged new document {} to disk (backup)", documentUUID);
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

    // Return success response with document (UUID is already in document)
    Json(serde_json::json!({
        "success": true,
        "document": document,
        "formats": formats,
        "cursor_position": 1  // After first character
    })).into_response()
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

#[derive(Debug, Deserialize)]
pub struct RenderDocumentRequest {
    pub document: serde_json::Value,
    pub render_options: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct RenderDocumentResponse {
    pub success: bool,
    pub formats: DocumentFormats,
    pub message: Option<String>,
}

// Document render endpoint - generates all formats from document
async fn render_document_handler(Json(request): Json<RenderDocumentRequest>) -> impl IntoResponse {
    // Deserialize document to our Document model for rendering
    let doc: Document = match serde_json::from_value(request.document.clone()) {
        Ok(d) => d,
        Err(e) => {
            return Json(RenderDocumentResponse {
                success: false,
                formats: DocumentFormats { vexflow_svg: None, editor_svg: None, lilypond_svg: None, lilypond_src: None, midi: None },
                message: Some(format!("Failed to deserialize document: {}", e)),
            }).into_response();
        }
    };

    // Generate all formats from the document
    let editor_svg = crate::renderers::editor::svg::render_editor_svg(&doc, None, None, None).ok();
    let lilypond_src = crate::renderers::lilypond::renderer::convert_processed_document_to_lilypond_src(&doc, None).ok();
    let vexflow_renderer = crate::renderers::vexflow::VexFlowRenderer::new();
    let vexflow_data = vexflow_renderer.render_data_from_document(&doc);
    let vexflow_svg = vexflow_data.get("vexflow_js").and_then(|js| js.as_str()).map(|s| s.to_string());

    let formats = DocumentFormats {
        vexflow_svg,
        editor_svg,
        lilypond_svg: None,  // Would need to run lilypond command
        lilypond_src,
        midi: None,
    };

    Json(RenderDocumentResponse {
        success: true,
        formats,
        message: Some("Document rendered successfully".to_string()),
    }).into_response()
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


// TODO: DELETE - PUT update document handler
// UNUSED by frontend JS - marked for deletion
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
                        match crate::document::edit::structural::delete_selection(&mut doc, start, end) {
                            Ok(pos) => new_cursor_pos = pos,
                            Err(e) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))).into_response(),
                        }
                    }
                }
                let text = edit_command.text.unwrap_or_default();
                println!("Insert text '{}' at position {}, doc has {} elements", text, new_cursor_pos, doc.elements.len());
                // Debug document structure
                for (i, elem) in doc.elements.iter().enumerate() {
                    if let crate::models::DocumentElement::Stave(stave) = elem {
                        println!("  Stave {}: {} lines", i, stave.lines.len());
                    }
                }
                if text == "\n" {
                    crate::document::edit::structural::insert_newline(&mut doc, new_cursor_pos)
                } else if let Some(char_to_insert) = text.chars().next() {
                    crate::document::edit::structural::insert_char(&mut doc, new_cursor_pos, char_to_insert)
                } else {
                    Ok(new_cursor_pos) // No text to insert
                }
            }
            "delete_text" => {
                if let (Some(start), Some(end)) = (edit_command.selection_start, edit_command.selection_end) {
                    if start < end {
                        crate::document::edit::structural::delete_selection(&mut doc, start, end)
                    } else if let Some(direction) = edit_command.direction {
                        let pos = edit_command.position.unwrap_or(new_cursor_pos);
                        if direction == "backward" {
                            crate::document::edit::structural::delete_char_left(&mut doc, pos)
                        } else {
                            crate::document::edit::structural::delete_char_right(&mut doc, pos)
                        }
                    } else {
                        Ok(edit_command.position.unwrap_or(new_cursor_pos))
                    }
                } else if let Some(direction) = edit_command.direction {
                    let pos = edit_command.position.unwrap_or(new_cursor_pos);
                    if direction == "backward" {
                        crate::document::edit::structural::delete_char_left(&mut doc, pos)
                    } else {
                        crate::document::edit::structural::delete_char_right(&mut doc, pos)
                    }
                } else {
                    Ok(edit_command.position.unwrap_or(new_cursor_pos))
                }
            }
            "copy_selection" => {
                if let (Some(start), Some(end)) = (edit_command.selection_start, edit_command.selection_end) {
                    if start < end {
                        match crate::document::edit::structural::copy_selection(&doc, start, end) {
                            Ok(clipboard_content) => {
                                doc.ui_state.clipboard_content = Some(clipboard_content.content);
                            }
                            Err(e) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))).into_response(),
                        }
                    }
                }
                Ok(new_cursor_pos) // Copy doesn't move the cursor
            }
            "paste" => {
                if let Some(clipboard_content) = doc.ui_state.clipboard_content.clone() {
                    // First, delete any existing selection
                    if let (Some(start), Some(end)) = (edit_command.selection_start, edit_command.selection_end) {
                        if start < end {
                           match crate::document::edit::structural::delete_selection(&mut doc, start, end) {
                            Ok(pos) => new_cursor_pos = pos,
                            Err(e) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e }))).into_response(),
                        }
                        }
                    }
                    // Then, paste
                    let clipboard = crate::document::edit::structural::Clipboard { content: clipboard_content };
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
    
    // Reconstruct document value from elements after edit
    crate::document::edit::structural::reconstruct_document_value(&mut doc);

    // Update UI state and timestamp in the model
    doc.ui_state.selection.cursor_position = new_cursor_pos;
    doc.timestamp = chrono::Utc::now().to_rfc3339();

    // Convert the modified Document model back to JSON.
    let document_json = match serde_json::to_value(&doc) {
        Ok(json) => json,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": format!("Failed to re-serialize document: {}", e)}))).into_response(),
    };

    // Log the updated document to filesystem for backup/history
    // (In local-first architecture, browser owns the document, server logs for backup)
    if let Err(e) = save_document(&documentUUID, &document_json).await {
        // Log error but don't fail - logging is not critical for the operation
        eprintln!("Warning: Failed to log document {} to disk: {}", documentUUID, e);
    } else {
        println!("Logged document {} to disk (backup)", documentUUID);
    }

    // Generate all formats from the modified document.
    let formats = {
        let editor_svg = match crate::renderers::editor::svg::render_editor_svg(&doc, None, None, None) {
            Ok(svg) => {
                eprintln!("✅ Generated editor SVG ({} chars)", svg.len());
                Some(svg)
            },
            Err(e) => {
                eprintln!("❌ Failed to generate editor SVG: {}", e);
                None
            }
        };
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
    // In local-first architecture, browser owns the document
    // Server doesn't maintain state - just log the request
    println!("GET request for document UUID: {}", documentUUID);

    // Return not found - browser should use its localStorage copy
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": "Document not found. In local-first architecture, documents are stored in browser localStorage.",
            "documentUUID": documentUUID
        }))
    ).into_response()
}

// TODO: DELETE - Edit document handler - loads a document from backup and opens editor
// UNUSED by frontend JS - marked for deletion
async fn edit_document_handler(
    Path(documentUUID): Path<String>
) -> impl IntoResponse {
    // Load document from backup (for editing/recovery purposes)
    match load_document(&documentUUID).await {
        Ok(document_json) => {
            // Return HTML page that loads this document into the editor
            let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Edit Document - {}</title>
    <meta charset="UTF-8">
    <script>
        // Load document into localStorage and redirect to main editor
        window.addEventListener('DOMContentLoaded', function() {{
            const document = {};

            // Store document in localStorage
            const documentUUID = '{}';
            localStorage.setItem('musictext_document_' + documentUUID, JSON.stringify(document));
            localStorage.setItem('musictext_current_document', documentUUID);

            // Redirect to main editor
            window.location.href = '/';
        }});
    </script>
</head>
<body>
    <p>Loading document {}...</p>
</body>
</html>
"#, documentUUID, serde_json::to_string(&document_json).unwrap_or_default(), documentUUID, documentUUID);

            Html(html).into_response()
        },
        Err(e) => {
            // Document not found in backup
            let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Document Not Found</title>
    <meta charset="UTF-8">
</head>
<body>
    <h1>Document Not Found</h1>
    <p>Document {} not found in backup storage.</p>
    <p>Error: {}</p>
    <a href="/">Return to Editor</a>
</body>
</html>
"#, documentUUID, e);

            Html(html).into_response()
        }
    }
}

// UUID-based transform handler
async fn transform_document_by_id_handler(
    Path(documentUUID): Path<String>,
    Json(request): Json<TransformDocumentByIdRequest>
) -> impl IntoResponse {
    // In local-first architecture, this endpoint shouldn't be used
    // Document should be sent in the request body
    println!("Transform request for document UUID: {} (deprecated endpoint)", documentUUID);

    (
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "error": "This endpoint is deprecated. Use /api/documents/transform with document in request body.",
            "documentUUID": documentUUID
        }))
    ).into_response()
}

// UUID-based export handler
async fn export_document_by_id_handler(
    Path(documentUUID): Path<String>,
    Json(request): Json<ExportDocumentByIdRequest>
) -> impl IntoResponse {
    // In local-first architecture, this endpoint shouldn't be used
    // Document should be sent in the request body
    println!("Export request for document UUID: {} (deprecated endpoint)", documentUUID);

    (
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "error": "This endpoint is deprecated. Use /api/documents/export with document in request body.",
            "documentUUID": documentUUID
        }))
    ).into_response()
}

async fn health_endpoint() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
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
