#![allow(missing_docs, dead_code, unused_imports)]

use core::iter::Peekable;

use lexer::{Token, TokenKind, Tokenizer};
use errors::*;


/// Peek at the next token, if its kind isn't one of the specified `$pattern`s,
/// return a `Error::SyntaxError` with the provided message.
macro_rules! lookahead {
    ($self:expr, $err_msg:expr, $( $pattern:pat )|*) => {
        match $self.peek() {
            $( Some($pattern) )|* => {},
            Some(_) => {
                let next = $self.tokens.peek().unwrap();
                return Err(Error::SyntaxError($err_msg, next.span()));
            }
            None => return Err(Error::UnexpectedEOF),
        }
    }
}


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
        if let Some(TokenKind::N) = self.peek() {
            let _ = self.tokens.next();

            lookahead!(self, r#"A "N" command must be followed by a number"#,
                       TokenKind::Number(_));

            match self.tokens
                      .next()
                      .expect("This should be unreachable")
                      .kind() {
                TokenKind::Number(n) => Ok(Some(n as u32)),
                _ => unreachable!(),
            }
        } else {
            Ok(None)
        }
    }

    fn command_type(&mut self) -> Result<(CommandKind, Number)> {
        lookahead!(self, "Expected a command type",
                   TokenKind::G | TokenKind::M | TokenKind::T);

        let kind = match self.unchecked_next() {
            TokenKind::G => CommandKind::G,
            TokenKind::M => CommandKind::M,
            TokenKind::T => CommandKind::T,
            _ => unreachable!(),
        };

        lookahead!(self, "Commands need to have a number", TokenKind::Number(_));

        let n = match self.unchecked_next() {
            TokenKind::Number(n) => {
                // TODO: Need to amend the lexer's definition of a Number
                if n == n as u32 as f32 {
                    Number::Integer(n as u32)
                } else {
                    unimplemented!();
                }
            }
            _ => unreachable!(),
        };

        Ok((kind, n))
    }

    fn args(&mut self) -> Result<Args> {
        Ok(Args::default())
    }

    fn peek(&mut self) -> Option<TokenKind> {
        self.tokens.peek().map(|t| t.kind().clone())
    }

    fn unchecked_next(&mut self) -> TokenKind {
        self.tokens
            .next()
            .expect("Should never get here because we always do a lookahead first")
            .kind()
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

    parser_test!(basic_g_code, command_type, "G20" => (CommandKind::G, Number::Integer(20)));
    parser_test!(m_command_type, command_type, "M02" => (CommandKind::M, Number::Integer(02)));
    parser_test!(t_command_type, command_type, "T20" => (CommandKind::T, Number::Integer(20)));
    parser_test!(FAIL: invalid_command_type, command_type, "N15");
    parser_test!(FAIL: command_type_with_no_number, command_type, "G X15.0");

    // TODO: Uncomment this when the lexer has been adjusted
    // parser_test!(gcode_with_decimal_command, command_type, "G91.1"
    //              => (CommandKind::G, Number::Decimal(91, 1)));
}
