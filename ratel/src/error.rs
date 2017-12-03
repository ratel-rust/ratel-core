use std::fmt::{self, Debug, Display};
use lexer::Token;

/// Error type used by the tokenizer and the parser internally.
#[derive(PartialEq, Clone)]
pub struct Error {
    pub token: Token,
    pub raw: Box<str>,
    pub start: usize,
    pub end: usize,
}

impl Debug for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unexpected {:?}({}) at {}:{}", &self.token, &*self.raw, self.start, self.end)
    }
}

/// Error type returned by `parser::parse`. This error will include
/// owned `String` of the source code where the error occurred, so
/// that a meaningful error can be printed out.
pub enum ParseError {
    UnexpectedEndOfProgram,
    UnexpectedToken {
        source: String,
        start: usize,
        end: usize,
    },
}

impl Debug for ParseError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Display for ParseError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::UnexpectedEndOfProgram => {
                try!(write!(f, "Unexpected end of program"))
            },

            ParseError::UnexpectedToken {
                ref source,
                start,
                end
            } => {
                let (lineno, line) = source[..start]
                                       .lines()
                                       .enumerate()
                                       .last()
                                       .unwrap_or((0, ""));

                let colno = line.chars().count();
                let token_len = source[start..end].chars().count();

                try!(writeln!(f, "Unexpected token at {}:{}\n", lineno + 1, colno + 1));

                let iter = source
                            .lines()
                            .enumerate()
                            .skip_while(|&(index, _)| index < lineno.saturating_sub(2))
                            .take_while(|&(index, _)| index < lineno + 3);

                let width = log10(lineno + 3);

                for (index, line) in iter {
                    if index == lineno {
                        try!(writeln!(f, "> {0:1$} | {2}", index+1, width, line));

                        for _ in 0..width {
                            try!(write!(f, " "));
                        }

                        try!(write!(f, "   | "));

                        for _ in 0..colno {
                            try!(write!(f, " "));
                        }

                        for _ in 0..token_len {
                            try!(write!(f, "^"));
                        }

                        try!(write!(f, "\n"));
                    } else {
                        try!(writeln!(f, "{0:1$} | {2}", index+1, width+2, line));
                    }
                }

            },
        }

        Ok(())
    }
}

fn log10(mut num: usize) -> usize {
    let mut log = 0;

    while num > 0 {
        log += 1;
        num /= 10;
    }

    log
}

pub type Result<T> = ::std::result::Result<T, Error>;

pub type ParseResult<T> = ::std::result::Result<T, ParseError>;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_format_unexpected_token_error () {
        let err = ParseError::UnexpectedToken {
            source: "foo".to_string(),
            start: 0,
            end: 1
        };

        let expected = "Unexpected token at 1:1\n\n> 1 | foo\n    | ^\n";

        assert_eq!(format!("{}", err), expected);
    }

}
