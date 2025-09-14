// Temporarily disable complex renderer modules
// pub mod renderer;
// pub mod templates;
pub mod generator;

// pub use renderer::*;
// pub use templates::*;
pub use generator::*;

// Simple function to render from our Document type
pub fn render_lilypond_from_document(document: &crate::parse::Document) -> String {
    // For now, generate a basic template with the input
    let input = document.source.value.as_ref()
        .map_or("(unknown)", |v| v.as_str());

    format!(
        "\\version \"2.24.0\"\n\\language \"english\"\n\n% Original notation source:\n% {}\n\n\\score {{\n  \\new Staff {{\n    \\fixed c' {{\n      \\key c \\major\n      \\time 4/4\n      \\autoBeamOff\n      c'4 d'4 e'4 f'4\n    }}\n  }}\n}}",
        input
    )
}