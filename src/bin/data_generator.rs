use rand::seq::SliceRandom;
use rand::Rng;
use serde::Serialize;
use svg::node::element::path::Data;
use svg::node::element::{Group, Path, Text};
use svg::Document;
use std::fs::File;
use std::io::Write;

const FONT_SIZE: f64 = 24.0;
const FONT_FAMILY: &str = "monospace";
const CHAR_WIDTH: f64 = FONT_SIZE * 0.6; // Approximate width of a character
const TEXT_Y_POS: f64 = 50.0;
const STROKE_WIDTH: f64 = 1.5;
const STROKE_COLOR: &str = "black";
const CANVAS_PADDING: f64 = 20.0;
const LINE_HEIGHT: f64 = 30.0;

#[derive(Serialize, Debug, Clone)]
struct WordData {
    text: String,
    start_char: usize,
    end_char: usize,
    bbox: [f64; 4], // [xmin, ymin, xmax, ymax]
}

#[derive(Serialize, Debug)]
struct AnnotationData {
    #[serde(rename = "type")]
    annotation_type: String,
    start_char: usize,
    end_char: usize,
    bbox: [f64; 4],
}

#[derive(Serialize, Debug)]
struct JsonOutput {
    image_filename: String,
    full_text: String,
    words: Vec<WordData>,
    annotations: Vec<AnnotationData>,
}

#[derive(Debug, Clone)]
struct Word {
    text: String,
    start_char_index: usize,
}

fn generate_sentence(rng: &mut impl Rng) -> (String, Vec<Word>) {
    let vocabulary = vec![
        "the", "quick", "brown", "fox", "jumps", "over", "lazy", "dog", "and", "a",
        "synthetic", "data", "generator", "creates", "text", "with", "loops", "arcs",
        "rust", "program", "is", "running", "now",
    ];
    let mut sentence = String::new();
    let mut words = Vec::new();
    let num_words = rng.gen_range(5..=10);

    for i in 0..num_words {
        let word_text = vocabulary.choose(rng).unwrap().to_string();
        let start_char_index = sentence.chars().count();
        sentence.push_str(&word_text);
        words.push(Word {
            text: word_text.clone(),
            start_char_index,
        });
        if i < num_words - 1 {
            sentence.push(' ');
        }
    }
    (sentence, words)
}

fn draw_lower_loop(word: &WordData) -> Path {
    let start_x = (word.start_char as f64 * CHAR_WIDTH);
    let end_x = (word.end_char as f64 * CHAR_WIDTH);
    let y = TEXT_Y_POS + FONT_SIZE * 0.4;
    let control_y = y + FONT_SIZE * 0.5;

    let data = Data::new()
        .move_to((start_x, y))
        .quadratic_curve_to(( (start_x + end_x) / 2.0, control_y, end_x, y ));

    Path::new()
        .set("fill", "none")
        .set("stroke", STROKE_COLOR)
        .set("stroke-width", STROKE_WIDTH)
        .set("d", data)
}

fn draw_upper_arc(start_char: usize, end_char: usize) -> Path {
    let start_x = start_char as f64 * CHAR_WIDTH + CHAR_WIDTH / 2.0;
    let end_x = end_char as f64 * CHAR_WIDTH + CHAR_WIDTH / 2.0;
    let y = TEXT_Y_POS - FONT_SIZE * 0.8;
    let control_y = y - FONT_SIZE * 0.5;

    let data = Data::new()
        .move_to((start_x, y))
        .quadratic_curve_to(( (start_x + end_x) / 2.0, control_y, end_x, y ));

    Path::new()
        .set("fill", "none")
        .set("stroke", STROKE_COLOR)
        .set("stroke-width", STROKE_WIDTH)
        .set("d", data)
}

fn main() {
    let mut rng = rand::thread_rng();
    let (sentence, words_info) = generate_sentence(&mut rng);

    let mut words_data = Vec::new();
    for word in &words_info {
        let start_char = word.start_char_index;
        let end_char = start_char + word.text.chars().count();
        let x_min = CANVAS_PADDING + start_char as f64 * CHAR_WIDTH;
        let y_min = TEXT_Y_POS - FONT_SIZE * 0.8; // Approximate top of text
        let x_max = CANVAS_PADDING + end_char as f64 * CHAR_WIDTH;
        let y_max = TEXT_Y_POS + FONT_SIZE * 0.2; // Approximate bottom of text
        words_data.push(WordData {
            text: word.text.clone(),
            start_char,
            end_char,
            bbox: [x_min, y_min, x_max, y_max],
        });
    }

    let mut annotations_data = Vec::new();

    let canvas_width = sentence.chars().count() as f64 * CHAR_WIDTH + 2.0 * CANVAS_PADDING;
    let canvas_height = LINE_HEIGHT * 4.0;

    let mut document = Document::new()
        .set("viewBox", (0, 0, canvas_width, canvas_height))
        .set("width", canvas_width)
        .set("height", canvas_height);

    let text_node = Text::new()
        .set("x", CANVAS_PADDING)
        .set("y", TEXT_Y_POS)
        .set("font-family", FONT_FAMILY)
        .set("font-size", FONT_SIZE)
        .add(svg::node::Text::new(&sentence));
    document = document.add(text_node);

    // Add lower loops
    for word in &words_data {
        if rng.gen_bool(0.3) {
            let loop_path = draw_lower_loop(word);
            let group = Group::new()
                .set("transform", format!("translate({}, 0)", CANVAS_PADDING))
                .add(loop_path);
            document = document.add(group);

            let bbox = [
                word.bbox[0],
                word.bbox[3], // y_max of word
                word.bbox[2],
                word.bbox[3] + FONT_SIZE * 0.5, // y_max + loop depth
            ];
            annotations_data.push(AnnotationData {
                annotation_type: "lower_loop".to_string(),
                start_char: word.start_char,
                end_char: word.end_char,
                bbox,
            });
        }
    }

    // Add upper arcs (slurs)
    let num_arcs_to_generate = rng.gen_range(1..=3);
    let mut arcs_generated = 0;
    let mut attempts = 0;
    while arcs_generated < num_arcs_to_generate && attempts < 100 { // Add an attempt limit to prevent infinite loops
        attempts += 1;
        if words_info.len() < 2 { continue; }

        let idx1 = rng.gen_range(0..words_info.len());
        let mut idx2 = rng.gen_range(0..words_info.len());
        while idx1 == idx2 {
            idx2 = rng.gen_range(0..words_info.len());
        }

        let word1 = &words_info[idx1];
        let word2 = &words_info[idx2];

        let char_idx1 = word1.start_char_index + rng.gen_range(0..word1.text.chars().count());
        let char_idx2 = word2.start_char_index + rng.gen_range(0..word2.text.chars().count());

        let (start_char, end_char) = if char_idx1 < char_idx2 { (char_idx1, char_idx2) } else { (char_idx2, char_idx1) };

        if (end_char - start_char) >= 3 && (end_char - start_char) <= 15 {
             let arc_path = draw_upper_arc(start_char, end_char);
             let group = Group::new()
                .set("transform", format!("translate({}, 0)", CANVAS_PADDING))
                .add(arc_path);
             document = document.add(group);

             let x_min = CANVAS_PADDING + start_char as f64 * CHAR_WIDTH;
             let y_min = TEXT_Y_POS - FONT_SIZE * 0.8 - FONT_SIZE * 0.5; // text top - arc height
             let x_max = CANVAS_PADDING + end_char as f64 * CHAR_WIDTH;
             let y_max = TEXT_Y_POS - FONT_SIZE * 0.8; // text top
             annotations_data.push(AnnotationData {
                annotation_type: "slur".to_string(),
                start_char,
                end_char,
                bbox: [x_min, y_min, x_max, y_max],
            });
            arcs_generated += 1;
        }
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let svg_filename = format!("synthetic_image_{}.svg", timestamp);
    let json_filename = format!("synthetic_image_{}.json", timestamp);

    svg::save(&svg_filename, &document).unwrap();
    println!("Generated image: {}", &svg_filename);

    let json_output = JsonOutput {
        image_filename: svg_filename,
        full_text: sentence,
        words: words_data,
        annotations: annotations_data,
    };

    let json_string = serde_json::to_string_pretty(&json_output).unwrap();
    let mut file = File::create(&json_filename).unwrap();
    file.write_all(json_string.as_bytes()).unwrap();
    println!("Generated label file: {}", &json_filename);
}
