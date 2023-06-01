use std::collections::HashMap;

use crate::lex::{Keyword, Lexer, Location, Token, TokenKind};

use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
pub enum Op {
    PushInt(u64),
    PushStrPtr(usize),
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
    If(Vec<Op>),
    While {
        condn: Vec<Op>,
        body: Vec<Op>,
    },
    Print,
    CallFn(usize),
    // We just copy the bound values to the return stack, so the only info needed by codegen is
    // actually just the number of bindings (`count`).
    Bind {
        count: usize,
        peek: bool,
        body: Vec<Op>,
    },
    // As per the previous comment, we can just use the index from the top of the stack of the
    // binding we want.
    PushBind(usize),
    //Ret(usize),        // the number of stack frames to drop
    Puts,
}

#[derive(Error, Debug)]
pub enum SyntaxError<'src> {
    #[error("unexpected end of input at {0}")]
    Eof(Location<'src>),
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
    #[error("{location}: `{kw}` does not make sense in the current context")]
    UnexpectedKeyword {
        kw: Keyword,
        location: Location<'src>,
    },
    #[error("{location}: {message}")]
    Generic {
        location: Location<'src>,
        message: &'static str,
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
    /// The function identifiers that are currently in scope.
    func_idents: Vec<&'src str>,
    /// String literals referencing directly into the source. Escape sequences handled by codegen.
    strings: Vec<&'src str>,
    bindings: Vec<&'src str>,
}

impl<'src> Context<'src> {
    fn insert_func_ident(&mut self, ident: &'src str) {
        self.lookup.insert(ident, self.lookup.len());
        self.func_idents.push(ident);
    }
}

pub fn parse_tokens<'src>(lexer: &mut Lexer<'src>) -> Result<Vec<Func<'src>>, SyntaxError<'src>> {
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

fn parse_block<'src>(
    lexer: &mut Lexer<'src>,
    ctx: &mut Context<'src>,
    terminator: Keyword,
) -> Result<Vec<Op>, SyntaxError<'src>> {
    let mut body = Vec::new();
    loop {
        let t = lexer.next().ok_or(SyntaxError::Eof(lexer.location()))?;

        match t.kind {
            TokenKind::Int(num) => body.push(Op::PushInt(num)),
            TokenKind::Keyword(kw) => {
                if kw == terminator {
                    break;
                }
                match kw {
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
                    Keyword::Fn => {
                        return Err(SyntaxError::Generic {
                            location: t.location,
                            message: "no function definitions outside of top-level",
                        })
                    }
                    Keyword::In => {
                        return Err(SyntaxError::UnexpectedKeyword {
                            location: t.location,
                            kw,
                        })
                    }
                    Keyword::If => {
                        body.push(Op::If(parse_block(lexer, ctx, Keyword::End)?));
                    }
                    Keyword::While => {
                        let condn = parse_block(lexer, ctx, Keyword::Do)?;
                        let loop_body = parse_block(lexer, ctx, Keyword::End)?;
                        body.push(Op::While {
                            condn,
                            body: loop_body,
                        });
                    }
                    Keyword::Let | Keyword::Peek => {
                        let bindings_count = ctx.bindings.len();
                        let mut count = 0;
                        loop {
                            let next_t = lexer
                                .next()
                                .ok_or_else(|| SyntaxError::Eof(lexer.location()))?;
                            match next_t.kind {
                                TokenKind::Identifier => {
                                    ctx.bindings.push(next_t.value);
                                    count += 1;
                                }
                                TokenKind::Keyword(Keyword::In) => break,
                                found => {
                                    return Err(SyntaxError::UnexpectedToken {
                                        expected: TokenKind::Identifier,
                                        location: next_t.location,
                                        found,
                                    })
                                }
                            }
                        }
                        body.push(Op::Bind {
                            count,
                            peek: matches!(t.kind, TokenKind::Keyword(Keyword::Peek)),
                            body: parse_block(lexer, ctx, Keyword::End)?,
                        });
                        // We can safely remove all the new bindings from ctx as the scope has
                        // ended.
                        ctx.bindings.drain(bindings_count..);
                    }
                    Keyword::Do | Keyword::End => {
                        return Err(SyntaxError::UnexpectedKeyword {
                            kw,
                            location: t.location,
                        })
                    }
                }
            }
            TokenKind::Identifier => {
                // FIXME: There is a bug here: if the ident with the same value appears somewhere
                // earlier in the parsing, but outside of this scope, it will be discovered first
                // by `contains`.
                if ctx.func_idents.contains(&t.value) {
                    let symbol = ctx.lookup.get(&t.value).unwrap_or_else(|| {
                        panic!("`{0}` is in scope => `{0}` is in nametable", t.value)
                    });
                    body.push(Op::CallFn(*symbol));
                } else if let Some(index) = ctx.bindings.iter().rev().position(|b| *b == t.value) {
                    body.push(Op::PushBind(index));
                } else {
                    return Err(SyntaxError::UnknownIdentifier {
                        identifier: t.value,
                        location: t.location,
                    });
                }
            }
            TokenKind::String => {
                // Small optimisation: if an equal string already exists as a literal then we don't
                // need to put it in the table twice.
                let index = ctx
                    .strings
                    .iter()
                    .position(|s| s == &t.value)
                    .unwrap_or(ctx.strings.len());
                if index == ctx.strings.len() {
                    ctx.strings.push(&t.value);
                }
                body.push(Op::PushStrPtr(index));
            }
            TokenKind::Char => {
                // For now, we are not supporting escapes in chars. This is a priority to support
                // once the new lexer/parser is merged into main.
                let value = t
                    .value
                    .strip_prefix('\'')
                    .expect("char literal only parsed with opening `'`")
                    .strip_suffix('\'')
                    .expect("char literal only parsed with closing `'`");
                if value.chars().count() != 1 {
                    return Err(SyntaxError::Generic{
                        location: t.location,
                        message: "all character literals should have a length of 1. Did you mean to use `\"`?"
                    });
                }
                let value = value.chars().next().expect("we just asserted count == 1") as u64;
                body.push(Op::PushInt(value));
            }
        }
    }
    Ok(body)
}

fn parse_fn<'src>(
    lexer: &mut Lexer<'src>,
    ctx: &mut Context<'src>,
) -> Result<Func<'src>, SyntaxError<'src>> {
    let t = lexer.expect_next(TokenKind::Identifier)?;
    let ident = t.value;
    ctx.insert_func_ident(ident);
    let _ = lexer.expect_next(TokenKind::Keyword(Keyword::In))?;

    let body = parse_block(lexer, ctx, Keyword::End)?;
    Ok(Func { ident, body })
}
