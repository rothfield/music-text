use notation_parser::*;

fn main() {
    let input = "Title\n\n .S\n\n|SS";
    
    match unified_parser(input) {
        Ok((document, _spatial_analysis_yaml)) => {
            println!("=== SIMPLIFIED PARSER RESULTS ===");
            println!("Detected system: {:?}", document.metadata.detected_system);
            println!("Number of nodes: {}", document.nodes.len());
            
            for (i, node) in document.nodes.iter().enumerate() {
                println!("\nNode {}: {} (line {})", i, node.node_type, node.row);
                println!("  Value: '{}'", node.value);
                println!("  Children: {}", node.nodes.len());
                
                for (j, child) in node.nodes.iter().enumerate() {
                    println!("    Child {}: {} = '{}' at ({}:{})", 
                        j, child.node_type, child.value, child.row, child.col);
                }
            }
            
            // Test outline generation
            println!("\n=== OUTLINE ===");
            println!("{}", document.to_html_outline(0));
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}