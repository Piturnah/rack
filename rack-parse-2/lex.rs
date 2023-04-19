

#[derive(Debug)]
pub struct Token<'source> {
    kind: TokenKind,
    value: &'source str,
    location: Location<'source>, // Can be a separate lifetime if needed.
}

#[derive(Debug)]
pub enum TokenKind {
    Keyword(Keyword),
    Identifier,
}

#[derive(Debug)]
struct Location<'f> {
    file: Option<&'f str>,
    pos: (usize, usize),
}

#[derive(Debug)]
pub enum Keyword {
    Fn,
    In,
    End,
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
    ($($str:literal => $word:tt),+) => {
        impl FromStr for Keyword {
            type Err = UnknownKeywordError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($str => Ok(Self::$word)),+,
                    _ => Err(UnknownKeywordError)
                }
            }
        }
        // fmt::Display impl can go here if necessary.
    }
}

keyword_str! {
    "fn" => Fn,
    "in" => In,
    "end" => End
}

#[derive(Debug, PartialEq, Eq)]
pub struct Lexer<'source> {
    cursor: usize,
    content: &'source str,
    line_begin: usize,
    line: usize,
    file: Option<&'source str>, // Can be a separate lifetime if needed.
}

impl<'source> Lexer<'source> {
    pub fn new(source: &'source str, file: Option<&'source str>) -> Self {
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
}

impl<'source> Iterator for Lexer<'source> {
    type Item = Token<'source>;
    fn next(&mut self) -> Option<Self::Item> {
        self.trim_left();

        let token_begin = self.cursor;
        while !self.content.chars().nth(self.cursor)?.is_whitespace() {
            self.cursor += 1;
        }

        // TODO: this may need to use bytes length instead of chars length.
        let value = &self.content[token_begin..self.cursor];
        let kind = if let Ok(keyword) = Keyword::from_str(value) {
            TokenKind::Keyword(keyword)
        } else {
            TokenKind::Identifier
        };
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
