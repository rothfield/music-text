use notation_parser::handwritten_lexer::tokenize_with_handwritten_lexer;

fn main() {
    let input = "| S R G M | P D N S' |";
    println!("Input: {}", input);
    println!();
    
    let tokens = tokenize_with_handwritten_lexer(input);
    
    for (i, token) in tokens.iter().enumerate() {
        println!("{}: {} = '{}' (line: {}, col: {})", 
                 i, token.token_type, token.value, token.line, token.col);
    }
}