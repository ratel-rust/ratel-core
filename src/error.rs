#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Error {
    UnexpectedEndOfProgram,
    UnexpectedToken {
        start: usize,
        end: usize,
    },
}

pub enum ParseError {
    UnexpectedEndOfProgram,
    UnexpectedToken {
        source: String,
        start: usize,
        end: usize,
    },
}

pub type Result<T> = ::std::result::Result<T, Error>;

pub type ParseResult<T> = ::std::result::Result<T, ParseError>;
