use crate::document::pest_interface::{Pair, Rule};
use crate::document::model::{ContentLine, MusicalElement, TextLine, NotationSystem};
use super::helpers::source_from_span;
use super::pitch::{transform_pitch, resolve_notation_system};

#[derive(Debug, Clone)]
struct UnderlineSpan {
    start_col: usize,
    end_col: usize,
}

fn detect_underline_spans(text_lines: &[TextLine]) -> Vec<UnderlineSpan> {
    let mut spans = Vec::new();
    
    for text_line in text_lines {
        let content = &text_line.content;
        let mut start_col = None;
        
        for (col, ch) in content.chars().enumerate() {
            if ch == '_' {
                if start_col.is_none() {
                    start_col = Some(col);
                }
            } else if let Some(start) = start_col {
                // End of underline span
                spans.push(UnderlineSpan {
                    start_col: start,
                    end_col: col - 1,
                });
                start_col = None;
            }
        }
        
        // Handle underline that goes to end of line
        if let Some(start) = start_col {
            spans.push(UnderlineSpan {
                start_col: start,
                end_col: content.len() - 1,
            });
        }
    }
    
    spans
}

fn is_position_in_spans(position: usize, spans: &[UnderlineSpan]) -> bool {
    spans.iter().any(|span| position >= span.start_col && position <= span.end_col)
}


fn detect_dominant_notation_system_from_elements(elements: &[MusicalElement]) -> NotationSystem {
    let mut counts = [0; 4]; // [Number, Western, Sargam, Bhatkhande]
    
    for element in elements {
        if let MusicalElement::Note(note) = element {
            let idx = match note.notation_system {
                NotationSystem::Number => 0,
                NotationSystem::Western => 1,
                NotationSystem::Sargam => 2,
                NotationSystem::Bhatkhande => 3,
            };
            counts[idx] += 1;
        }
    }
    
    // Priority: Bhatkhande > others (most specific)
    if counts[3] > 0 {
        NotationSystem::Bhatkhande
    } else {
        // Find the system with maximum count
        let max_idx = counts.iter().enumerate().max_by_key(|(_, count)| *count).map(|(idx, _)| idx).unwrap_or(1);
        match max_idx {
            0 => NotationSystem::Number,
            1 => NotationSystem::Western,
            2 => NotationSystem::Sargam,
            3 => NotationSystem::Bhatkhande,
            _ => NotationSystem::Western, // Default fallback
        }
    }
}

fn update_element_notation_systems(elements: &mut [MusicalElement], dominant_system: NotationSystem) {
    for element in elements {
        if let MusicalElement::Note(note) = element {
            // Update ambiguous notes to use the dominant system
            let resolved_system = resolve_notation_system(&note.syllable, dominant_system);
            note.notation_system = resolved_system;
        }
    }
}


pub(super) fn transform_content_line(
    content_pair: Pair<Rule>, 
    text_lines_before: &[TextLine], 
    text_lines_after: &[TextLine]
) -> Result<(ContentLine, NotationSystem), String> {
    let mut elements = Vec::new();
    let source = source_from_span(content_pair.as_span());
    
    // First pass: create content line with default/detected notation systems
    // Detect spatial annotations from text lines
    let slur_spans = detect_underline_spans(text_lines_before);
    let beat_group_spans = detect_underline_spans(text_lines_after);
    
    // Get the content line span information before consuming the pair
    let content_start = content_pair.as_span().start();
    
    for inner_pair in content_pair.into_inner() {
        // Calculate the actual column position within the content line text
        let element_start_pos = inner_pair.as_span().start() - content_start;
        let in_slur = is_position_in_spans(element_start_pos, &slur_spans);
        let in_beat_group = is_position_in_spans(element_start_pos, &beat_group_spans);
        
        match inner_pair.as_rule() {
            Rule::musical_element_no_barline => {
                elements.push(transform_musical_element_no_barline(inner_pair, in_slur, in_beat_group, NotationSystem::Number)?);
            }
            Rule::barline => {
                // Add the structural barline as an element
                elements.push(MusicalElement::Barline {
                    source: source_from_span(inner_pair.as_span()),
                    in_slur,
                    in_beat_group,
                });
            }
            Rule::musical_element => {
                elements.push(transform_musical_element(inner_pair, in_slur, in_beat_group, NotationSystem::Number)?);
            }
            _ => {}
        }
    }
    
    // Second pass: detect dominant notation system from parsed notes and update them
    let dominant_notation_system = detect_dominant_notation_system_from_elements(&elements);
    update_element_notation_systems(&mut elements, dominant_notation_system);
    
    Ok((ContentLine { elements, source }, dominant_notation_system))
}

fn transform_musical_element(element_pair: Pair<Rule>, in_slur: bool, in_beat_group: bool, context_notation_system: NotationSystem) -> Result<MusicalElement, String> {
    for inner_pair in element_pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::pitch => {
                return transform_pitch(inner_pair, in_slur, in_beat_group, context_notation_system);
            }
            Rule::barline => {
                return Ok(MusicalElement::Barline {
                    source: source_from_span(inner_pair.as_span()),
                    in_slur,
                    in_beat_group,
                });
            }
            Rule::space => {
                let space_count = inner_pair.as_str().len();
                return Ok(MusicalElement::Space { 
                    count: space_count,
                    source: source_from_span(inner_pair.as_span()),
                    in_slur,
                    in_beat_group,
                });
            }
            Rule::dash => {
                return Ok(MusicalElement::Dash {
                    source: source_from_span(inner_pair.as_span()),
                    in_slur,
                    in_beat_group,
                });
            }
            _ => {}
        }
    }
    Err("Unknown musical element".to_string())
}

fn transform_musical_element_no_barline(element_pair: Pair<Rule>, in_slur: bool, in_beat_group: bool, context_notation_system: NotationSystem) -> Result<MusicalElement, String> {
    for inner_pair in element_pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::pitch => {
                return transform_pitch(inner_pair, in_slur, in_beat_group, context_notation_system);
            }
            Rule::space => {
                let space_count = inner_pair.as_str().len();
                return Ok(MusicalElement::Space { 
                    count: space_count,
                    source: source_from_span(inner_pair.as_span()),
                    in_slur,
                    in_beat_group,
                });
            }
            Rule::dash => {
                return Ok(MusicalElement::Dash {
                    source: source_from_span(inner_pair.as_span()),
                    in_slur,
                    in_beat_group,
                });
            }
            _ => {}
        }
    }
    Err("Unknown musical element (no barline)".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::model::{TextLine, Source, Position};

    #[test]
    fn test_detect_underline_spans() {
        let text_lines = vec![
            TextLine {
                content: "1___2".to_string(),
                source: Source {
                    value: "1___2".to_string(),
                    position: Position { line: 1, column: 1 },
                },
            },
        ];
        
        let spans = detect_underline_spans(&text_lines);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].start_col, 1);
        assert_eq!(spans[0].end_col, 3);
    }

    #[test] 
    fn test_is_position_in_spans() {
        let spans = vec![
            UnderlineSpan { start_col: 1, end_col: 3 },
            UnderlineSpan { start_col: 5, end_col: 7 },
        ];
        
        assert!(!is_position_in_spans(0, &spans));
        assert!(is_position_in_spans(1, &spans));
        assert!(is_position_in_spans(2, &spans)); 
        assert!(is_position_in_spans(3, &spans));
        assert!(!is_position_in_spans(4, &spans));
        assert!(is_position_in_spans(5, &spans));
        assert!(is_position_in_spans(7, &spans));
        assert!(!is_position_in_spans(8, &spans));
    }

    #[test]
    fn test_spatial_attributes_basic() {
        use crate::document::parse_document;
        
        // Test basic functionality - spatial attributes should default to false
        let input = "|1 2";
        let document = parse_document(input).unwrap();
        
        assert_eq!(document.staves.len(), 1);
        let stave = &document.staves[0];
        
        // Should have no text lines
        assert_eq!(stave.text_lines_before.len(), 0);
        assert_eq!(stave.text_lines_after.len(), 0);
        
        // Check content line elements
        let elements = &stave.content_line.elements;
        
        // Find the Note elements and check their spatial attributes are false
        let mut found_notes = 0;
        for element in elements {
            if let MusicalElement::Note(note) = element {
                assert_eq!(note.in_slur, false);
                assert_eq!(note.in_beat_group, false);
                found_notes += 1;
            }
        }
        
        assert_eq!(found_notes, 2); // Should find notes "1" and "2"
    }

    #[test]
    fn test_spatial_analysis_with_mock_data() {
        use crate::document::model::{TextLine, Source, Position};
        
        // Test the spatial analysis logic directly with mock data
        let text_lines_before = vec![
            TextLine {
                content: " ___  ".to_string(), // Underline from col 1 to col 3
                source: Source {
                    value: " ___  ".to_string(),
                    position: Position { line: 1, column: 1 },
                },
            },
        ];
        
        let slur_spans = detect_underline_spans(&text_lines_before);
        assert_eq!(slur_spans.len(), 1);
        assert_eq!(slur_spans[0].start_col, 1);
        assert_eq!(slur_spans[0].end_col, 3);
        
        // Test position checking
        assert!(!is_position_in_spans(0, &slur_spans)); // Before underline
        assert!(is_position_in_spans(1, &slur_spans));  // Start of underline
        assert!(is_position_in_spans(2, &slur_spans));  // Middle of underline
        assert!(is_position_in_spans(3, &slur_spans));  // End of underline
        assert!(!is_position_in_spans(4, &slur_spans)); // After underline
    }

    #[test]
    fn test_spatial_analysis_ignores_content_line() {
        use crate::document::model::{TextLine, Source, Position};
        
        // This test demonstrates that spatial analysis correctly analyzes TEXT LINES
        // and ignores the content line itself, mapping positions correctly.
        
        // Mock text lines with underlines that would correspond to content line positions
        let text_lines_before = vec![
            TextLine {
                // Imagine content line is "|1 2 3" (positions: |=0, 1=1, space=2, 2=3, space=4, 3=5)
                // Underline at positions 1-3 should cover note "1" and the space after it
                content: " ___   ".to_string(), // Underline from col 1 to col 3
                source: Source {
                    value: " ___   ".to_string(),
                    position: Position { line: 1, column: 1 },
                },
            },
        ];
        
        let text_lines_after = vec![
            TextLine {
                // Beat group underline at positions 3-5 should cover note "2" and note "3"  
                content: "   ___".to_string(), // Underline from col 3 to col 5
                source: Source {
                    value: "   ___".to_string(),
                    position: Position { line: 2, column: 1 },
                },
            },
        ];
        
        // Test the underline detection
        let slur_spans = detect_underline_spans(&text_lines_before);
        let beat_spans = detect_underline_spans(&text_lines_after);
        
        println!("Slur spans detected: {:?}", slur_spans);
        println!("Beat spans detected: {:?}", beat_spans);
        
        assert_eq!(slur_spans.len(), 1);
        assert_eq!(beat_spans.len(), 1);
        
        // Test position mapping for a simulated content line "|1 2 3"
        // Content positions: | = 0, 1 = 1, (space) = 2, 2 = 3, (space) = 4, 3 = 5
        
        // Position 1 (note "1") should be in slur only
        assert!(is_position_in_spans(1, &slur_spans));
        assert!(!is_position_in_spans(1, &beat_spans));
        
        // Position 3 (note "2") is at the boundary - in BOTH spans (overlapping)
        println!("Position 3 in slur spans? {}", is_position_in_spans(3, &slur_spans));
        println!("Position 3 in beat spans? {}", is_position_in_spans(3, &beat_spans));
        assert!(is_position_in_spans(3, &slur_spans));   // End of slur
        assert!(is_position_in_spans(3, &beat_spans));   // Start of beat group
        
        // Position 5 (note "3") should be in beat group only
        assert!(!is_position_in_spans(5, &slur_spans));
        assert!(is_position_in_spans(5, &beat_spans));
        
        // Position 2 should be in slur only
        assert!(is_position_in_spans(2, &slur_spans));
        assert!(!is_position_in_spans(2, &beat_spans));
        
        // Position 4 should be in beat group only
        assert!(!is_position_in_spans(4, &slur_spans));
        assert!(is_position_in_spans(4, &beat_spans));
        
        println!("✓ Spatial analysis correctly maps text line underlines to content positions");
        println!("✓ Slur spans: {:?}", slur_spans);
        println!("✓ Beat group spans: {:?}", beat_spans);
    }

    #[test]
    fn test_parse_with_text_line() {
        use crate::document::parse_document;
        
        // Test with a pre-text line - trying different formats
        let test_cases = vec![
            "text\n|1 2",
            "___\n|1 2",
            "1___2\n|1 2",
        ];
        
        for (i, input) in test_cases.iter().enumerate() {
            println!("Testing case {}: '{}'", i, input);
            match parse_document(input) {
                Ok(document) => {
                    println!("  Success! staves: {}, text_lines_before: {}", 
                        document.staves.len(), 
                        document.staves[0].text_lines_before.len()
                    );
                    if !document.staves[0].text_lines_before.is_empty() {
                        println!("  text_line content: '{}'", 
                            document.staves[0].text_lines_before[0].content
                        );
                    }
                }
                Err(e) => {
                    println!("  Error: {}", e);
                }
            }
        }
    }
}