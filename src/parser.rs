use core::iter::Peekable;
use core::num::Float;

use errors::*;
use argument::Argument;


/// A parser which takes a stream of characters and parses them as gcode
/// instructions.
pub struct Parser<I>
    where I: Iterator<Item = char>
{
    stream: Peekable<I>,
}

impl<I> Parser<I>
    where I: Iterator<Item = char>
{
    pub fn new(stream: I) -> Parser<I> {
        Parser { stream: stream.peekable() }
    }

    fn parse_g_code(&mut self) -> Result<u32> {
        self.expect('G')?;
        let (n, _) = self.parse_integer()?;
        Ok(n)
    }

    /// Parse an integer, returning the integer and its length.
    fn parse_integer(&mut self) -> Result<(u32, u32)> {
        let mut n = 0;
        let mut counter = 0;

        while let Some(peek) = self.stream.peek().cloned() {
            if !peek.is_digit(10) {
                break;
            }

            // these unwraps are actually safe because we've already checked
            let next = self.stream.next().unwrap().to_digit(10).unwrap();

            // TODO: What happens when this overflows?
            n = n * 10 + next;
            counter += 1;
        }

        Ok((n, counter))
    }


    fn parse_argument(&mut self) -> Result<Argument> {
        macro_rules! consume_and_variant {
            ($self:expr, $variant:path) => {
                {
                    let _ = $self.stream.next();
                    let arg = $self.parse_number()?;
                    Ok($variant(arg))
                }
            };
        }

        let next = self.stream.peek().cloned().ok_or(Error::UnexpectedEOF)?;

        match next {
            'X' => consume_and_variant!(self, Argument::X),
            'Y' => consume_and_variant!(self, Argument::Y),
            'Z' => consume_and_variant!(self, Argument::Z),
            'F' => consume_and_variant!(self, Argument::Feed),
            _ => unimplemented!(),
        }
    }

    /// Parse a number which **must** contain a decimal point.
    fn parse_number(&mut self) -> Result<f64> {
        let (integer_part, _) = self.parse_integer()?;
        self.expect('.')?;

        match self.parse_integer() {
            Err(_) => Ok(integer_part as f64),
            Ok((fractional_part, length)) => {
                Ok(float_from_integers(integer_part, fractional_part, length))
            }
        }
    }

    fn expect(&mut self, character: char) -> Result<char> {
        match self.stream.peek().cloned() {
            Some(c) if c == character => {}
            Some(_) => return Err(Error::Expected(character)),
            None => return Err(Error::UnexpectedEOF),
        }

        let _ = self.stream.next();
        Ok(character)
    }
}

/// Create a `f64` from its integer part and fractional part.
fn float_from_integers(integer_part: u32, fractional_part: u32, fractional_length: u32) -> f64 {
    let n = integer_part as f64;
    n + (fractional_part as f64 / 10.0.powi(fractional_length as i32))
}


#[cfg(test)]
mod tests {
    use super::*;

    /// A helper macro for generating parser tests. It will create a new
    /// `Parser` from the `chars()` of the `$src`, run the specified method,
    /// then assert that the unwrapped result is `$should_be`.
    macro_rules! parse_test {
        ($name:ident, $method:ident, $src:expr => $should_be:expr) => {
            #[test]
            fn $name() {
                let src = $src;
                let should_be = $should_be;

                let mut parser = Parser::new(src.chars());
                let got = parser.$method().unwrap();

                assert_eq!(got, should_be);
            }
        }
    }

    parse_test!(parse_integer, parse_integer, "123" => (123, 3));
    parse_test!(parse_integer_part_of_number, parse_integer, "123.456" => (123, 3));
    parse_test!(reads_a_g_code, parse_g_code, "G90" => 90);
    parse_test!(reads_a_decimal, parse_number, "12.34" => 12.34);
    parse_test!(reads_a_decimal_with_lots_of_significant_zeroes, parse_number, "12.00001" => 12.00001);
    parse_test!(reads_number_with_only_trailing_dot, parse_number, "12." => 12.0);
    parse_test!(reads_x_argument, parse_argument, "X12.3" => Argument::X(12.3));

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
}
