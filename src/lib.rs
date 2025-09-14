pub mod parse;
pub mod web;
pub mod rhythm;
pub mod renderers;
pub mod pipeline;
pub mod models;

pub fn parse(input: &str, _system: Option<&str>) -> Result<parse::Document, parse::ParseError> {
    parse::parse_document(input)
}

// Re-export the pipeline function for the web server
pub use pipeline::process_notation;