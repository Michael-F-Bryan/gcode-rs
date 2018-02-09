use core::str;

use command::{ArgumentKind, Arguments, Command, Kind};
use helpers::AsciiExt;


#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Parser {
    // internal parser state...
}

impl Parser {
    pub fn new() -> Parser {
        Parser::default()
    }

    pub fn parse<'a, 'this: 'a>(&'this mut self, data: &'a [u8]) -> Gcodes<'a> {
        Gcodes {
            data: data,
            current_index: 0,
            parser: self,
        }
    }

    /// Try to parse a single command, updating the parser's internal state
    /// and also returning the number of bytes read.
    fn parse_single_command(&mut self, mut src: &[u8]) -> Option<(Command, usize)> {
        let original_len = src.len();
        let line_no = match line_number(src) {
            Some((n, bytes_read)) => {
                src = &src[bytes_read..];
                Some(n)
            }
            None => None,
        };

        src = skip_whitespace(src);

        let ((kind, major, minor), bytes_read) = match command_name(src) {
            Some(thing) => thing,
            None => return None,
        };
        src = &src[bytes_read..];
        src = skip_whitespace(src);

        let mut args = Arguments::default();

        while let Some((arg_kind, value, bytes_read)) = argument(src) {
            src = &src[bytes_read..];
            args.set(arg_kind, value);

            src = skip_whitespace(src);
        }

        let cmd = Command {
            line: line_no,
            kind: kind,
            number: major,
            secondary_number: minor,
            args: args,
        };
        let total_bytes_read = original_len - src.len();

        Some((cmd, total_bytes_read))
    }
}

#[derive(Debug, PartialEq)]
pub struct Gcodes<'a> {
    data: &'a [u8],
    current_index: usize,
    parser: &'a mut Parser,
}

impl<'a> Iterator for Gcodes<'a> {
    type Item = Command;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let src = &self.data[self.current_index..];
            if src.is_empty() {
                return None;
            }

            match self.parser.parse_single_command(src) {
                Some((cmd, bytes_read)) => {
                    self.current_index += bytes_read;
                    return Some(cmd);
                }
                None => {
                    // we couldn't parse something, try to step past it
                    self.current_index += 1;
                    continue;
                }
            }
        }
    }
}

fn command_name(mut src: &[u8]) -> Option<((Kind, u32, Option<u32>), usize)> {
    let original_len = src.len();

    let kind = match command_kind(src) {
        Some(k) => k,
        None => return None,
    };
    src = &src[1..];

    let (number, bytes_read) = match may_be_integer_or_decimal(src) {
        Some(n) => n,
        None => return None,
    };

    let (major, minor) = number.to_major_minor();

    src = &src[bytes_read..];

    let total_bytes_read = original_len - src.len();

    Some(((kind, major, minor), total_bytes_read))
}

/// Try to parse a command kind from the beginning of a byte string.
fn command_kind(src: &[u8]) -> Option<Kind> {
    let letter = match src.first() {
        Some(l) => l,
        None => return None,
    };

    match letter.uppercase() {
        b'G' => Some(Kind::G),
        b'M' => Some(Kind::M),
        b'T' => Some(Kind::T),
        _ => None,
    }
}

/// Try to parse an integer from the beginning of a bytestring.
fn integer(src: &[u8]) -> Option<(u32, usize)> {
    match take_while(src, |b| b.is_numeric()) {
        Some(number) => {
            let bytes_read = number.len();
            let parsed = str::from_utf8(number).unwrap().parse().unwrap();
            Some((parsed, bytes_read))
        }
        None => None,
    }
}

/// Try to parse a line number.
fn line_number(src: &[u8]) -> Option<(u32, usize)> {
    if src.is_empty() {
        return None;
    }

    if src[0].uppercase() != b'N' {
        return None;
    }

    integer(&src[1..]).map(|(n, bytes_read)| (n, bytes_read + 1))
}

/// Get the substring starting at the beginning where every character satisfies
/// some predicate.
fn take_while<'a, F>(src: &'a [u8], mut pred: F) -> Option<&'a [u8]>
where
    F: FnMut(u8) -> bool,
{
    let mut current_index = 0;

    while let Some(byte) = src.get(current_index) {
        if pred(*byte) {
            current_index += 1;
        } else {
            break;
        }
    }

    if current_index == 0 {
        None
    } else {
        Some(&src[..current_index])
    }
}

/// Get a substring which skips past any leading whitespace.
fn skip_whitespace(src: &[u8]) -> &[u8] {
    let spaces = take_while(src, |b| b.is_whitespace())
        .map(|spaces| spaces.len())
        .unwrap_or(0);

    &src[spaces..]
}

fn decimal(src: &[u8]) -> Option<(&[u8], usize)> {
    let mut seen_decimal = false;

    let got = take_while(src, |b| if b.is_numeric() {
        true
    } else if b == b'.' {
        if seen_decimal {
            // we only want to consume 1 decimal point, so if we've already
            // seen one and we see another, bail
            false
        } else {
            // otherwise we mark that we've seen one and continue
            seen_decimal = true;
            true
        }
    } else {
        // anything other than a digit or decimal we don't want
        false
    });

    match got {
        Some(bytes) => if bytes.contains(&b'.') {
            Some((
                bytes,
                bytes.iter().position(|&byte| byte == b'.').unwrap(),
            ))
        } else {
            None
        },
        None => None,
    }
}

fn argument(src: &[u8]) -> Option<(ArgumentKind, f32, usize)> {
    if src.is_empty() {
        return None;
    }

    let kind = match src[0].uppercase() {
        b'X' => ArgumentKind::X,
        b'Y' => ArgumentKind::Y,
        _ => return None,
    };

    let (value, _) = match decimal(&src[1..]) {
        Some(v) => v,
        None => return None,
    };

    let bytes_read = 1 + value.len();

    match str::from_utf8(value).unwrap().parse() {
        Ok(value) => Some((kind, value, bytes_read)),
        Err(_) => None,
    }
}

/// Something which may be either an integer, or two integers separated by
/// a dot (i.e. a decimal number).
#[derive(Debug, Copy, Clone, PartialEq)]
enum MaybeInteger {
    Integer(u32),
    Decimal(u32, u32),
}

impl MaybeInteger {
    fn to_major_minor(&self) -> (u32, Option<u32>) {
        match *self {
            MaybeInteger::Integer(maj) => (maj, None),
            MaybeInteger::Decimal(maj, min) => (maj, Some(min)),
        }
    }
}

fn may_be_integer_or_decimal(src: &[u8]) -> Option<(MaybeInteger, usize)> {
    if let Some((bytes, decimal_point)) = decimal(src) {
        let (start, end) = bytes.split_at(decimal_point);
        let second_number = &end[1..];
        if second_number.is_empty() {
            return None;
        }

        let first = integer(start).unwrap();
        let second = integer(second_number).unwrap();
        let maybe = MaybeInteger::Decimal(first.0, second.0);
        return Some((maybe, bytes.len()));
    }

    integer(src).map(|(n, bytes_read)| (MaybeInteger::Integer(n), bytes_read))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_g90() {
        let src = b"G90 X10.0";
        let should_be = Command {
            kind: Kind::G,
            number: 90,
            args: Arguments::default().set_x(10.0),
            ..Default::default()
        };

        let mut parser = Parser::new();
        let got = parser.parse(src).next().unwrap();

        assert_eq!(got, should_be);
    }

    #[test]
    fn parse_a_number() {
        let inputs = vec![("1", Some(1)), ("30", Some(30)), ("foo", None)];

        for (src, should_be) in inputs {
            let should_be = should_be.map(|n| (n, src.len()));

            let got = integer(src.as_bytes());
            assert_eq!(got, should_be);
        }
    }

    #[test]
    fn parse_a_command_kind() {
        let inputs = vec![
            ("G", Some(Kind::G)),
            ("g", Some(Kind::G)),
            ("T", Some(Kind::T)),
            ("t", Some(Kind::T)),
            ("n", None),
            ("M", Some(Kind::M)),
            ("m", Some(Kind::M)),
            ("asd", None),
            ("123", None),
        ];

        for (src, should_be) in inputs {
            let got = command_kind(src.as_bytes());
            assert_eq!(got, should_be);
        }
    }

    #[test]
    fn parse_a_line_number() {
        let inputs = vec![
            ("N1", Some(1)),
            ("N123", Some(123)),
            ("n123", Some(123)),
            ("G90", None),
            ("3.14", None),
        ];

        for (src, should_be) in inputs {
            let should_be = should_be.map(|n| (n, src.len()));

            let got = line_number(src.as_bytes());
            assert_eq!(got, should_be);
        }
    }

    #[test]
    fn parse_a_decimal() {
        let inputs = vec![
            ("12.3", Some(2)),
            ("1.23", Some(1)),
            ("1.", Some(1)),
            ("G90", None),
            (".5", Some(0)),
        ];

        for (src, should_be) in inputs {
            let should_be = should_be.map(|decimal_point| (src.as_bytes(), decimal_point));

            let got = decimal(src.as_bytes());
            assert_eq!(got, should_be);
        }
    }

    #[test]
    fn parse_a_command_name() {
        let inputs = vec![
            ("G90", Some((Kind::G, 90, None))),
            ("G90.5", Some((Kind::G, 90, Some(5)))),
            ("M90", Some((Kind::M, 90, None))),
            ("90", None),
            ("G", None),
            ("T12", Some((Kind::T, 12, None))),
        ];

        for (src, should_be) in inputs {
            let should_be = should_be.map(|v| (v, src.len()));

            let got = command_name(src.as_bytes());
            assert_eq!(got, should_be);
        }
    }

    #[test]
    fn maybe_integer() {
        let inputs = vec![
            ("1.2", Some(MaybeInteger::Decimal(1, 2))),
            ("1", Some(MaybeInteger::Integer(1))),
            ("1.", None),
            ("G90.1", None),
        ];

        for (src, should_be) in inputs {
            let should_be = should_be.map(|v| (v, src.len()));

            let got = may_be_integer_or_decimal(src.as_bytes());
            assert_eq!(got, should_be);
        }
    }

    #[test]
    fn argument_parsing() {
        let inputs = vec![
            ("X10.0", Some((ArgumentKind::X, 10.0))),
            ("Y10.", Some((ArgumentKind::Y, 10.0))),
            ("G90", None),
        ];

        for (src, should_be) in inputs {
            let should_be = should_be.map(|(kind, value)| (kind, value, src.len()));

            let got = argument(src.as_bytes());
            assert_eq!(got, should_be);
        }
    }
}
