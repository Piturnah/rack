use std::collections::HashMap;

use crate::lex::{Keyword, Lexer, Location, TokenKind};

use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
pub enum Op {
    PushInt(u64),
    //PushStrPtr(usize),
    Plus,
    Minus,
    DivMod,
    Dup,
    Drop,
    Swap,
    Over,
    Equals,
    Neq,
    Not,
    GreaterThan,
    LessThan,
    Or,
    And,
    ReadByte,
    //If(Option<usize>),
    //While(Option<Box<Op>>),
    Print,
    CallFn(usize),
    //Ret(usize),        // the number of stack frames to drop
    Puts, // Later move to stdlib?
          //Bind(usize, bool), // (number of variables to bind, are we peeking)
          //PushBind(usize),   // index of binding to push
          //Unbind(usize),
}

#[derive(Error, Debug)]
pub enum SyntaxError<'src> {
    #[error("unexpected end of input at {0}")]
    EOF(Location<'src>),
    #[error("{location}: {found} is not allowed at the top level")]
    UnexpectedTopLevel {
        found: TokenKind,
        location: Location<'src>,
    },
    #[error("unexpected token at {location} (expected {expected} but found {found})")]
    UnexpectedToken {
        expected: TokenKind,
        found: TokenKind,
        location: Location<'src>,
    },
    #[error("{location}: `{identifier}` is an unknown name in the current context")]
    UnknownIdentifier {
        identifier: &'src str,
        location: Location<'src>,
    },
}

#[derive(Debug)]
pub struct Func<'src> {
    ident: &'src str,
    body: Vec<Op>,
}

// NOTE: Can probably be moved, for example can make a `Parser` struct and move related functions
// to impl block?
#[derive(Debug, Default)]
pub struct Context<'src> {
    /// Nametable.
    lookup: HashMap<&'src str, usize>,
    /// The idents that are currently in scope.
    idents: Vec<&'src str>,
}

impl<'src> Context<'src> {
    fn insert_ident(&mut self, ident: &'src str) {
        self.lookup.insert(ident, self.lookup.len());
        self.idents.push(ident);
    }
}

pub fn parse_tokens<'lex, 'src>(
    lexer: &'lex mut Lexer<'src>,
) -> Result<Vec<Func<'src>>, SyntaxError<'src>> {
    let mut funcs = Vec::new();
    let mut ctx = Context::default();
    loop {
        let Some(t) = lexer.next() else { break };
        match t.kind {
            TokenKind::Keyword(Keyword::Fn) => funcs.push(parse_fn(lexer, &mut ctx)?),
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

fn parse_block<'lex, 'src>(
    lexer: &'lex mut Lexer<'src>,
    ctx: &mut Context<'src>,
) -> Result<Vec<Op>, SyntaxError<'src>> {
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
                Keyword::Dup => body.push(Op::Dup),
                Keyword::Drop => body.push(Op::Drop),
                Keyword::Swap => body.push(Op::Swap),
                Keyword::Over => body.push(Op::Over),
                Keyword::Equals => body.push(Op::Equals),
                Keyword::Neq => body.push(Op::Neq),
                Keyword::Not => body.push(Op::Not),
                Keyword::GreaterThan => body.push(Op::GreaterThan),
                Keyword::LessThan => body.push(Op::LessThan),
                Keyword::Or => body.push(Op::Or),
                Keyword::And => body.push(Op::And),
                Keyword::ReadByte => body.push(Op::ReadByte),
                Keyword::Puts => body.push(Op::Puts),
                Keyword::DivMod => body.push(Op::DivMod),
                Keyword::Div => {
                    body.push(Op::DivMod);
                    body.push(Op::Drop);
                }
                Keyword::Mod => {
                    body.push(Op::DivMod);
                    body.push(Op::Swap);
                    body.push(Op::Drop);
                }
                Keyword::Fn => panic!("no function definitions outside of top-level"),
                Keyword::In => panic!(),
            },
            TokenKind::Identifier => {
                if ctx.idents.contains(&t.value) {
                    let symbol = ctx.lookup.get(&t.value).unwrap_or_else(|| {
                        panic!("`{0}` is in scope => `{0}` is in nametable", t.value)
                    });
                    body.push(Op::CallFn(*symbol));
                } else {
                    return Err(SyntaxError::UnknownIdentifier {
                        identifier: t.value,
                        location: t.location,
                    });
                }
            }
        }
    }
    Ok(body)
}

fn parse_fn<'lex, 'src>(
    lexer: &'lex mut Lexer<'src>,
    ctx: &mut Context<'src>,
) -> Result<Func<'src>, SyntaxError<'src>> {
    let t = lexer.expect_next(TokenKind::Identifier)?;
    let ident = t.value;
    ctx.insert_ident(ident);
    let _ = lexer.expect_next(TokenKind::Keyword(Keyword::In))?;

    let body = parse_block(lexer, ctx)?;
    Ok(Func { ident, body })
}
