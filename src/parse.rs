//! Low level parsing routines.

use core::fmt::Debug;
use core::str::{self, FromStr};
use types::Number;

/// Uses `FromStr` to unconditionally convert a string of bytes into some `T`.
fn parse_bytes<T>(bytes: &[u8]) -> T
where
    T: FromStr,
    T::Err: Debug,
{
    let s = if cfg!(debug_assertions) {
        str::from_utf8(bytes).expect("Input should alway be UTF-8")
    } else {
        unsafe { str::from_utf8_unchecked(bytes) }
    };

    s.parse().expect("unreachable")
}

fn digits(i: &[u8]) -> ::nom::IResult<&[u8], &[u8]> {
    let mut remaining = i;

    while !remaining.is_empty() {
        if remaining[0].is_ascii_digit() {
            remaining = &remaining[1..];
        } else {
            break;
        }
    }

    let num_bytes = i.len() - remaining.len();

    if num_bytes == 0 {
        Err(::nom::Err::Incomplete(::nom::Needed::Size(1)))
    } else {
        let matched = &i[..num_bytes];
        Ok((remaining, matched))
    }
}

named!(decimal_number<&[u8], Number>, do_parse!(
    major: map!(digits, parse_bytes) >>
    char!('.') >>
    minor: map!(digits, parse_bytes) >>
    (Number::Decimal(major, minor))
));

named!(integer_number<&[u8], Number>, map!(map!(digits, parse_bytes), |maj| Number::Integer(maj)));

named!(pub number<&[u8], Number>, alt!(complete!(decimal_number) | integer_number));

named!(
    g_word<&[u8], Number>,
    do_parse!(
        tag_no_case!("g") >>
        num: number >>
        (num)
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_an_integer() {
        let src = b"123";
        let should_be = Number::Integer(123);

        let (_, got) = number(src).unwrap();
        assert_eq!(got, should_be);
    }

    #[test]
    fn parse_a_decimal_number() {
        let src = b"123.45";
        let should_be = Number::Decimal(123, 45);

        let (_, got) = number(src).unwrap();
        assert_eq!(got, should_be);
    }

    #[test]
    fn recognise_g_commands() {
        let inputs = vec![
            ("G90", Number::Integer(90)),
            ("g9", Number::Integer(9)),
            ("G32.1", Number::Decimal(32, 1)),
        ];

        for (src, should_be) in inputs {
            let (_, got) = g_word(src.as_bytes()).unwrap();
            assert_eq!(got, should_be);
        }
    }

    #[test]
    fn invalid_g_commands() {
        let inputs = vec![
            "M", "x12.3", " ", "", "N5", "%", "$", "\0", "G", "g", "1.23"
        ];

        for src in inputs {
            let _ = g_word(src.as_bytes()).unwrap_err();
        }
    }
}
