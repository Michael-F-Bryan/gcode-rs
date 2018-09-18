use core::iter::Peekable;
use lexer::Lexer;
use types::{Gcode, Mnemonic, Word};

/// Parse a string of text into a stream of `Gcode`s.
pub fn parse(src: &str) -> Parser {
    Parser::new(src)
}

/// A gcode parser which is extremely permissive in what input it will accept.
#[derive(Debug, Clone)]
pub struct Parser<'input> {
    lexer: Peekable<Lexer<'input>>,
}

impl<'input> Parser<'input> {
    pub(crate) fn new(src: &'input str) -> Parser<'input> {
        Parser {
            lexer: Lexer::new(src).peekable(),
        }
    }

    #[cfg(test)]
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
        let mut last_n = None;

        while let Some(next_letter) = self.peek().map(|w| w.letter) {
            let got = match next_letter {
                'O' => parse_o(&mut self.lexer),
                'T' => parse_t(&mut self.lexer),
                'M' => parse_machine(&mut self.lexer),
                'G' => parse_g(&mut self.lexer),
                'N' => {
                    last_n = self.lexer.next();
                    continue;
                }
                c => unimplemented!("Unknown code, {:?}", c),
            };

            match last_n {
                Some(n) => {
                    return Some(
                        got.with_line_number(
                            n.value.abs().trunc() as u32,
                            n.span,
                        ),
                    )
                }
                None => return Some(got),
            }
        }

        None
    }
}

fn parse_o<I>(iter: &mut Peekable<I>) -> Gcode
where
    I: Iterator<Item = Word>,
{
    do_parse(iter, Mnemonic::ProgramNumber, false)
}

fn parse_t<I>(iter: &mut Peekable<I>) -> Gcode
where
    I: Iterator<Item = Word>,
{
    do_parse(iter, Mnemonic::ToolChange, false)
}

fn parse_machine<I>(iter: &mut Peekable<I>) -> Gcode
where
    I: Iterator<Item = Word>,
{
    do_parse(iter, Mnemonic::MachineRoutine, true)
}

fn parse_g<I>(iter: &mut Peekable<I>) -> Gcode
where
    I: Iterator<Item = Word>,
{
    do_parse(iter, Mnemonic::General, true)
}

fn do_parse<I>(
    iter: &mut Peekable<I>,
    mnemonic: Mnemonic,
    takes_args: bool,
) -> Gcode
where
    I: Iterator<Item = Word>,
{
    let word = iter.next().expect("Already checked");

    let mut code = Gcode::new(mnemonic, word.value, word.span);

    if takes_args {
        parse_args(iter, &mut code);
    }

    code
}

fn parse_args<I>(iter: &mut Peekable<I>, code: &mut Gcode)
where
    I: Iterator<Item = Word>,
{
    let current_line = code.span().source_line;

    while iter
        .peek()
        .map(|w| w.span.source_line == current_line && is_arg(w.letter))
        .unwrap_or(false)
    {
        let arg = iter.next().expect("Already checked");
        code.add_argument(arg);
    }
}

fn is_arg(c: char) -> bool {
    // gcodes are kinda all over the place, so we just assume all letters
    // except mnemonics are argument material
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

    parse_test!(parse_a_program_number, "O123" => Gcode::new(Mnemonic::ProgramNumber, 123.0, Span::new(0, 4, 0)));
    parse_test!(parse_a_tool_change, "T6" => Gcode::new(Mnemonic::ToolChange, 6.0, Span::new(0, 2, 0)));
    parse_test!(parse_a_machine_code, "M30" => Gcode::new(Mnemonic::MachineRoutine, 30.0, Span::new(0, 3, 0)));

    parse_test!(parse_a_gcode_with_arguments, "G01 X100 Y50.0" => 
                Gcode::new(Mnemonic::General, 1.0, Span::new(0, 14, 0))
                    .with_argument(Word::new('X', 100.0, Span::new(4, 8, 0)))
                    .with_argument(Word::new('Y', 50.0, Span::new(9, 14, 0))));
}
