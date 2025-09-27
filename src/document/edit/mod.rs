use crate::parse::Document;
use serde_json;

pub mod octave;
pub mod text;
pub mod structural;

/// Execute an edit operation on a document
/// This is the main entry point for all document edit operations
pub fn execute_edit(
    document: &mut Document,
    edit_type: &str,
    target_uuids: &[String],
    params: &serde_json::Value,
) -> Result<(), String> {
    match edit_type {
        "set_octave" => {
            let octave_type = params
                .get("octave_type")
                .and_then(|v| v.as_str())
                .unwrap_or("higher");

            octave::apply_octave_edit(document, target_uuids, octave_type)
        }
        "apply_slur" => {
            // Future: slur::apply_slur_edit(document, target_uuids)
            Ok(())
        }
        _ => Err(format!("Unknown edit type: {}", edit_type)),
    }
}