use crate::parse::SyntaxError;

use std::{error::Error, fmt, num::ParseIntError, str::FromStr};

#[derive(Debug)]
pub struct Token<'src> {
    pub kind: TokenKind,
    pub value: &'src str,
    pub location: Location<'src>, // Can be a separate lifetime if needed.
}

impl<'src> Token<'src> {
    /// Map the token to a Result of Self and a syntax error if the type of token isn't the same
    /// as `expected`.
    pub fn expect_kind(self, expected: TokenKind) -> Result<Self, SyntaxError<'src>> {
        if self.kind != expected {
            Err(SyntaxError::UnexpectedToken {
                expected,
                found: self.kind,
                location: self.location,
            })
        } else {
            Ok(self)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {
    Keyword(Keyword),
    Int(u64),
    Identifier,
    String,
    Char,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Keyword(kw) => write!(f, "keyword {kw}"),
            Self::Int(_) => write!(f, "int"),
            Self::Identifier => write!(f, "identifier"),
            Self::String => write!(f, "string literal"),
            Self::Char => write!(f, "character literal"),
        }
    }
}

#[derive(Debug)]
pub struct Location<'f> {
    pub file: Option<&'f str>,
    pub pos: (usize, usize),
}

impl<'f> fmt::Display for Location<'f> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (row, col) = self.pos;
        if let Some(file) = self.file {
            write!(f, "{file}:{row}:{col}")
        } else {
            write!(f, "{row}:{col}")
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Keyword {
    Fn,
    In,
    End,
    Plus,
    Minus,
    Print,
    Drop,
    Dup,
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
    Puts,
    DivMod,
    Div,
    Mod,
    If,
    While,
    Do,
    Let,
    Peek,
    Ret,
}

#[derive(Debug)]
pub struct UnknownKeywordError;

impl fmt::Display for UnknownKeywordError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unknown keyword")
    }
}

impl Error for UnknownKeywordError {}

macro_rules! keyword_str {
    ($($str:literal => $word:tt),+,) => {
        impl FromStr for Keyword {
            type Err = UnknownKeywordError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($str => Ok(Self::$word)),+,
                    _ => Err(UnknownKeywordError)
                }
            }
        }
        impl fmt::Display for Keyword {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    $(Self::$word => write!(f, $str)),+
                }
            }
        }
    }
}

keyword_str! {
    "fn" => Fn,
    "in" => In,
    "end" => End,
    "+" => Plus,
    "-" => Minus,
    "print" => Print,
    "drop" => Drop,
    "dup" => Dup,
    "swap" => Swap,
    "over" => Over,
    "=" => Equals,
    "!=" => Neq,
    "not" => Not,
    ">" => GreaterThan,
    "<" => LessThan,
    "or" => Or,
    "and" => And,
    "@" => ReadByte,
    "puts" => Puts,
    "divmod" => DivMod,
    "/" => Div,
    "%" => Mod,
    "if" => If,
    "while" => While,
    "do" => Do,
    "let" => Let,
    "peek" => Peek,
    "ret" => Ret,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Lexer<'src> {
    cursor: usize,
    content: &'src str,
    line_begin: usize,
    line: usize,
    file: Option<&'src str>, // Can be a separate lifetime if needed.
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str, file: Option<&'src str>) -> Self {
        Lexer {
            content: source,
            cursor: 0,
            line_begin: 0,
            line: 1,
            file,
        }
    }

    fn trim_left(&mut self) {
        loop {
            match self.content.chars().nth(self.cursor) {
                Some(c) if c.is_whitespace() => {
                    self.cursor += 1;
                    if c == '\n' {
                        self.line += 1;
                        self.line_begin = self.cursor;
                    }
                }
                _ => break,
            }
        }
    }

    pub fn location<'lex>(&'lex self) -> Location<'src> {
        Location {
            file: self.file,
            pos: (self.line, self.cursor - self.line_begin + 1),
        }
    }

    pub fn expect_next<'lex>(
        &'lex mut self,
        expected: TokenKind,
    ) -> Result<Token<'src>, SyntaxError<'src>>
    where
        'src: 'lex,
    {
        self.next()
            .ok_or(SyntaxError::Eof(self.location()))?
            .expect_kind(expected)
    }
}

impl<'src> Iterator for Lexer<'src> {
    type Item = Token<'src>;
    fn next(&mut self) -> Option<Self::Item> {
        self.trim_left();

        let mut string_literal = false;
        let mut char_literal = false;

        let token_begin = self.cursor;
        let mut next_c = self.content.chars().nth(self.cursor)?;
        if next_c == '"' {
            string_literal = true;
            self.cursor = self
                .content
                .chars()
                .skip(self.cursor + 1)
                .position(|c| c == '"')?
                + 1
                + self.cursor;
        }
        if next_c == '\'' {
            char_literal = true;
            self.cursor = self
                .content
                .chars()
                .skip(self.cursor + 1)
                .position(|c| c == '\'')?
                + 1
                + self.cursor;
        }
        if is_separator(next_c) {
            self.cursor += 1;
        } else {
            while !is_separator(next_c) && !next_c.is_whitespace() {
                self.cursor += 1;
                next_c = self.content.chars().nth(self.cursor)?;
            }
        }

        // TODO: this may need to use bytes length instead of chars length.
        let value = &self.content[token_begin..self.cursor];
        let kind = if string_literal {
            TokenKind::String
        } else if char_literal {
            TokenKind::Char
        } else if let Ok(keyword) = Keyword::from_str(value) {
            TokenKind::Keyword(keyword)
        } else if let Ok(num) = parse_int(value) {
            TokenKind::Int(num)
        } else {
            TokenKind::Identifier
        };
        // TODO: refactor to use `Self::location` method.
        let location = Location {
            file: self.file,
            pos: (self.line, token_begin - self.line_begin + 1),
        };

        Some(Token {
            kind,
            value,
            location,
        })
    }
}

fn parse_int(s: &str) -> Result<u64, ParseIntError> {
    if let Some(s) = s.strip_prefix("0x") {
        u64::from_str_radix(s, 16)
    } else if let Some(s) = s.strip_prefix("0o") {
        u64::from_str_radix(s, 8)
    } else if let Some(s) = s.strip_prefix("0b") {
        u64::from_str_radix(s, 2)
    } else {
        s.parse::<u64>()
    }
}

/// A separator is a char token that separates tokens.
fn is_separator(c: char) -> bool {
    ['+', '-', '*', '-', '/'].contains(&c)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn trim_left() {
        let mut l = Lexer::new("\n\n  \n\thello\n", None);
        l.trim_left();
        assert_eq!(
            l,
            Lexer {
                line: 4,
                content: "\n\n  \n\thello\n",
                line_begin: 5,
                cursor: 6,
                file: None
            }
        );
    }
}
