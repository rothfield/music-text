// Raw AST structures for Phase 1 parsing
// These represent the raw parsed structure before position-based classification

use serde::{Deserialize, Serialize};
use crate::models::Position;
use crate::ast::{ContentLine, LyricsLine};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawStave {
    pub pre_content_lines: Vec<RawAnnotationLine>,
    pub content_line: ContentLine,
    pub post_content_lines: Vec<RawAnnotationLine>,
    pub position: Option<Position>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawAnnotationLine {
    pub content: RawAnnotationContent,
    pub position: Option<Position>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RawAnnotationContent {
    Upper(Vec<UpperItem>),
    Lower(Vec<LowerItem>),
    Lyrics(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UpperItem {
    OctaveMarker { marker: String, position: Option<Position> },
    Tala { marker: String, position: Option<Position> },
    Ornament { pitches: Vec<String>, position: Option<Position> },
    Chord { chord: String, position: Option<Position> },
    Slur { underscores: String, position: Option<Position> },
    Ending { ending: String, position: Option<Position> },
    Mordent { position: Option<Position> },
    Space { count: usize, position: Option<Position> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LowerItem {
    OctaveMarker { marker: String, position: Option<Position> },
    KommalIndicator { position: Option<Position> },
    BeatGrouping { underscores: String, position: Option<Position> },
    Space { count: usize, position: Option<Position> },
}