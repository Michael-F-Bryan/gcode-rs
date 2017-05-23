use core::iter::Peekable;


use errors::*;

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

    fn parse_g_code(&mut self) -> Result<usize> {
        self.expect('G')?;

        Ok(0)
    }

    fn parse_integer(&mut self) -> Result<u32> {
        let mut n = 0;

        while let Some(peek) = self.stream.peek().cloned() {
            if !peek.is_digit(10) {
                break;
            }

            // these unwraps are actually safe because we've already checked
            let next = self.stream.next().unwrap().to_digit(10).unwrap();

            // TODO: What happens when this overflows?
            n = n * 10 + next;
        }

        Ok(n)
    }

    fn expect(&mut self, character: char) -> Result<char> {
        match self.stream.peek().cloned() {
            Some(c) if c == character => {}
            Some(other) => return Err(Error::Expected(character)),
            None => return Err(Error::UnexpectedEOF),
        }

        let _ = self.stream.next();
        Ok(character)
    }
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

    parse_test!(parse_integer, parse_integer, "123" => 123);
    parse_test!(reads_a_g_code, parse_g_code, "G90" => 90);
}
