use std::fmt;

/// Error type used by the tokenizer and the parser internally.
#[derive(Debug)]
pub enum Error {
    UnexpectedEndOfProgram,
    UnexpectedToken {
        start: usize,
        end: usize,
    },
}

/// Error type returned by `parser::parse`. This error will include
/// owned `String` of the source code where the error occured, so
/// that a meaningful error can be printed out.
pub enum ParseError {
    UnexpectedEndOfProgram,
    UnexpectedToken {
        source: String,
        start: usize,
        end: usize,
    },
}

impl fmt::Debug for ParseError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for ParseError {
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
                                       .expect("Must always have at least one line.");

                let colno = line.chars().count();
                let token_len = source[start..end].chars().count();

                try!(writeln!(f, "Unexpected token at {}:{}\n", lineno + 1, colno + 1));

                let iter = source
                            .lines()
                            .enumerate()
                            .skip_while(|&(index, _)| index < lineno.saturating_sub(2))
                            .take_while(|&(index, _)| index < lineno + 3);

                for (index, line) in iter {
                    if index == lineno {
                        try!(writeln!(f, "> {:4} | {}", index+1, line));

                        try!(write!(f, "       | "));

                        for _ in 0..colno {
                            try!(write!(f, " "));
                        }

                        for _ in 0..token_len {
                            try!(write!(f, "^"));
                        }

                        try!(write!(f, "\n"));
                    } else {
                        try!(writeln!(f, "{:6} | {}", index+1, line));
                    }
                }

            },
        }

        Ok(())
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;

pub type ParseResult<T> = ::std::result::Result<T, ParseError>;
