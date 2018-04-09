use parse::{Block, ParseError};

/// Parse the entire source string and check whether there are any errors.
pub fn validate(src: &str) -> Result<(), ParseError> {
    for line in src.lines() {
        let b = Block::new(line);

        for maybe_word in b.words() {
            if let Err(e) = maybe_word {
                return Err(e);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::prelude::v1::*;

    #[test]
    fn validate_a_bunch_of_gcode_expressions() {
        let inputs = vec!["", "\n", "( just a comment)", "G10", "G01 X500.0 Y90Z-52"];

        for src in inputs {
            validate(src).map_err(|e| (e, src)).unwrap();
        }
    }

    #[test]
    fn check_a_bunch_of_invalid_gcodes() {
        let inputs = vec!["$", "( half a comment\n", "GG30", "G10 X 5.3"];

        for src in inputs {
            validate(src).unwrap_err();
        }
    }

    quickcheck! {
        fn validation_doesnt_panic(text: String) -> bool {
            let _ = validate(&text);
            true
        }
    }
}
