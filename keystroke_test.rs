use notation_parser::*;

fn test_incremental_parsing(input: &str) {
    println!("=== PARSING: '{}' ===", input.replace('\n', "\\n"));
    
    match unified_parser(input) {
        Ok(document) => {
            println!("Nodes: {}", document.nodes.len());
            println!("System: {:?}", document.metadata.detected_system);
            
            // Show outline
            println!("OUTLINE:");
            println!("{}", document.to_html_outline(0).replace("<br>", "\n").replace("&nbsp;", " "));
        }
        Err(e) => {
            println!("ERROR: {}", e);
        }
    }
    println!();
}

fn main() {
    let keystrokes = vec![
        "T",
        "Ti", 
        "Tit",
        "Titl",
        "Title",
        "Title\n",
        "Title\n\n",
        "Title\n\n ",
        "Title\n\n .",
        "Title\n\n .S",
        "Title\n\n .S\n",
        "Title\n\n .S\n\n",
        "Title\n\n .S\n\n|",
        "Title\n\n .S\n\n|S",
        "Title\n\n .S\n\n|SS",
    ];
    
    for keystroke in keystrokes {
        test_incremental_parsing(keystroke);
    }
}