use std::fs;

mod lex;
mod parse;

use lex::Lexer;

fn main() {
    let source = fs::read_to_string("test.rk").unwrap();
    let mut lexer = Lexer::new(&source, Some("test.rk"));
    //println!("{:#?}", lexer.collect::<Vec<_>>());
    println!("{:#?}", parse::parse_tokens(&mut lexer));
}