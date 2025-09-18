use crate::parse::model::{Document, DocumentElement, StaveLine};
use crate::rhythm::{Item, analyzer::process_rhythm_immutable};
use crate::rhythm::types::ParsedElement;

/// Pipeline step: Enhance document staves with rhythm analysis
///
/// This function takes the spatially-processed Document and creates beat structures
/// directly from the enhanced ParsedElements, preserving spatial assignments like octave markers.
pub fn analyze_rhythm(mut document: Document) -> Result<Document, String> {
    for element in &mut document.elements {
        if let DocumentElement::Stave(stave) = element {
            // Find the content line in this stave and convert it to rhythm items
            for line in &stave.lines {
                match line {
                    StaveLine::Content(content_elements) => {
                        // Process the spatially-enhanced elements directly, preserving spatial data
                        let rhythm_items = process_rhythm_immutable(content_elements);
                        // TODO: rhythm_items field removed, beats now in ContentLine elements
                        break; // Assume one content line per stave
                    }
                    StaveLine::ContentLine(content_line) => {
                        // Already has elements! Extract beats from them
                        let beats: Vec<_> = content_line.elements.iter()
                            .filter_map(|elem| {
                                if let crate::parse::model::ContentElement::Beat(beat) = elem {
                                    Some(beat.clone())
                                } else {
                                    None
                                }
                            })
                            .collect();
                        // TODO: rhythm_items field removed, beats now in ContentLine elements
                        break;
                    }
                    _ => {}
                }
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