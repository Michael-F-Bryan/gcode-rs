//! Some helper traits because a lot of the useful unicode stuff isn't
//! included in `core::char`.


/// An extension trait which allows you to do common character manipulation,
/// assuming your text is purely ascii.
pub trait AsciiExt {
    fn is_whitespace(&self) -> bool;
    fn is_alphabetic(&self) -> bool;
    fn uppercase(&self) -> Self;
    fn lowercase(&self) -> Self;
}

impl AsciiExt for char {
    fn is_whitespace(&self) -> bool {
        match *self {
            '\r' | ' ' | '\t' | '\n' => true,
            _ => false,
        }
    }

    fn is_alphabetic(&self) -> bool {
        match *self {
            'a'...'z' | 'A'...'Z' => true,
            _ => false,
        }
    }

    fn uppercase(&self) -> Self {
        match *self {
            'a'...'z' => {
                let diff = b'a' - b'A';
                (*self as u8 - diff) as Self
            }
            other => other,
        }
    }

    fn lowercase(&self) -> Self {
        match *self {
            'A'...'Z' => {
                let diff = b'a' - b'A';
                (*self as u8 + diff) as Self
            }
            other => other,
        }
    }
}

impl AsciiExt for u8 {
    fn is_whitespace(&self) -> bool {
        (*self as char).is_whitespace()
    }

    fn is_alphabetic(&self) -> bool {
        (*self as char).is_alphabetic()
    }

    fn uppercase(&self) -> Self {
        (*self as char).uppercase() as u8
    }

    fn lowercase(&self) -> Self {
        (*self as char).lowercase() as u8
    }
}


/// Create a `f32` from its integer part and fractional part.
pub fn float_from_integers(integer_part: u32, fractional_part: u32, fractional_length: u32) -> f32 {
    let n = integer_part as f32;
    if fractional_part == 0 {
        n
    } else {
        let ten_shifted = pow(10, fractional_length) as f32;
        n + (fractional_part as f32 / ten_shifted)
    }
}

fn pow(n: u32, exp: u32) -> u32 {
    (1..exp).fold(n, |acc, _| acc * n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_alphabetic() {
        let inputs = [
            ('a', true),
            ('b', true),
            ('z', true),
            ('x', true),
            ('A', true),
            ('B', true),
            ('Z', true),
            ('X', true),
            (' ', false),
            ('!', false),
            ('.', false),
        ];

        for &(src, should_be) in &inputs {
            assert_eq!(src.is_alphabetic(), should_be);
        }
    }

    #[test]
    fn test_float_from_integers() {
        let inputs = [
            ((12, 34, 2), 12.34),
            ((1, 0, 0), 1.0),
            ((12345, 54321, 5), 12345.54321),
            ((1000, 0001, 4), 1000.0001),
        ];

        for &((integer, frac, length), should_be) in &inputs {
            let got = float_from_integers(integer, frac, length);
            println!("({}, {}) => {}", integer, frac, should_be);
            assert_eq!(got, should_be);
        }
    }

    #[test]
    fn swapping_case() {
        let inputs = [('a', 'A'), ('m', 'M'), ('$', '$'), ('z', 'Z'), ('s', 'S')];

        for &(left, right) in &inputs {
            assert_eq!(left.uppercase(), right);
            assert_eq!(right.lowercase(), left);
        }
    }
}
