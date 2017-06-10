//! The main parsing module for this crate.

use core::iter::Peekable;

use lexer::{Token, TokenKind};
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


/// A parser which turns a stream of gcode tokens into a stream of commands.
#[derive(Debug)]
pub struct Parser<I>
    where I: Iterator<Item = Token>
{
    tokens: Peekable<I>,
}

impl<I> Parser<I>
    where I: Iterator<Item = Token>
{
    /// Create a new parser using the provided stream of tokens.
    pub fn new(tokens: I) -> Parser<I> {
        Parser { tokens: tokens.peekable() }
    }

    fn next_command(&mut self) -> Result<Line> {
        if let Ok(number) = self.program_number() {
            return Ok(Line::ProgramNumber(number));
        };

        let line_number = self.line_number()?;
        let (kind, number) = self.command_type()?;
        let args = self.args()?;

        let cmd = Command {
            kind,
            number,
            args,
            line_number,
        };
        Ok(Line::Cmd(cmd))
    }

    fn program_number(&mut self) -> Result<u32> {
        lookahead!(self, "Expected an \"O\"", TokenKind::O);
        let _ = self.tokens.next();

        lookahead!(self, "Expected a program number", TokenKind::Number(_));

        match self.unchecked_next() {
            TokenKind::Number(n) => Ok(n as u32),
            _ => unreachable!(),
        }
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
                    // FIXME: This is wrong, read above TODO
                    Number::Decimal(n as u32, 0)
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
        TokenKind::FeedRate | TokenKind::I | TokenKind::J | TokenKind::H |
        TokenKind::P | TokenKind::E);

        let kind = match self.unchecked_next() {
            TokenKind::X => ArgumentKind::X,
            TokenKind::Y => ArgumentKind::Y,
            TokenKind::Z => ArgumentKind::Z,
            TokenKind::S => ArgumentKind::S,
            TokenKind::I => ArgumentKind::I,
            TokenKind::J => ArgumentKind::J,
            TokenKind::H => ArgumentKind::H,
            TokenKind::P => ArgumentKind::P,
            TokenKind::FeedRate => ArgumentKind::F,
            TokenKind::E => ArgumentKind::E,
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
                    _ => {
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
        self.tokens.peek().map(|t| t.kind())
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
    type Item = Result<Line>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.next_command() {
            Err(Error::UnexpectedEOF) => None,
            other => Some(other),
        }
    }
}

/// A single line of gcode.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Line {
    /// A program number.
    ProgramNumber(u32),
    /// An actual command.
    Cmd(Command),
}

/// A type which can either be an integer or a float.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Number {
    /// A plain integer.
    Integer(u32),
    /// A floating point number, represented as the integer before and after
    /// the decimal point.
    Decimal(u32, u32),
}

/// A single command.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Command {
    /// Which kind of `Command` is this?
    pub kind: CommandKind,
    /// The command's number.
    pub number: Number,
    /// All arguments passed to the command.
    pub args: Args,
    /// The line number the command is on (if any).
    pub line_number: Option<u32>,
}

/// A *good ol' bag-o-floats* which contains all the possible arguments and their values.
#[derive(Debug, Copy, Clone, Default, PartialEq)]
#[allow(missing_docs)]
pub struct Args {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
    pub s: Option<f32>,
    pub t: Option<f32>,
    pub f: Option<f32>,
    pub i: Option<f32>,
    pub j: Option<f32>,
    pub h: Option<f32>,
    pub p: Option<f32>,
    pub e: Option<f32>,
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
            ArgumentKind::H => self.h = Some(value),
            ArgumentKind::P => self.p = Some(value),
            ArgumentKind::E => self.e = Some(value),
        }
    }
}

/// The type of argument provided.
#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(missing_docs)]
enum ArgumentKind {
    X,
    Y,
    Z,
    F,
    S,
    I,
    J,
    H,
    P,
    E,
}

/// The type of command.
#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(missing_docs)]
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
    parser_test!(e_argument, argument, "E10.0" => (ArgumentKind::E, 10.0));

    parser_test!(program_number, program_number, "O500" => 500);


    parser_test!(basic_command, next_command, "N15 G10 X-2.0" => Line::Cmd(Command {
        kind: CommandKind::G,
        number: Number::Integer(10),
        args: Args {
            x: Some(-2.0),
            ..Default::default()
        },
        line_number: Some(15),
    }));

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
        quick_parser_quickcheck!(program_number);
        quick_parser_quickcheck!(line_number);
        quick_parser_quickcheck!(args);
        quick_parser_quickcheck!(argument);
        quick_parser_quickcheck!(next_command);
    }
}
