use crate::document::pest_interface::{Pairs, Pair, Rule};
use crate::document::model::{Document, DocumentElement, StaffGroup, StaffGroupType, NamedStave, Source, Position};
use super::stave::{transform_stave, transform_simple_content_to_stave};

pub(super) fn transform_document(pairs: Pairs<Rule>) -> Result<Document, String> {
    let mut content = Vec::new();
    let mut document_source = Source {
        value: String::new(),
        position: Position { line: 1, column: 1 },
    };
    
    for pair in pairs {
        if pair.as_rule() == Rule::document {
            document_source = Source {
                value: pair.as_str().to_string(),
                position: {
                    let (line, column) = pair.as_span().start_pos().line_col();
                    Position { line, column }
                },
            };
            
            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::mixed_content => {
                        for mixed_line_pair in inner_pair.into_inner() {
                            match mixed_line_pair.as_rule() {
                                Rule::mixed_line => {
                                    for line_content in mixed_line_pair.into_inner() {
                                        match line_content.as_rule() {
                                            Rule::stave => {
                                                content.push(DocumentElement::SingleStave(transform_stave(line_content)?));
                                            }
                                            Rule::simple_content_line => {
                                                // Check if this is a musical line (3+ adjacent pitches)
                                                if let Some(_system) = detect_musical_line(&line_content) {
                                                    content.push(DocumentElement::SingleStave(transform_simple_content_to_stave(line_content)?));
                                                }
                                                // Otherwise ignore as text
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                Rule::staff_group => {
                                    content.push(DocumentElement::StaffGroup(transform_staff_group(mixed_line_pair)?));
                                }
                                Rule::blank_line => {
                                    // Ignore blank lines
                                }
                                _ => {}
                            }
                        }
                    }
                    Rule::EOI => {} // End of input, ignore
                    _ => {}
                }
            }
        }
    }
    
    Ok(Document { content, source: document_source })
}

/// Transform a staff_group Pair into a StaffGroup
fn transform_staff_group(pair: Pair<Rule>) -> Result<StaffGroup, String> {
    let source = Source {
        value: pair.as_str().to_string(),
        position: {
            let (line, column) = pair.as_span().start_pos().line_col();
            Position { line, column }
        },
    };

    let mut group_type = None;
    let mut staves = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::staff_group_start => {
                for start_inner in inner.into_inner() {
                    if start_inner.as_rule() == Rule::group_type {
                        let group_type_str = start_inner.as_str();
                        group_type = StaffGroupType::from_str(group_type_str);
                        if group_type.is_none() {
                            return Err(format!("Unknown staff group type: {}", group_type_str));
                        }
                    }
                }
            }
            Rule::staff_group_content => {
                for named_stave_pair in inner.into_inner() {
                    if named_stave_pair.as_rule() == Rule::named_stave {
                        staves.push(transform_named_stave(named_stave_pair)?);
                    }
                }
            }
            Rule::staff_group_end => {
                // End of staff group, nothing to do
            }
            _ => {}
        }
    }

    let group_type = group_type.ok_or("Missing staff group type")?;

    Ok(StaffGroup {
        group_type,
        staves,
        source,
    })
}

/// Transform a named_stave Pair into a NamedStave
fn transform_named_stave(pair: Pair<Rule>) -> Result<NamedStave, String> {
    let source = Source {
        value: pair.as_str().to_string(),
        position: {
            let (line, column) = pair.as_span().start_pos().line_col();
            Position { line, column }
        },
    };

    let mut staff_name = None;
    let mut stave = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::staff_name => {
                staff_name = Some(inner.as_str().to_string());
            }
            Rule::stave => {
                stave = Some(transform_stave(inner)?);
            }
            _ => {}
        }
    }

    let name = staff_name.ok_or("Missing staff name")?;
    let stave = stave.ok_or("Missing stave content")?;

    Ok(NamedStave {
        name,
        stave,
        source,
    })
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum NotationSystem {
    Number,
    Western,
    Sargam,
    Bhatkhande,
}

/// Detect if a simple content line has 3+ adjacent pitches from same notation system
fn detect_musical_line(pair: &Pair<Rule>) -> Option<NotationSystem> {
    let mut pitch_count = 0;
    let mut current_system = None;
    
    for element in pair.clone().into_inner() {
        if element.as_rule() == Rule::musical_element_no_barline {
            for inner in element.into_inner() {
                match inner.as_rule() {
                    Rule::pitch => {
                        // Pitch rule contains the actual pitch type as child
                        for pitch_type in inner.into_inner() {
                            if let Some(system) = rule_to_system(pitch_type.as_rule()) {
                                if current_system.is_none() || current_system == Some(system) {
                                    current_system = Some(system);
                                    pitch_count += 1;
                                    if pitch_count >= 3 {
                                        return Some(system);
                                    }
                                } else {
                                    // Different system, reset
                                    pitch_count = 1;
                                    current_system = Some(system);
                                }
                            }
                        }
                    }
                    Rule::space => {
                        // Spaces don't break the sequence
                    }
                    _ => {
                        // Non-pitch element breaks the sequence
                        pitch_count = 0;
                        current_system = None;
                    }
                }
            }
        }
    }
    
    None
}

fn rule_to_system(rule: Rule) -> Option<NotationSystem> {
    match rule {
        Rule::number_pitch => Some(NotationSystem::Number),
        Rule::western_pitch => Some(NotationSystem::Western),
        Rule::sargam_pitch => Some(NotationSystem::Sargam),
        Rule::bhatkhande_pitch => Some(NotationSystem::Bhatkhande),
        _ => None,
    }
}