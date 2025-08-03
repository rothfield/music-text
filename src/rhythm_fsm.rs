use crate::models::Node;
use std::collections::HashMap;

fn next_power_of_two(n: usize) -> usize {
    if n <= 1 {
        return 1;
    }
    let mut power = 1;
    while power < n {
        power *= 2;
    }
    power
}

fn previous_power_of_two(n: usize) -> usize {
    if n <= 1 {
        return 1;
    }
    let mut power = 1;
    while power * 2 < n {
        power *= 2;
    }
    power
}

fn gcd(a: usize, b: usize) -> usize {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

fn reduce_fraction(numerator: usize, denominator: usize) -> (usize, usize) {
    let g = gcd(numerator, denominator);
    (numerator / g, denominator / g)
}

#[derive(Debug, Clone)]
struct Element {
    node: Node, // Store the full rich token
    subdivisions: usize,
    duration: Option<String>,
}

#[derive(Debug)]
struct Beat {
    divisions: usize,
    elements: Vec<Element>,
    tied_to_previous: bool,
}

#[derive(Debug)]
enum OutputItem {
    Beat(Beat),
    Barline,
    Breathmark, // New output type for tick character
}

#[derive(Debug, PartialEq)]
enum State {
    S0,
    InBeat,
    Halt,
}

struct FSM {
    state: State,
    output: Vec<OutputItem>,
    current_beat: Option<Beat>,
}

impl FSM {
    fn new() -> Self {
        Self {
            state: State::S0,
            output: vec![],
            current_beat: None,
        }
    }

    fn process(&mut self, nodes: Vec<&Node>) {
        let mut iter = nodes.into_iter().peekable();
        while let Some(node) = iter.next() {
            match self.state {
                State::S0 => {
                    if self.is_barline(node) {
                        self.emit_barline();
                    } else if self.is_beat_separator(node) {
                        // beat_separator, no-op
                    } else if self.is_breathmark(node) {
                        self.emit_breathmark();
                    } else if self.is_dash(node) {
                        self.start_beat_dash();
                    } else if self.is_pitch(node) {
                        self.start_beat_pitch(node);
                    }
                    // Unknown tokens stay in same state (S0)
                },
                State::InBeat => {
                    if self.is_barline(node) || self.is_beat_separator(node) {
                        self.finish_beat();
                        if self.is_barline(node) {
                            self.emit_barline();
                        }
                        self.state = State::S0;
                    } else if self.is_breathmark(node) {
                        self.finish_beat();
                        self.emit_breathmark();
                        self.state = State::S0;
                    } else if self.is_dash(node) {
                        self.extend_last_element();
                    } else if self.is_pitch(node) {
                        self.add_pitch_to_beat(node);
                    }
                    // Unknown tokens stay in same state (InBeat)
                },
                State::Halt => break,
            }
        }

        if self.state == State::InBeat {
            self.finish_beat();
        }
        self.state = State::Halt;
    }

    fn start_beat_pitch(&mut self, node: &Node) {
        let mut beat = Beat { divisions: 1, elements: vec![], tied_to_previous: false };
        beat.elements.push(Element { node: node.clone(), subdivisions: 1, duration: None });
        self.current_beat = Some(beat);
        self.state = State::InBeat;
    }

    fn start_beat_dash(&mut self) {
        let last_node = self.find_last_non_dash_node();
        if let Some(node) = last_node {
            // Found previous pitch or rest - extend it
            let is_pitch = node.node_type == "PITCH" || node.pitch_code.is_some();
            let mut beat = Beat { 
                divisions: 1, 
                elements: vec![], 
                tied_to_previous: is_pitch  // Only tie if previous was a pitch
            };
            beat.elements.push(Element { node, subdivisions: 1, duration: None });
            self.current_beat = Some(beat);
            self.state = State::InBeat;
        } else {
            // No previous element - create rest
            let rest_node = Node {
                node_type: "REST".to_string(),
                value: "r".to_string(),
                row: 0,
                col: 0,
                nodes: vec![],
                pitch_code: None,
                octave: None,
                dash_consumed: false,
                divisions: 1,
            };
            let mut beat = Beat { divisions: 1, elements: vec![], tied_to_previous: false };
            beat.elements.push(Element { node: rest_node, subdivisions: 1, duration: None });
            self.current_beat = Some(beat);
            self.state = State::InBeat;
        }
    }

    fn extend_last_element(&mut self) {
        if let Some(beat) = &mut self.current_beat {
            beat.divisions += 1;
            if let Some(last) = beat.elements.last_mut() {
                last.subdivisions += 1;
            }
        }
    }

    fn add_pitch_to_beat(&mut self, node: &Node) {
        if let Some(beat) = &mut self.current_beat {
            beat.divisions += 1;
            beat.elements.push(Element { node: node.clone(), subdivisions: 1, duration: None });
        }
    }

    fn finish_beat(&mut self) {
        if let Some(mut beat) = self.current_beat.take() {
            for el in &mut beat.elements {
                let (reduced_num, reduced_denom) = reduce_fraction(el.subdivisions, beat.divisions);
                el.duration = Some(format!("{}/{}", reduced_num, reduced_denom));
            }
            self.output.push(OutputItem::Beat(beat));
        }
    }

    fn emit_barline(&mut self) {
        self.output.push(OutputItem::Barline);
    }

    fn emit_breathmark(&mut self) {
        self.output.push(OutputItem::Breathmark);
    }

    fn find_last_non_dash_node(&self) -> Option<Node> {
        for item in self.output.iter().rev() {
            match item {
                OutputItem::Barline | OutputItem::Breathmark => break,
                OutputItem::Beat(beat) => {
                    for el in beat.elements.iter().rev() {
                        return Some(el.node.clone());
                    }
                }
            }
        }
        None
    }

    // Helper methods to identify token types
    fn is_barline(&self, node: &Node) -> bool {
        node.node_type == "BARLINE" || node.value == "|"
    }

    fn is_beat_separator(&self, node: &Node) -> bool {
        // Both spaces and newlines separate beats
        // Check node type instead of exact value to handle multiple spaces
        node.node_type == "WHITESPACE" || node.node_type == "NEWLINE" || node.value == "\n"
    }

    fn is_breathmark(&self, node: &Node) -> bool {
        node.value == "'"
    }

    fn is_dash(&self, node: &Node) -> bool {
        node.value == "-"
    }

    fn is_pitch(&self, node: &Node) -> bool {
        node.node_type == "PITCH" || node.pitch_code.is_some()
    }
}

pub fn test_fsm() {
    use crate::pitch::PitchCode;
    
    println!("Testing FSM with Node objects:");
    
    // Create test nodes
    let s_node = Node {
        node_type: "PITCH".to_string(),
        value: "S".to_string(),
        row: 0,
        col: 0,
        divisions: 0,
        dash_consumed: false,
        nodes: Vec::new(),
        pitch_code: Some(PitchCode::N1),
        octave: Some(0),
    };
    
    let dash_node = Node {
        node_type: "DASH".to_string(),
        value: "-".to_string(),
        row: 0,
        col: 1,
        divisions: 0,
        dash_consumed: false,
        nodes: Vec::new(),
        pitch_code: None,
        octave: None,
    };
    
    let barline_node = Node {
        node_type: "BARLINE".to_string(),
        value: "|".to_string(),
        row: 0,
        col: 2,
        divisions: 0,
        dash_consumed: false,
        nodes: Vec::new(),
        pitch_code: None,
        octave: None,
    };
    
    let mut fsm = FSM::new();
    fsm.process(vec![&s_node, &dash_node, &barline_node]);

    for item in fsm.output {
        println!("{:?}", item);
    }
}

pub fn convert_to_lilypond_with_fsm(nodes: Vec<&Node>) -> Result<String, String> {
    use crate::pitch::{pitchcode_to_english_lilypond};
    
    let mut fsm = FSM::new();
    fsm.process(nodes);
    
    let mut result = Vec::new();
    
    for item in fsm.output {
        match item {
            OutputItem::Beat(beat) => {
                let mut beat_notes = Vec::new();
                for element in beat.elements {
                    // Convert duration fraction to LilyPond duration
                    let lilypond_duration = match element.duration.as_deref() {
                        Some("1/1") => "4",  // Full beat = quarter note
                        Some("1/2") => "8",  // Half beat = eighth note  
                        Some("2/2") => "4",  // Two halves = quarter note
                        Some("1/3") => "8",  // Third of beat = eighth note (will need tuplet)
                        Some("1/4") => "16", // Quarter of beat = sixteenth note
                        Some("2/4") => "8",  // Two quarters = eighth note
                        Some("3/4") => "4.", // Three quarters = dotted quarter
                        _ => "8", // default
                    };
                    
                    // Convert pitch using rich node data
                    let pitch_str = if let Some(pitch_code) = element.node.pitch_code {
                        let base_note = pitchcode_to_english_lilypond(pitch_code);
                        let octave_suffix = match element.node.octave.unwrap_or(0) {
                            -1 => ",",
                            0 => "",
                            1 => "'",
                            _ => "",
                        };
                        format!("{}{}", base_note, octave_suffix)
                    } else {
                        element.node.value.clone()
                    };
                    
                    let note_str = format!("{}{}", pitch_str, lilypond_duration);
                    
                    // Add tie prefix if this beat is tied to previous
                    if beat.tied_to_previous && beat_notes.is_empty() {
                        beat_notes.push(format!("~ {}", note_str));
                    } else {
                        beat_notes.push(note_str);
                    }
                }
                result.extend(beat_notes);
            }
            OutputItem::Barline => {
                result.push("|".to_string());
            }
            OutputItem::Breathmark => {
                result.push("\\breathe".to_string());
            }
        }
    }
    
    Ok(result.join(" "))
}

// Main integration function - replaces group_nodes_into_lines_and_beats
pub fn group_nodes_with_fsm(nodes: &[Node], lines_of_music: &Vec<usize>) -> Vec<Node> {
    eprintln!("FSM_INTEGRATION: group_nodes_with_fsm called with {} nodes", nodes.len());
    let mut result = Vec::new();
    let mut nodes_by_line: HashMap<usize, Vec<&Node>> = HashMap::new();
    
    // Group nodes by line
    for node in nodes {
        nodes_by_line.entry(node.row).or_default().push(node);
    }
    
    // Process each line
    let mut sorted_lines: Vec<_> = nodes_by_line.into_iter().collect();
    sorted_lines.sort_by_key(|(line_num, _)| *line_num);
    
    for (line_num, line_nodes) in sorted_lines {
        // A line is musical if it's in the pre-identified list OR if it contains any PITCH nodes.
        let is_musical = lines_of_music.contains(&line_num) || 
                        line_nodes.iter().any(|n| n.node_type == "PITCH");
        
        if is_musical {
            // This is a line of music - use FSM to process rhythm
            let fsm_result = create_music_line_with_fsm(line_num, line_nodes);
            result.push(fsm_result);
        } else {
            // Non-musical line - create simple LINE node
            let mut line_node = Node::new("LINE".to_string(), "".to_string(), line_num, 0);
            for &node in &line_nodes {
                line_node.nodes.push(node.clone());
            }
            result.push(line_node);
        }
    }
    
    result
}

fn create_music_line_with_fsm(line_num: usize, line_nodes: Vec<&Node>) -> Node {
    // Sort nodes by column
    let mut sorted_nodes = line_nodes;
    sorted_nodes.sort_by_key(|n| n.col);
    
    // Use FSM to process the rhythm
    let mut fsm = FSM::new();
    fsm.process(sorted_nodes.clone());
    
    // Convert FSM output back to Node structure
    let mut line_node = Node::new("LINE".to_string(), "".to_string(), line_num, 0);
    
    for item in fsm.output {
        match item {
            OutputItem::Beat(beat) => {
                let mut beat_node = Node::new("BEAT".to_string(), "".to_string(), line_num, 0);
                beat_node.divisions = beat.divisions;
                
                for (i, element) in beat.elements.iter().enumerate() {
                    let mut element_node = element.node.clone();
                    element_node.divisions = element.subdivisions;
                    // Mark the first note in a tied beat
                    if i == 0 && beat.tied_to_previous {
                        element_node.dash_consumed = true;
                    }
                    // For tuplets, use standard note durations that VexFlow can handle
                    let is_tuplet = beat.divisions > 1 && (beat.divisions & (beat.divisions - 1)) != 0;
                    let note_fraction = if is_tuplet {
                        // For tuplets, keep numerator (subdivisions) but change denominator to power of 2
                        let prev_power_of_2 = previous_power_of_two(beat.divisions);
                        let (reduced_num, reduced_denom) = reduce_fraction(element.subdivisions, prev_power_of_2 * 4);
                        format!("{}/{}", reduced_num, reduced_denom)
                    } else {
                        // Normal case: use element subdivisions in numerator
                        let (reduced_num, reduced_denom) = reduce_fraction(element.subdivisions, beat.divisions * 4);
                        format!("{}/{}", reduced_num, reduced_denom)
                    };
                    element_node.value = format!("{}[{}]", element_node.value.split('[').next().unwrap_or(&element_node.value), note_fraction);
                    eprintln!("FSM: Processing {}", element_node.value);
                    beat_node.nodes.push(element_node);
                }
                
                line_node.nodes.push(beat_node);
            }
            OutputItem::Barline => {
                let barline_node = Node::new("BARLINE".to_string(), "|".to_string(), line_num, 0);
                line_node.nodes.push(barline_node);
            }
            OutputItem::Breathmark => {
                let breathmark_node = Node::new("BREATHMARK".to_string(), "'".to_string(), line_num, 0);
                line_node.nodes.push(breathmark_node);
            }
        }
    }
    
    line_node
}