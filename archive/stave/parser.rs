use crate::parse::Document;
use crate::rhythm::process_rhythm_batch;

/// Pipeline step: Enhance document staves with rhythm analysis
/// 
/// This function takes the Document from document_parser and enhances the ParsedElements
/// in-place with rhythm analysis data, maintaining the same Document structure.
pub fn analyze_rhythm(mut document: Document) -> Result<Document, String> {
    // Collect all content lines for batch processing
    let all_content_lines: Vec<&Vec<crate::rhythm::types::ParsedElement>> = document.staves
        .iter()
        .map(|stave| &stave.content_line)
        .collect();
    
    // Process rhythm for all staves in batch
    let all_rhythm_items = process_rhythm_batch(&all_content_lines);
    
    // Preserve Beat structures for renderers (don't mutate content_line)  
    for (stave_idx, stave) in document.staves.iter_mut().enumerate() {
        if let Some(rhythm_items) = all_rhythm_items.get(stave_idx) {
            // Store Beat structures for renderers to use directly
            stave.rhythm_items = Some(rhythm_items.clone());
        }
    }
    
    Ok(document)
}

// Old flattening function removed - renderers now use rhythm_items directly

/// A stave that has been processed through the rhythm FSM
/// Contains beat-grouped elements instead of flat element lists
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessedStave {
    pub text_lines_before: Vec<crate::parse::TextLine>,
    pub rhythm_items: Vec<crate::rhythm::Item>,
    pub text_lines_after: Vec<crate::parse::TextLine>,
    pub upper_lines: Vec<crate::parse::model::UpperLine>,  // Preserve for rendering unknown tokens
    pub lower_lines: Vec<crate::parse::model::LowerLine>,  // Preserve for rendering unknown tokens
    pub lyrics_lines: Vec<crate::parse::model::LyricsLine>,
    pub notation_system: crate::parse::model::NotationSystem,
    pub source: crate::parse::model::Source,
    pub begin_multi_stave: bool,  // True if this stave begins a multi-stave group
    pub end_multi_stave: bool,    // True if this stave ends a multi-stave group
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_document;

    #[test]
    fn test_analyze_rhythm_single_stave() {
        let input = "|1 2 3";
        let document = parse_document(input).unwrap();
        let analyzed = analyze_rhythm(document).unwrap();
        
        // Should return the same staves as input with rhythm analysis
        assert_eq!(analyzed.staves.len(), 1);
        assert!(analyzed.staves[0].rhythm_items.is_some()); // Should have rhythm items
    }

    #[test]
    fn test_analyze_rhythm_multi_stave() {
        let input = "|1 2\n\n|3 4";
        let document = parse_document(input).unwrap();
        let analyzed = analyze_rhythm(document).unwrap();
        
        // Should return both staves
        assert_eq!(analyzed.staves.len(), 2);
    }
}