use crate::parse::model::{Document, DocumentElement, StaveLine};
use crate::rhythm::process_rhythm_batch;

/// Pipeline step: Enhance document staves with rhythm analysis
///
/// This function takes the Document and enhances staves with rhythm analysis data,
/// adapted to work with the current Document structure (elements/StaveLine format).
pub fn analyze_rhythm(mut document: Document) -> Result<Document, String> {
    // Collect all content lines for batch processing
    let mut all_content_lines = Vec::new();

    for element in &document.elements {
        if let DocumentElement::Stave(stave) = element {
            // Find the content line in this stave
            for line in &stave.lines {
                if let StaveLine::Content(content_elements) = line {
                    all_content_lines.push(content_elements);
                    break; // Assume one content line per stave
                }
            }
        }
    }

    // Process rhythm for all staves in batch
    let all_rhythm_items = process_rhythm_batch(&all_content_lines);

    // Store Beat structures for renderers
    let mut rhythm_item_index = 0;
    for element in &mut document.elements {
        if let DocumentElement::Stave(stave) = element {
            if let Some(rhythm_items) = all_rhythm_items.get(rhythm_item_index) {
                // Store Beat structures for renderers to use directly
                stave.rhythm_items = Some(rhythm_items.clone());
                rhythm_item_index += 1;
            }
        }
    }

    Ok(document)
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

        // Should have rhythm items stored in staves
        for element in &analyzed.elements {
            if let DocumentElement::Stave(stave) = element {
                assert!(stave.rhythm_items.is_some(), "Should have rhythm items");
            }
        }
    }

    #[test]
    fn test_analyze_rhythm_multi_stave() {
        let input = "|1 2\n\n|3 4";
        let document = parse_document(input).unwrap();
        let analyzed = analyze_rhythm(document).unwrap();

        // Should have rhythm items for each stave
        let stave_count = analyzed.elements.iter()
            .filter(|e| matches!(e, DocumentElement::Stave(_)))
            .count();
        assert_eq!(stave_count, 2);
    }
}