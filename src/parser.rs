use core::iter::{FilterMap, Peekable};

use lexer::{Token, TokenKind, Tokenizer};
use errors::*;


pub struct Parser<I>
    where I: Iterator<Item = Token>
{
    tokens: Peekable<I>,
}

impl<I> Parser<I>
    where I: Iterator<Item = Token>
{
    fn new(tokens: I) -> Parser<I> {
        Parser { tokens: tokens.peekable() }
    }

    /// A convenience function for creating a parser directly from a stream of
    /// characters.
    #[cfg(nightly)]
    fn from_source<C, F>(src: C) -> Parser<impl Iterator<Item = Token>>
        where C: Iterator<Item = char>
    {
        let tok = Tokenizer::new(src);
        Parser { tokens: tok.filter_map(|t| t.ok()).peekable() }
    }

    fn next_command(&mut self) -> Result<Command> {
        let line_number = self.line_number()?;
        let (kind, number) = self.command_type()?;
        let args = self.args()?;

        Ok(Command {
               kind,
               number,
               args,
               line_number,
           })
    }

    fn line_number(&mut self) -> Result<Option<u32>> {
        // TODO: make legit
        Ok(None)
    }

    fn command_type(&mut self) -> Result<(CommandKind, Number)> {
        // TODO: make legit
        Ok((CommandKind::G, Number::Integer(20)))
    }

    fn args(&mut self) -> Result<Args> {
        Ok(Args::default())
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Number {
    Integer(u32),
    Decimal(u32, u32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    kind: CommandKind,
    number: Number,
    args: Args,
    line_number: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Args {
    x: Option<f32>,
    y: Option<f32>,
    z: Option<f32>,
    s: Option<f32>,
    t: Option<f32>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CommandKind {
    G,
    M,
    T,
}


#[cfg(test)]
mod tests {
    use super::*;
    use lexer::TokenKind;

    macro_rules! tokens {
        ($src:expr) => {
            {
                let got: Result<::std::vec::Vec<Token>> = ::lexer::Tokenizer::new($src.chars()).collect();
                assert!(got.is_ok(), "Invalid source code");
                got.unwrap().into_iter()
            }
        }
    }

    #[test]
    fn basic_command() {
        let src = tokens!("G20"); // Set units to inches
        let mut parser = Parser::new(src);

        let should_be = Command {
            kind: CommandKind::G,
            number: Number::Integer(20),
            args: Args::default(),
            line_number: None,
        };

        let got = parser.next_command().unwrap();

        assert_eq!(got, should_be);
    }
}
