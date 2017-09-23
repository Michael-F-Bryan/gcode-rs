#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Command {
    pub kind: Kind,
    pub number: u32,
    pub secondary_number: Option<u32>,
    pub line: Option<u32>,
    pub args: Arguments,
}

impl Default for Command {
    fn default() -> Self {
        Command {
            kind: Kind::G,
            number: 90,
            secondary_number: None,
            args: Arguments::default(),
            line: None,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    G,
    M,
    T,
}


#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Arguments {
    pub x: f32,
}

impl Arguments {
    pub fn set(&mut self, kind: ArgumentKind, value: f32) {
        match kind {
            ArgumentKind::X => self.x = value,
            _ => unimplemented!(),
        }
    }

    pub fn set_x(mut self, x: f32) -> Self {
        self.x = x;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArgumentKind {
    X,
    Y,
}
