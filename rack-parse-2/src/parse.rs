use crate::lex::{Keyword, Lexer, Location, Token, TokenKind};

use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
pub enum Op {
    PushInt(u64),
    //PushStrPtr(usize),
    Plus,
    Minus,
    //DivMod,
    //Dup,
    //Drop,
    //Swap,
    //Over,
    //Equals,
    //Neq,
    //Not,
    //GreaterThan,
    //LessThan,
    //Or,
    //And,
    //ReadByte,
    //If(Option<usize>),
    //While(Option<Box<Op>>),
    //End(Box<Op>),
    Print,
    //CallFn(usize),
    //Ret(usize),        // the number of stack frames to drop
    //Puts,              // Later move to stdlib?
    //Bind(usize, bool), // (number of variables to bind, are we peeking)
    //PushBind(usize),   // index of binding to push
    //Unbind(usize),
}

#[derive(Error, Debug)]
pub enum SyntaxError<'f> {
    #[error("unexpected end of input at {0}")]
    EOF(Location<'f>),
    #[error("{location}: {found} is not allowed at the top level")]
    UnexpectedTopLevel {
        found: TokenKind,
        location: Location<'f>,
    },
    #[error("unexpected token at {location} (expected {expected} but found {found})")]
    UnexpectedToken {
        expected: TokenKind,
        found: TokenKind,
        location: Location<'f>,
    },
}

#[derive(Debug)]
pub struct Func<'src> {
    ident: &'src str,
    body: Vec<Op>,
}

pub fn parse_tokens<'lex, 'src>(
    lexer: &'lex mut Lexer<'src>,
) -> Result<Vec<Func<'src>>, SyntaxError<'src>> {
    let mut funcs = Vec::new();
    loop {
        let Some(t) = lexer.next() else { break };
        match t.kind {
            TokenKind::Keyword(Keyword::Fn) => funcs.push(parse_fn(lexer)?),
            _ => {
                return Err(SyntaxError::UnexpectedTopLevel {
                    found: t.kind,
                    location: t.location,
                })
            }
        }
    }
    Ok(funcs)
}

fn parse_block<'lex, 'src>(lexer: &'lex mut Lexer<'src>) -> Result<Vec<Op>, SyntaxError<'src>> {
    // TODO: replace all of these unwraps and panics with error handling.
    let mut body = Vec::new();
    loop {
        let t = lexer.next().ok_or(SyntaxError::EOF(lexer.location()))?;

        match t.kind {
            TokenKind::Int(num) => body.push(Op::PushInt(num)),
            TokenKind::Keyword(kw) => match kw {
                Keyword::End => break,
                Keyword::Plus => body.push(Op::Plus),
                Keyword::Minus => body.push(Op::Minus),
                Keyword::Print => body.push(Op::Print),
                Keyword::Fn => panic!("no function definitions outside of top-level"),
                Keyword::In => panic!(),
            },
            kind => todo!("parsing of {kind:?} in body"),
        }
    }
    Ok(body)
}

fn parse_fn<'lex, 'src>(lexer: &'lex mut Lexer<'src>) -> Result<Func<'src>, SyntaxError<'src>> {
    let t = lexer.expect_next(TokenKind::Identifier)?;
    let ident = t.value;
    let _ = lexer.expect_next(TokenKind::Keyword(Keyword::In))?;

    let body = parse_block(lexer)?;
    Ok(Func { ident, body })
}
