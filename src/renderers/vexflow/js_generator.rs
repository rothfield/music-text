/// VexFlow JavaScript code generator
/// Generates self-executing JavaScript that creates VexFlow notation

use crate::parse::model::{Beat, BeatElement, Note, Stave, StaveLine, ContentElement};

pub struct VexFlowJSGenerator {
    js_code: String,
    note_counter: usize,
    voice_counter: usize,
}

impl VexFlowJSGenerator {
    pub fn new() -> Self {
        Self {
            js_code: String::new(),
            note_counter: 0,
            voice_counter: 0,
        }
    }

    pub fn generate_for_stave(&mut self, stave: &Stave, container_id: &str) -> String {
        self.js_code.clear();
        self.note_counter = 0;
        self.voice_counter = 0;

        // Wrap in IIFE to avoid bare return statement
        self.add_line("(function() {");

        // Setup renderer and context
        self.add_line(&format!(
            "  const container = document.getElementById('{}');",
            container_id
        ));
        self.add_line("  if (!container) return;");
        self.add_line("  container.innerHTML = '';");
        self.add_line("");

        self.add_line("  const { Renderer, Stave, StaveNote, Voice, Formatter, Tuplet, Beam } = Vex.Flow;");
        self.add_line("");

        self.add_line("  const renderer = new Renderer(container, Renderer.Backends.SVG);");
        self.add_line("  const canvasWidth = 800;");
        self.add_line("  const canvasHeight = 200;");
        self.add_line("  renderer.resize(canvasWidth, canvasHeight);");
        self.add_line("  const context = renderer.getContext();");
        self.add_line("  context.scale(0.9, 0.9);");
        self.add_line("");

        // Create stave
        self.add_line("  const stave = new Stave(10, 40, 700);");
        self.add_line("  stave.addClef('treble');");
        self.add_line("  stave.setContext(context);");
        self.add_line("  stave.draw();");
        self.add_line("");

        // Process content lines
        let mut all_notes = Vec::new();
        let mut tuplets = Vec::new();
        let mut beams = Vec::new();

        for line in &stave.lines {
            if let StaveLine::ContentLine(content_line) = line {
                for element in &content_line.elements {
                    match element {
                        ContentElement::Beat(beat) => {
                            if beat.is_tuplet == Some(true) {
                                let (tuplet_note_names, tuplet_obj) = self.generate_tuplet(beat);
                                if let Some(tuplet_name) = tuplet_obj {
                                    tuplets.push(tuplet_name);
                                }
                                // Tuplets handle their own beaming/bracketing, no additional beams needed
                                all_notes.extend(tuplet_note_names);
                            } else {
                                let beat_notes = self.generate_beat_notes(beat);
                                // Only beam if the beat contains beamable notes (eighth or shorter)
                                if beat_notes.len() >= 2 && self.is_beat_beamable(beat) {
                                    beams.push(beat_notes.clone());
                                }
                                all_notes.extend(beat_notes);
                            }
                        }
                        ContentElement::Barline(_) => {
                            // Skip barlines for now
                        }
                        ContentElement::Whitespace(_) => {
                            // Skip whitespace
                        }
                        ContentElement::UnknownToken(_) => {
                            // Skip unknown tokens (behave like whitespace)
                        }
                    }
                }
            }
        }

        // Create voice and add all notes
        if !all_notes.is_empty() {
            self.add_line("  // Create voice and add notes");
            let voice_name = self.next_voice_name();
            self.add_line(&format!(
                "  const {} = new Voice({{ num_beats: 4, beat_value: 4, resolution: Vex.Flow.RESOLUTION }});",
                voice_name
            ));
            self.add_line(&format!("  {}.setStrict(false);", voice_name));

            let notes_array = format!("[{}]", all_notes.join(", "));
            self.add_line(&format!("  {}.addTickables({});", voice_name, notes_array));
            self.add_line("");

            // Format and draw
            self.add_line("  // Format and draw");
            self.add_line(&format!(
                "  const formatter = new Formatter().joinVoices([{}]);",
                voice_name
            ));
            self.add_line(&format!("  formatter.format([{}], 600);", voice_name));
            self.add_line(&format!("  {}.draw(context, stave);", voice_name));
            self.add_line("");
        }

        // Draw tuplets
        if !tuplets.is_empty() {
            self.add_line("  // Draw tuplets");
            for tuplet in tuplets {
                self.add_line(&format!("  {}.setContext(context).draw();", tuplet));
            }
            self.add_line("");
        }

        // Draw beams
        if !beams.is_empty() {
            self.add_line("  // Draw beams");
            for beam_notes in beams {
                let beam_name = format!("beam_{}", self.note_counter);
                self.note_counter += 1;
                let notes_array = format!("[{}]", beam_notes.join(", "));
                self.add_line(&format!(
                    "    const {} = new Beam({});",
                    beam_name, notes_array
                ));
                self.add_line(&format!("    {}.setContext(context).draw();", beam_name));
            }
        }

        // Close the IIFE
        self.add_line("})();");

        self.js_code.clone()
    }

    fn generate_tuplet(&mut self, beat: &Beat) -> (Vec<String>, Option<String>) {
        let mut note_names = Vec::new();

        // Generate individual notes with their actual durations
        for element in &beat.elements {
            match element {
                BeatElement::Note(note) => {
                    let note_name = self.next_note_name();
                    let (key, _accidentals) = self.note_to_vexflow_key(note);

                    // Use actual note duration (like in generate_beat_notes)
                    let duration = self.duration_to_vexflow_duration(
                        note.numerator.unwrap_or(1),
                        note.denominator.unwrap_or(4)
                    );

                    self.add_line(&format!(
                        "  const {} = new StaveNote({{ keys: ['{}'], duration: '{}' }});",
                        note_name, key, duration
                    ));
                    note_names.push(note_name.clone());
                }
                BeatElement::Dash(dash) => {
                    // Only process dashes that have rhythm data (starting dashes)
                    if let (Some(numer), Some(denom)) = (dash.numerator, dash.denominator) {
                        let rest_name = self.next_note_name();
                        let duration = self.duration_to_vexflow_duration(numer, denom);

                        self.add_line(&format!(
                            "  const {} = new StaveNote({{ keys: ['b/4'], duration: '{}r' }});",
                            rest_name, duration
                        ));
                        note_names.push(rest_name.clone());
                    }
                    // Skip dashes without rhythm data (extenders)
                }
                _ => {
                    // Skip other elements
                }
            }
        }

        // Create tuplet
        let tuplet_obj = if !note_names.is_empty() {
            let tuplet_name = format!("tuplet_{}", self.note_counter);
            self.note_counter += 1;

            let (num_notes, notes_occupied) = beat.tuplet_ratio.unwrap_or((3, 2));
            let notes_array = format!("[{}]", note_names.join(", "));

            self.add_line(&format!(
                "  const {} = new Tuplet({}, {{ notes_occupied: {}, num_notes: {}, bracketed: true }});",
                tuplet_name, notes_array, notes_occupied, num_notes
            ));

            Some(tuplet_name)
        } else {
            None
        };

        (note_names, tuplet_obj)
    }

    fn generate_beat_notes(&mut self, beat: &Beat) -> Vec<String> {
        let mut note_names = Vec::new();

        for element in &beat.elements {
            match element {
                BeatElement::Note(note) => {
                    let note_name = self.next_note_name();
                    let (key, _accidentals) = self.note_to_vexflow_key(note);

                    // Use simple numerator/denominator duration
                    let duration = self.duration_to_vexflow_duration(
                        note.numerator.unwrap_or(1),
                        note.denominator.unwrap_or(4)
                    );

                    self.add_line(&format!(
                        "  const {} = new StaveNote({{ keys: ['{}'], duration: '{}' }});",
                        note_name, key, duration
                    ));
                    note_names.push(note_name);
                }
                BeatElement::Dash(dash) => {
                    // Only process dashes that have rhythm data (starting dashes)
                    if let (Some(numer), Some(denom)) = (dash.numerator, dash.denominator) {
                        let rest_name = self.next_note_name();
                        let duration = self.duration_to_vexflow_duration(numer, denom);

                        self.add_line(&format!(
                            "  const {} = new StaveNote({{ keys: ['b/4'], duration: '{}r' }});",
                            rest_name, duration
                        ));
                        note_names.push(rest_name);
                    }
                    // Skip dashes without rhythm data (extenders)
                }
                _ => {
                    // Skip other elements
                }
            }
        }

        note_names
    }


    /// Check if a beat contains only notes that can be beamed (eighth notes or shorter)
    fn is_beat_beamable(&self, beat: &Beat) -> bool {
        for element in &beat.elements {
            match element {
                BeatElement::Note(note) => {
                    let denominator = note.denominator.unwrap_or(4);
                    // Only eighth notes (8) and shorter (16, 32, etc.) can be beamed
                    // Quarter notes (4), half notes (2), and whole notes (1) cannot
                    if denominator < 8 {
                        return false;
                    }
                }
                BeatElement::Dash(dash) => {
                    // Check rests with rhythm data
                    if let Some(denom) = dash.denominator {
                        if denom < 8 {
                            return false;
                        }
                    }
                }
                _ => {}
            }
        }
        true
    }

    fn note_to_vexflow_key(&self, note: &Note) -> (String, Vec<String>) {
        let degree = self.pitch_code_to_degree(note.pitch_code);
        self.degree_to_vexflow_key(degree, note.octave)
    }

    fn pitch_code_to_degree(&self, pitch_code: crate::models::PitchCode) -> crate::models::Degree {
        use crate::models::PitchCode::*;
        use crate::models::Degree;

        match pitch_code {
            N1bb => Degree::N1bb, N1b => Degree::N1b, N1 => Degree::N1, N1s => Degree::N1s, N1ss => Degree::N1ss,
            N2bb => Degree::N2bb, N2b => Degree::N2b, N2 => Degree::N2, N2s => Degree::N2s, N2ss => Degree::N2ss,
            N3bb => Degree::N3bb, N3b => Degree::N3b, N3 => Degree::N3, N3s => Degree::N3s, N3ss => Degree::N3ss,
            N4bb => Degree::N4bb, N4b => Degree::N4b, N4 => Degree::N4, N4s => Degree::N4s, N4ss => Degree::N4ss,
            N5bb => Degree::N5bb, N5b => Degree::N5b, N5 => Degree::N5, N5s => Degree::N5s, N5ss => Degree::N5ss,
            N6bb => Degree::N6bb, N6b => Degree::N6b, N6 => Degree::N6, N6s => Degree::N6s, N6ss => Degree::N6ss,
            N7bb => Degree::N7bb, N7b => Degree::N7b, N7 => Degree::N7, N7s => Degree::N7s, N7ss => Degree::N7ss,
        }
    }

    fn degree_to_vexflow_key(&self, degree: crate::models::Degree, octave: i8) -> (String, Vec<String>) {
        use crate::models::Degree::*;

        let (base_note, accidental) = match degree {
            N1bb => ("C", Some("bb")),  N1b => ("C", Some("b")),   N1 => ("C", None),
            N1s => ("C", Some("#")),    N1ss => ("C", Some("##")),
            N2bb => ("D", Some("bb")),  N2b => ("D", Some("b")),   N2 => ("D", None),
            N2s => ("D", Some("#")),    N2ss => ("D", Some("##")),
            N3bb => ("E", Some("bb")),  N3b => ("E", Some("b")),   N3 => ("E", None),
            N3s => ("E", Some("#")),    N3ss => ("E", Some("##")),
            N4bb => ("F", Some("bb")),  N4b => ("F", Some("b")),   N4 => ("F", None),
            N4s => ("F", Some("#")),    N4ss => ("F", Some("##")),
            N5bb => ("G", Some("bb")),  N5b => ("G", Some("b")),   N5 => ("G", None),
            N5s => ("G", Some("#")),    N5ss => ("G", Some("##")),
            N6bb => ("A", Some("bb")),  N6b => ("A", Some("b")),   N6 => ("A", None),
            N6s => ("A", Some("#")),    N6ss => ("A", Some("##")),
            N7bb => ("B", Some("bb")),  N7b => ("B", Some("b")),   N7 => ("B", None),
            N7s => ("B", Some("#")),    N7ss => ("B", Some("##")),
        };

        let vexflow_octave = 4 + octave;
        let key = format!("{}/{}", base_note, vexflow_octave);

        let accidentals = if let Some(acc) = accidental {
            vec![acc.to_string()]
        } else {
            vec![]
        };

        (key, accidentals)
    }

    fn duration_to_vexflow_duration(&self, numerator: u32, denominator: u32) -> String {
        // Direct numerator/denominator mapping to VexFlow durations
        match (numerator, denominator) {
            (1, 1) => "w",        // whole note
            (1, 2) => "h",        // half note
            (1, 4) => "q",        // quarter note
            (1, 8) => "8",        // eighth note
            (1, 12) => "8",       // triplet eighth (show as 8th, tuplet bracket handles timing)
            (1, 16) => "16",      // sixteenth note
            (1, 20) => "16",      // quintuplet sixteenth
            (1, 24) => "16",      // sextuplet sixteenth
            (1, 28) => "16",      // septuplet sixteenth
            (1, 32) => "32",      // thirty-second note
            (1, 48) => "32",      // triplet thirty-second
            (1, 64) => "64",      // sixty-fourth note
            _ => {
                // Algorithmic mapping based on denominator
                if denominator >= 96 { "64" }
                else if denominator >= 48 { "32" }
                else if denominator >= 24 { "16" }
                else if denominator >= 12 { "8" }
                else { "q" }
            }
        }.to_string()
    }


    fn next_note_name(&mut self) -> String {
        let name = format!("note_{}", self.note_counter);
        self.note_counter += 1;
        name
    }

    fn next_voice_name(&mut self) -> String {
        let name = format!("voice_{}", self.voice_counter);
        self.voice_counter += 1;
        name
    }

    fn add_line(&mut self, line: &str) {
        self.js_code.push_str(line);
        self.js_code.push('\n');
    }
}