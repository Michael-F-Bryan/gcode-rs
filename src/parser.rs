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
/// command ::= line_number command_name args
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

impl<I> Parser<I>
    where I: Iterator<Item = Token>
{
    pub fn new(stream: I) -> Parser<I> {
        Parser { stream: stream.peekable() }
    }

    fn command_name(&mut self) -> Result<CommandType> {
        match self.peek() {
            Some(TokenKind::G) => Ok(CommandType::G),
            Some(TokenKind::M) => Ok(CommandType::M),
            Some(_) => {
                // We need to do a proper peek at the next token
                // in order to construct a proper error message
                let next = self.stream.peek().unwrap();
                Err(Error::SyntaxError("Expected a command type", next.span()))
            }
            None => Err(Error::UnexpectedEOF),
        }
    }

    fn line_number(&mut self) -> Result<Option<u32>> {
        if self.peek() != Some(TokenKind::N) {
            return Ok(None);
        }

        let _ = self.stream.next();

        match self.peek() {
            Some(TokenKind::Integer(_)) => {}
            Some(_) => {
                let next = self.stream.peek().unwrap();
                return Err(Error::SyntaxError("Expected a line number", next.span()));
            }
            None => return Err(Error::UnexpectedEOF),
        }

        match self.stream.next().map(|t| t.kind()) {
            Some(TokenKind::Integer(n)) => Ok(Some(n)),
            _ => unreachable!(),
        }
    }

    fn arg_kind(&mut self) -> Result<ArgumentKind> {
        match self.peek() {
            Some(TokenKind::X) |
            Some(TokenKind::Y) |
            Some(TokenKind::Z) |
            Some(TokenKind::R) |
            Some(TokenKind::FeedRate) => {}
            Some(_) => {
                let next = self.stream.next().unwrap();
                return Err(Error::SyntaxError("Expected an argument kind", next.span()));
            }
            None => return Err(Error::UnexpectedEOF),
        }

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
            match self.peek() {
                Some(TokenKind::Number(_)) => {}
                Some(_) => {
                    let next = self.stream.peek().unwrap();
                    return Err(Error::SyntaxError("A command argument must always have an argument value",
                                                  next.span()));
                }
                None => return Err(Error::UnexpectedEOF),
            }

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
        unimplemented!()
    }

    fn peek(&mut self) -> Option<TokenKind> {
        self.stream.peek().map(|t| t.kind())
    }
}

#[derive(Debug, PartialEq)]
pub struct Command {
    span: Span,
    line_number: Option<usize>,
    command_type: CommandType,
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
}
