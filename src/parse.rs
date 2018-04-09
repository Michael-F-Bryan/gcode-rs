//! Low level parsing routines.

/// A single block of gcodes, usually one line.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Block<'a> {
    src: &'a str,
    line_number: Option<usize>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Word {
    pub mnemonic: Mnemonic,
    pub value: Number,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Mnemonic {
    G,
}

#[derive(Debug, Copy, Clone, PartialEq)]
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

fn parse_integer(src: &str) -> Result<(&str, u16), ParseError> {
    let (rest, number) = take_while(src, |c| (c as u8).is_ascii_digit())
        .ok_or(ParseError::Expected("one or more digits"))?;

    let n = number.parse().expect("never fails");

    Ok((rest, n))
}

fn parse_number(src: &str) -> Result<(&str, Number), ParseError> {
    if let Ok((rest, (a, b))) = parse_decimal(src) {
        return Ok((rest, Number::Decimal(a, b)));
    }

    if let Ok((rest, i)) = parse_integer(src) {
        return Ok((rest, Number::Integer(i)));
    }

    Err(ParseError::Expected("A number"))
}

fn parse_decimal(src: &str) -> Result<(&str, (u16, u16)), ParseError> {
    let (mut rest, integer_part) = parse_integer(src)?;

    if rest.starts_with('.') {
        rest = &rest[1..];
    } else {
        return Err(ParseError::Expected("A decimal point"));
    }

    let (rest, decimal_part) = parse_integer(rest)?;

    let dec = (integer_part, decimal_part);
    Ok((rest, dec))
}

fn take_while<F>(src: &str, mut predicate: F) -> Option<(&str, &str)>
where
    F: FnMut(char) -> bool,
{
    let mut cursor = 0;

    for (ix, c) in src.char_indices() {
        if predicate(c) {
            cursor = ix + c.len_utf8();
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_an_integer() {
        let inputs = vec![("1", 1), ("123", 123), ("1234.567", 1234)];

        for (src, should_be) in inputs {
            let (_, got) = parse_integer(src).unwrap();
            assert_eq!(got, should_be);
        }
    }

    #[test]
    fn parse_a_full_decimal() {
        let src = "12.3";
        let should_be = (12, 3);

        let (_, got) = parse_decimal(src).unwrap();

        assert_eq!(got, should_be);
    }

    #[test]
    fn parse_proper_numbers() {
        let inputs = vec![("1", Number::Integer(1)), ("12.3", Number::Decimal(12, 3))];

        for (src, should_be) in inputs {
            let (_, got) = parse_number(src).unwrap();
            assert_eq!(got, should_be);
        }
    }
}
