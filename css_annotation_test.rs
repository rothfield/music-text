/// End-to-end test for CSS annotation system
///
/// This test demonstrates the complete flow:
/// 1. HTML input with annotations
/// 2. Parse HTML to extract text + CSS classes
/// 3. Parse music normally
/// 4. Assign CSS classes to parsed elements
/// 5. Verify annotations are preserved in the response

use music_text::{process_annotated_notation};

fn main() {
    // Test HTML input with slur annotations (like from the UI)
    let html_input = r#"<span class="cm-note begin-slur">1</span><span class="cm-note">2</span><span class="cm-note end-slur forte">3</span>"#;

    println!("Input HTML: {}", html_input);

    match process_annotated_notation(html_input) {
        Ok(result) => {
            println!("\n✅ Processing successful!");
            println!("Clean text: {}", result.document.elements.len());

            // Print the document structure to verify CSS classes are assigned
            println!("\nDocument structure:");
            for (i, element) in result.document.elements.iter().enumerate() {
                if let music_text::parse::model::DocumentElement::Stave(stave) = element {
                    println!("  Stave {}: {} lines", i, stave.lines.len());

                    for (j, line) in stave.lines.iter().enumerate() {
                        if let music_text::parse::model::StaveLine::ContentLine(content_line) = line {
                            println!("    Content line {}: {} elements", j, content_line.elements.len());

                            for (k, content_element) in content_line.elements.iter().enumerate() {
                                if let music_text::parse::model::ContentElement::Beat(beat) = content_element {
                                    println!("      Beat {}: {} beat elements, classes: {:?}", k, beat.elements.len(), beat.css_classes);

                                    for (l, beat_element) in beat.elements.iter().enumerate() {
                                        match beat_element {
                                            music_text::parse::model::BeatElement::Note(note) => {
                                                println!("        Note {}: '{}' at char {}, classes: {:?}",
                                                    l, note.value.as_ref().unwrap_or(&"?".to_string()), note.char_index, note.css_classes);
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Verify that the JSON serialization includes CSS classes
            println!("\nJSON response includes CSS classes:");
            let json = serde_json::to_string_pretty(&result.document).unwrap();
            if json.contains("css_classes") {
                println!("✅ CSS classes are included in JSON response");
            } else {
                println!("❌ CSS classes are NOT included in JSON response");
            }
        }
        Err(e) => {
            println!("❌ Processing failed: {}", e);
        }
    }
}