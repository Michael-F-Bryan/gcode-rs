use core::iter::Peekable;
use lexer::Lexer;
use types::{Gcode, Mnemonic, Word};

/// Parse a string of text into a stream of `Gcode`s.
pub fn parse<'input>(src: &'input str) -> impl Iterator<Item = Gcode> + 'input {
    Parser::new(src)
}

/// A gcode parser which is extremely permissive in what input it will accept.
#[derive(Debug, Clone)]
struct Parser<'input> {
    lexer: Peekable<Lexer<'input>>,
}

impl<'input> Parser<'input> {
    pub fn new(src: &'input str) -> Parser<'input> {
        Parser {
            lexer: Lexer::new(src).peekable(),
        }
    }

    fn is_finished(&mut self) -> bool {
        self.peek().is_none()
    }

    fn peek(&mut self) -> Option<&Word> {
        self.lexer.peek()
    }
}

impl<'input> Iterator for Parser<'input> {
    type Item = Gcode;

    fn next(&mut self) -> Option<Self::Item> {
        match self.peek()?.letter {
            'O' => Some(parse_o(&mut self.lexer)),
            'T' => Some(parse_t(&mut self.lexer)),
            'M' => Some(parse_machine(&mut self.lexer)),
            'G' => Some(parse_g(&mut self.lexer)),
            _ => unimplemented!(),
        }
    }
}

fn parse_o<I>(iter: &mut Peekable<I>) -> Gcode 
    where I: Iterator<Item = Word>
{
    let word = iter.next().expect("Already checked");

    Gcode::new(Mnemonic::ProgramNumber, word.number, word.span)
}

fn parse_t<I>(iter: &mut Peekable<I>) -> Gcode 
    where I: Iterator<Item = Word>
{
    let word = iter.next().expect("Already checked");

    Gcode::new(Mnemonic::ToolChange, word.number, word.span)
}

fn parse_machine<I>(iter: &mut Peekable<I>) -> Gcode 
    where I: Iterator<Item = Word>
{
    let word = iter.next().expect("Already checked");

    Gcode::new(Mnemonic::MachineRoutine, word.number, word.span)
}

fn parse_g<I>(iter: &mut Peekable<I>) -> Gcode 
    where I: Iterator<Item = Word>
{
    let word = iter.next().expect("Already checked");
    let mut code = Gcode::new(Mnemonic::MachineRoutine, word.number, word.span);

    parse_args(iter, &mut code);

    code
}

fn parse_args<I>(iter: &mut Peekable<I>, code: &mut Gcode)
    where I: Iterator<Item = Word>
{
    let current_line = code.span.source_line;

    while iter.peek().map(|w| w.span.source_line == current_line && is_arg(w.letter)).unwrap_or(false) {
        let arg = iter.next().expect("Already checked");
        code.add_argument(arg);
    }
}

fn is_arg(c: char) -> bool {
    // we just assume all letters except mnemonics are argument material
    match c.to_ascii_uppercase() {
        'O' | 'M' | 'T' | 'G' => false,
        other if other.is_ascii_alphabetic() => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::Span;

    macro_rules! parse_test {
        ($name:ident, $src:expr => $should_be:expr) => {
            #[test]
            fn $name() {
                let src = $src;
                let should_be = $should_be;

                let mut parser = Parser::new(src);

                assert!(!parser.is_finished());
                let got = parser.next().unwrap();
                assert!(parser.is_finished());

                assert_eq!(got, should_be);
            }
        };
    }

    parse_test!(parse_a_program_number, "O123" => Gcode {
            mnemonic: Mnemonic::ProgramNumber,
            number: 123.0,
            span: Span::new(0, 4, 0),
            ..Default::default()
        });

    parse_test!(parse_a_tool_change, "T6" => Gcode {
            mnemonic: Mnemonic::ToolChange,
            number: 6.0,
            span: Span::new(0, 2, 0),
            ..Default::default()
        });

    parse_test!(parse_a_machine_code, "M30" => Gcode {
            mnemonic: Mnemonic::MachineRoutine,
            number: 30.0,
            span: Span::new(0, 3, 0),
            ..Default::default()
        });

    parse_test!(parse_a_gcode_with_arguments, "G01 X100 Y50.0" => Gcode {
            mnemonic: Mnemonic::MachineRoutine,
            number: 1.0,
            span: Span::new(0, 14, 0),
            arguments: vec![
                Word { 
                    letter: 'X', 
                    number: 100.0, 
                span: Span::new(4, 8, 0),
                },
                Word { 
                    letter: 'Y', 
                    number: 50.0, 
                    span: Span::new(9, 14, 0) 
                },
            ].into_iter()
                .collect()
        });
}
