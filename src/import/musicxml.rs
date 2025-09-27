use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};

use crate::models::core::{Document, DocumentElement, Stave, StaveLine, TextLine};
use crate::models::notation::NotationSystem;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportOptions {
    pub prefer_minor: bool,
}

impl Default for ImportOptions {
    fn default() -> Self { Self { prefer_minor: false } }
}

// Map fifths to tonic (major by default), supports -7..7
fn tonic_for_fifths(fifths: i32, prefer_minor: bool) -> (&'static str, bool) {
    // Circle of fifths ordered majors
    // index = fifths mod 12 mapped to known range
    let majors = [
        "C","G","D","A","E","B","F#","C#","G#","D#","A#","F"
    ];
    let minors = [
        "A","E","B","F#","C#","G#","D#","A#","F","C","G","D"
    ];
    let idx = ((fifths % 12) + 12) % 12;
    if prefer_minor { (minors[idx as usize], true) } else { (majors[idx as usize], false) }
}

// Build scale from tonic: returns degrees N1..N7 as pitch names with accidentals
fn scale_for_tonic(tonic: &str, minor: bool) -> [String;7] {
    // Semitone steps for major/minor scales
    let steps = if minor { [2,1,2,2,1,2,2] } else { [2,2,1,2,2,2,1] };
    let pitch_classes = ["C","C#","D","D#","E","F","F#","G","G#","A","A#","B"];
    // Map tonic to index
    let mut idx = pitch_classes.iter().position(|p| *p == tonic)
        .unwrap_or(0);
    let mut out: [String;7] = Default::default();
    out[0] = pitch_classes[idx].to_string();
    for d in 1..7 {
        idx = (idx + steps[d-1] as usize) % 12;
        out[d] = pitch_classes[idx].to_string();
    }
    out
}

fn midi_to_step_alter(step: &str, alter: i32) -> String {
    match alter {
        -2 => format!("{}bb", step),
        -1 => format!("{}b", step),
         0 => step.to_string(),
         1 => format!("{}#", step),
         2 => format!("{}##", step),
         _ => step.to_string(),
    }
}

fn pitch_to_pc(step: &str, alter: i32) -> String { midi_to_step_alter(step, alter) }

fn degree_for_pitch(pc: &str, scale: &[String;7]) -> Option<usize> {
    for i in 0..7 { if scale[i] == pc { return Some(i+1);} }
    None
}

pub fn import_musicxml_to_document(xml: &str, opts: Option<ImportOptions>) -> anyhow::Result<Document> {
    let opts = opts.unwrap_or_default();
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut buf = Vec::new();
    let mut in_key = false;
    let mut fifths: i32 = 0;

    // Simple pass to find key fifths (first occurrence)
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = e.local_name();
                if name.as_ref() == b"key" { in_key = true; }
            }
            Ok(Event::End(e)) => {
                if e.local_name().as_ref() == b"key" { break; }
            }
            Ok(Event::Empty(_)) => {}
            Ok(Event::Text(t)) => {
                if in_key { // next text may be in <fifths>
                    // unfortunately quick_xml requires tracking exact element; do minimalistic: reparse small slice
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    // Fallback: regex-like search for <fifths>n</fifths>
    if let Some(start) = xml.find("<fifths>") {
        if let Some(end) = xml[start..].find("</fifths>") {
            let val = &xml[start+8..start+end];
            if let Ok(n) = val.trim().parse::<i32>() { fifths = n; }
        }
    }

    let (tonic, is_minor) = tonic_for_fifths(fifths, opts.prefer_minor);
    let scale = scale_for_tonic(tonic, is_minor);

    // Parse time signature and divisions for rhythm grouping
    let divisions: i32 = find_tag_text(xml, "divisions").and_then(|s| s.parse().ok()).unwrap_or(1);
    let beats: i32 = find_tag_text(xml, "beats").and_then(|s| s.parse().ok()).unwrap_or(4);
    let beat_type: i32 = find_tag_text(xml, "beat-type").and_then(|s| s.parse().ok()).unwrap_or(4);
    let beat_div: i32 = ((4 * divisions) / beat_type.max(1)).max(1);

    // Collect monophonic notes in order from first part/measure/note (exclude rests, chords)
    // Build beats by accumulating note/rest durations into beat_div windows
    let mut beats_out: Vec<String> = Vec::new();
    let mut current_beat = String::new();
    let mut remaining_in_beat = beat_div;
    let mut rdr = Reader::from_str(xml);
    rdr.trim_text(true);
    let mut b = Vec::new();
    let mut in_note = false; let mut is_rest = false; let mut is_chord = false;
    let mut step: Option<String> = None; let mut alter: i32 = 0;

    loop {
        match rdr.read_event_into(&mut b) {
            Ok(Event::Start(e)) => {
                let n = e.local_name();
                if n.as_ref() == b"note" { in_note = true; is_rest=false; is_chord=false; step=None; alter=0; }
            }
            Ok(Event::Empty(e)) => {
                let n = e.local_name();
                if in_note && n.as_ref() == b"rest" { is_rest = true; }
                if in_note && n.as_ref() == b"chord" { is_chord = true; }
            }
            Ok(Event::End(e)) => {
                if e.local_name().as_ref() == b"note" {
                    // finalize will be handled after duration extraction below
                    in_note = false;
                }
            }
            Ok(Event::Text(t)) => {
                if in_note {
                    let txt = t.unescape().unwrap_or_default().to_string();
                    // Detect current subelement by peeking previous start tag in raw xml; 
                    // Minimal approach: look back a small window
                    // Prefer robust second pass: naive string search around this text
                }
            }
            Ok(Event::Start(ref e)) if in_note => {
                let ln = e.local_name();
                if ln.as_ref() == b"step" {
                    if let Ok(Event::Text(t2)) = rdr.read_event_into(&mut b) {
                        step = Some(t2.unescape().unwrap_or_default().to_string());
                    }
                    // consume end
                    let _ = rdr.read_event_into(&mut b);
                } else if ln.as_ref() == b"alter" {
                    if let Ok(Event::Text(t2)) = rdr.read_event_into(&mut b) {
                        alter = t2.unescape().unwrap_or_default().to_string().parse::<i32>().unwrap_or(0);
                    }
                    let _ = rdr.read_event_into(&mut b);
                } else if ln.as_ref() == b"duration" {
                    // duration in divisions
                    let mut dur_div: i32 = 0;
                    if let Ok(Event::Text(t2)) = rdr.read_event_into(&mut b) {
                        dur_div = t2.unescape().unwrap_or_default().to_string().parse::<i32>().unwrap_or(0);
                    }
                    let _ = rdr.read_event_into(&mut b);
                    if !is_chord { // monophonic
                        if is_rest {
                            // fill beat with dashes for the duration
                            let mut rem = dur_div;
                            while rem > 0 {
                                if remaining_in_beat == beat_div && current_beat.is_empty() {
                                    // starting a rest beat: leading dash
                                    current_beat.push('-');
                                    remaining_in_beat -= 1;
                                    rem -= 1;
                                    continue;
                                }
                                // extend rest within beat
                                current_beat.push('-');
                                remaining_in_beat -= 1;
                                rem -= 1;
                                if remaining_in_beat == 0 {
                                    beats_out.push(std::mem::take(&mut current_beat));
                                    remaining_in_beat = beat_div;
                                }
                            }
                        } else if let Some(s) = &step {
                            let pc = pitch_to_pc(s, alter);
                            // map to Nk with accidentals by comparing to key scale
                            let token = map_pc_to_degree_with_acc(&pc, &scale);
                            let mut rem = dur_div;
                            // place token at the start of a subdivision; if beat is mid, still place as new subdivision
                            if remaining_in_beat == 0 { remaining_in_beat = beat_div; }
                            // insert the note symbol
                            current_beat.push_str(&token);
                            remaining_in_beat -= 1;
                            rem -= 1;
                            // sustain remainder with dashes possibly across beats
                            while rem > 0 {
                                if remaining_in_beat == 0 {
                                    beats_out.push(std::mem::take(&mut current_beat));
                                    remaining_in_beat = beat_div;
                                    // cross-beat sustain: leading dash
                                    current_beat.push('-');
                                    remaining_in_beat -= 1;
                                    rem -= 1;
                                    continue;
                                }
                                current_beat.push('-');
                                remaining_in_beat -= 1;
                                rem -= 1;
                                if remaining_in_beat == 0 {
                                    beats_out.push(std::mem::take(&mut current_beat));
                                    remaining_in_beat = beat_div;
                                }
                            }
                        }
                    }
                } else if ln.as_ref() == b"rest" { is_rest = true; }
                else if ln.as_ref() == b"chord" { is_chord = true; }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        b.clear();
    }

    // flush last beat
    if !current_beat.is_empty() { beats_out.push(current_beat); }

    // Build a single-stave Document in Number notation with one content line as Text for now (simple)
    let content_line = if beats_out.is_empty() { "".to_string() } else { beats_out.join(" ") };

    let stave = Stave {
        value: None,
        char_index: 0,
        notation_system: NotationSystem::Number,
        line: 0,
        column: 0,
        index_in_line: 0,
        index_in_doc: 0,
        lines: vec![
            StaveLine::Text(TextLine{ value: Some(content_line), char_index: 0 })
        ],
    };

    let document = Document {
        document_uuid: None,
        value: None,
        char_index: 0,
        title: None,
        author: None,
        directives: Default::default(),
        elements: vec![DocumentElement::Stave(stave)],
        ui_state: Default::default(),
        timestamp: String::new(),
    };

    Ok(document)
}

fn find_tag_text(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    if let Some(i) = xml.find(&open) {
        if let Some(endrel) = xml[i+open.len()..].find(&format!("</{}>", tag)) {
            return Some(xml[i+open.len()..i+open.len()+endrel].to_string());
        }
    }
    None
}

fn map_pc_to_degree_with_acc(pc: &str, scale: &[String;7]) -> String {
    // compute semitone index for pc and for each scale degree; choose nearest with delta in [-6,6]
    let pcs = ["C","C#","D","D#","E","F","F#","G","G#","A","A#","B"];
    let idx = pcs.iter().position(|p| *p == pc).unwrap_or(0) as i32;
    let mut best_deg = 1; let mut best_delta: i32 = 12;
    for (i, sd) in scale.iter().enumerate() {
        let si = pcs.iter().position(|p| *p == sd).unwrap_or(0) as i32;
        let mut d = (idx - si) % 12; let mut d2 = d;
        if d2 > 6 { d2 -= 12; }
        if d2 < -6 { d2 += 12; }
        if d2.abs() < best_delta.abs() { best_delta = d2; best_deg = (i+1) as i32; }
    }
    let acc = match best_delta {
        0 => "".to_string(),
        1 => "#".to_string(),
        2 => "##".to_string(),
        -1 => "b".to_string(),
        -2 => "bb".to_string(),
        _ => {
            // fallback clamp
            if best_delta > 0 { "#".to_string() } else { "b".to_string() }
        }
    };
    format!("N{}{}", best_deg, acc)
}
