use std::fs;

mod lex;
mod parse;

use lex::Lexer;

fn main() {
    let source = fs::read_to_string("test.rk").unwrap();
    let mut lexer = Lexer::new(&source, Some("test.rk"));
    match parse::parse_tokens(&mut lexer) {
        Ok(ast) => println!("{ast:#?}"),
        Err(e) => println!("{e}"),
    }
}
