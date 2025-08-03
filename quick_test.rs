use notation_parser::*;

fn main() {
    let input = "Title\n\n .S\n\n|SS";
    
    match unified_parser(input) {
        Ok(document) => {
            println!("=== FINAL RESULT ===");
            println!("Nodes: {}", document.nodes.len());
            
            for (i, node) in document.nodes.iter().enumerate() {
                println!("Node {}: TYPE='{}' VALUE='{}'", i, node.node_type, node.value);
                for (j, child) in node.nodes.iter().enumerate() {
                    println!("  Child {}: TYPE='{}' VALUE='{}'", j, child.node_type, child.value);
                }
            }
            
            println!("\nOUTLINE:");
            println!("{}", document.to_html_outline(0).replace("<br>", "\n").replace("&nbsp;", " "));
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}