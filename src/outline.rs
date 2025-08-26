// src/outline.rs
// HAML-style outline generator for document structure visualization

use crate::models::{Document, Node};
use crate::pitch::pitchcode_to_string;
use std::fs::File;
use std::io::{self, Write};

pub trait ToOutline {
    fn to_outline(&self, indent_level: usize) -> String;
    fn to_html_outline(&self, indent_level: usize) -> String;
}

impl ToOutline for Document {
    fn to_outline(&self, indent_level: usize) -> String {
        let mut output = String::new();
        let indent = "  ".repeat(indent_level);
        
        // Document header
        output.push_str(&format!("{}document\n", indent));
        
        // Metadata section
        if self.metadata.title.is_some() || !self.metadata.directives.is_empty() {
            output.push_str(&format!("{}  metadata\n", indent));
            
            if let Some(title) = &self.metadata.title {
                output.push_str(&format!("{}    title: {}\n", indent, title.text));
            }
            
            for directive in &self.metadata.directives {
                output.push_str(&format!("{}    {}: {}\n", 
                    indent, directive.key.to_lowercase(), directive.value));
            }
        }
        
        // Nodes section
        if !self.nodes.is_empty() {
            output.push_str(&format!("{}  nodes\n", indent));
            for node in &self.nodes {
                output.push_str(&node.to_outline(indent_level + 2));
            }
        }
        
        output
    }

    fn to_html_outline(&self, indent_level: usize) -> String {
        let text_outline = self.to_outline(indent_level);
        let mut html_output = String::new();
        
        for line in text_outline.lines() {
            let trimmed = line.trim_start();
            let spaces = line.len() - trimmed.len();
            let indent = "&nbsp;".repeat(spaces);
            
            let colored_line = if trimmed == "document" {
                format!("<span class=\"document\">{}</span>", trimmed)
            } else if trimmed == "nodes" || trimmed == "metadata" {
                format!("<span class=\"section\">{}</span>", trimmed)
            } else if trimmed == "pitch" {
                format!("<span class=\"pitch\">{}</span>", trimmed)
            } else if trimmed == "beat" {
                format!("<span class=\"beat\">{}</span>", trimmed)
            } else if trimmed == "musical-line" || trimmed == "line" {
                format!("<span class=\"musical-line\">musical-line</span>")
            } else if trimmed == "barline" {
                format!("<span class=\"barline\">{}</span>", trimmed)
            } else if trimmed == "divisions" || trimmed == "pitch-code" || trimmed == "octave" {
                format!("<span class=\"property\">{}</span>", trimmed)
            } else if trimmed.starts_with("S") || trimmed.starts_with("R") || trimmed.starts_with("G") || trimmed.starts_with("M") || trimmed.starts_with("P") || trimmed.starts_with("D") || trimmed.starts_with("N") {
                format!("<span class=\"pitch-value\">{}</span>", trimmed)
            } else if trimmed.chars().all(|c| c.is_ascii_digit() || c == '-') {
                format!("<span class=\"number\">{}</span>", trimmed)
            } else if trimmed == "." || trimmed == ":" || trimmed == "'" {
                format!("<span class=\"octave-marker\">{}</span>", trimmed)
            } else {
                format!("<span class=\"outline-text\">{}</span>", trimmed)
            };
            
            html_output.push_str(&format!("{}{}<br>\n", indent, colored_line));
        }
        
        html_output
    }
}

impl ToOutline for Node {
    fn to_outline(&self, indent_level: usize) -> String {
        match self.node_type.as_str() {
            "PITCH" => self.pitch_to_outline(indent_level),
            "BEAT" => self.beat_to_outline(indent_level),
            "MUSICAL_LINE" => self.musical_line_to_outline(indent_level),
            "BARLINE" => self.barline_to_outline(indent_level),
            "WHITESPACE" => self.space_to_outline(indent_level),
            "NEWLINE" => self.line_to_outline(indent_level),
            "WORD" => self.word_to_outline(indent_level),
            "LINE" => self.regular_line_to_outline(indent_level),
            _ => self.generic_to_outline(indent_level),
        }
    }

    fn to_html_outline(&self, indent_level: usize) -> String {
        let text_outline = self.to_outline(indent_level);
        let mut html_output = String::new();
        
        for line in text_outline.lines() {
            let trimmed = line.trim_start();
            let spaces = line.len() - trimmed.len();
            let indent = "&nbsp;".repeat(spaces);
            
            let colored_line = if trimmed == "pitch" {
                format!("<span class=\"pitch\">{}</span>", trimmed)
            } else if trimmed == "beat" {
                format!("<span class=\"beat\">{}</span>", trimmed)
            } else if trimmed == "musical-line" || trimmed == "line" {
                format!("<span class=\"musical-line\">musical-line</span>")
            } else if trimmed == "barline" {
                format!("<span class=\"barline\">{}</span>", trimmed)
            } else if trimmed == "divisions" || trimmed == "pitch-code" || trimmed == "octave" {
                format!("<span class=\"property\">{}</span>", trimmed)
            } else if trimmed.starts_with("S") || trimmed.starts_with("R") || trimmed.starts_with("G") || trimmed.starts_with("M") || trimmed.starts_with("P") || trimmed.starts_with("D") || trimmed.starts_with("N") {
                format!("<span class=\"pitch-value\">{}</span>", trimmed)
            } else if trimmed.chars().all(|c| c.is_ascii_digit() || c == '-') {
                format!("<span class=\"number\">{}</span>", trimmed)
            } else if trimmed == "." || trimmed == ":" || trimmed == "'" {
                format!("<span class=\"octave-marker\">{}</span>", trimmed)
            } else {
                format!("<span class=\"outline-text\">{}</span>", trimmed)
            };
            
            html_output.push_str(&format!("{}{}<br>\n", indent, colored_line));
        }
        
        html_output
    }
}

pub fn generate_outline(document: &Document, filename: &str) -> io::Result<()> {
    let output = document.to_outline(0);
    
    // Write to file
    let mut file = File::create(filename)?;
    file.write_all(output.as_bytes())?;
    
    Ok(())
}

impl Node {
    fn pitch_to_outline(&self, indent_level: usize) -> String {
        let mut output = String::new();
        let indent = "  ".repeat(indent_level);
        
        output.push_str(&format!("{}pitch\n", indent));
        output.push_str(&format!("{}  {}\n", indent, self.value));
        
        // Pitch-specific attributes
        if let Some(pitch_code) = &self.pitch_code {
            output.push_str(&format!("{}  pitch-code\n", indent));
            output.push_str(&format!("{}    {}\n", indent, pitchcode_to_string(*pitch_code)));
        }
        
        if let Some(octave) = self.octave {
            if octave != 0 {
                output.push_str(&format!("{}  octave\n", indent));
                output.push_str(&format!("{}    {}\n", indent, octave));
            }
        }
        
        if self.divisions > 0 {
            output.push_str(&format!("{}  divisions\n", indent));
            output.push_str(&format!("{}    {}\n", indent, self.divisions));
        }
        
        // Process children - categorize lyrics, ornaments, etc.
        for child in &self.nodes {
            match child.node_type.as_str() {
                "WORD" => {
                    output.push_str(&format!("{}  lyric\n", indent));
                    output.push_str(&format!("{}    {}\n", indent, child.value));
                }
                "ORNAMENT" => {
                    output.push_str(&format!("{}  ornament\n", indent));
                    if !child.value.trim().is_empty() {
                        output.push_str(&format!("{}    {}\n", indent, child.value));
                    }
                    for ornament_child in &child.nodes {
                        output.push_str(&ornament_child.to_outline(indent_level + 2));
                    }
                }
                "OCTAVE_MARKER" => {
                    output.push_str(&format!("{}  octave\n", indent));
                    output.push_str(&format!("{}    {}\n", indent, child.value));
                }
                _ => {
                    output.push_str(&child.to_outline(indent_level + 1));
                }
            }
        }
        
        output
    }
    
    fn beat_to_outline(&self, indent_level: usize) -> String {
        let mut output = String::new();
        let indent = "  ".repeat(indent_level);
        
        output.push_str(&format!("{}beat\n", indent));
        
        if self.divisions > 0 {
            output.push_str(&format!("{}  divisions\n", indent));
            output.push_str(&format!("{}    {}\n", indent, self.divisions));
        }
        
        // Process children
        for child in &self.nodes {
            output.push_str(&child.to_outline(indent_level + 1));
        }
        
        output
    }
    
    fn musical_line_to_outline(&self, indent_level: usize) -> String {
        let mut output = String::new();
        let indent = "  ".repeat(indent_level);
        
        output.push_str(&format!("{}musical-line\n", indent));
        
        // Process children
        for child in &self.nodes {
            output.push_str(&child.to_outline(indent_level + 1));
        }
        
        output
    }
    
    fn barline_to_outline(&self, indent_level: usize) -> String {
        let mut output = String::new();
        let indent = "  ".repeat(indent_level);
        
        output.push_str(&format!("{}barline\n", indent));
        output.push_str(&format!("{}  {}\n", indent, self.value));
        
        output
    }
    
    fn space_to_outline(&self, indent_level: usize) -> String {
        let mut output = String::new();
        let indent = "  ".repeat(indent_level);
        
        output.push_str(&format!("{}space\n", indent));
        
        output
    }
    
    fn line_to_outline(&self, indent_level: usize) -> String {
        let mut output = String::new();
        let indent = "  ".repeat(indent_level);
        
        output.push_str(&format!("{}line\n", indent));
        
        output
    }
    
    fn word_to_outline(&self, indent_level: usize) -> String {
        let mut output = String::new();
        let indent = "  ".repeat(indent_level);
        
        output.push_str(&format!("{}word\n", indent));
        output.push_str(&format!("{}  {}\n", indent, self.value));
        
        output
    }
    
    fn regular_line_to_outline(&self, indent_level: usize) -> String {
        let mut output = String::new();
        
        // Skip non-musical lines (line-X format) but process their children
        if self.value.starts_with("line-") {
            // Process children at the same indent level to flatten them
            for child in &self.nodes {
                output.push_str(&child.to_outline(indent_level));
            }
        } else {
            output.push_str(&self.generic_to_outline(indent_level));
        }
        
        output
    }
    
    fn generic_to_outline(&self, indent_level: usize) -> String {
        let mut output = String::new();
        let indent = "  ".repeat(indent_level);
        
        let node_type = self.node_type.to_lowercase();
        output.push_str(&format!("{}{}\n", indent, node_type));
        
        // Show value if not empty/whitespace
        if !self.value.trim().is_empty() 
            && self.value != "\n" 
            && !self.value.chars().all(|c| c.is_whitespace()) {
            output.push_str(&format!("{}  {}\n", indent, self.value));
        }
        
        // Process children
        for child in &self.nodes {
            output.push_str(&child.to_outline(indent_level + 1));
        }
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Metadata, Node, Title, Directive};
    
    #[test]
    fn test_outline_generation() {
        let document = Document {
            metadata: Metadata {
                title: Some(Title {
                    text: "Test Song".to_string(),
                    row: 1,
                    col: 1,
                }),
                directives: vec![
                    Directive {
                        key: "Key".to_string(),
                        value: "C".to_string(),
                        row: 2,
                        col: 1,
                    }
                ],
                detected_system: Some("Western".to_string()),
                attributes: std::collections::HashMap::new(),
            },
            nodes: vec![
                Node::new("LINE".to_string(), "music-line-1".to_string(), 3, 1),
            ],
            notation_system: Some("Western".to_string()),
        };
        
        let result = generate_outline(&document, "test.outline");
        assert!(result.is_ok());
    }
}