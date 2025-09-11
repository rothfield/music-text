// Classification logic to convert raw parse results into final AST
// Phase 2 of the two-phase parsing approach

use crate::ast::{Stave, AnnotationLine, AnnotationItem, LyricsLine};
use crate::ast::raw::{RawStave, RawAnnotationLine, RawAnnotationContent, UpperItem, LowerItem};
use crate::models::Position;

#[derive(Debug)]
pub enum ClassificationError {
    InvalidPreContentLine,
    InvalidPostContentLine,
    PositionLost,
}

impl std::fmt::Display for ClassificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClassificationError::InvalidPreContentLine => write!(f, "Invalid pre-content line"),
            ClassificationError::InvalidPostContentLine => write!(f, "Invalid post-content line"),
            ClassificationError::PositionLost => write!(f, "Position information lost during classification"),
        }
    }
}

impl std::error::Error for ClassificationError {}

/// Convert raw stave (from Phase 1 parsing) into final classified stave
pub fn classify_raw_stave(raw_stave: RawStave) -> Result<Stave, ClassificationError> {
    let mut stave = Stave {
        upper_lines: Vec::new(),
        content_line: raw_stave.content_line,
        lower_lines: Vec::new(),
        lyrics_lines: Vec::new(),
        position: raw_stave.position,
    };
    
    // Convert pre-content lines to upper_lines
    for raw_line in raw_stave.pre_content_lines {
        match raw_line.content {
            RawAnnotationContent::Upper(items) => {
                let annotation_line = convert_upper_items_to_annotation_line(items, raw_line.position)?;
                stave.upper_lines.push(annotation_line);
            }
            RawAnnotationContent::Lyrics(syllables) => {
                let lyrics_line = LyricsLine { syllables };
                stave.lyrics_lines.push(lyrics_line);
            }
            _ => return Err(ClassificationError::InvalidPreContentLine),
        }
    }
    
    // Convert post-content lines to lower_lines/lyrics_lines
    for raw_line in raw_stave.post_content_lines {
        match raw_line.content {
            RawAnnotationContent::Lower(items) => {
                let annotation_line = convert_lower_items_to_annotation_line(items, raw_line.position)?;
                stave.lower_lines.push(annotation_line);
            }
            RawAnnotationContent::Lyrics(syllables) => {
                let lyrics_line = LyricsLine { syllables };
                stave.lyrics_lines.push(lyrics_line);
            }
            _ => return Err(ClassificationError::InvalidPostContentLine),
        }
    }
    
    Ok(stave)
}

/// Convert upper items to annotation line  
fn convert_upper_items_to_annotation_line(
    items: Vec<UpperItem>, 
    position: Option<Position>
) -> Result<AnnotationLine, ClassificationError> {
    let mut annotation_items = Vec::new();
    
    for item in items {
        let annotation_item = match item {
            UpperItem::OctaveMarker { marker, position } => {
                AnnotationItem::UpperOctaveMarker { marker, position }
            }
            UpperItem::Tala { marker, position } => {
                AnnotationItem::Tala { marker, position }
            }
            UpperItem::Ornament { pitches, position } => {
                AnnotationItem::Ornament { pitches, position }
            }
            UpperItem::Chord { chord, position } => {
                AnnotationItem::Chord { chord, position }
            }
            UpperItem::Slur { underscores, position } => {
                AnnotationItem::Slur { underscores, position }
            }
            UpperItem::Ending { ending, position } => {
                AnnotationItem::Ending { ending, position }
            }
            UpperItem::Mordent { position } => {
                AnnotationItem::Mordent { position }
            }
            UpperItem::Space { count, position } => {
                AnnotationItem::Space { count, position }
            }
        };
        annotation_items.push(annotation_item);
    }
    
    Ok(AnnotationLine { items: annotation_items })
}

/// Convert lower items to annotation line
fn convert_lower_items_to_annotation_line(
    items: Vec<LowerItem>,
    position: Option<Position>
) -> Result<AnnotationLine, ClassificationError> {
    let mut annotation_items = Vec::new();
    
    for item in items {
        let annotation_item = match item {
            LowerItem::OctaveMarker { marker, position } => {
                AnnotationItem::LowerOctaveMarker { marker, position }
            }
            LowerItem::KommalIndicator { position } => {
                // Create a symbol for kommal indicator
                AnnotationItem::Symbol { symbol: "_".to_string(), position }
            }
            LowerItem::BeatGrouping { underscores, position } => {
                AnnotationItem::BeatGrouping { underscores, position }
            }
            LowerItem::Space { count, position } => {
                AnnotationItem::Space { count, position }
            }
        };
        annotation_items.push(annotation_item);
    }
    
    Ok(AnnotationLine { items: annotation_items })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{ContentLine, Measure, Beat};
    
    #[test]
    fn test_classify_upper_octave_marker() {
        let raw_stave = RawStave {
            pre_content_lines: vec![
                RawAnnotationLine {
                    content: RawAnnotationContent::Upper(vec![
                        UpperItem::OctaveMarker { 
                            marker: ".".to_string(), 
                            position: Some(Position { row: 1, col: 1 }) 
                        }
                    ]),
                    position: Some(Position { row: 1, col: 1 }),
                }
            ],
            content_line: ContentLine { line_number: None, measures: Vec::new() },
            post_content_lines: Vec::new(),
            position: None,
        };
        
        let stave = classify_raw_stave(raw_stave).unwrap();
        
        assert_eq!(stave.upper_lines.len(), 1);
        assert!(matches!(
            &stave.upper_lines[0].items[0],
            AnnotationItem::UpperOctaveMarker { marker, .. } if marker == "."
        ));
    }
    
    #[test]
    fn test_classify_lower_octave_marker() {
        let raw_stave = RawStave {
            pre_content_lines: Vec::new(),
            content_line: ContentLine { line_number: None, measures: Vec::new() },
            post_content_lines: vec![
                RawAnnotationLine {
                    content: RawAnnotationContent::Lower(vec![
                        LowerItem::OctaveMarker { 
                            marker: ".".to_string(), 
                            position: Some(Position { row: 2, col: 1 }) 
                        }
                    ]),
                    position: Some(Position { row: 2, col: 1 }),
                }
            ],
            position: None,
        };
        
        let stave = classify_raw_stave(raw_stave).unwrap();
        
        assert_eq!(stave.lower_lines.len(), 1);
        assert!(matches!(
            &stave.lower_lines[0].items[0],
            AnnotationItem::LowerOctaveMarker { marker, .. } if marker == "."
        ));
    }
    
    #[test]
    fn test_position_preservation() {
        let raw_stave = RawStave {
            pre_content_lines: Vec::new(),
            content_line: ContentLine { line_number: None, measures: Vec::new() },
            post_content_lines: vec![
                RawAnnotationLine {
                    content: RawAnnotationContent::Lower(vec![
                        LowerItem::OctaveMarker { 
                            marker: ".".to_string(), 
                            position: Some(Position { row: 2, col: 1 }) 
                        }
                    ]),
                    position: Some(Position { row: 2, col: 1 }),
                }
            ],
            position: None,
        };
        
        let stave = classify_raw_stave(raw_stave).unwrap();
        
        // Verify position data preserved
        let lower_marker = &stave.lower_lines[0].items[0];
        if let AnnotationItem::LowerOctaveMarker { position: Some(pos), .. } = lower_marker {
            assert_eq!(pos.row, 2);
            assert_eq!(pos.col, 1);
        } else {
            panic!("Expected LowerOctaveMarker with position");
        }
    }
}