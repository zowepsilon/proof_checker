mod lexer;
mod parser;
mod ast;
mod checker;
mod unification;

use lexer::Lexer;
use parser::Parser;
use checker::Checker;

fn main() {
    use std::env;

    let mut args = env::args();

    let _ = args.next();
    let source_filename = args.next().expect("please provide a source file");
    let source = std::fs::read_to_string(source_filename).unwrap();
    let tokens = Lexer::new(&source).lex().unwrap();
    let program = Parser::new(tokens).parse().unwrap();

    let mut checker = Checker::new();
    for stmt in program {
        match checker.check(stmt) {
            Ok((name, _)) => println!("Ok: {name}"),
            Err(e) => {
                use checker::CheckingError::*;

                match e {
                    IncorrectProof => eprintln!("ERROR: incorrect proof"),
                    UnknownVariable(f, var) => eprintln!("Unknown variable `{}` in {:?}", var, f),
                    CannotInfer(v) => eprintln!("ERROR: cannot infer variable `{v}` as it is not in the proposition"),
                    UnificationError => eprintln!("ERROR: TODO unification error message"),
                }

                return;
            }
        }
    }
}
