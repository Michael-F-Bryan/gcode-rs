pub struct GCode {
    number: Number,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Number {
    Integer(u16),
    Decimal(u16, u16),
}

impl Number {
    pub fn major(&self) -> u16 {
        match *self {
            Number::Integer(m) | Number::Decimal(m, _) => m,
        }
    }

    pub fn minor(&self) -> Option<u16> {
        match *self {
            Number::Integer(_) => None,
            Number::Decimal(_, min) => Some(min),
        }
    }
}
