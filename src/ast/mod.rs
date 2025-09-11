// AST module - Abstract Syntax Tree structures
// Two-phase parsing: Raw AST (Phase 1) → Classified AST (Phase 2)

pub mod raw;

use serde::{Deserialize, Serialize};
use crate::models::Position;

// Final classified AST structures (Phase 2)

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stave {
    pub upper_lines: Vec<AnnotationLine>,
    pub content_line: ContentLine,
    pub lower_lines: Vec<AnnotationLine>,
    pub lyrics_lines: Vec<LyricsLine>,
    pub position: Option<Position>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnnotationLine {
    pub items: Vec<AnnotationItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnnotationItem {
    UpperOctaveMarker { marker: String, position: Option<Position> },
    LowerOctaveMarker { marker: String, position: Option<Position> },
    Tala { marker: String, position: Option<Position> },
    Ornament { pitches: Vec<String>, position: Option<Position> },
    Chord { chord: String, position: Option<Position> },
    Slur { underscores: String, position: Option<Position> },
    Ending { ending: String, position: Option<Position> },
    Mordent { position: Option<Position> },
    BeatGrouping { underscores: String, position: Option<Position> },
    Symbol { symbol: String, position: Option<Position> },
    Space { count: usize, position: Option<Position> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentLine {
    pub line_number: Option<u32>,
    pub measures: Vec<Measure>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Measure {
    pub beats: Vec<Beat>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Beat {
    pub elements: Vec<BeatElement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BeatElement {
    Note { pitch: String },
    Rest,
    Dash,
    Breath,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LyricsLine {
    pub syllables: Vec<String>,
}

// Re-export raw types for convenience
pub use raw::{RawStave, RawAnnotationLine, RawAnnotationContent, UpperItem, LowerItem};