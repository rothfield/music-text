use crate::parse::Document;
use uuid::Uuid;

/// Apply octave edit to specific elements in the document
pub fn apply_octave_edit(
    document: &mut Document,
    target_uuids: &[String],
    octave_type: &str,
) -> Result<(), String> {
    // Convert string UUIDs to Uuid objects
    let target_uuids: Result<Vec<Uuid>, _> = target_uuids
        .iter()
        .map(|s| s.parse::<Uuid>())
        .collect();

    let target_uuids = target_uuids.map_err(|e| format!("Invalid UUID format: {}", e))?;

    // Get octave value from octave type
    let octave_value = match octave_type {
        "lowest" => -2,   // LL - Lowest octave (: below)
        "lower" => -1,    // L - Lower octave (. below)
        "middle" => 0,    // M - Middle octave (no markers)
        "higher" => 1,    // U - Upper octave (. above)
        "highest" => 2,   // HH - Highest octave (: above)
        _ => return Err(format!("Unknown octave type: {}", octave_type)),
    };

    // Find and modify notes with matching UUIDs (can be Beat UUIDs or Note UUIDs)
    let mut modified_count = 0;

    // Debug: println!("🔧 Octave edit: Looking for {} target UUIDs: {:?}", target_uuids.len(), target_uuids);

    for element in &mut document.elements {
        if let crate::models::core::DocumentElement::Stave(stave) = element {
            for line in &mut stave.lines {
                modified_count += modify_notes_in_stave_line(line, &target_uuids, octave_value);
            }
        }
    }

    // Debug: println!("🔧 Octave edit: Modified {} notes", modified_count);

    if modified_count == 0 {
        return Err(format!("No notes found with the provided UUIDs. Searched for {} UUIDs in document.", target_uuids.len()));
    }

    Ok(())
}

/// Recursively find and modify notes in a stave line
fn modify_notes_in_stave_line(
    line: &mut crate::models::core::StaveLine,
    target_uuids: &[Uuid],
    octave_value: i8,
) -> usize {
    use crate::models::core::StaveLine;
    let mut modified_count = 0;

    match line {
        StaveLine::ContentLine(content_line) => {
            for element in &mut content_line.elements {
                if let crate::models::elements::ContentElement::Beat(beat) = element {
                    // Debug: println!("🔧 Checking Beat UUID: {}", beat.id);

                    // Check if the Beat itself is selected
                    if target_uuids.contains(&beat.id) {
                        // Debug: println!("🔧 Found matching Beat UUID: {}", beat.id);
                        // Apply octave to all notes in this beat
                        for beat_element in &mut beat.elements {
                            if let crate::models::elements::BeatElement::Note(note) = beat_element {
                                note.octave = octave_value;
                                modified_count += 1;
                            }
                        }
                    } else {
                        // Check individual Note UUIDs within the beat
                        for beat_element in &mut beat.elements {
                            if let crate::models::elements::BeatElement::Note(note) = beat_element {
                                // Debug: println!("🔧 Checking Note UUID: {}", note.id);
                                if target_uuids.contains(&note.id) {
                                    // Debug: println!("🔧 Found matching Note UUID: {}", note.id);
                                    note.octave = octave_value;
                                    modified_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        StaveLine::Content(_parsed_elements) => {
            // Legacy rhythm elements don't have UUIDs, skip them
            // TODO: Convert legacy elements to ContentLine with UUIDs if needed
        }
        _ => {
            // Other line types (Text, Upper, Lower, Lyrics, etc.) don't contain notes
        }
    }

    modified_count
}