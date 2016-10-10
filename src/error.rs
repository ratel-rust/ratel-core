#[derive(Copy, Clone, Debug)]
pub enum Error {
    UnexpectedEndOfProgram,
    UnexpectedToken {
        start: usize,
        end: usize,
    },
}

pub enum ParsingError {
    UnexpectedEndOfProgram,
    UnexpectedToken {
        start: usize,
        end: usize,
    },
}

pub type Result<T> = ::std::result::Result<T, Error>;
