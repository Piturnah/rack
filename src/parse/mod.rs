use std::process;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, digit1, hex_digit1, multispace0},
    combinator::map,
    error::ParseError,
    multi::{many0, many1},
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};
use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug, PartialEq, Eq)]
pub struct Func {
    pub name: String,
    pub block: Vec<Op>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token<'a> {
    pub position: Span<'a>,
    pub op: Op,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Op {
    PushInt(u64),
    Plus,
    Minus,
}

// Whitespace delimited combinator from nom docs.
fn ws<'a, F: 'a, O, E: ParseError<Span<'a>>>(
    inner: F,
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, O, E>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

fn nom_op(s: Span) -> IResult<Span, Op> {
    alt((nom_push_int, nom_plus, nom_minus))(s)
}

fn nom_plus(s: Span) -> IResult<Span, Op> {
    map(ws(tag("+")), |_| Op::Plus)(s)
}

fn nom_minus(s: Span) -> IResult<Span, Op> {
    map(ws(tag("-")), |_| Op::Minus)(s)
}

macro_rules! nom_number_literal {
    ($($name:ident, $prefix:literal => $base:literal),+,) => {
        fn nom_push_int(s: Span) -> IResult<Span, Op> {
            map(
                alt(($($name),+, nom_decimal_literal)),
                Op::PushInt,
            )(s)
        }

        $(
            // TODO: I'd rather be able to interpolate the `$name` into a longer fn name such as
            // `nom_($name)_literal`.
            fn $name(s: Span) -> IResult<Span, u64> {
                map(ws(preceded(tag($prefix), hex_digit1)), |digits| {
                    u64::from_str_radix(&digits, $base).unwrap_or_else(|e| {
                        eprintln!("Could not parse {} literal: {e}", stringify!($name));
                        process::exit(1);
                    })
                })(s)
            }
        )+
    }
}

nom_number_literal! {
    hex,    "0x" => 16,
    octal,  "0o" => 8,
    binary, "0b" => 2,
}

fn nom_decimal_literal(s: Span) -> IResult<Span, u64> {
    map(ws(digit1), |digits| {
        digits.parse::<u64>().unwrap_or_else(|e| {
            eprintln!("Could not parse number literal: {e}");
            process::exit(1);
        })
    })(s)
}

fn nom_block(s: Span) -> IResult<Span, Vec<Op>> {
    many0(nom_op)(s)
}

pub fn nom_fn(s: Span) -> IResult<Span, Func> {
    map(
        tuple((
            ws(delimited(
                tag("fn"),
                ws(many1(alt((digit1, alpha1)))),
                tag("in"),
            )),
            ws(terminated(nom_block, tag("end"))),
        )),
        |(name, block)| Func {
            name: name
                .iter()
                .map(|s| s.fragment().to_owned())
                .collect::<String>(),
            block,
        },
    )(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_int_literal() {
        assert_eq!(
            nom_push_int(Span::new("4")).map(|r| r.1),
            Ok(Op::PushInt(4))
        );
        assert_eq!(
            nom_push_int(Span::new("0x1a3 ")).map(|r| r.1),
            Ok(Op::PushInt(0x1a3))
        );
        assert_eq!(
            nom_push_int(Span::new("\t0o4")).map(|r| r.1),
            Ok(Op::PushInt(4))
        );
        assert_eq!(
            nom_push_int(Span::new("0b0101\n\n")).map(|r| r.1),
            Ok(Op::PushInt(5))
        );
    }

    #[test]
    fn parse_prog() {
        use Op::*;

        assert_eq!(
            nom_block(Span::new("0x4 5 +  5")).map(|r| r.1),
            Ok(vec![PushInt(4), PushInt(5), Plus, PushInt(5)])
        );
    }

    #[test]
    fn parse_fn() {
        assert_eq!(
            nom_fn(Span::new(
                "
fn main in
  1 1 -
end"
            ))
            .map(|r| r.1),
            Ok(Func {
                name: "main".to_string(),
                block: vec![Op::PushInt(1), Op::PushInt(1), Op::Minus]
            })
        )
    }
}
