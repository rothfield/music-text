// AST types for the pest grammar-based music-text parser
// Based on doremi-script grammar but with updated terminology

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::models::Position;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    /// Key-value attributes (key: C, time: 4/4, author: John, etc.)
    pub attributes: HashMap<String, String>,
    
    /// Musical staves (the main content)
    pub staves: Vec<Stave>,
    
    /// Detected notation system
    pub notation_system: NotationSystem,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NotationSystem {
    Sargam,    // S R G M P D N
    Number,    // 1 2 3 4 5 6 7
    Western,   // C D E F G A B  
    Abc,       // ABC notation variant
    Doremi,    // d r m f s l t
    Mixed,     // Multiple systems detected
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stave {
    /// Lines above the content (ornaments, octave markers, chords, etc.)
    pub upper_lines: Vec<AnnotationLine>,
    
    /// The main musical content line
    pub content_line: ContentLine,
    
    /// Lines below the content (octave markers, etc.)
    pub lower_lines: Vec<AnnotationLine>,
    
    /// Lyrics lines
    pub lyrics_lines: Vec<LyricsLine>,
    
    /// Position of this stave in the source text
    pub position: Option<Position>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnnotationLine {
    pub items: Vec<AnnotationItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnnotationItem {
    /// Upper octave marker (., :, *, ') - only in upper lines
    UpperOctaveMarker {
        marker: String,
        position: Option<Position>,
    },
    
    /// Tala marker (0-6, +)
    Tala {
        marker: String,
        position: Option<Position>,
    },
    
    /// Ornament sequences (pitch sequences above notes)
    Ornament {
        pitches: Vec<String>,
        position: Option<Position>,
    },
    
    /// Chord notation [Cmaj7]
    Chord {
        chord: String,
        position: Option<Position>,
    },
    
    /// Mordent marker (~)
    Mordent {
        position: Option<Position>,
    },
    
    /// Upper line dots and symbols
    Symbol {
        symbol: String,
        position: Option<Position>,
    },
    
    /// 1st, 2nd ending markers (1.---, 2.___)
    Ending {
        ending: String,
        position: Option<Position>,
    },
    
    /// Slur marking (upper line underscores)
    Slur {
        underscores: String,
        position: Option<Position>,
    },
    
    /// Beat grouping marking (lower line underscores)  
    BeatGrouping {
        underscores: String,
        position: Option<Position>,
    },
    
    /// Lower octave marker (., :, *, ') - only in lower lines
    LowerOctaveMarker {
        marker: String,
        position: Option<Position>,
    },
    
    /// Whitespace (for alignment)
    Space {
        count: usize,
        position: Option<Position>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentLine {
    /// Optional line number (1), 2), etc.)
    pub line_number: Option<u32>,
    
    /// Musical measures
    pub measures: Vec<Measure>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Measure {
    /// Opening barline (optional)
    pub start_barline: Option<Barline>,
    
    /// Beats within the measure
    pub beats: Vec<Beat>,
    
    /// Closing barline (optional)
    pub end_barline: Option<Barline>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Beat {
    /// Beat elements (notes, dashes, slurs)
    pub elements: Vec<BeatElement>,
    // Rhythm analysis fields (populated by FSM)
    pub divisions: Option<usize>,           // Total subdivisions in beat (e.g., 5 for "1-2-3")
    pub is_tuplet: Option<bool>,            // True if not power of 2
    pub tuplet_ratio: Option<(usize, usize)>, // e.g., (5, 4) for 5-tuplet
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SlurType {
    BeginSlur,  // First note under a slur
    InSlur,     // Subsequent notes under the same slur
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BeatElement {
    /// Musical pitch
    Pitch {
        value: String,           // S, 1, C, etc.
        accidental: Option<String>, // #, b
        syllable: Option<String>,   // Assigned syllable from lyrics
        slur_type: Option<SlurType>, // Slur marking from spatial analysis
        octave: i8,                 // Octave number (-4 to 4, default 0)
        // Rhythm analysis fields (populated by FSM)
        subdivisions: Option<usize>, // How many subdivisions this note occupies (e.g., 2 for "1-")
        is_tied: Option<bool>,       // True if tied to next same pitch
        // Source position
        position: Option<Position>,  // Line/column in source text
    },
    
    /// Note extension dash
    Dash {
        position: Option<Position>,  // Line/column in source text
    },
    
    /// Rest (standalone dash with no preceding note)
    Rest {
        subdivisions: Option<usize>, // How many subdivisions this rest occupies
        position: Option<Position>,  // Line/column in source text
    },
    
    /// Slur start
    SlurStart {
        position: Option<Position>,  // Line/column in source text
    },
    
    /// Slur end  
    SlurEnd {
        position: Option<Position>,  // Line/column in source text
    },
    
    /// Whitespace within delimited beats
    Space {
        position: Option<Position>,  // Line/column in source text
    },
    
    /// Breath mark (')
    BreathMark {
        position: Option<Position>,  // Line/column in source text
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Barline {
    Single,           // |
    Double,           // ||
    Final,            // |]
    ReverseFinal,     // [|
    LeftRepeat,       // |:
    RightRepeat,      // :|
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LyricsLine {
    pub syllables: Vec<String>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
            staves: Vec::new(),
            notation_system: NotationSystem::Mixed,
        }
    }
}

/// Print a human-readable AST structure
pub fn print_ast(doc: &Document) {
    println!("Document:");
    println!("  Notation System: {:?}", doc.notation_system);
    
    if !doc.attributes.is_empty() {
        println!("  Attributes:");
        for (key, value) in &doc.attributes {
            println!("    {}: {}", key, value);
        }
    }
    
    println!("  Staves: {}", doc.staves.len());
    for (i, stave) in doc.staves.iter().enumerate() {
        println!("    Stave {}:", i + 1);
        
        if !stave.upper_lines.is_empty() {
            println!("      Upper Lines: {}", stave.upper_lines.len());
        }
        
        println!("      Content Line:");
        if let Some(line_num) = stave.content_line.line_number {
            println!("        Line Number: {}", line_num);
        }
        println!("        Measures: {}", stave.content_line.measures.len());
        
        for (j, measure) in stave.content_line.measures.iter().enumerate() {
            println!("          Measure {}: {} beats", j + 1, measure.beats.len());
            for (k, beat) in measure.beats.iter().enumerate() {
                println!("            Beat {}: {} elements", k + 1, beat.elements.len());
            }
        }
        
        if !stave.lower_lines.is_empty() {
            println!("      Lower Lines: {}", stave.lower_lines.len());
        }
        
        if !stave.lyrics_lines.is_empty() {
            println!("      Lyrics Lines: {}", stave.lyrics_lines.len());
            for (j, lyrics) in stave.lyrics_lines.iter().enumerate() {
                println!("        Line {}: {} syllables", j + 1, lyrics.syllables.len());
            }
        }
    }
}