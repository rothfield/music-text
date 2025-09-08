// Pest-based parser for musical notation
// Converts pest parse tree to our custom AST

use pest::{Parser, iterators::Pair};
use pest_derive::Parser;
// use std::collections::HashMap; // Not needed for current implementation

use crate::ast::*;
use crate::ast::raw::*;
use crate::models::{Degree, Position};
use crate::spatial_parser::{assign_octave_markers, assign_syllables_to_notes, analyze_slurs};
use crate::classifier::{classify_raw_stave, ClassificationError};

#[derive(Parser)]
#[grammar = "grammar/notation.pest"]
pub struct MusicTextParser;

pub type ParseError = pest::error::Error<Rule>;

/// Extract line/column position from a Pest pair
fn extract_position(pair: &Pair<Rule>) -> Option<Position> {
    let (line, col) = pair.line_col();
    Some(Position { row: line, col: col })
}

/// Helper function to choose the appropriate grammar rule based on system
fn parse_with_system<'a>(input: &'a str, system: &str) -> Result<pest::iterators::Pairs<'a, Rule>, ParseError> {
    match system {
        "sargam" => MusicTextParser::parse(Rule::sargam_document, input),
        "number" => MusicTextParser::parse(Rule::number_document, input),
        "western" => MusicTextParser::parse(Rule::western_document, input),
        "abc" => MusicTextParser::parse(Rule::abc_document, input),
        "doremi" => MusicTextParser::parse(Rule::doremi_document, input),
        "auto" | _ => MusicTextParser::parse(Rule::document, input),
    }
}

pub fn parse_notation(input: &str, system: &str) -> Result<Document, ParseError> {
    let mut parser = DocumentBuilder::new();
    
    // Use shared parsing logic
    let pairs = parse_with_system(input, system)?;
    
    for pair in pairs {
        parser.process_document(pair)?;
    }
    
    let mut document = parser.build();
    analyze_slurs(&mut document);
    assign_octave_markers(&mut document);
    assign_syllables_to_notes(&mut document);
    Ok(document)
}

/// Parse notation returning both raw AST and spatially-processed AST
pub fn parse_notation_with_stages(input: &str, system: &str) -> Result<(Document, Document), ParseError> {
    let mut parser = DocumentBuilder::new();
    
    // Use shared parsing logic
    let pairs = parse_with_system(input, system)?;
    
    for pair in pairs {
        parser.process_document(pair)?;
    }
    
    let raw_document = parser.build();
    let mut spatial_document = raw_document.clone();
    
    // Apply spatial processing to the copy
    analyze_slurs(&mut spatial_document);
    assign_octave_markers(&mut spatial_document);
    assign_syllables_to_notes(&mut spatial_document);
    
    Ok((raw_document, spatial_document))
}


struct DocumentBuilder {
    document: Document,
}

impl DocumentBuilder {
    fn new() -> Self {
        Self {
            document: Document::new(),
        }
    }
    
    fn build(self) -> Document {
        self.document
    }
    
    fn process_document(&mut self, pair: Pair<Rule>) -> Result<(), ParseError> {
        match pair.as_rule() {
            Rule::document | 
            Rule::sargam_document | 
            Rule::number_document | 
            Rule::western_document | 
            Rule::abc_document | 
            Rule::doremi_document => {
                for inner_pair in pair.into_inner() {
                    match inner_pair.as_rule() {
                        Rule::attribute_section => {
                            self.process_attributes(inner_pair)?;
                        }
                        Rule::staves_section => {
                            self.process_staves(inner_pair)?;
                        }
                        Rule::sargam_composition => {
                            self.document.notation_system = NotationSystem::Sargam;
                            self.process_composition(inner_pair)?;
                        }
                        Rule::number_composition => {
                            self.document.notation_system = NotationSystem::Number;
                            self.process_composition(inner_pair)?;
                        }
                        Rule::western_composition => {
                            self.document.notation_system = NotationSystem::Western;
                            self.process_composition(inner_pair)?;
                        }
                        Rule::abc_composition => {
                            self.document.notation_system = NotationSystem::Abc;
                            self.process_composition(inner_pair)?;
                        }
                        Rule::doremi_composition => {
                            self.document.notation_system = NotationSystem::Doremi;
                            self.process_composition(inner_pair)?;
                        }
                        Rule::EOI => break,
                        _ => {}
                    }
                }
            }
            _ => {
                return Err(pest::error::Error::new_from_pos(
                    pest::error::ErrorVariant::CustomError {
                        message: format!("Unexpected rule: {:?}", pair.as_rule()),
                    },
                    pair.as_span().start_pos(),
                ));
            }
        }
        Ok(())
    }
    
    fn process_composition(&mut self, pair: Pair<Rule>) -> Result<(), ParseError> {
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::attribute_section => {
                    self.process_attributes(inner_pair)?;
                }
                Rule::sargam_stave | 
                Rule::number_stave | 
                Rule::western_stave | 
                Rule::abc_stave | 
                Rule::doremi_stave |
                Rule::stave => {
                    let stave = self.process_stave(inner_pair)?;
                    self.document.staves.push(stave);
                }
                Rule::empty_line => {
                    // Skip empty lines between staves
                }
                _ => {}
            }
        }
        Ok(())
    }
    
    fn process_attributes(&mut self, pair: Pair<Rule>) -> Result<(), ParseError> {
        for inner_pair in pair.into_inner() {
            if inner_pair.as_rule() == Rule::attribute_line {
                let mut key = String::new();
                let mut value = String::new();
                
                for attr_pair in inner_pair.into_inner() {
                    match attr_pair.as_rule() {
                        Rule::attribute_key => key = attr_pair.as_str().to_string(),
                        Rule::attribute_value => value = attr_pair.as_str().trim().to_string(),
                        _ => {}
                    }
                }
                
                if !key.is_empty() {
                    self.document.attributes.insert(key, value);
                }
            }
        }
        Ok(())
    }
    
    fn process_staves(&mut self, pair: Pair<Rule>) -> Result<(), ParseError> {
        for inner_pair in pair.into_inner() {
            if inner_pair.as_rule() == Rule::stave {
                let stave = self.process_stave(inner_pair)?;
                self.document.staves.push(stave);
            }
        }
        Ok(())
    }
    
    fn process_stave(&mut self, pair: Pair<Rule>) -> Result<Stave, ParseError> {
        let position = extract_position(&pair);
        let raw_stave = self.parse_raw_stave(pair)?;
        let final_stave = self.classify_raw_stave(raw_stave)?;
        Ok(final_stave)
    }
    
    fn parse_raw_stave(&mut self, pair: Pair<Rule>) -> Result<RawStave, ParseError> {
        let mut raw_stave = RawStave {
            pre_content_lines: Vec::new(),
            content_line: ContentLine { line_number: None, measures: Vec::new() },
            post_content_lines: Vec::new(),
            position: extract_position(&pair),
        };
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::pre_content_line | 
                Rule::sargam_pre_content_line | 
                Rule::number_pre_content_line | 
                Rule::western_pre_content_line | 
                Rule::abc_pre_content_line | 
                Rule::doremi_pre_content_line => {
                    let raw_line = self.parse_pre_content_line(inner_pair)?;
                    raw_stave.pre_content_lines.push(raw_line);
                }
                Rule::content_line | 
                Rule::sargam_content_line | 
                Rule::number_content_line | 
                Rule::western_content_line | 
                Rule::abc_content_line | 
                Rule::doremi_content_line => {
                    raw_stave.content_line = self.process_content_line(inner_pair)?;
                }
                Rule::post_content_line | 
                Rule::sargam_post_content_line | 
                Rule::number_post_content_line | 
                Rule::western_post_content_line | 
                Rule::abc_post_content_line | 
                Rule::doremi_post_content_line => {
                    let raw_line = self.parse_post_content_line(inner_pair)?;
                    raw_stave.post_content_lines.push(raw_line);
                }
                // Handle legacy rules for backward compatibility
                Rule::upper_line | 
                Rule::sargam_upper_line | 
                Rule::number_upper_line | 
                Rule::western_upper_line | 
                Rule::abc_upper_line | 
                Rule::doremi_upper_line => {
                    // Treat as pre-content line
                    let annotation_line = self.process_upper_line(inner_pair)?;
                    let raw_line = self.convert_annotation_line_to_raw_upper(annotation_line)?;
                    raw_stave.pre_content_lines.push(raw_line);
                }
                Rule::lower_line => {
                    // Treat as post-content line
                    let annotation_line = self.process_lower_line(inner_pair)?;
                    let raw_line = self.convert_annotation_line_to_raw_lower(annotation_line)?;
                    raw_stave.post_content_lines.push(raw_line);
                }
                Rule::lyrics_line => {
                    // Treat as post-content line
                    let lyrics = self.process_lyrics_line(inner_pair)?;
                    let raw_line = RawAnnotationLine {
                        content: RawAnnotationContent::Lyrics(lyrics.syllables),
                        position: None,
                    };
                    raw_stave.post_content_lines.push(raw_line);
                }
                _ => {}
            }
        }
        
        Ok(raw_stave)
    }
    
    fn classify_raw_stave(&mut self, raw_stave: RawStave) -> Result<Stave, ParseError> {
        classify_raw_stave(raw_stave).map_err(|err| {
            pest::error::Error::new_from_pos(
                pest::error::ErrorVariant::CustomError {
                    message: format!("Classification error: {}", err),
                },
                pest::Position::from_start(""),
            )
        })
    }
    
    fn process_content_line(&mut self, pair: Pair<Rule>) -> Result<ContentLine, ParseError> {
        let mut content_line = ContentLine {
            line_number: None,
            measures: Vec::new(),
        };
        
        let mut current_measure = Measure {
            start_barline: None,
            beats: Vec::new(),
            end_barline: None,
        };
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::line_number => {
                    let num_str = inner_pair.as_str().trim_end_matches(')');
                    content_line.line_number = num_str.parse().ok();
                }
                Rule::barline => {
                    let barline = self.process_barline(inner_pair)?;
                    if current_measure.beats.is_empty() && current_measure.start_barline.is_none() {
                        current_measure.start_barline = Some(barline);
                    } else {
                        current_measure.end_barline = Some(barline);
                        content_line.measures.push(current_measure);
                        current_measure = Measure {
                            start_barline: None,
                            beats: Vec::new(),
                            end_barline: None,
                        };
                    }
                }
                Rule::measure | 
                Rule::sargam_measure | 
                Rule::number_measure | 
                Rule::western_measure | 
                Rule::abc_measure | 
                Rule::doremi_measure => {
                    let beats = self.process_measure(inner_pair)?;
                    current_measure.beats.extend(beats);
                }
                _ => {}
            }
        }
        
        // Add the last measure if it has content
        if !current_measure.beats.is_empty() || current_measure.start_barline.is_some() {
            content_line.measures.push(current_measure);
        }
        
        Ok(content_line)
    }
    
    fn process_measure(&mut self, pair: Pair<Rule>) -> Result<Vec<Beat>, ParseError> {
        let mut beats = Vec::new();
        let mut current_beat_elements = Vec::new();
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::beat | Rule::simple_beat |
                Rule::sargam_beat | Rule::sargam_simple_beat |
                Rule::number_beat | Rule::number_simple_beat |
                Rule::western_beat | Rule::western_simple_beat |
                Rule::abc_beat | Rule::abc_simple_beat |
                Rule::doremi_beat | Rule::doremi_simple_beat => {
                    let beat = self.process_beat(inner_pair)?;
                    beats.push(beat);
                }
                Rule::begin_slur => {
                    let position = extract_position(&inner_pair);
                    current_beat_elements.push(BeatElement::SlurStart { position });
                }
                Rule::end_slur => {
                    let position = extract_position(&inner_pair);
                    current_beat_elements.push(BeatElement::SlurEnd { position });
                }
                _ => {}
            }
        }
        
        // If we collected loose elements, make them into an undelimited beat
        if !current_beat_elements.is_empty() {
            beats.push(Beat {
                elements: current_beat_elements,
                divisions: None,
                is_tuplet: None,
                tuplet_ratio: None,
            });
        }
        
        Ok(beats)
    }
    
    fn process_beat(&mut self, pair: Pair<Rule>) -> Result<Beat, ParseError> {
        let mut elements = Vec::new();
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::simple_beat | Rule::sargam_simple_beat | Rule::number_simple_beat | 
                Rule::western_simple_beat | Rule::abc_simple_beat | Rule::doremi_simple_beat => {
                    // Process all beat_items within the simple_beat
                    for beat_item_pair in inner_pair.into_inner() {
                        if let Some(element) = self.process_beat_item(beat_item_pair)? {
                            elements.push(element);
                        }
                    }
                }
                Rule::beat_item | Rule::sargam_beat_item | Rule::number_beat_item | 
                Rule::western_beat_item | Rule::abc_beat_item | Rule::doremi_beat_item => {
                    // Handle loose beat_items directly
                    if let Some(element) = self.process_beat_item(inner_pair)? {
                        elements.push(element);
                    }
                }
                _ => {}
            }
        }
        
        Ok(Beat {
            elements,
            divisions: None,
            is_tuplet: None,
            tuplet_ratio: None,
        })
    }
    
    fn process_beat_item(&mut self, pair: Pair<Rule>) -> Result<Option<BeatElement>, ParseError> {
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::pitch | Rule::sargam_pitch | Rule::number_pitch | Rule::western_pitch | Rule::abc_pitch | Rule::doremi_pitch => {
                    return self.process_beat_element(inner_pair);
                }
                Rule::dash => {
                    let position = extract_position(&inner_pair);
                    return Ok(Some(BeatElement::Dash { position }));
                }
                Rule::begin_slur => {
                    let position = extract_position(&inner_pair);
                    return Ok(Some(BeatElement::SlurStart { position }));
                }
                Rule::end_slur => {
                    let position = extract_position(&inner_pair);
                    return Ok(Some(BeatElement::SlurEnd { position }));
                }
                Rule::breath_mark => {
                    let position = extract_position(&inner_pair);
                    return Ok(Some(BeatElement::BreathMark { position }));
                }
                _ => {}
            }
        }
        Ok(None)
    }
    
    fn process_beat_element(&mut self, pair: Pair<Rule>) -> Result<Option<BeatElement>, ParseError> {
        let position = extract_position(&pair);
        
        match pair.as_rule() {
            Rule::pitch |
            Rule::sargam_pitch |
            Rule::number_pitch |
            Rule::western_pitch |
            Rule::abc_pitch |
            Rule::doremi_pitch => {
                let (value, accidental) = self.parse_pitch(pair)?;
                Ok(Some(BeatElement::Pitch { 
                    value, 
                    accidental, 
                    syllable: None, 
                    slur_type: None, 
                    octave: 0, 
                    subdivisions: None, 
                    is_tied: None, 
                    position 
                }))
            }
            Rule::dash => {
                Ok(Some(BeatElement::Dash { position }))
            }
            Rule::begin_slur => {
                Ok(Some(BeatElement::SlurStart { position }))
            }
            Rule::end_slur => {
                Ok(Some(BeatElement::SlurEnd { position }))
            }
            _ => {
                // Skip whitespace and other non-musical elements
                Ok(None)
            }
        }
    }
    
    fn parse_pitch(&mut self, pair: Pair<Rule>) -> Result<(String, Option<String>), ParseError> {
        let pitch_str = pair.as_str();
        
        // Check for accidentals
        if pitch_str.ends_with('#') {
            Ok((pitch_str[..pitch_str.len()-1].to_string(), Some("#".to_string())))
        } else if pitch_str.ends_with('b') {
            Ok((pitch_str[..pitch_str.len()-1].to_string(), Some("b".to_string())))
        } else {
            Ok((pitch_str.to_string(), None))
        }
    }
    
    fn process_barline(&mut self, pair: Pair<Rule>) -> Result<Barline, ParseError> {
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::single_barline => return Ok(Barline::Single),
                Rule::double_barline => return Ok(Barline::Double),
                Rule::final_barline => return Ok(Barline::Final),
                Rule::reverse_final_barline => return Ok(Barline::ReverseFinal),
                Rule::left_repeat => return Ok(Barline::LeftRepeat),
                Rule::right_repeat => return Ok(Barline::RightRepeat),
                _ => {}
            }
        }
        
        // Default fallback
        Ok(Barline::Single)
    }
    
    fn process_upper_line(&mut self, pair: Pair<Rule>) -> Result<AnnotationLine, ParseError> {
        let mut items = Vec::new();
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::upper_line_item | Rule::sargam_upper_line_item => {
                    for item_pair in inner_pair.into_inner() {
                        if let Some(item) = self.process_upper_line_item(item_pair)? {
                            items.push(item);
                        }
                    }
                }
                _ => {
                    if let Some(item) = self.process_upper_line_item(inner_pair)? {
                        items.push(item);
                    }
                }
            }
        }
        
        Ok(AnnotationLine { items })
    }
    
    fn process_upper_line_item(&mut self, pair: Pair<Rule>) -> Result<Option<AnnotationItem>, ParseError> {
        let position = extract_position(&pair);
        
        match pair.as_rule() {
            Rule::upper_line_dot => Ok(Some(AnnotationItem::UpperOctaveMarker { 
                marker: pair.as_str().to_string(),
                position
            })),
            Rule::upper_line_two_dots => Ok(Some(AnnotationItem::UpperOctaveMarker { 
                marker: pair.as_str().to_string(),
                position
            })),
            Rule::tala => Ok(Some(AnnotationItem::Tala { 
                marker: pair.as_str().to_string(),
                position
            })),
            Rule::mordent => Ok(Some(AnnotationItem::Mordent { position })),
            Rule::ending => Ok(Some(AnnotationItem::Ending { 
                ending: pair.as_str().to_string(),
                position
            })),
            Rule::slur => Ok(Some(AnnotationItem::Slur { 
                underscores: pair.as_str().to_string(),
                position
            })),
            Rule::beat_grouping => Ok(Some(AnnotationItem::BeatGrouping { 
                underscores: pair.as_str().to_string(),
                position
            })),
            Rule::chord => {
                let chord_str = pair.as_str();
                // Remove [ and ] brackets
                let chord_content = &chord_str[1..chord_str.len()-1];
                Ok(Some(AnnotationItem::Chord { 
                    chord: chord_content.to_string(),
                    position
                }))
            }
            Rule::ornament | Rule::sargam_ornament => {
                let ornament_pitches = self.process_ornament(pair)?;
                Ok(Some(AnnotationItem::Ornament { 
                    pitches: ornament_pitches,
                    position
                }))
            }
            _ => {
                // Handle whitespace
                if pair.as_str().chars().all(char::is_whitespace) {
                    Ok(Some(AnnotationItem::Space { 
                        count: pair.as_str().len(),
                        position
                    }))
                } else {
                    Ok(Some(AnnotationItem::Symbol { 
                        symbol: pair.as_str().to_string(),
                        position
                    }))
                }
            }
        }
    }
    
    fn process_ornament(&mut self, pair: Pair<Rule>) -> Result<Vec<String>, ParseError> {
        let mut pitches = Vec::new();
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::ornament_pitch => {
                    for pitch_pair in inner_pair.into_inner() {
                        pitches.push(pitch_pair.as_str().to_string());
                    }
                }
                Rule::pitch | Rule::sargam_pitch => {
                    pitches.push(inner_pair.as_str().to_string());
                }
                _ => {}
            }
        }
        
        Ok(pitches)
    }
    
    fn process_lower_line(&mut self, pair: Pair<Rule>) -> Result<AnnotationLine, ParseError> {
        let mut items = Vec::new();
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::lower_line_item => {
                    for item_pair in inner_pair.into_inner() {
                        if let Some(item) = self.process_lower_line_item(item_pair)? {
                            items.push(item);
                        }
                    }
                }
                _ => {
                    if let Some(item) = self.process_lower_line_item(inner_pair)? {
                        items.push(item);
                    }
                }
            }
        }
        
        Ok(AnnotationLine { items })
    }
    
    fn process_lower_line_item(&mut self, pair: Pair<Rule>) -> Result<Option<AnnotationItem>, ParseError> {
        let position = extract_position(&pair);
        
        match pair.as_rule() {
            Rule::lower_line_dot => Ok(Some(AnnotationItem::LowerOctaveMarker { 
                marker: pair.as_str().to_string(),
                position
            })),
            Rule::lower_line_two_dots => Ok(Some(AnnotationItem::LowerOctaveMarker { 
                marker: pair.as_str().to_string(),
                position
            })),
            Rule::kommal_indicator => Ok(Some(AnnotationItem::Symbol { 
                symbol: pair.as_str().to_string(),
                position
            })),
            Rule::beat_grouping => Ok(Some(AnnotationItem::BeatGrouping { 
                underscores: pair.as_str().to_string(),
                position
            })),
            _ => {
                // Handle whitespace
                if pair.as_str().chars().all(char::is_whitespace) {
                    Ok(Some(AnnotationItem::Space { 
                        count: pair.as_str().len(),
                        position
                    }))
                } else {
                    Ok(None)
                }
            }
        }
    }
    
    fn process_lyrics_line(&mut self, pair: Pair<Rule>) -> Result<LyricsLine, ParseError> {
        let mut syllables = Vec::new();
        
        for inner_pair in pair.into_inner() {
            if inner_pair.as_rule() == Rule::syllable {
                syllables.push(inner_pair.as_str().to_string());
            }
        }
        
        Ok(LyricsLine { syllables })
    }
    
    /// Parse pre-content line using upper_grammar context
    fn parse_pre_content_line(&mut self, pair: Pair<Rule>) -> Result<RawAnnotationLine, ParseError> {
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::upper_grammar | 
                Rule::sargam_upper_grammar | 
                Rule::number_upper_grammar | 
                Rule::western_upper_grammar | 
                Rule::abc_upper_grammar | 
                Rule::doremi_upper_grammar => {
                    return Ok(RawAnnotationLine {
                        content: RawAnnotationContent::Upper(self.parse_upper_items(inner_pair)?),
                        position: extract_position(&pair),
                    });
                }
                _ => {}
            }
        }
        Err(pest::error::Error::new_from_pos(
            pest::error::ErrorVariant::CustomError {
                message: "Expected upper_grammar in pre_content_line".to_string(),
            },
            pair.as_span().start_pos(),
        ))
    }
    
    /// Parse post-content line using lower_grammar or lyrics_grammar context
    fn parse_post_content_line(&mut self, pair: Pair<Rule>) -> Result<RawAnnotationLine, ParseError> {
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::lower_grammar | 
                Rule::sargam_lower_grammar | 
                Rule::number_lower_grammar | 
                Rule::western_lower_grammar | 
                Rule::abc_lower_grammar | 
                Rule::doremi_lower_grammar => {
                    return Ok(RawAnnotationLine {
                        content: RawAnnotationContent::Lower(self.parse_lower_items(inner_pair)?),
                        position: extract_position(&pair),
                    });
                }
                Rule::lyrics_grammar => {
                    return Ok(RawAnnotationLine {
                        content: RawAnnotationContent::Lyrics(self.parse_lyrics_items(inner_pair)?),
                        position: extract_position(&pair),
                    });
                }
                _ => {}
            }
        }
        Err(pest::error::Error::new_from_pos(
            pest::error::ErrorVariant::CustomError {
                message: "Expected lower_grammar or lyrics_grammar in post_content_line".to_string(),
            },
            pair.as_span().start_pos(),
        ))
    }
    
    /// Parse upper grammar items
    fn parse_upper_items(&mut self, pair: Pair<Rule>) -> Result<Vec<UpperItem>, ParseError> {
        let mut items = Vec::new();
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::upper_item | 
                Rule::sargam_upper_item | 
                Rule::number_upper_item | 
                Rule::western_upper_item | 
                Rule::abc_upper_item | 
                Rule::doremi_upper_item => {
                    if let Some(item) = self.parse_upper_item(inner_pair)? {
                        items.push(item);
                    }
                }
                _ => {}
            }
        }
        
        Ok(items)
    }
    
    /// Parse individual upper item
    fn parse_upper_item(&mut self, pair: Pair<Rule>) -> Result<Option<UpperItem>, ParseError> {
        let position = extract_position(&pair);
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::upper_octave_marker => {
                    return Ok(Some(UpperItem::OctaveMarker { 
                        marker: inner_pair.as_str().to_string(),
                        position
                    }));
                }
                Rule::tala => {
                    return Ok(Some(UpperItem::Tala { 
                        marker: inner_pair.as_str().to_string(),
                        position
                    }));
                }
                Rule::ornament | Rule::sargam_ornament | Rule::number_ornament | 
                Rule::western_ornament | Rule::abc_ornament | Rule::doremi_ornament => {
                    let pitches = self.process_ornament(inner_pair)?;
                    return Ok(Some(UpperItem::Ornament { pitches, position }));
                }
                Rule::chord => {
                    return Ok(Some(UpperItem::Chord { 
                        chord: inner_pair.as_str().to_string(),
                        position
                    }));
                }
                Rule::slur => {
                    return Ok(Some(UpperItem::Slur { 
                        underscores: inner_pair.as_str().to_string(),
                        position
                    }));
                }
                Rule::ending => {
                    return Ok(Some(UpperItem::Ending { 
                        ending: inner_pair.as_str().to_string(),
                        position
                    }));
                }
                Rule::mordent => {
                    return Ok(Some(UpperItem::Mordent { position }));
                }
                _ => {
                    // Handle spaces - count consecutive spaces
                    let text = inner_pair.as_str();
                    if text.chars().all(|c| c == ' ' || c == '\t') {
                        return Ok(Some(UpperItem::Space { 
                            count: text.len(),
                            position
                        }));
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    /// Parse lower grammar items
    fn parse_lower_items(&mut self, pair: Pair<Rule>) -> Result<Vec<LowerItem>, ParseError> {
        let mut items = Vec::new();
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::lower_item | 
                Rule::sargam_lower_item | 
                Rule::number_lower_item | 
                Rule::western_lower_item | 
                Rule::abc_lower_item | 
                Rule::doremi_lower_item => {
                    if let Some(item) = self.parse_lower_item(inner_pair)? {
                        items.push(item);
                    }
                }
                _ => {}
            }
        }
        
        Ok(items)
    }
    
    /// Parse individual lower item
    fn parse_lower_item(&mut self, pair: Pair<Rule>) -> Result<Option<LowerItem>, ParseError> {
        let position = extract_position(&pair);
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::lower_octave_marker => {
                    return Ok(Some(LowerItem::OctaveMarker { 
                        marker: inner_pair.as_str().to_string(),
                        position
                    }));
                }
                Rule::kommal_indicator => {
                    return Ok(Some(LowerItem::KommalIndicator { position }));
                }
                Rule::beat_grouping => {
                    return Ok(Some(LowerItem::BeatGrouping { 
                        underscores: inner_pair.as_str().to_string(),
                        position
                    }));
                }
                _ => {
                    // Handle spaces - count consecutive spaces
                    let text = inner_pair.as_str();
                    if text.chars().all(|c| c == ' ' || c == '\t') {
                        return Ok(Some(LowerItem::Space { 
                            count: text.len(),
                            position
                        }));
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    /// Parse lyrics items
    fn parse_lyrics_items(&mut self, pair: Pair<Rule>) -> Result<Vec<String>, ParseError> {
        let mut syllables = Vec::new();
        
        for inner_pair in pair.into_inner() {
            if inner_pair.as_rule() == Rule::syllable {
                syllables.push(inner_pair.as_str().to_string());
            }
        }
        
        Ok(syllables)
    }
    
    /// Convert annotation line to raw upper for backward compatibility
    fn convert_annotation_line_to_raw_upper(&mut self, annotation_line: AnnotationLine) -> Result<RawAnnotationLine, ParseError> {
        let mut items = Vec::new();
        
        for item in annotation_line.items {
            let raw_item = match item {
                AnnotationItem::UpperOctaveMarker { marker, position } => {
                    UpperItem::OctaveMarker { marker, position }
                }
                AnnotationItem::Tala { marker, position } => {
                    UpperItem::Tala { marker, position }
                }
                AnnotationItem::Ornament { pitches, position } => {
                    UpperItem::Ornament { pitches, position }
                }
                AnnotationItem::Chord { chord, position } => {
                    UpperItem::Chord { chord, position }
                }
                AnnotationItem::Slur { underscores, position } => {
                    UpperItem::Slur { underscores, position }
                }
                AnnotationItem::Ending { ending, position } => {
                    UpperItem::Ending { ending, position }
                }
                AnnotationItem::Mordent { position } => {
                    UpperItem::Mordent { position }
                }
                AnnotationItem::Space { count, position } => {
                    UpperItem::Space { count, position }
                }
                _ => {
                    return Err(pest::error::Error::new_from_pos(
                        pest::error::ErrorVariant::CustomError {
                            message: "Invalid annotation item for upper line".to_string(),
                        },
                        pest::Position::from_start(""),
                    ));
                }
            };
            items.push(raw_item);
        }
        
        Ok(RawAnnotationLine {
            content: RawAnnotationContent::Upper(items),
            position: None,
        })
    }
    
    /// Convert annotation line to raw lower for backward compatibility  
    fn convert_annotation_line_to_raw_lower(&mut self, annotation_line: AnnotationLine) -> Result<RawAnnotationLine, ParseError> {
        let mut items = Vec::new();
        
        for item in annotation_line.items {
            let raw_item = match item {
                AnnotationItem::LowerOctaveMarker { marker, position } => {
                    LowerItem::OctaveMarker { marker, position }
                }
                AnnotationItem::BeatGrouping { underscores, position } => {
                    LowerItem::BeatGrouping { underscores, position }
                }
                AnnotationItem::Space { count, position } => {
                    LowerItem::Space { count, position }
                }
                AnnotationItem::Symbol { symbol, position } if symbol == "_" => {
                    LowerItem::KommalIndicator { position }
                }
                _ => {
                    return Err(pest::error::Error::new_from_pos(
                        pest::error::ErrorVariant::CustomError {
                            message: "Invalid annotation item for lower line".to_string(),
                        },
                        pest::Position::from_start(""),
                    ));
                }
            };
            items.push(raw_item);
        }
        
        Ok(RawAnnotationLine {
            content: RawAnnotationContent::Lower(items),
            position: None,
        })
    }
}





/// Simple pitch to LilyPond conversion for testing
fn convert_pitch_to_lilypond(pitch: &str, octave: i8) -> String {
    // Convert sargam to western note names for LilyPond
    let base_note = match pitch.to_uppercase().as_str() {
        "S" => "c",
        "R" => "d", 
        "G" => "e",
        "M" => "f",
        "P" => "g",
        "D" => "a",
        "N" => "b",
        _ => "c", // Default fallback
    };
    
    // Apply octave markers
    let octave_suffix = match octave {
        -2 => ",,",    // Double lower octave  
        -1 => ",",     // Lower octave
        0 => "",       // Normal octave
        1 => "'",      // Higher octave
        2 => "''",     // Double higher octave
        _ => "",       // Fallback
    };
    
    format!("{}{}", base_note, octave_suffix)
}

/// Convert Degree enum to LilyPond notation
fn convert_degree_to_lilypond(degree: Degree, octave: i8) -> String {
    let base_note = match degree {
        Degree::N1 => "c",
        Degree::N2 => "d",
        Degree::N3 => "e", 
        Degree::N4 => "f",
        Degree::N5 => "g",
        Degree::N6 => "a",
        Degree::N7 => "b",
        _ => "c", // Default fallback for flats/sharps
    };
    
    let octave_suffix = match octave {
        -2 => ",,",
        -1 => ",",
        0 => "",
        1 => "'",
        2 => "''",
        _ => "",
    };
    
    format!("{}{}", base_note, octave_suffix)
}

/// DEBUG: Test raw pest parser for debugging parsing issues
pub fn debug_pest_parse(input: &str) {
    println!("=== DEBUGGING PEST PARSE FOR: '{}' ===", input);
    
    // Test with number_document rule
    println!("--- Testing number_document rule ---");
    match MusicTextParser::parse(Rule::number_document, input) {
        Ok(pairs) => {
            println!("SUCCESS: Parsed with number_document rule");
            debug_print_parse_tree(pairs, 0);
        }
        Err(e) => {
            println!("ERROR: Failed to parse with number_document rule");
            println!("Error: {}", e);
        }
    }
    
    println!();
    
    // Test with generic document rule  
    println!("--- Testing generic document rule ---");
    match MusicTextParser::parse(Rule::document, input) {
        Ok(pairs) => {
            println!("SUCCESS: Parsed with document rule");
            debug_print_parse_tree(pairs, 0);
        }
        Err(e) => {
            println!("ERROR: Failed to parse with document rule");
            println!("Error: {}", e);
        }
    }
    
    println!();
    
    // Test individual components
    println!("--- Testing individual components ---");
    
    // Test number_pitch directly
    println!("Testing number_pitch:");
    match MusicTextParser::parse(Rule::number_pitch, input) {
        Ok(pairs) => {
            println!("SUCCESS: Parsed with number_pitch rule");
            debug_print_parse_tree(pairs, 0);
        }
        Err(e) => {
            println!("ERROR: Failed to parse with number_pitch rule");
            println!("Error: {}", e);
        }
    }
    
    println!();
    
    // Test pitch directly
    println!("Testing pitch:");
    match MusicTextParser::parse(Rule::pitch, input) {
        Ok(pairs) => {
            println!("SUCCESS: Parsed with pitch rule");
            debug_print_parse_tree(pairs, 0);
        }
        Err(e) => {
            println!("ERROR: Failed to parse with pitch rule");  
            println!("Error: {}", e);
        }
    }
    
    println!();
    
    // Test number_beat_item directly
    println!("Testing number_beat_item:");
    match MusicTextParser::parse(Rule::number_beat_item, input) {
        Ok(pairs) => {
            println!("SUCCESS: Parsed with number_beat_item rule");
            debug_print_parse_tree(pairs, 0);
        }
        Err(e) => {
            println!("ERROR: Failed to parse with number_beat_item rule");
            println!("Error: {}", e);
        }
    }
    
    println!();
    
    // Test number_simple_beat directly
    println!("Testing number_simple_beat:");
    match MusicTextParser::parse(Rule::number_simple_beat, input) {
        Ok(pairs) => {
            println!("SUCCESS: Parsed with number_simple_beat rule");
            debug_print_parse_tree(pairs, 0);
        }
        Err(e) => {
            println!("ERROR: Failed to parse with number_simple_beat rule");
            println!("Error: {}", e);
        }
    }
    
    println!("=== END DEBUG ===");
}

fn debug_print_parse_tree(pairs: pest::iterators::Pairs<Rule>, indent: usize) {
    for pair in pairs {
        let indent_str = "  ".repeat(indent);
        println!("{}Rule: {:?}", indent_str, pair.as_rule());
        println!("{}Text: '{}'", indent_str, pair.as_str());
        println!("{}Span: {:?}", indent_str, pair.as_span());
        
        let inner_pairs = pair.into_inner();
        if inner_pairs.len() > 0 {
            println!("{}Inner rules:", indent_str);
            debug_print_parse_tree(inner_pairs, indent + 1);
        }
    }
}

/// TEST: Create a test document with lower octave markers to verify FSM integration
pub fn test_lower_octave_integration() {
    use crate::ast::*;
    use std::collections::HashMap;
    
    // Create a document with lower octave markers manually
    let mut document = Document {
        attributes: HashMap::new(),
        staves: vec![
            Stave {
                upper_lines: vec![],
                content_line: ContentLine {
                    line_number: None,
                    measures: vec![
                        Measure {
                            start_barline: None,
                            beats: vec![
                                Beat {
                                    elements: vec![
                                        BeatElement::Pitch { 
                                            value: "S".to_string(), 
                                            accidental: None, 
                                            syllable: None, 
                                            slur_type: None, 
                                            octave: 0,
                                            subdivisions: None,
                                            is_tied: None,
                                            position: None
                                        }
                                    ],
                                    divisions: None,
                                    is_tuplet: None,
                                    tuplet_ratio: None,
                                },
                                Beat {
                                    elements: vec![
                                        BeatElement::Pitch { 
                                            value: "R".to_string(), 
                                            accidental: None, 
                                            syllable: None, 
                                            slur_type: None, 
                                            octave: 0,
                                            subdivisions: None,
                                            is_tied: None,
                                            position: None
                                        }
                                    ],
                                    divisions: None,
                                    is_tuplet: None,
                                    tuplet_ratio: None,
                                },
                            ],
                            end_barline: None,
                        }
                    ],
                },
                lower_lines: vec![
                    AnnotationLine {
                        items: vec![
                            AnnotationItem::LowerOctaveMarker { 
                                marker: ".".to_string(),
                                position: None
                            },
                            AnnotationItem::Space { 
                                count: 1,
                                position: None
                            },
                            AnnotationItem::LowerOctaveMarker { 
                                marker: ":".to_string(),
                                position: None
                            },
                        ],
                    }
                ],
                lyrics_lines: vec![],
                position: None,
            }
        ],
        notation_system: NotationSystem::Sargam,
    };
    
    println!("BEFORE octave assignment:");
    for stave in &document.staves {
        for measure in &stave.content_line.measures {
            for (beat_idx, beat) in measure.beats.iter().enumerate() {
                for element in &beat.elements {
                    if let BeatElement::Pitch { value, octave, .. } = element {
                        println!("Beat {}: {} has octave: {}", beat_idx, value, octave);
                    }
                }
            }
        }
    }
    
    // Apply octave markers
    assign_octave_markers(&mut document);
    
    println!("Testing FSM pipeline: AST -> ParsedElements -> FSM -> LilyPond...");
    // Convert AST to ParsedElements first
    let parsed_elements = crate::ast_to_parsed::convert_ast_to_parsed_elements(&document);
    println!("ParsedElements:");
    for (i, elem) in parsed_elements.iter().enumerate() {
        println!("  Element {}: {:?}", i, elem);
    }
    
    // Convert ParsedElements to FSM output
    let fsm_items = crate::rhythm_fsm::convert_parsed_to_fsm_output(&parsed_elements);
    println!("FSM output:");
    for (i, item) in fsm_items.iter().enumerate() {
        println!("  Item {}: {:?}", i, item);
    }
    
    // Convert FSM to LilyPond
    println!("LilyPond generation from FSM:");
    for item in &fsm_items {
        match item {
            crate::parser_v2_fsm::Item::Beat(beat) => {
                for element in &beat.elements {
                    match &element.event {
                        crate::parser_v2_fsm::Event::Note { degree, octave, .. } => {
                            let lilypond_note = convert_degree_to_lilypond(*degree, *octave);
                            println!("  {:?} (octave {}) -> LilyPond: {}", degree, octave, lilypond_note);
                        },
                        crate::parser_v2_fsm::Event::Rest => {
                            println!("  Rest -> LilyPond: r");
                        },
                    }
                }
            },
            _ => {} // Skip other item types for now
        }
    }
    
    println!("AFTER octave assignment:");
    for stave in &document.staves {
        for measure in &stave.content_line.measures {
            for (beat_idx, beat) in measure.beats.iter().enumerate() {
                for element in &beat.elements {
                    if let BeatElement::Pitch { value, octave, .. } = element {
                        println!("Beat {}: {} has octave: {}", beat_idx, value, octave);
                    }
                }
            }
        }
    }
}


