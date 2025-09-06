use axum::{
    extract::Query,
    response::Json,
    routing::get,
    Router,
};
use serde::Serialize;
use std::collections::HashMap;
use tower_http::{cors::CorsLayer, services::ServeDir};
use music_text::{parse_notation, pest_pair_to_json, process_notation};


#[derive(Serialize)]
struct ParseResponse {
    success: bool,
    pest_output: Option<serde_json::Value>,
    parsed_document: Option<serde_json::Value>,
    processed_staves: Option<serde_json::Value>,
    minimal_lilypond: Option<String>,
    full_lilypond: Option<String>,
    lilypond_svg: Option<String>,
    vexflow: Option<serde_json::Value>,
    error: Option<String>,
}

// pest_pair_to_json is now imported from parser module

async fn parse_text(Query(params): Query<HashMap<String, String>>) -> Json<ParseResponse> {
    let input = params.get("input").cloned().unwrap_or_default();
    
    if input.trim().is_empty() {
        return Json(ParseResponse {
            success: true,
            pest_output: None,
            parsed_document: None,
            processed_staves: None,
            minimal_lilypond: None,
            full_lilypond: None,
            lilypond_svg: None,
            vexflow: None,
            error: None,
        });
    }
    
    // Get PEST output
    let pest_result = match parse_notation(&input) {
        Ok(pairs) => {
            let result: Vec<serde_json::Value> = pairs
                .map(|pair| pest_pair_to_json(&pair))
                .collect();
            Some(serde_json::Value::Array(result))
        }
        Err(e) => {
            return Json(ParseResponse {
                success: false,
                pest_output: None,
                parsed_document: None,
                processed_staves: None,
                minimal_lilypond: None,
                full_lilypond: None,
                lilypond_svg: None,
                vexflow: None,
                error: Some(format!("{}", e)),
            });
        }
    };
    
    // Get pipeline processing result
    let (parsed_doc, processed_staves) = match process_notation(&input) {
        Ok(result) => (
            Some(serde_json::to_value(result.parsed_document).unwrap()),
            Some(serde_json::to_value(result.processed_staves).unwrap()),
        ),
        Err(_) => (None, None),
    };
    
    // Generate dummy data for other formats
    let minimal_lilypond = Some(format!("\\version \"2.24.0\"\n{{ {} }}", 
        input.chars().filter(|c| c.is_numeric() || c.is_alphabetic())
            .map(|c| format!("{} ", c))
            .collect::<String>()));
    
    let full_lilypond = Some(format!(
        "\\version \"2.24.0\"\n\\paper {{\n  #(set-paper-size \"a4\")\n}}\n\\score {{\n  \\new Staff {{\n    \\clef treble\n    {} \n  }}\n  \\layout {{ }}\n  \\midi {{ }}\n}}", 
        input.chars().filter(|c| c.is_numeric() || c.is_alphabetic())
            .map(|c| format!("{}4 ", c.to_lowercase()))
            .collect::<String>()
    ));
    
    let lilypond_svg = Some(format!(
        "<svg width='500' height='200' xmlns='http://www.w3.org/2000/svg'>
        <rect width='500' height='200' fill='#fffffb' stroke='#333' stroke-width='1'/>
        
        <!-- Title -->
        <text x='20' y='25' font-family='serif' font-size='16' font-weight='bold' fill='#333'>Bach-style Musical Notation (Demo)</text>
        <text x='20' y='45' font-family='monospace' font-size='11' fill='#666'>Input: {}</text>
        
        <!-- Staff lines -->
        <line x1='40' y1='80' x2='460' y2='80' stroke='#333' stroke-width='1'/>
        <line x1='40' y1='90' x2='460' y2='90' stroke='#333' stroke-width='1'/>
        <line x1='40' y1='100' x2='460' y2='100' stroke='#333' stroke-width='1'/>
        <line x1='40' y1='110' x2='460' y2='110' stroke='#333' stroke-width='1'/>
        <line x1='40' y1='120' x2='460' y2='120' stroke='#333' stroke-width='1'/>
        
        <!-- Treble clef -->
        <text x='50' y='115' font-family='serif' font-size='40' fill='#333'>𝄞</text>
        
        <!-- Notes (Bach-inspired pattern) -->
        <circle cx='100' cy='90' r='4' fill='#333'/>
        <line x1='104' y1='90' x2='104' y2='60' stroke='#333' stroke-width='2'/>
        
        <circle cx='130' cy='95' r='4' fill='#333'/>
        <line x1='134' y1='95' x2='134' y2='65' stroke='#333' stroke-width='2'/>
        
        <circle cx='160' cy='85' r='4' fill='#333'/>
        <line x1='164' y1='85' x2='164' y2='55' stroke='#333' stroke-width='2'/>
        
        <circle cx='190' cy='90' r='4' fill='#333'/>
        <line x1='194' y1='90' x2='194' y2='60' stroke='#333' stroke-width='2'/>
        
        <!-- Bar line -->
        <line x1='220' y1='80' x2='220' y2='120' stroke='#333' stroke-width='2'/>
        
        <!-- More notes -->
        <circle cx='250' cy='100' r='4' fill='#333'/>
        <line x1='254' y1='100' x2='254' y2='70' stroke='#333' stroke-width='2'/>
        
        <circle cx='280' cy='105' r='4' fill='#333'/>
        <line x1='284' y1='105' x2='284' y2='75' stroke='#333' stroke-width='2'/>
        
        <circle cx='310' cy='95' r='4' fill='#333'/>
        <line x1='314' y1='95' x2='314' y2='65' stroke='#333' stroke-width='2'/>
        
        <!-- Time signature -->
        <text x='80' y='95' font-family='serif' font-size='18' fill='#333'>4</text>
        <text x='80' y='110' font-family='serif' font-size='18' fill='#333'>4</text>
        
        <!-- Footer -->
        <text x='20' y='160' font-family='serif' font-size='12' fill='#666'>Generated by Music Text Parser</text>
        <text x='20' y='180' font-family='serif' font-size='10' fill='#999'>In the style of J.S. Bach • Placeholder SVG</text>
        </svg>", 
        input.chars().take(40).collect::<String>()
    ));
    
    let vexflow = Some(serde_json::json!({
        "notes": input.chars().filter(|c| c.is_numeric() || c.is_alphabetic())
            .take(8)
            .map(|c| {
                serde_json::json!({
                    "keys": [format!("{}/4", c.to_lowercase())],
                    "duration": "q"
                })
            })
            .collect::<Vec<_>>(),
        "time_signature": "4/4",
        "dummy": true
    }));
    
    Json(ParseResponse {
        success: true,
        pest_output: pest_result,
        parsed_document: parsed_doc,
        processed_staves,
        minimal_lilypond,
        full_lilypond,
        lilypond_svg,
        vexflow,
        error: None,
    })
}

// parse_document function removed - now integrated into parse_text

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/parse", get(parse_text))
        .nest_service("/", ServeDir::new("webapp"))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
        .await
        .unwrap();
        
    println!("Server running on http://127.0.0.1:3001");
    
    axum::serve(listener, app).await.unwrap();
}