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


#[derive(Debug)]
pub struct Parser<I>
    where I: Iterator<Item = Token>
{
    tokens: Peekable<I>,
}

impl<I> Parser<I>
    where I: Iterator<Item = Token>
{
    pub fn new(tokens: I) -> Parser<I> {
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
                    return Err(Error::SyntaxError("Commands with a decimal aren't supported at the moment",
                                                  Default::default()));
                }
            }
            _ => unreachable!(),
        };

        Ok((kind, n))
    }

    fn args(&mut self) -> Result<Args> {
        let mut a = Args::default();

        while let Ok((kind, value)) = self.argument() {
            a.set(kind, value);
        }

        Ok(a)
    }

    fn argument(&mut self) -> Result<(ArgumentKind, f32)> {
        lookahead!(self, "Expected an argument kind",
        TokenKind::X | TokenKind::Y | TokenKind::Z | TokenKind::S | 
        TokenKind::FeedRate | TokenKind::I | TokenKind::J);

        let kind = match self.unchecked_next() {
            TokenKind::X => ArgumentKind::X,
            TokenKind::Y => ArgumentKind::Y,
            TokenKind::Z => ArgumentKind::Z,
            TokenKind::S => ArgumentKind::S,
            TokenKind::I => ArgumentKind::I,
            TokenKind::J => ArgumentKind::J,
            TokenKind::FeedRate => ArgumentKind::F,
            _ => unreachable!(),
        };

        // Check for a negative number
        let is_negative = if self.peek() == Some(TokenKind::Minus) {
            let _ = self.tokens.next();
            true
        } else {
            false
        };

        let n = match self.tokens.next() {
            Some(t) => {
                match t.kind() {
                    TokenKind::Number(number) => number,
                    other => {
                        return Err(Error::SyntaxError("All arguments must be followed by a number",
                                                      t.span()))
                    }
                }
            }
            None => return Err(Error::UnexpectedEOF),
        };

        if is_negative {
            Ok((kind, -1.0 * n))
        } else {
            Ok((kind, n))
        }
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

impl<I> Iterator for Parser<I>
    where I: Iterator<Item = Token>
{
    type Item = Result<Command>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.next_command() {
            Err(Error::UnexpectedEOF) => None,
            other => Some(other),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Number {
    Integer(u32),
    Decimal(u32, u32),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Command {
    kind: CommandKind,
    number: Number,
    args: Args,
    line_number: Option<u32>,
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Args {
    x: Option<f32>,
    y: Option<f32>,
    z: Option<f32>,
    s: Option<f32>,
    t: Option<f32>,
    f: Option<f32>,
    i: Option<f32>,
    j: Option<f32>,
}

impl Args {
    fn set(&mut self, kind: ArgumentKind, value: f32) {
        match kind {
            ArgumentKind::X => self.x = Some(value),
            ArgumentKind::Y => self.y = Some(value),
            ArgumentKind::Z => self.z = Some(value),
            ArgumentKind::S => self.s = Some(value),
            ArgumentKind::F => self.f = Some(value),
            ArgumentKind::I => self.i = Some(value),
            ArgumentKind::J => self.j = Some(value),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum ArgumentKind {
    X,
    Y,
    Z,
    F,
    S,
    I,
    J,
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


    parser_test!(negative_x_argument, argument, "X-10.0" => (ArgumentKind::X, -10.0));
    parser_test!(x_argument, argument, "X10.0" => (ArgumentKind::X, 10.0));
    parser_test!(y_argument, argument, "Y10.0" => (ArgumentKind::Y, 10.0));
    parser_test!(z_argument, argument, "Z3.14" => (ArgumentKind::Z, 3.14));
    parser_test!(s_argument, argument, "S10.0" => (ArgumentKind::S, 10.0));
    parser_test!(i_argument, argument, "I10" => (ArgumentKind::I, 10.0));
    parser_test!(j_argument, argument, "J10.0" => (ArgumentKind::J, 10.0));


    parser_test!(basic_command, next_command, "N15 G10 X-2.0" => Command {
        kind: CommandKind::G,
        number: Number::Integer(10),
        args: Args {
            x: Some(-2.0),
            ..Default::default()
        },
        line_number: Some(15),
    });

    #[allow(trivial_casts)]
    mod qc {
        use super::*;
        use std::prelude::v1::*;

        macro_rules! quick_parser_quickcheck {
            ($method:ident) => (
                quickcheck!{
                    fn $method(tokens: Vec<Token>) -> () {
                    let mut parser = Parser::new(tokens.into_iter());
                    let _ = parser.$method();
                    }
                }
            )
        }

        quick_parser_quickcheck!(command_type);
        quick_parser_quickcheck!(line_number);
        quick_parser_quickcheck!(args);
        quick_parser_quickcheck!(argument);
        quick_parser_quickcheck!(next_command);
    }
}
