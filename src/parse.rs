//! Low level parsing routines.

use core::fmt::{self, Display, Formatter};
use core::str::{self, FromStr};

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

/// A single block of gcodes, usually one line.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Block<'a> {
    src: &'a str,
    line_number: Option<usize>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Word {
    G(Number),
    N(Number),
    T(Number),

    X(f32),
    Y(f32),
    Z(f32),

    I(f32),
    J(f32),
    K(f32),
}

impl Display for Word {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Word::G(n) => write!(f, "G{}", n),
            Word::N(n) => write!(f, "N{}", n),
            Word::T(n) => write!(f, "T{}", n),
            Word::X(n) => write!(f, "X{}", n),
            Word::Y(n) => write!(f, "Y{}", n),
            Word::Z(n) => write!(f, "Z{}", n),
            Word::I(n) => write!(f, "I{}", n),
            Word::J(n) => write!(f, "J{}", n),
            Word::K(n) => write!(f, "K{}", n),
        }
    }
}

impl FromStr for Word {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Word, Self::Err> {
        let (rest, got) = parse_word(s.as_bytes())?;

        if rest.is_empty() {
            Err(ParseError::UnexpectedEOF)
        } else {
            Ok(got)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Number {
    Integer(u32),
    Decimal(u32, u32),
}

impl Number {
    pub fn major(&self) -> u32 {
        match *self {
            Number::Integer(maj) | Number::Decimal(maj, _) => maj,
        }
    }

    pub fn minor(&self) -> Option<u32> {
        match *self {
            Number::Integer(_) => None,
            Number::Decimal(_, min) => Some(min),
        }
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Number::Integer(n) => n.fmt(f),
            Number::Decimal(a, b) => write!(f, "{}.{}", a, b),
        }
    }
}

impl FromStr for Number {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Number, Self::Err> {
        let (rest, got) = parse_number(s.as_bytes())?;

        if rest.is_empty() {
            Err(ParseError::UnexpectedEOF)
        } else {
            Ok(got)
        }
    }
}

/// Skips whitespace (excluding newlines).
fn skip_whitespace(src: &[u8]) -> &[u8] {
    match take_while(src, |c| c != b'\n' && c.is_ascii_whitespace()) {
        Some((rest, _)) => rest,
        None => src,
    }
}

fn skip_comments_to_newline(src: &[u8]) -> &[u8] {
    if src.starts_with(b";") {
        let rest = take_while(&src[1..], |c| c != b'\n')
            .map(|(rest, _)| rest)
            .unwrap_or(src);
        return rest;
    }

    if src.starts_with(b"(") {
        match take_while(src, |c| c != b')' && c != b'\n') {
            Some((rest, _)) => {
                if rest.starts_with(b")") {
                    &rest[1..]
                } else {
                    src
                }
            }
            _ => unimplemented!(),
        }
    } else {
        src
    }
}

fn parse_word(src: &[u8]) -> Result<(&[u8], Word), ParseError> {
    let mnemonic = src.first().ok_or(ParseError::UnexpectedEOF)?;
    let rest = &src[1..];

    fn num_parse<F>(rest: &[u8], func: F) -> Result<(&[u8], Word), ParseError>
    where
        F: Fn(Number) -> Word,
    {
        parse_number(rest).map(|(r, n)| (r, func(n)))
    }

    fn float_parse<F>(rest: &[u8], func: F) -> Result<(&[u8], Word), ParseError>
    where
        F: Fn(f32) -> Word,
    {
        parse_float(rest).map(|(r, n)| (r, func(n)))
    }

    match mnemonic.to_ascii_uppercase() {
        b'G' => num_parse(rest, Word::G),
        b'T' => num_parse(rest, Word::T),
        b'N' => num_parse(rest, Word::N),

        b'X' => float_parse(rest, Word::X),
        b'Y' => float_parse(rest, Word::Y),
        b'Z' => float_parse(rest, Word::Z),

        b'I' => float_parse(rest, Word::I),
        b'J' => float_parse(rest, Word::J),
        b'K' => float_parse(rest, Word::K),
        _ => Err(ParseError::Expected("A word")),
    }
}

fn parse_integer(src: &[u8]) -> Result<(&[u8], u32), ParseError> {
    let (rest, number) = take_while(src, |c| (c as u8).is_ascii_digit())
        .ok_or(ParseError::Expected("one or more digits"))?;

    Ok((rest, parse_from_str(number)))
}

fn parse_from_str<F>(src: &[u8]) -> F
where
    F: FromStr,
{
    let parsed = unsafe { str::from_utf8_unchecked(src).parse() };
    match parsed {
        Ok(f) => f,
        Err(_) => panic!("This should always parse: {}", str::from_utf8(src).unwrap()),
    }
}

fn parse_number(src: &[u8]) -> Result<(&[u8], Number), ParseError> {
    if src.is_empty() {
        return Err(ParseError::UnexpectedEOF);
    }

    if let Ok((rest, (a, b))) = parse_decimal(src) {
        return Ok((rest, Number::Decimal(a, b)));
    }

    if let Ok((rest, i)) = parse_integer(src) {
        return Ok((rest, Number::Integer(i)));
    }

    Err(ParseError::Expected("A number"))
}

fn parse_decimal(src: &[u8]) -> Result<(&[u8], (u32, u32)), ParseError> {
    let (mut rest, integer_part) = parse_integer(src)?;

    if rest.starts_with(b".") {
        rest = &rest[1..];
    } else {
        return Err(ParseError::Expected("A decimal point"));
    }

    let (rest, decimal_part) = parse_integer(rest)?;

    let dec = (integer_part, decimal_part);
    Ok((rest, dec))
}

fn parse_float(mut src: &[u8]) -> Result<(&[u8], f32), ParseError> {
    let negative = if src.starts_with(b"-") {
        src = &src[1..];
        true
    } else {
        false
    };

    let (rest, _) = parse_number(src)?;
    let bytes_read = src.len() - rest.len();
    let float_as_str = &src[..bytes_read];

    let f: f32 = parse_from_str(float_as_str);
    if negative {
        Ok((rest, -f))
    } else {
        Ok((rest, f))
    }
}

fn take_while<F>(src: &[u8], mut predicate: F) -> Option<(&[u8], &[u8])>
where
    F: FnMut(u8) -> bool,
{
    let mut cursor = 0;

    for (ix, &c) in src.into_iter().enumerate() {
        if predicate(c) {
            cursor = ix + 1;
        } else {
            break;
        }
    }

    let (keep, rest) = src.split_at(cursor);

    if !keep.is_empty() {
        Some((rest, keep))
    } else {
        None
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ParseError {
    Expected(&'static str),
    NumberTooBig,
    UnexpectedEOF,
}

#[cfg(test)]
impl Arbitrary for Number {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        if g.gen_weighted_bool(2) {
            Number::Integer(g.gen())
        } else {
            Number::Decimal(g.gen(), g.gen())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_an_integer() {
        let inputs = vec![("1", 1), ("123", 123), ("1234.567", 1234)];

        for (src, should_be) in inputs {
            let (_, got) = parse_integer(src.as_bytes()).unwrap();
            assert_eq!(got, should_be);
        }
    }

    #[test]
    fn parse_a_full_decimal() {
        let src = "12.3";
        let should_be = (12, 3);

        let (_, got) = parse_decimal(src.as_bytes()).unwrap();

        assert_eq!(got, should_be);
    }

    #[test]
    fn parse_proper_numbers() {
        let inputs = vec![("1", Number::Integer(1)), ("12.3", Number::Decimal(12, 3))];

        for (src, should_be) in inputs {
            let (_, got) = parse_number(src.as_bytes()).unwrap();
            assert_eq!(got, should_be);
        }
    }

    #[test]
    fn parse_some_floats() {
        let inputs = vec![
            ("1", 1.0),
            ("10", 10.0),
            ("1.23", 1.23),
            ("0.5", 0.5),
            ("1.", 1.0),
            ("-3.14", -3.14),
            // (".5", 0.5),
        ];

        for (src, should_be) in inputs {
            let (_, got) = parse_float(src.as_bytes()).map_err(|_| src).unwrap();
            assert_eq!(got, should_be);
        }
    }

    #[test]
    fn parse_some_valid_words() {
        let inputs = vec![
            ("G90", Word::G(Number::Integer(90))),
            ("N05", Word::N(Number::Integer(5))),
            ("T50", Word::T(Number::Integer(50))),
            ("G91.5", Word::G(Number::Decimal(91, 5))),
            ("X91.5", Word::X(91.5)),
            ("y-5.0", Word::Y(-5.0)),
            ("Z5", Word::Z(5.0)),
            ("Z5.", Word::Z(5.0)),
            ("i-3.14", Word::I(-3.14)),
        ];

        for (src, should_be) in inputs {
            let (_, got) = parse_word(src.as_bytes()).map_err(|_| src).unwrap();
            assert_eq!(got, should_be);
        }
    }

    #[test]
    fn skip_whitespace_junk() {
        let inputs = vec![
            (" asd", "asd"),
            ("\t \n", "\n"),
            ("    1  \n", "1  \n"),
            ("\n  ", "\n  "),
        ];

        for (src, should_be) in inputs {
            let got = skip_whitespace(src.as_bytes());
            let got = str::from_utf8(got).unwrap();
            assert_eq!(got, should_be);
        }
    }

    #[test]
    fn skip_some_comments() {
        let inputs = vec![("(asd)f", "f"), ("; comment\n", "\n"), ("asd", "asd")];

        for (src, should_be) in inputs {
            let got = skip_comments_to_newline(src.as_bytes());
            let got = str::from_utf8(got).unwrap();
            assert_eq!(got, should_be);
        }
    }

    quickcheck! {
        fn round_trip_int_parsing(n: u32) -> bool {
            let src = format!("{}", n);
            let (_, got) = parse_integer(src.as_bytes()).unwrap();

            got == n
        }

        fn float_parsing_is_reversible(n: f32) -> bool {
            let src = format!("{}", n);
            let (_, got) = parse_float(src.as_bytes()).unwrap();

            got == n
        }

        fn round_trip_number(n: Number) -> bool {
            let src = format!("{}", n);
            let (_, got) = parse_number(src.as_bytes()).unwrap();

            got == n
        }
    }
}
