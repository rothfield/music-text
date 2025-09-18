// Simplified FSM for core rhythm logic only
// Does NOT create beats - just processes dash extension and grouping

use crate::models::{ParsedElement, Degree, Position};

#[derive(Debug, Clone)]
pub enum SimpleItem {
    Note { degree: Degree, octave: i8, subdivisions: u32 },
    Rest { subdivisions: u32 },
    Barline { style: String },
}

pub fn process_elements_simple(elements: &[ParsedElement]) -> Vec<SimpleItem> {
    let mut result = Vec::new();
    let mut i = 0;
    
    while i < elements.len() {
        match &elements[i] {
            ParsedElement::Note { degree, octave, .. } => {
                // Count subdivisions by looking ahead for dashes
                let mut subdivisions = 1;
                let mut j = i + 1;
                
                // Count consecutive dashes
                while j < elements.len() {
                    if let ParsedElement::Dash { .. } = &elements[j] {
                        subdivisions += 1;
                        j += 1;
                    } else {
                        break;
                    }
                }
                
                result.push(SimpleItem::Note {
                    degree: *degree,
                    octave: *octave,
                    subdivisions,
                });
                
                i = j; // Skip the processed dashes
            },
            ParsedElement::Dash { .. } => {
                // Standalone dash = rest
                let mut subdivisions = 1;
                let mut j = i + 1;
                
                // Count consecutive dashes
                while j < elements.len() {
                    if let ParsedElement::Dash { .. } = &elements[j] {
                        subdivisions += 1;
                        j += 1;
                    } else {
                        break;
                    }
                }
                
                result.push(SimpleItem::Rest { subdivisions });
                i = j;
            },
            ParsedElement::Barline { style, .. } => {
                result.push(SimpleItem::Barline { style: style.clone() });
                i += 1;
            },
            ParsedElement::Whitespace { .. } => {
                // Skip whitespace - it's just for beat separation in Pest
                i += 1;
            },
            _ => {
                // Skip other elements 
                i += 1;
            }
        }
    }
    
    result
}

// Simple tuplet detection based on total subdivisions
pub fn detect_tuplets(items: &[SimpleItem]) -> bool {
    let total_subdivisions: u32 = items.iter()
        .map(|item| match item {
            SimpleItem::Note { subdivisions, .. } => *subdivisions,
            SimpleItem::Rest { subdivisions } => *subdivisions,
            SimpleItem::Barline { .. } => 0,
        })
        .sum();
    
    // Tuplet if not a power of 2
    total_subdivisions > 0 && (total_subdivisions & (total_subdivisions - 1)) != 0
}

pub fn calculate_tuplet_ratio(total_subdivisions: u32) -> (u32, u32) {
    if total_subdivisions <= 1 {
        return (1, 1);
    }
    
    // Find largest power of 2 less than total_subdivisions
    let mut denominator = 1;
    while denominator * 2 < total_subdivisions {
        denominator *= 2;
    }
    
    (total_subdivisions, denominator)
}