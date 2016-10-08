#[derive(Copy, Clone, Debug)]
pub struct Error {
    pub line: usize,
    pub column: usize,
}

pub type Result<T> = ::std::result::Result<T, Error>;
