#![allow(missing_docs, dead_code, unused_imports)]

use core::iter::Peekable;

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
        if let Some(TokenKind::N) = self.peek() {
            let _ = self.tokens.next();

            match self.tokens.next() {
                Some(t) => {
                    match t.kind() {
                        TokenKind::Number(n) => Ok(Some(n as u32)),
                        _ => {
                            Err(Error::SyntaxError("A \"N\" command must be followed by a number",
                                                   t.span()))
                        }
                    }
                }
                None => Err(Error::UnexpectedEOF),
            }
        } else {
            Ok(None)
        }
    }

    fn command_type(&mut self) -> Result<(CommandKind, Number)> {
        // TODO: make legit
        Ok((CommandKind::G, Number::Integer(20)))
    }

    fn args(&mut self) -> Result<Args> {
        Ok(Args::default())
    }

    fn peek(&mut self) -> Option<TokenKind> {
        self.tokens.peek().map(|t| t.kind().clone())
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

    macro_rules! tokens {
        ($src:expr) => {
            {
                let got: Result<::std::vec::Vec<Token>> = ::lexer::Tokenizer::new($src.chars()).collect();
                assert!(got.is_ok(), "Invalid source code");
                got.unwrap().into_iter()
            }
        }
    }

    macro_rules! parser_test {
        ($name:ident, $method:ident, $src:expr => $should_be:expr) => {
            #[test]
            fn $name() {
                let src = tokens!($src);
                let mut parser = Parser::new(src);

                let should_be = $should_be;
                let got = parser.$method().unwrap();

                assert_eq!(got, should_be);
            }
        };
        (FAIL: $name:ident, $method:ident, $src:expr) => {
            #[test]
            fn $name() {
                let src = tokens!($src);
                let mut parser = Parser::new(src);

                assert!(parser.$method().is_err());
            }
        };
    }


    parser_test!(parse_line_number, line_number, "N10" => Some(10));
    parser_test!(no_line_number, line_number, "G10" => None);
    parser_test!(FAIL: invalid_line_number, line_number, "N");

    parser_test!(parse_entire_basic_command, next_command, "G20"
                 => Command {
                        kind: CommandKind::G,
                        number: Number::Integer(20),
                        args: Args::default(),
                        line_number: None,
                    });
}
