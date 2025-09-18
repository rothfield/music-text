pub mod parse;
pub mod web;
pub mod rhythm;
pub mod renderers;
pub mod pipeline;
pub mod old_pipeline;
pub mod models;
pub mod stave_analyzer;
pub mod spatial;
pub mod tree_functions;

pub fn parse(input: &str, _system: Option<&str>) -> Result<parse::Document, parse::ParseError> {
    parse::parse_document(input)
}

// Re-export the pipeline function for the web server
pub use pipeline::process_notation;