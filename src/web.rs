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
    pub ui_state: Option<serde_json::Value>,  // UI state to preserve in new document
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
    pub formats: Option<DocumentFormats>,
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
    pub document: serde_json::Value,
    pub format: String,
    pub content: String,
    pub message: Option<String>,
}

// Document storage - on-disk for durability
type DocumentStore = std::path::PathBuf;


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

// ARCHIVED: Template rendering helper - kept for reference
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
        // Rails-style document processing endpoints
        .route("/api/documents/render", post(render_document_handler))
        .route("/api/documents/transform", post(transform_document_handler))
        .route("/api/documents/export", post(export_document_handler))
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


// Document creation endpoint
async fn create_document_handler(
    Query(query): Query<CreateDocumentQuery>,
    Json(request): Json<CreateDocumentRequest>
) -> impl IntoResponse {
    let documentUUID = Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();

    // Check if content is provided - if yes, parse it; if no, create empty document
    println!("Document creation request content: {:?}", request.content);
    let document = if let Some(content) = &request.content {
        // Create minimal valid structure with provided content
        crate::parse::Document {
            document_uuid: Some(documentUUID.clone()),
            id: Uuid::new_v4(),
            value: Some(content.clone()),
            title: None,
            author: None,
            directives: std::collections::HashMap::new(),
            elements: vec![
                crate::models::DocumentElement::Stave(crate::models::Stave {
                    id: Uuid::new_v4(),
                    value: Some(content.clone()),
                    notation_system: crate::models::NotationSystem::Number,
                    line: 0,
                    column: 0,
                    index_in_doc: 0,
                    index_in_line: 0,
                    lines: vec![
                        crate::models::StaveLine::ContentLine(crate::models::ContentLine {
                            id: Uuid::new_v4(),
                            value: Some(content.clone()),
                            elements: vec![],
                        })
                    ],
                })
            ],
            ui_state: crate::models::UIState::default(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    } else {
        // Create empty document with empty stave for insertion
        println!("Creating empty document with UUID-enabled stave");
        crate::parse::Document {
            document_uuid: Some(documentUUID.clone()),
            id: Uuid::new_v4(),
            value: Some(String::new()),
            title: None,
            author: None,
            directives: std::collections::HashMap::new(),
            elements: vec![
                crate::models::DocumentElement::Stave(crate::models::Stave {
                    id: Uuid::new_v4(),
                    value: Some(String::new()),
                    notation_system: crate::models::NotationSystem::Number,
                    line: 1,
                    column: 1,
                    index_in_doc: 0,
                    index_in_line: 0,
                    lines: vec![
                        crate::models::StaveLine::ContentLine(crate::models::ContentLine {
                            id: Uuid::new_v4(),
                            value: Some(String::new()),
                            elements: vec![],  // Empty - will accept insertion
                        })
                    ],
                })
            ],
            ui_state: crate::models::UIState::default(),
            timestamp: timestamp.clone(),
        }
    };

    // Convert the document to JSON
    let mut document_value = serde_json::to_value(&document).unwrap_or_else(|_| {
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
        // Use provided UI state or create default with preserved cursor position
        let ui_state = if let Some(provided_ui_state) = request.ui_state {
            // Use the provided UI state but ensure cursor position is updated for content length
            let mut ui_state_value = provided_ui_state;
            if let serde_json::Value::Object(ref mut ui_map) = ui_state_value {
                if let Some(serde_json::Value::Object(ref mut selection)) = ui_map.get_mut("selection") {
                    selection.insert("cursor_position".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(
                            request.content.as_ref().map(|c| c.len()).unwrap_or(0)
                        )));
                }
            }
            ui_state_value
        } else {
            // Create default UI state
            serde_json::json!({
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
            })
        };
        map.insert("ui_state".to_string(), ui_state);
    }

    let final_document = document_value;

    // Log the new document to filesystem for backup/history
    if let Err(e) = save_document(&documentUUID, &final_document).await {
        eprintln!("Warning: Failed to log new document {} to disk: {}", documentUUID, e);
    } else {
        println!("Created and logged new document {} to disk (backup)", documentUUID);
    }

    // Generate formats only if requested in query parameters
    let requested_formats = query.representations.as_deref().unwrap_or("");
    let generate_all = requested_formats.is_empty(); // If no specific formats requested, generate all

    let editor_svg = if generate_all || requested_formats.contains("editor_svg") {
        crate::renderers::editor::svg::render_editor_svg(&document, None, None, None).ok()
    } else {
        None
    };

    let lilypond_src = if generate_all || requested_formats.contains("lilypond") {
        crate::renderers::lilypond::renderer::convert_processed_document_to_lilypond_src(&document, None).ok()
    } else {
        None
    };

    let vexflow_svg = if generate_all || requested_formats.contains("vexflow") {
        let vexflow_renderer = crate::renderers::vexflow::VexFlowRenderer::new();
        let vexflow_data = vexflow_renderer.render_data_from_document(&document);
        vexflow_data.get("vexflow_js").and_then(|js| js.as_str()).map(|s| s.to_string())
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
        document: final_document,
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
        "insert_text" => {
            // Text editing command - deserialize document to Document model for editing
            let mut doc: Document = match serde_json::from_value(request.document.clone()) {
                Ok(d) => d,
                Err(e) => {
                    return Json(TransformDocumentResponse {
                        success: false,
                        document: request.document,
                        updated_elements: vec![],
                        message: Some(format!("Failed to deserialize document for text editing: {}", e)),
                        svg: None,
                        formats: None,
                    });
                }
            };

            let mut new_cursor_pos = doc.ui_state.selection.cursor_position;

            // Extract parameters for text editing
            let text = request.parameters
                .as_ref()
                .and_then(|p| p.get("text"))
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let target_uuid = request.parameters
                .as_ref()
                .and_then(|p| p.get("target_uuid"))
                .and_then(|v| v.as_str());

            let element_position = request.parameters
                .as_ref()
                .and_then(|p| p.get("element_position"))
                .and_then(|v| v.as_u64())
                .map(|v| v as usize)
                .unwrap_or(0);

            let selection_start = request.parameters
                .as_ref()
                .and_then(|p| p.get("selection_start"))
                .and_then(|v| v.as_u64())
                .map(|v| v as usize);

            let selection_end = request.parameters
                .as_ref()
                .and_then(|p| p.get("selection_end"))
                .and_then(|v| v.as_u64())
                .map(|v| v as usize);

            // Handle selection deletion before insertion (standard editor behavior)
            if let (Some(start), Some(end)) = (selection_start, selection_end) {
                if start < end {
                    match crate::document::edit::structural::delete_selection(&mut doc, start, end) {
                        Ok(pos) => new_cursor_pos = pos,
                        Err(e) => {
                            return Json(TransformDocumentResponse {
                                success: false,
                                document: request.document,
                                updated_elements: vec![],
                                message: Some(format!("Failed to delete selection: {}", e)),
                                svg: None,
                        formats: None,
                            });
                        }
                    }
                }
            }

            // Parse the input text to get musical elements
            let parsed_elements = if !text.is_empty() {
                match crate::document::line_parser::content_line_parser::unused_parse_content_line(
                    text,
                    0,
                    doc.get_detected_notation_systems().first().cloned().unwrap_or(crate::models::NotationSystem::Number),
                    0,
                ) {
                    Ok(content_line) => content_line.elements,
                    Err(e) => {
                        return Json(TransformDocumentResponse {
                            success: false,
                            document: request.document,
                            updated_elements: vec![],
                            message: Some(format!("Failed to parse input text '{}': {}", text, e)),
                            svg: None,
                        formats: None,
                        });
                    }
                }
            } else {
                vec![]
            };

            // Insert parsed elements at the target location
            let result = if let Some(uuid_str) = target_uuid {
                // UUID-based insertion (preferred)
                insert_elements_at_uuid(&mut doc, uuid_str, element_position, parsed_elements)
            } else {
                // Fallback to character position-based insertion
                // Insert each character individually at the specified position
                let mut current_position = element_position;
                for ch in text.chars() {
                    match crate::document::edit::structural::insert_char(&mut doc, current_position, ch) {
                        Ok(new_pos) => current_position = new_pos,
                        Err(e) => return Json(TransformDocumentResponse {
                            success: false,
                            document: request.document,
                            updated_elements: vec![],
                            message: Some(format!("Character insertion failed: {}", e)),
                            svg: None,
                        formats: None,
                        }),
                    }
                }
                Ok(current_position)
            };

            match result {
                Ok(cursor_pos) => {
                    new_cursor_pos = cursor_pos;

                    // Reconstruct document value from elements after edit
                    crate::document::edit::structural::reconstruct_document_value(&mut doc);

                    // Analyze rhythm patterns
                    println!("Running rhythm analysis on document...");
                    match crate::rhythm::analyzer::analyze_rhythm_into_document(&mut doc) {
                        Ok(()) => {
                            println!("Rhythm analysis completed successfully");
                        }
                        Err(e) => {
                            eprintln!("Warning: Rhythm analysis failed: {}", e);
                            // Continue anyway - rhythm analysis failure shouldn't break the operation
                        }
                    }

                    // Update UI state and timestamp
                    doc.ui_state.selection.cursor_position = new_cursor_pos;
                    doc.timestamp = chrono::Utc::now().to_rfc3339();

                    // Convert back to JSON
                    updated_document = serde_json::to_value(&doc).unwrap_or(request.document);

                    // Generate SVG
                    svg_content = crate::renderers::editor::svg::render_editor_svg(&doc, None, None, None).ok();
                }
                Err(e) => {
                    return Json(TransformDocumentResponse {
                        success: false,
                        document: request.document,
                        updated_elements: vec![],
                        message: Some(format!("Failed to insert text: {}", e)),
                        svg: None,
                        formats: None,
                    });
                }
            }
        }
        "delete_text" => {
            // Text deletion command
            let mut doc: Document = match serde_json::from_value(request.document.clone()) {
                Ok(d) => d,
                Err(e) => {
                    return Json(TransformDocumentResponse {
                        success: false,
                        document: request.document,
                        updated_elements: vec![],
                        message: Some(format!("Failed to deserialize document for text deletion: {}", e)),
                        svg: None,
                        formats: None,
                    });
                }
            };

            let mut new_cursor_pos = doc.ui_state.selection.cursor_position;

            // Extract parameters
            let position = request.parameters
                .as_ref()
                .and_then(|p| p.get("position"))
                .and_then(|v| v.as_u64())
                .map(|v| v as usize)
                .unwrap_or(new_cursor_pos);

            let direction = request.parameters
                .as_ref()
                .and_then(|p| p.get("direction"))
                .and_then(|v| v.as_str())
                .unwrap_or("backward");

            let selection_start = request.parameters
                .as_ref()
                .and_then(|p| p.get("selection_start"))
                .and_then(|v| v.as_u64())
                .map(|v| v as usize);

            let selection_end = request.parameters
                .as_ref()
                .and_then(|p| p.get("selection_end"))
                .and_then(|v| v.as_u64())
                .map(|v| v as usize);

            // Handle deletion
            let result = if let (Some(start), Some(end)) = (selection_start, selection_end) {
                if start < end {
                    crate::document::edit::structural::delete_selection(&mut doc, start, end)
                } else if direction == "backward" {
                    crate::document::edit::structural::delete_char_left(&mut doc, position)
                } else {
                    crate::document::edit::structural::delete_char_right(&mut doc, position)
                }
            } else if direction == "backward" {
                crate::document::edit::structural::delete_char_left(&mut doc, position)
            } else {
                crate::document::edit::structural::delete_char_right(&mut doc, position)
            };

            match result {
                Ok(cursor_pos) => {
                    new_cursor_pos = cursor_pos;

                    // Reconstruct document value from elements after edit
                    crate::document::edit::structural::reconstruct_document_value(&mut doc);

                    // Analyze rhythm patterns
                    println!("Running rhythm analysis on document...");
                    match crate::rhythm::analyzer::analyze_rhythm_into_document(&mut doc) {
                        Ok(()) => {
                            println!("Rhythm analysis completed successfully");
                        }
                        Err(e) => {
                            eprintln!("Warning: Rhythm analysis failed: {}", e);
                            // Continue anyway - rhythm analysis failure shouldn't break the operation
                        }
                    }

                    // Update UI state and timestamp
                    doc.ui_state.selection.cursor_position = new_cursor_pos;
                    doc.timestamp = chrono::Utc::now().to_rfc3339();

                    // Convert back to JSON
                    updated_document = serde_json::to_value(&doc).unwrap_or(request.document);

                    // Generate SVG
                    svg_content = crate::renderers::editor::svg::render_editor_svg(&doc, None, None, None).ok();
                }
                Err(e) => {
                    return Json(TransformDocumentResponse {
                        success: false,
                        document: request.document,
                        updated_elements: vec![],
                        message: Some(format!("Failed to delete text: {}", e)),
                        svg: None,
                        formats: None,
                    });
                }
            }
        }
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
                        formats: None,
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
                        formats: None,
            });
        }
    }

    // Update timestamp
    if let Some(timestamp_field) = updated_document.get_mut("timestamp") {
        *timestamp_field = serde_json::Value::String(chrono::Utc::now().to_rfc3339());
    }

    // Generate all formats for tab updates
    let formats = if let Ok(doc) = serde_json::from_value::<crate::parse::Document>(updated_document.clone()) {
        let editor_svg = crate::renderers::editor::svg::render_editor_svg(&doc, None, None, None).ok();
        let lilypond_src = crate::renderers::lilypond::renderer::convert_processed_document_to_lilypond_src(&doc, None).ok();
        let vexflow_renderer = crate::renderers::vexflow::VexFlowRenderer::new();
        let vexflow_data = vexflow_renderer.render_data_from_document(&doc);
        let vexflow_svg = vexflow_data.get("vexflow_js").and_then(|js| js.as_str()).map(|s| s.to_string());

        Some(DocumentFormats {
            vexflow_svg,
            editor_svg: editor_svg.clone(),
            lilypond_svg: None,  // Would need to run lilypond command
            lilypond_src,
            midi: None,
        })
    } else {
        None
    };

    Json(TransformDocumentResponse {
        success: true,
        document: updated_document,
        updated_elements,
        message: Some(format!("Applied {} to {} elements", request.command_type, request.target_uuids.len())),
        svg: svg_content,
        formats,
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
    pub document: serde_json::Value,
    pub formats: DocumentFormats,
    pub message: Option<String>,
}

// Document render endpoint - generates all formats from document
async fn render_document_handler(Json(request): Json<RenderDocumentRequest>) -> impl IntoResponse {
    // Deserialize document to our Document model for rendering
    let mut doc: Document = match serde_json::from_value(request.document.clone()) {
        Ok(d) => d,
        Err(e) => {
            return Json(RenderDocumentResponse {
                success: false,
                document: request.document,
                formats: DocumentFormats { vexflow_svg: None, editor_svg: None, lilypond_svg: None, lilypond_src: None, midi: None },
                message: Some(format!("Failed to deserialize document: {}", e)),
            }).into_response();
        }
    };

    // Update timestamp to indicate rendering occurred
    doc.timestamp = chrono::Utc::now().to_rfc3339();

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

    // Convert updated document back to JSON
    let updated_document = serde_json::to_value(&doc).unwrap_or(request.document);

    Json(RenderDocumentResponse {
        success: true,
        document: updated_document,
        formats,
        message: Some("Document rendered successfully".to_string()),
    }).into_response()
}

// Document export endpoint
async fn export_document_handler(Json(request): Json<ExportDocumentRequest>) -> impl IntoResponse {
    // Deserialize document and update metadata for export tracking
    let mut doc: Document = match serde_json::from_value(request.document.clone()) {
        Ok(d) => d,
        Err(e) => {
            return Json(ExportDocumentResponse {
                success: false,
                document: request.document,
                format: request.format,
                content: String::new(),
                message: Some(format!("Failed to deserialize document: {}", e)),
            }).into_response();
        }
    };

    // Update timestamp and export metadata
    doc.timestamp = chrono::Utc::now().to_rfc3339();

    match request.format.as_str() {
        "lilypond-png" => {
            // Generate LilyPond source
            match crate::renderers::lilypond::renderer::convert_processed_document_to_lilypond_src(&doc, None) {
                Ok(lilypond_src) => {
                    // Run LilyPond to generate PNG
                    match generate_lilypond_png(&lilypond_src).await {
                        Ok(png_base64) => {
                            let updated_document = serde_json::to_value(&doc).unwrap_or(request.document);
                            Json(ExportDocumentResponse {
                                success: true,
                                document: updated_document,
                                format: request.format,
                                content: png_base64,
                                message: Some("LilyPond PNG generated successfully".to_string()),
                            }).into_response()
                        }
                        Err(e) => {
                            Json(ExportDocumentResponse {
                                success: false,
                                document: request.document,
                                format: request.format,
                                content: String::new(),
                                message: Some(format!("Failed to generate PNG: {}", e)),
                            }).into_response()
                        }
                    }
                }
                Err(e) => {
                    Json(ExportDocumentResponse {
                        success: false,
                        document: request.document,
                        format: request.format,
                        content: String::new(),
                        message: Some(format!("Failed to generate LilyPond source: {}", e)),
                    }).into_response()
                }
            }
        }
        "lilypond" => {
            // Generate LilyPond source
            match crate::renderers::lilypond::renderer::convert_processed_document_to_lilypond_src(&doc, None) {
                Ok(lilypond_src) => {
                    let updated_document = serde_json::to_value(&doc).unwrap_or(request.document);
                    Json(ExportDocumentResponse {
                        success: true,
                        document: updated_document,
                        format: request.format,
                        content: lilypond_src,
                        message: Some("LilyPond source generated successfully".to_string()),
                    }).into_response()
                }
                Err(e) => {
                    Json(ExportDocumentResponse {
                        success: false,
                        document: request.document,
                        format: request.format,
                        content: String::new(),
                        message: Some(format!("Failed to generate LilyPond source: {}", e)),
                    }).into_response()
                }
            }
        }
        _ => {
            let format_str = request.format.clone();
            Json(ExportDocumentResponse {
                success: false,
                document: request.document,
                format: request.format,
                content: "".to_string(),
                message: Some(format!("Unsupported export format: {}", format_str)),
            }).into_response()
        }
    }
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



async fn health_endpoint() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

/// Insert parsed elements into the document at the specified UUID location
fn insert_elements_at_uuid(
    doc: &mut Document,
    target_uuid: &str,
    element_position: usize,
    elements: Vec<crate::models::ContentElement>,
) -> Result<usize, String> {
    use uuid::Uuid;

    // Parse the target UUID
    let target_uuid = match Uuid::parse_str(target_uuid) {
        Ok(uuid) => uuid,
        Err(_) => return Err(format!("Invalid UUID format: {}", target_uuid)),
    };

    // Find the target element in the document tree
    for stave in &mut doc.elements {
        if let crate::models::DocumentElement::Stave(stave) = stave {
            for line in &mut stave.lines {
                if let crate::models::StaveLine::ContentLine(content_line) = line {
                    // Check if the target UUID matches the content line itself
                    if content_line.id == target_uuid {
                        // Insert elements at the beginning of this content line
                        for (i, element) in elements.into_iter().enumerate() {
                            content_line.elements.insert(i, element);
                        }
                        return Ok(0); // Return cursor position at beginning
                    }

                    // Check individual elements within the content line
                    for (content_idx, content_element) in content_line.elements.iter_mut().enumerate() {
                        match content_element {
                            crate::models::ContentElement::Beat(beat) => {
                                // Check if this is the target beat
                                if beat.id == target_uuid {
                                    // Insert elements into this beat
                                    let insert_position = element_position.min(beat.elements.len());

                                    // Convert ContentElements to BeatElements
                                    for element in elements {
                                        match element {
                                            crate::models::ContentElement::Beat(source_beat) => {
                                                // Insert all elements from the source beat
                                                for (i, beat_element) in source_beat.elements.into_iter().enumerate() {
                                                    beat.elements.insert(insert_position + i, beat_element);
                                                }
                                            }
                                            _ => {
                                                // For non-beat elements, we'd need additional logic
                                                return Err("Cannot insert non-beat content into a beat".to_string());
                                            }
                                        }
                                    }

                                    // Return new cursor position (for now, just return current position)
                                    return Ok(0); // TODO: Calculate proper cursor position
                                }

                                // Also check individual elements within the beat
                                for (beat_element_idx, beat_element) in beat.elements.iter().enumerate() {
                                    let element_uuid = match beat_element {
                                        crate::models::BeatElement::Note(note) => note.id,
                                        crate::models::BeatElement::Dash(dash) => dash.id,
                                        crate::models::BeatElement::BreathMark(breath) => breath.id,
                                        crate::models::BeatElement::Rest(rest) => rest.id,
                                    };

                                    if element_uuid == target_uuid {
                                        // Insert at this specific element position
                                        // For now, insert after this element
                                        for element in elements {
                                            match element {
                                                crate::models::ContentElement::Beat(source_beat) => {
                                                    for (i, new_beat_element) in source_beat.elements.into_iter().enumerate() {
                                                        beat.elements.insert(beat_element_idx + 1 + i, new_beat_element);
                                                    }
                                                }
                                                _ => {
                                                    return Err("Cannot insert non-beat content into beat element".to_string());
                                                }
                                            }
                                        }
                                        return Ok(0); // TODO: Calculate proper cursor position
                                    }
                                }
                            }
                            _ => {
                                // Handle other content element types if needed
                            }
                        }
                    }
                }
            }
        }
    }

    Err(format!("Element with UUID {} not found in document", target_uuid))
}


// New API endpoint for canvas SVG with cursor position support
async fn render_editor_svg(Json(request): Json<CanvasSvgRequest>) -> impl IntoResponse {
    // This endpoint now assumes the client provides or the server derives the Document elsewhere;
    // for compatibility, keep it simple: return 400 since raw input_text is removed.
    (StatusCode::BAD_REQUEST, "render_editor_svg requires a Document; raw input_text is not supported").into_response()
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
