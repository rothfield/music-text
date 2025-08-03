// src/colorizer/mod.rs
// ANSI colorization functionality extracted from display module

use std::collections::HashMap;
use std::fs;
use regex::Regex;
use colored::*;
use crate::models::Metadata;

pub fn parse_css_for_ansi(css_path: &str) -> HashMap<String, (String, bool)> {
    let mut styles = HashMap::new();
    let content = fs::read_to_string(css_path).unwrap_or_default();
    let rule_regex =
        Regex::new(r"\.token-([a-zA-Z0-9_-]+)\s*\{\s*color:\s*([a-zA-Z]+)\s*;(?:\s*/\*\s*(.*?)\s*\*/)?\s*\}")
            .unwrap();

    for cap in rule_regex.captures_iter(&content) {
        let token_name = cap[1].to_uppercase().replace("-", "_");
        let color = cap[2].to_lowercase();
        let reverse = cap.get(3).map_or(false, |m| m.as_str().contains("reverse"));
        styles.insert(token_name, (color, reverse));
    }
    styles
}

pub fn colorize_string(s: &str, color: &str, reverse: bool) -> String {
    let mut colored_s = match color {
        "yellow" => s.yellow(),
        "white" => s.white(),
        "green" => s.green(),
        "darkcyan" => s.cyan(),
        "red" => s.red(),
        "magenta" => s.magenta(),
        "blue" => s.blue(),
        "brown" => s.truecolor(165, 142, 142),
        _ => s.normal(),
    };
    if reverse {
        colored_s = colored_s.on_truecolor(50, 50, 50); // Dark grey background for reverse
    }
    colored_s.to_string()
}

pub fn colorize_title(text: &str, color: &str) -> String {
    let colored_title = match color {
        "yellow" => text.yellow().bold().underline(),
        "white" => text.white().bold().underline(),
        "green" => text.green().bold().underline(),
        "darkcyan" => text.cyan().bold().underline(),
        "red" => text.red().bold().underline(),
        "magenta" => text.magenta().bold().underline(),
        "blue" => text.blue().bold().underline(),
        "brown" => text.truecolor(165, 42, 42).bold().underline(),
        _ => text.normal().bold().underline(),
    };
    colored_title.to_string()
}

pub fn colorize_beat_element(text: &str, color: &str, reverse: bool) -> String {
    let colored_val = match color {
        "yellow" => text.yellow().underline(),
        "white" => text.white().underline(),
        "green" => text.green().underline(),
        "darkcyan" => text.cyan().underline(),
        "red" => text.red().underline(),
        "magenta" => text.magenta().underline(),
        "blue" => text.blue().underline(),
        "brown" => text.truecolor(165, 42, 42).underline(),
        _ => text.normal().underline(),
    };
    if reverse {
        colored_val.on_truecolor(50, 50, 50).to_string()
    } else {
        colored_val.to_string()
    }
}

pub fn generate_legend_string(
    styles: &HashMap<String, (String, bool)>,
    used_tokens: &HashMap<String, String>,
    metadata: Option<&Metadata>,
    for_flattener: bool,
) -> String {
    let mut legend = String::new();
    legend.push_str(&format!("{}\n", "--- Active Token Legend ---".bold()));
    let mut sorted_tokens: Vec<_> = used_tokens.iter().collect();
    sorted_tokens.sort_by_key(|(k, _v)| *k);

    for (token_type, sample_value) in sorted_tokens {
        if let Some((color, reverse)) = styles.get(token_type as &str) {
            legend.push_str(&format!(
                "- {}: {}\n",
                token_type,
                colorize_string(sample_value, color, *reverse)
            ));
        }
    }

    if let Some(meta) = metadata {
        if meta.title.is_some() {
            if let Some((color, _)) = styles.get("TITLE") {
                legend.push_str(&format!(
                    "- {}: {}\n",
                    "TITLE",
                    colorize_string("Title Text", color, false).bold().underline()
                ));
            }
        }
        if !meta.directives.is_empty() {
            if let Some((color, _)) = styles.get("DIRECTIVE_KEY") {
                legend.push_str(&format!(
                    "- {}: {}\n",
                    "DIRECTIVE_KEY",
                    colorize_string("key:", color, false)
                ));
            }
            if let Some((color, _)) = styles.get("DIRECTIVE_VALUE") {
                legend.push_str(&format!(
                    "- {}: {}\n",
                    "DIRECTIVE_VALUE",
                    colorize_string("value", color, false)
                ));
            }
        }
    }

    if for_flattener {
        legend.push_str(&format!(
            "- {}: {}\n",
            "UNASSIGNED",
            colorize_string(" ", "white", true)
        ));
    }

    legend.push_str(&format!("{}\n", "---------------------------".bold()));
    legend
}