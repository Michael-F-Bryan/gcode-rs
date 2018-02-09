
#[macro_export]
macro_rules! gcode {
    (N $n:expr $(, $($rest:tt)* )*) => {
        $crate::command::Command {
            line: Some($n),
            ..gcode!($( $($rest)* ),*)
        }
    };
    (G $n:expr $(, $($rest:tt)* )*) => {{
        $crate::command::Command {
            kind: $crate::command::Kind::G,
            number: $n,
            ..gcode!($( $($rest)* ),*)
        }
    }};
    (X $value:expr $(, $($rest:tt)* )*) => {{
        let mut original = gcode!($( $($rest)* ),*);
        original.args.x = $value;
        original
    }};
    () => {
        $crate::command::Command::default()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use command::{Command, Kind};

    #[test]
    fn g_90() {
        let got = gcode!(G 90);

        assert_eq!(got.number, 90);
        assert_eq!(got.kind, Kind::G);
    }

    #[test]
    fn line_number() {
        let should_be = Some(50);

        let got = gcode!(N 50);
        assert_eq!(got.line, should_be);
    }

    #[test]
    fn with_argument() {
        let got = gcode!(G 90, X 3.14);

        assert_eq!(got.args.x, 3.14);
    }
}
