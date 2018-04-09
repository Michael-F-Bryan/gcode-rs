//! Low level parsing routines.

use core::str::{self, FromStr};

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
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Number {
    Integer(u16),
    Decimal(u16, u16),
}

impl Number {
    pub fn major(&self) -> u16 {
        match *self {
            Number::Integer(maj) | Number::Decimal(maj, _) => maj,
        }
    }

    pub fn minor(&self) -> Option<u16> {
        match *self {
            Number::Integer(_) => None,
            Number::Decimal(_, min) => Some(min),
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
    let mnemonic = src.first().ok_or(ParseError::EOF)?;
    let rest = &src[1..];

    match mnemonic {
        b'g' | b'G' => parse_number(rest).map(|(r, n)| (r, Word::G(n))),
        b'n' | b'N' => parse_number(rest).map(|(r, n)| (r, Word::N(n))),
        _ => Err(ParseError::Expected("A word")),
    }
}

fn parse_integer(src: &[u8]) -> Result<(&[u8], u16), ParseError> {
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
        return Err(ParseError::EOF);
    }

    if let Ok((rest, (a, b))) = parse_decimal(src) {
        return Ok((rest, Number::Decimal(a, b)));
    }

    if let Ok((rest, i)) = parse_integer(src) {
        return Ok((rest, Number::Integer(i)));
    }

    Err(ParseError::Expected("A number"))
}

fn parse_decimal(src: &[u8]) -> Result<(&[u8], (u16, u16)), ParseError> {
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
    EOF,
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
            ("G91.5", Word::G(Number::Decimal(91, 5))),
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
}
