mod lexer;
mod parser;
mod ast;

use lexer::Lexer;
use parser::Parser;

fn main() {
    use std::env;

    let mut args = env::args();

    let _ = args.next();
    let source_filename = args.next().expect("please provide a source file");
    let source = std::fs::read_to_string(source_filename).unwrap();
    let tokens = Lexer::new(&source).lex().unwrap();
    let program = Parser::new(tokens).parse().unwrap();
    
    for stmt in program {
        println!("{stmt:?}");
    }
}
