/// HTML parser for extracting text and CSS classes character by character
///
/// This module implements character-by-character HTML parsing to extract:
/// 1. Clean text content (without HTML tags)
/// 2. CSS classes for each character position
///
/// Example:
/// Input: `<span class="begin-slur forte">12</span><span class="end-slur">3</span>`
/// Output:
/// - text: "123"
/// - classes: [["begin-slur", "forte"], ["begin-slur", "forte"], ["end-slur"]]

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HtmlParseResult {
    pub text: String,
    pub character_classes: Vec<Vec<String>>,
    pub character_styles: Vec<HashMap<String, String>>,
}

#[derive(Debug)]
enum ParseState {
    InText,
    InTag,
    InAttribute,
}

/// Parse HTML and extract text with character-by-character CSS class mapping
pub fn parse_html_with_classes(html: &str) -> HtmlParseResult {
    let mut text = String::new();
    let mut character_classes = Vec::new();
    let mut character_styles = Vec::new();
    let mut current_classes = Vec::new(); // CSS class stack
    let mut current_styles = HashMap::new(); // CSS style stack

    let mut state = ParseState::InText;
    let mut chars = html.chars().peekable();
    let mut tag_buffer = String::new();

    while let Some(ch) = chars.next() {
        match state {
            ParseState::InText => {
                if ch == '<' {
                    state = ParseState::InTag;
                    tag_buffer.clear();
                } else {
                    // Regular character - add to text and record current classes and styles
                    text.push(ch);
                    character_classes.push(current_classes.clone());
                    character_styles.push(current_styles.clone());
                }
            }
            ParseState::InTag => {
                if ch == '>' {
                    // End of tag - process it
                    process_tag(&tag_buffer, &mut current_classes, &mut current_styles);
                    state = ParseState::InText;
                } else {
                    tag_buffer.push(ch);
                }
            }
            ParseState::InAttribute => {
                // Handle attributes if needed (for now, just collect in tag_buffer)
                tag_buffer.push(ch);
                if ch == '>' {
                    process_tag(&tag_buffer, &mut current_classes, &mut current_styles);
                    state = ParseState::InText;
                }
            }
        }
    }

    HtmlParseResult {
        text,
        character_classes,
        character_styles,
    }
}

/// Process a complete HTML tag and update the current CSS class and style stacks
fn process_tag(tag: &str, current_classes: &mut Vec<String>, current_styles: &mut HashMap<String, String>) {
    let tag = tag.trim();

    if tag.starts_with('/') {
        // Closing tag - remove classes and styles that were added by the corresponding opening tag
        if tag == "/span" {
            // Clear all current classes and styles when closing a span
            // This is simplified but works for non-nested spans
            current_classes.clear();
            current_styles.clear();
        }
    } else {
        // Opening tag - extract class and style attributes
        if let Some(classes) = extract_class_attribute(tag) {
            for class in classes {
                if !current_classes.contains(&class) {
                    current_classes.push(class);
                }
            }
        }

        if let Some(styles) = extract_style_attribute(tag) {
            for (property, value) in styles {
                current_styles.insert(property, value);
            }
        }
    }
}

/// Extract CSS classes from a tag's class attribute
/// Example: `span class="begin-slur forte"` -> Some(vec!["begin-slur", "forte"])
fn extract_class_attribute(tag: &str) -> Option<Vec<String>> {
    // Simple regex-free parsing for class attribute
    if let Some(class_start) = tag.find("class=") {
        let after_equals = &tag[class_start + 6..].trim_start();

        // Handle both quoted and unquoted attributes
        let (quote_char, class_content) = if after_equals.starts_with('"') {
            ('"', &after_equals[1..])
        } else if after_equals.starts_with('\'') {
            ('\'', &after_equals[1..])
        } else {
            // Unquoted - find next space
            let end = after_equals.find(' ').unwrap_or(after_equals.len());
            return Some(after_equals[..end].split_whitespace().map(String::from).collect());
        };

        // Find closing quote
        if let Some(quote_end) = class_content.find(quote_char) {
            let classes_str = &class_content[..quote_end];
            return Some(classes_str.split_whitespace().map(String::from).collect());
        }
    }

    None
}

/// Extract CSS styles from a tag's style attribute
/// Example: `span style="--begin-slur: true; color: red"` -> Some(HashMap with {"--begin-slur": "true", "color": "red"})
fn extract_style_attribute(tag: &str) -> Option<HashMap<String, String>> {
    // Simple parsing for style attribute
    if let Some(style_start) = tag.find("style=") {
        let after_equals = &tag[style_start + 6..].trim_start();

        // Handle both quoted and unquoted attributes
        let (quote_char, style_content) = if after_equals.starts_with('"') {
            ('"', &after_equals[1..])
        } else if after_equals.starts_with('\'') {
            ('\'', &after_equals[1..])
        } else {
            // Unquoted - find next space or end
            let end = after_equals.find(' ').unwrap_or(after_equals.len());
            let content = &after_equals[..end];
            return Some(parse_css_declarations(content));
        };

        // Find closing quote
        if let Some(quote_end) = style_content.find(quote_char) {
            let styles_str = &style_content[..quote_end];
            return Some(parse_css_declarations(styles_str));
        }
    }

    None
}

/// Parse CSS declarations from style attribute content
/// Example: "--begin-slur: true; color: red" -> HashMap with {"--begin-slur": "true", "color": "red"}
fn parse_css_declarations(css: &str) -> HashMap<String, String> {
    let mut styles = HashMap::new();

    for declaration in css.split(';') {
        let declaration = declaration.trim();
        if let Some(colon_pos) = declaration.find(':') {
            let property = declaration[..colon_pos].trim().to_string();
            let value = declaration[colon_pos + 1..].trim().to_string();
            if !property.is_empty() && !value.is_empty() {
                styles.insert(property, value);
            }
        }
    }

    styles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_span() {
        let html = r#"<span class="begin-slur">123</span>"#;
        let result = parse_html_with_classes(html);

        assert_eq!(result.text, "123");
        assert_eq!(result.character_classes.len(), 3);
        assert_eq!(result.character_classes[0], vec!["begin-slur"]);
        assert_eq!(result.character_classes[1], vec!["begin-slur"]);
        assert_eq!(result.character_classes[2], vec!["begin-slur"]);
        assert_eq!(result.character_styles.len(), 3);
        assert!(result.character_styles[0].is_empty());
    }

    #[test]
    fn test_multiple_classes() {
        let html = r#"<span class="begin-slur forte">12</span>"#;
        let result = parse_html_with_classes(html);

        assert_eq!(result.text, "12");
        assert_eq!(result.character_classes[0], vec!["begin-slur", "forte"]);
        assert_eq!(result.character_classes[1], vec!["begin-slur", "forte"]);
    }

    #[test]
    fn test_style_attribute() {
        let html = r#"<span style="--begin-slur: true; color: red">12</span>"#;
        let result = parse_html_with_classes(html);

        assert_eq!(result.text, "12");
        assert_eq!(result.character_styles.len(), 2);
        assert_eq!(result.character_styles[0].get("--begin-slur"), Some(&"true".to_string()));
        assert_eq!(result.character_styles[0].get("color"), Some(&"red".to_string()));
        assert_eq!(result.character_styles[1].get("--begin-slur"), Some(&"true".to_string()));
    }

    #[test]
    fn test_class_and_style_combined() {
        let html = r#"<span class="begin-slur" style="--begin-slur: true">12</span>"#;
        let result = parse_html_with_classes(html);

        assert_eq!(result.text, "12");
        assert_eq!(result.character_classes[0], vec!["begin-slur"]);
        assert_eq!(result.character_styles[0].get("--begin-slur"), Some(&"true".to_string()));
    }

    #[test]
    fn test_nested_spans() {
        let html = r#"<span class="begin-slur">1<span class="forte">2</span>3</span>"#;
        let result = parse_html_with_classes(html);

        assert_eq!(result.text, "123");
        assert_eq!(result.character_classes[0], vec!["begin-slur"]);
        assert_eq!(result.character_classes[1], vec!["begin-slur", "forte"]);
        assert_eq!(result.character_classes[2], vec!["begin-slur"]);
    }

    #[test]
    fn test_plain_text() {
        let html = "123 456";
        let result = parse_html_with_classes(html);

        assert_eq!(result.text, "123 456");
        assert_eq!(result.character_classes.len(), 7);
        for classes in result.character_classes {
            assert!(classes.is_empty());
        }
    }

    #[test]
    fn test_mixed_content() {
        let html = r#"1<span class="begin-slur">23</span>4"#;
        let result = parse_html_with_classes(html);

        assert_eq!(result.text, "1234");
        assert!(result.character_classes[0].is_empty()); // '1'
        assert_eq!(result.character_classes[1], vec!["begin-slur"]); // '2'
        assert_eq!(result.character_classes[2], vec!["begin-slur"]); // '3'
        assert!(result.character_classes[3].is_empty()); // '4'
    }
}