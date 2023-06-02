use std::{borrow::Cow, collections::HashMap};

use crate::lex::{Keyword, Lexer, Location, TokenKind};

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
    // We hold the number of additional stack frames to drop (due to let/peek bindings).
    Ret(usize),
    Puts,
}

#[derive(Error, Debug, Clone)]
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
pub struct Program<'src> {
    pub funcs: Vec<Func<'src>>,
    pub ctx: Context<'src>,
}

#[derive(Debug)]
pub struct Func<'src> {
    pub ident: &'src str,
    pub body: Vec<Op>,
}

#[derive(Debug, Default)]
pub struct Context<'src> {
    /// Nametable.
    pub lookup: HashMap<&'src str, usize>,
    /// The function identifiers that are currently in scope.
    pub func_idents: Vec<&'src str>,
    /// String literals referencing directly into the source. Escape sequences handled by codegen.
    pub strings: Vec<Cow<'src, str>>,
    bindings: Vec<&'src str>,
}

impl<'src> Context<'src> {
    fn insert_func_ident(&mut self, ident: &'src str) {
        self.lookup.insert(ident, self.lookup.len());
        self.func_idents.push(ident);
    }
}

pub fn parse_tokens<'src>(lexer: &mut Lexer<'src>) -> Result<Program<'src>, SyntaxError<'src>> {
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
    Ok(Program { funcs, ctx })
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
                    Keyword::True => body.push(Op::PushInt(1)),
                    Keyword::False => body.push(Op::PushInt(0)),
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
                    Keyword::Ret => body.push(Op::Ret(ctx.bindings.len())),
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
                // Clean the strings - involves stripping the delimiting " and escaping \s.
                // Must be done now so that we have an accurate length for `Op::PushInt`.
                let value = t
                    .value
                    .strip_prefix("\"")
                    .expect("string literal only lexed with opening `\"`")
                    .strip_suffix("\"")
                    .expect("string literal only lexed with closed `\"`");

                let value = if t.value.contains('\\') {
                    Cow::Owned(
                        value
                            .replace("\\n", "\n")
                            .replace("\\n", "\n")
                            .replace("\\t", "\t")
                            .replace("\\0", "\0"),
                    )
                } else {
                    Cow::Borrowed(value)
                };
                let len = value.len();

                // Small optimisation: if an equal string already exists as a literal then we don't
                // need to put it in the table twice.
                let index = ctx
                    .strings
                    .iter()
                    .position(|s| *s == value)
                    .unwrap_or(ctx.strings.len());

                if index == ctx.strings.len() {
                    ctx.strings.push(value);
                }
                body.push(Op::PushInt(len as u64));
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

    let mut body = parse_block(lexer, ctx, Keyword::End)?;
    body.push(Op::Ret(0));
    Ok(Func { ident, body })
}
