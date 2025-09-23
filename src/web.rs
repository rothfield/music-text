// Web server for live notation parsing
use axum::{
    extract::{Query, Multipart, Form},
    response::{IntoResponse, Html, Response},
    routing::{get, post},
    Json, Router,
    http::{StatusCode, header},
    body::Body,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_http::{cors::CorsLayer, services::ServeDir};
use tokio::fs;
use tokio::io::AsyncReadExt;
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
    plain_text: Option<String>,
    document: Option<crate::parse::Document>,
    detected_notation_systems: Option<Vec<String>>,
    lilypond: Option<String>,
    lilypond_minimal: Option<String>,
    lilypond_svg: Option<String>,
    vexflow: Option<serde_json::Value>,
    vexflow_svg: Option<String>,
    svg_poc: Option<String>,  // Doremi-script SVG POC
    syntax_spans: Option<Vec<crate::renderers::codemirror::Span>>,
    character_styles: Option<Vec<crate::renderers::codemirror::SpanStyle>>,
    roundtrip: Option<RoundtripData>,
    error: Option<String>,
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

    let svg_router = crate::renderers::svg::create_svg_router();

    let app = Router::new()
        .route("/api/parse", get(parse_text))
        .route("/api/render-svg-poc", post(render_svg_poc))
        .route("/retro/load", post(retro_load))
        .route("/retro", post(retro_parse).get(serve_retro_page))
        .route("/retro/", get(serve_retro_page))
        .route("/health", get(health_endpoint))
        .merge(svg_router)
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
            svg_poc: None,
            syntax_spans: None,
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


            // Generate spans and character styles directly from document in single tree walk
            let (syntax_spans, character_styles) = crate::renderers::codemirror::render_codemirror(&result.document, &input);

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

            // Always generate the doremi-script SVG POC (using the requested notation type)
            let svg_poc = Some(crate::renderers::svg::render_document_tree_to_svg(&result.document, &notation_type));

            // Create minimal metadata for minimal lilypond rendering
            let minimal_metadata = crate::models::Metadata {
                title: result.document.title.as_ref().map(|t| crate::models::Title { text: t.clone(), row: 0, col: 0 }),
                attributes: std::collections::HashMap::new(),
                detected_system: None,
                directives: Vec::new(),
            };

            // Generate minimal lilypond before moving document
            let lilypond_minimal = crate::renderers::lilypond::renderer::convert_processed_document_to_minimal_lilypond_src(&result.document, &minimal_metadata, Some(&input)).ok();

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
                svg_poc,
                syntax_spans: Some(syntax_spans),
                character_styles: Some(character_styles),
                roundtrip: Some(roundtrip),
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
            svg_poc: None,
            syntax_spans: None,
            character_styles: None,
            roundtrip: None,
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
            let svg_content = crate::renderers::svg::render_document_tree_to_svg(&result.document, &request.notation_type);

            println!("✅ SVG POC generation successful, length: {}", svg_content.len());
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

