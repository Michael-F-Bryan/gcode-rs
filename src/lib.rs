#![no_std]

mod parser;
pub use parser::Parser;
pub use errors::*;

mod errors {
    pub type Result<T> = ::core::result::Result<T, Error>;

    #[derive(Debug, Clone, PartialEq)]
    pub enum Error {
        Expected(char),
        UnexpectedEOF,
    }
}
