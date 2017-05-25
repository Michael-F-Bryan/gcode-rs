#![allow(missing_docs, dead_code)]

use core::iter::Peekable;
use arrayvec::ArrayVec;

use lexer::{Token, Span, TokenKind};
use errors::*;


type ArgBuffer = ArrayVec<[Argument; 10]>;


/// A parser which takes a stream of characters and parses them as gcode
/// instructions.
///
/// The grammar currently being used is roughly as follows:
///
/// ```text
/// start ::= command
///
/// command ::= line_number command_name args
///
/// command_name ::= command_type INTEGER
///
/// command_type ::= G
///                | M
///
/// args ::= args arg
///
/// arg ::= arg_kind number
///       | <epsilon>
///
/// arg_kind ::= X
///            | Y
///            | Z
///            | F
///            | R
///
/// line_number ::= N INTEGER
///               | <epsilon>
/// ```
///
/// I've tried to keep the grammar
pub struct Parser<I>
    where I: Iterator<Item = Token>
{
    stream: Peekable<I>,
}

/// Peek at the next token, if its kind isn't one of the specified `$pattern`s,
/// return a `Error::SyntaxError` with the provided message.
macro_rules! lookahead {
    ($self:expr, $err_msg:expr, $( $pattern:pat )|*) => {
        match $self.peek() {
            $( Some($pattern) )|* => {},
            Some(_) => {
                let next = $self.stream.next().unwrap();
                return Err(Error::SyntaxError($err_msg, next.span()));
            }
            None => return Err(Error::UnexpectedEOF),
        }
    }
}

impl<I> Parser<I>
    where I: Iterator<Item = Token>
{
    pub fn new(stream: I) -> Parser<I> {
        Parser { stream: stream.peekable() }
    }

    pub fn parse(&mut self) -> Result<Command> {
        self.command()
    }

    fn command(&mut self) -> Result<Command> {
        let span = Span::default(); // TODO: actually get from next token

        let line_number = self.line_number()?;
        let (command_type, command_number) = self.command_name()?;
        let args = self.args()?;

        let cmd = Command {
            span,
            line_number,
            command_type,
            args,
            command_number,
        };
        Ok(cmd)
    }

    fn command_name(&mut self) -> Result<(CommandType, u32)> {
        let ty = self.command_type()?;

        lookahead!(self, "Commands should be followed by an integer", TokenKind::Integer(_));

        if let TokenKind::Integer(n) = self.stream.next().unwrap().kind() {
            Ok((ty, n))
        } else {
            unreachable!()
        }
    }

    fn command_type(&mut self) -> Result<CommandType> {
        lookahead!(self, "Expected a command type", TokenKind::G | TokenKind::M);

        match self.stream.next().unwrap().kind() {
            TokenKind::G => Ok(CommandType::G),
            TokenKind::M => Ok(CommandType::M),
            _ => unreachable!(),
        }
    }

    fn line_number(&mut self) -> Result<Option<u32>> {
        if self.peek() != Some(TokenKind::N) {
            return Ok(None);
        }

        let _ = self.stream.next();

        lookahead!(self, "Expected a line number", TokenKind::Integer(_));

        match self.stream.next().map(|t| t.kind()) {
            Some(TokenKind::Integer(n)) => Ok(Some(n)),
            _ => unreachable!(),
        }
    }

    fn arg_kind(&mut self) -> Result<ArgumentKind> {
        lookahead!(self,
                   "Expected an argument kind",
                   TokenKind::X | TokenKind::Y | TokenKind::Z | TokenKind::R | TokenKind::FeedRate);

        match self.stream.next().unwrap().kind() {
            TokenKind::X => Ok(ArgumentKind::X),
            TokenKind::Y => Ok(ArgumentKind::Y),
            TokenKind::Z => Ok(ArgumentKind::Z),
            TokenKind::R => Ok(ArgumentKind::R),
            TokenKind::FeedRate => Ok(ArgumentKind::FeedRate),
            _ => unreachable!(),
        }
    }

    fn arg(&mut self) -> Result<Option<Argument>> {
        if let Ok(kind) = self.arg_kind() {
            // look ahead and check we have a number
            lookahead!(self, "A command argument must always have an argument value",
                       TokenKind::Number(_));

            let next = self.stream.next().unwrap();

            let n = match next.kind() {
                TokenKind::Number(n) => n,
                _ => unreachable!(),
            };

            Ok(Some(Argument {
                        kind: kind,
                        value: n,
                    }))

        } else {
            Ok(None)
        }
    }

    fn args(&mut self) -> Result<ArgBuffer> {
        let mut buffer = ArgBuffer::new();

        while let Ok(Some(arg)) = self.arg() {
            buffer.push(arg);
        }

        Ok(buffer)
    }

    fn peek(&mut self) -> Option<TokenKind> {
        self.stream.peek().map(|t| t.kind())
    }
}

impl<I> Iterator for Parser<I>
    where I: Iterator<Item = Token>
{
    type Item = Result<Command>;

    fn next(&mut self) -> Option<Self::Item> {
        let got = self.parse();

        if got == Err(Error::UnexpectedEOF) {
            None
        } else {
            Some(got)
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Command {
    span: Span,
    line_number: Option<u32>,
    command_type: CommandType,
    command_number: u32,
    args: ArgBuffer,
}


#[derive(Clone, Debug, PartialEq)]
struct Argument {
    kind: ArgumentKind,
    value: f32,
}

#[derive(Clone, Debug, PartialEq)]
enum ArgumentKind {
    X,
    Y,
    Z,

    R,
    FeedRate,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum CommandType {
    G,
    M,
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexer::TokenKind;

    #[test]
    fn parse_no_line_number() {
        let src = vec![];
        let should_be = None;

        let mut parser = Parser::new(src.into_iter());

        let got = parser.line_number().unwrap();

        assert_eq!(got, should_be);
    }

    #[test]
    fn parse_line_number() {
        let src = [TokenKind::N, TokenKind::Integer(10)];
        let should_be = Some(10);

        let tokens = src.iter().map(|&t| t.into());
        let mut parser = Parser::new(tokens);

        let got = parser.line_number().unwrap();

        assert_eq!(got, should_be);
    }

    #[test]
    fn parse_argument_kind() {
        let src = vec![([TokenKind::X], ArgumentKind::X),
                       ([TokenKind::Y], ArgumentKind::Y),
                       ([TokenKind::Z], ArgumentKind::Z),
                       ([TokenKind::R], ArgumentKind::R),
                       ([TokenKind::FeedRate], ArgumentKind::FeedRate)];

        for (tokens, should_be) in src {
            println!("{:?} => {:?}", tokens, should_be);

            let mut parser = Parser::new(tokens.iter().map(|&k| k.into()));
            let got = parser.arg_kind().unwrap();
            assert_eq!(got, should_be);
        }
    }

    #[test]
    fn parse_empty_arg() {
        let src = vec![];
        let mut parser = Parser::new(src.into_iter());
        let got = parser.arg().unwrap();
        assert!(got.is_none());
    }

    #[test]
    fn parse_x_arg() {
        let src = vec![TokenKind::X, TokenKind::Number(3.14)];
        let should_be = Argument {
            kind: ArgumentKind::X,
            value: 3.14,
        };

        let tokens = src.iter().map(|&k| k.into());
        let mut parser = Parser::new(tokens);

        let got = parser.arg().unwrap().unwrap();
        assert_eq!(got, should_be);
    }

    #[test]
    fn parse_empty_args() {
        let src = vec![];
        let mut parser = Parser::new(src.into_iter());
        let got = parser.args().unwrap();
        assert!(got.is_empty());
    }

    #[test]
    fn parse_single_args() {
        let src = vec![TokenKind::X, TokenKind::Number(3.14)];
        let should_be = Argument {
            kind: ArgumentKind::X,
            value: 3.14,
        };

        let tokens = src.iter().map(|&k| k.into());
        let mut parser = Parser::new(tokens);

        let got = parser.args().unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0], should_be);
    }

    #[test]
    fn parse_multiple_args() {
        let src = vec![TokenKind::X,
                       TokenKind::Number(3.14),
                       TokenKind::Y,
                       TokenKind::Number(2.1828)];
        let mut should_be = ArgBuffer::new();
        should_be.push(Argument {
                           kind: ArgumentKind::X,
                           value: 3.14,
                       });
        should_be.push(Argument {
                           kind: ArgumentKind::Y,
                           value: 2.1828,
                       });

        let tokens = src.iter().map(|&k| k.into());
        let mut parser = Parser::new(tokens);

        let got = parser.args().unwrap();
        assert_eq!(got, should_be);
    }

    #[test]
    fn parse_basic_command() {
        let src = vec![TokenKind::G, TokenKind::Integer(90)]; // G90
        let should_be = Command {
            span: (0, 0).into(),
            command_type: CommandType::G,
            command_number: 90,
            args: ArgBuffer::new(),
            line_number: None,
        };

        let tokens = src.iter().map(|&t| t.into());
        let mut parser = Parser::new(tokens);

        let got = parser.command().unwrap();

        assert_eq!(got, should_be);
    }

    #[test]
    fn parse_normal_g01() {
        let src = vec![TokenKind::N,
                       TokenKind::Integer(10),
                       TokenKind::G,
                       TokenKind::Integer(91),
                       TokenKind::X,
                       TokenKind::Number(1.0),
                       TokenKind::Y,
                       TokenKind::Number(3.1415),
                       TokenKind::Z,
                       TokenKind::Number(-20.0)];
        let mut should_be = Command {
            span: (0, 0).into(),
            command_type: CommandType::G,
            command_number: 91,
            args: ArgBuffer::new(),
            line_number: Some(10),
        };

        should_be
            .args
            .push(Argument {
                      kind: ArgumentKind::X,
                      value: 1.0,
                  });
        should_be
            .args
            .push(Argument {
                      kind: ArgumentKind::Y,
                      value: 3.1415,
                  });
        should_be
            .args
            .push(Argument {
                      kind: ArgumentKind::Z,
                      value: -20.0,
                  });

        let tokens = src.iter().map(|&t| t.into());
        let mut parser = Parser::new(tokens);

        let got = parser.command().unwrap();

        assert_eq!(got, should_be);
    }

    #[test]
    fn parse_command_and_name() {
        let src = [TokenKind::G, TokenKind::Integer(0)];
        let should_be = (CommandType::G, 0);

        let tokens = src.iter().map(|&t| t.into());
        let mut parser = Parser::new(tokens);

        let got = parser.command_name().unwrap();

        assert_eq!(got, should_be);
    }
}
