//! Some helper traits because a lot of the useful unicode stuff isn't
//! included in `core::char`.


pub trait MaybeWhitespace {
    fn is_whitespace(&self) -> bool;
}

impl MaybeWhitespace for char {
    fn is_whitespace(&self) -> bool {
        match *self {
            '\r' | ' ' | '\t' | '\n' => true,
            _ => false,
        }
    }
}

pub trait MaybeAlphabetic {
    fn is_alphabetic(&self) -> bool;
}

impl MaybeAlphabetic for char {
    fn is_alphabetic(&self) -> bool {
        match *self {
            'a'...'z' | 'A'...'Z' => true,
            _ => false,
        }
    }
}

pub trait AsciiSwapCase {
    fn uppercase(&self) -> Self;
    fn lowercase(&self) -> Self;
}

impl AsciiSwapCase for char {
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


#[cfg(feature = "nightly")]
pub mod lines {
    use lexer::Tokenizer;
    use low_level::BasicParser;
    use high_level::{type_check, Line};

    /// A high level helper function which will parse a stream of characters into
    /// GCodes.
    ///
    /// Where possible, you probably want to use this where possible to make life
    /// easier.
    ///
    ///
    /// # Note
    ///
    /// This requires the `nightly` feature because we use `conservative_impl_trait`,
    /// which is currently behind a feature gate.
    ///
    /// It will also silently ignore parsing, lexing, or type-checking errors.
    pub fn parse<I>(src: I) -> impl Iterator<Item = Line>
        where I: Iterator<Item = char>
    {
        let lexer = Tokenizer::new(src);
        let tokens = lexer.filter_map(|t| t.ok());

        let parser = BasicParser::new(tokens);
        let commands = parser
            .filter_map(|line| line.ok())
            .map(|c| type_check(c))
            .filter_map(|line| line.ok());

        Lines::new(commands)
    }

    #[derive(Debug)]
    pub struct Lines<I>
        where I: Iterator<Item = Line>
    {
        lines: I,
    }

    impl<I> Lines<I>
        where I: Iterator<Item = Line>
    {
        fn new(lines: I) -> Lines<I> {
            Lines { lines: lines }
        }
    }

    impl<I> Iterator for Lines<I>
        where I: Iterator<Item = Line>
    {
        type Item = Line;

        fn next(&mut self) -> Option<Self::Item> {
            self.lines.next()
        }
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
        let inputs = [('a', true),
                      ('b', true),
                      ('z', true),
                      ('x', true),
                      ('A', true),
                      ('B', true),
                      ('Z', true),
                      ('X', true),

                      (' ', false),
                      ('!', false),
                      ('.', false)];

        for &(src, should_be) in &inputs {
            assert_eq!(src.is_alphabetic(), should_be);
        }
    }

    #[test]
    fn test_float_from_integers() {
        let inputs = [((12, 34, 2), 12.34),
                      ((1, 0, 0), 1.0),
                      ((12345, 54321, 5), 12345.54321),
                      ((1000, 0001, 4), 1000.0001)];

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
