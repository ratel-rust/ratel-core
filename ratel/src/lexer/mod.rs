mod token;
mod labels;
mod util;

pub use lexer::token::*;

use lexer::labels::*;
use lexer::token::Token::*;

use std::str;
use error::Error;
use toolshed::Arena;

macro_rules! expect_byte {
    ($lex:ident) => ({
        match $lex.read_byte() {
            0 => return $lex.token = UnexpectedEndOfProgram,
            _ => $lex.bump()
        }
    });
}

macro_rules! unwind_loop {
    ($iteration:expr) => ({
        $iteration
        $iteration
        $iteration
        $iteration
        $iteration

        loop {
            $iteration
            $iteration
            $iteration
            $iteration
            $iteration
        }
    })
}

/// Contextual check describing which Automatic Semicolon Insertion rules can be applied.
#[derive(Clone, Copy, PartialEq)]
pub enum Asi {
    /// Current token is a semicolon. Parser should consume it and finalize the statement.
    ExplicitSemicolon,

    /// Current token is not a semicolon, but previous token is either followed by a
    /// line termination, or allows semicolon insertion itself. Parser should finalize the
    /// statement without consuming the current token.
    ImplicitSemicolon,

    /// Current token is not a semicolon, and no semicolon insertion rules were triggered.
    /// Parser should continue parsing the statement or error.
    NoSemicolon,
}

type ByteHandler = Option<for<'arena> fn(&mut Lexer<'arena>)>;

/// Lookup table mapping any incoming byte to a handler function defined below.
static BYTE_HANDLERS: [ByteHandler; 256] = [
//   0    1    2    3    4    5    6    7    8    9    A    B    C    D    E    F   //
    EOF, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, // 0
    ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, // 1
    ___, EXL, QOT, ERR, IDT, PRC, AMP, QOT, PNO, PNC, ATR, PLS, COM, MIN, PRD, SLH, // 2
    ZER, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, COL, SEM, LSS, EQL, MOR, QST, // 3
    ERR, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, // 4
    IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, BTO, IDT, BTC, CRT, IDT, // 5
    TPL, IDT, L_B, L_C, L_D, L_E, L_F, IDT, IDT, L_I, IDT, IDT, L_L, IDT, L_N, IDT, // 6
    L_P, IDT, L_R, L_S, L_T, L_U, L_V, L_W, IDT, L_Y, IDT, BEO, PIP, BEC, TLD, ERR, // 7
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // 8
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // 9
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // A
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // B
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // C
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // D
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // E
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // F
];

const ___: ByteHandler = None;

const ERR: ByteHandler = Some(|lex| {
    lex.bump();

    lex.token = UnexpectedToken;
});

const EOF: ByteHandler = Some(|lex| {
    lex.asi = Asi::ImplicitSemicolon;

    lex.token = EndOfProgram;
});

// ;
const SEM: ByteHandler = Some(|lex| {
    lex.bump();

    lex.asi = Asi::ExplicitSemicolon;

    lex.token = Semicolon;
});

// :
const COL: ByteHandler = Some(|lex| {
    lex.bump();

    lex.token = Colon;
});

// ,
const COM: ByteHandler = Some(|lex| {
    lex.bump();

    lex.token = Comma;
});

// (
const PNO: ByteHandler = Some(|lex| {
    lex.bump();

    lex.token = ParenOpen;
});

// )
const PNC: ByteHandler = Some(|lex| {
    lex.bump();

    lex.asi = Asi::ImplicitSemicolon;

    lex.token = ParenClose;
});

// [
const BTO: ByteHandler = Some(|lex| {
    lex.bump();

    lex.token = BracketOpen;
});

// ]
const BTC: ByteHandler = Some(|lex| {
    lex.bump();

    lex.token = BracketClose;
});

// {
const BEO: ByteHandler = Some(|lex| {
    lex.bump();

    lex.token = BraceOpen;
});

// }
const BEC: ByteHandler = Some(|lex| {
    lex.bump();

    lex.asi = Asi::ImplicitSemicolon;

    lex.token = BraceClose;
});

// =
const EQL: ByteHandler = Some(|lex| {
    lex.token = match lex.next_byte() {
        b'=' => {
            match lex.next_byte() {
                b'=' => {
                    lex.bump();

                    OperatorStrictEquality
                },

                _ => OperatorEquality
            }
        },

        b'>' => {
            lex.bump();

            OperatorFatArrow
        },

        _ => OperatorAssign
    };
});

// !
const EXL: ByteHandler = Some(|lex| {
    lex.token = match lex.next_byte() {
        b'=' => {
            match lex.next_byte() {
                b'=' => {
                    lex.bump();

                    OperatorStrictInequality
                },

                _ => OperatorInequality
            }
        },

        _ => OperatorLogicalNot
    };
});

// <
const LSS: ByteHandler = Some(|lex| {
    lex.token = match lex.next_byte() {
        b'<' => {
            match lex.next_byte() {
                b'=' => {
                    lex.bump();

                    OperatorBSLAssign
                },

                _ => OperatorBitShiftLeft
            }
        },

        b'=' => {
            lex.bump();

            OperatorLesserEquals
        },

        _ => OperatorLesser
    };
});

// >
const MOR: ByteHandler = Some(|lex| {
    lex.token = match lex.next_byte() {
        b'>' => {
            match lex.next_byte() {
                b'>' => {
                    match lex.next_byte() {
                        b'=' => {
                            lex.bump();

                            OperatorUBSRAssign
                        }

                        _ => OperatorUBitShiftRight
                    }
                },

                b'=' => {
                    lex.bump();

                    OperatorBSRAssign
                },

                _ => OperatorBitShiftRight
            }
        },

        b'=' => {
            lex.bump();

            OperatorGreaterEquals
        },

        _ => OperatorGreater
    };
});

// ?
const QST: ByteHandler = Some(|lex| {
    lex.bump();

    lex.token = OperatorConditional;
});

// ~
const TLD: ByteHandler = Some(|lex| {
    lex.bump();

    lex.token = OperatorBitwiseNot;
});

// ^
const CRT: ByteHandler = Some(|lex| {
    lex.token = match lex.next_byte() {
        b'=' => {
            lex.bump();

            OperatorBitXorAssign
        },

        _ => OperatorBitwiseXor
    };
});

// &
const AMP: ByteHandler = Some(|lex| {
    lex.token = match lex.next_byte() {
        b'&' => {
            lex.bump();

            OperatorLogicalAnd
        },

        b'=' => {
            lex.bump();

            OperatorBitAndAssign
        },

        _ => OperatorBitwiseAnd
    };
});

// |
const PIP: ByteHandler = Some(|lex| {
    lex.token = match lex.next_byte() {
        b'|' => {
            lex.bump();

            OperatorLogicalOr
        },

        b'=' => {
            lex.bump();

            OperatorBitOrAssign
        },

        _ => OperatorBitwiseOr
    };
});

// +
const PLS: ByteHandler = Some(|lex| {
    lex.token = match lex.next_byte() {
        b'+' => {
            lex.bump();

            OperatorIncrement
        },

        b'=' => {
            lex.bump();

            OperatorAddAssign
        },

        _ => OperatorAddition
    };
});

// -
const MIN: ByteHandler = Some(|lex| {
    lex.token = match lex.next_byte() {
        b'-' => {
            lex.bump();

            OperatorDecrement
        },

        b'=' => {
            lex.bump();

            OperatorSubtractAssign
        },

        _ => OperatorSubtraction
    };
});

// *
const ATR: ByteHandler = Some(|lex| {
    lex.token = match lex.next_byte() {
        b'*' => {
            match lex.next_byte() {
                b'=' => {
                    lex.bump();

                    OperatorExponentAssign
                },

                _ => OperatorExponent
            }
        },

        b'=' => {
            lex.bump();

            OperatorMultiplyAssign
        },

        _ => OperatorMultiplication
    };
});

// /
const SLH: ByteHandler = Some(|lex| {
    lex.token = match lex.next_byte() {
        // regular comment
        b'/' => {
            // Keep consuming bytes until new line or end of source
            unwind_loop!({
                match lex.next_byte() {
                    0 | b'\n' => {
                        return lex.consume();
                    }
                    _ => {}
                }
            });
        },

        // block comment
        b'*' => {
            lex.bump();
            // Keep consuming bytes until */ happens in a row
            unwind_loop!({
                match lex.read_byte() {
                    b'*' => {
                        match lex.next_byte() {
                            b'/' => {
                                lex.bump();
                                return lex.consume();
                            },
                            0 => return lex.token = UnexpectedEndOfProgram,
                            _ => {}
                        }
                    },
                    0 => return lex.token = UnexpectedEndOfProgram,
                    _ => lex.bump()
                }
            });
        },

        b'=' => {
            lex.bump();

            OperatorDivideAssign
        }

        _ => OperatorDivision
    };
});

// %
const PRC: ByteHandler = Some(|lex| {
    lex.token = match lex.next_byte() {
        b'=' => {
            lex.bump();

            OperatorRemainderAssign
        },

        _ => OperatorRemainder
    };
});

// Unicode character
const UNI: ByteHandler = Some(|lex| {
    let start = lex.index;

    // TODO: unicodes with different lengths
    let first = lex.slice_source(start, start + 4).chars().next().expect("Has to have one");

    if !first.is_alphanumeric() {
        return lex.token = UnexpectedToken;
    }

    // `read_label` bumps one at the beginning,
    // so we subtract it here.
    lex.index += first.len_utf8() - 1;

    lex.read_label();

    lex.token = Identifier;
});

// 0
const ZER: ByteHandler = Some(|lex| {
    match lex.next_byte() {
        b'b' | b'B' => {
            lex.bump();

            return lex.read_binary();
        },

        b'o' | b'O' => {
            lex.bump();

            return lex.read_octal();
        },

        b'x' | b'X' => {
            lex.bump();

            return lex.read_hexadec();
        },

        _ => {}
    }

    loop {
        match lex.read_byte() {
            b'0'...b'9' => {
                lex.bump();
            },
            b'.' => {
                lex.bump();

                return lex.read_float();
            },
            b'e' | b'E' => {
                lex.bump();

                return lex.read_scientific();
            }
            _ => break,
        }
    }

    lex.token = LiteralNumber;
});

// 1 to 9
const DIG: ByteHandler = Some(|lex| {
    unwind_loop!({
        match lex.next_byte() {
            b'0'...b'9' => {},
            b'.' => {
                lex.bump();

                return lex.read_float();
            },
            b'e' | b'E' => {
                lex.bump();

                return lex.read_scientific();
            },
            _ => {
                return lex.token = LiteralNumber;
            },
        }
    });
});

// .
const PRD: ByteHandler = Some(|lex| {
    match lex.next_byte() {
        b'0'...b'9' => {
            lex.bump();

            lex.read_float()
        },

        b'.' => {
            lex.token = match lex.next_byte() {
                b'.' => {
                    lex.bump();

                    OperatorSpread
                },

                _ => UnexpectedToken
            }
        },

        _ => lex.read_accessor()
    };
});

// " or '
const QOT: ByteHandler = Some(|lex| {
    let style = lex.read_byte();

    lex.bump();

    unwind_loop!({
        match lex.read_byte() {
            ch if ch == style => {
                lex.bump();
                return lex.token = LiteralString;
            },
            b'\\' => {
                lex.bump();
                expect_byte!(lex);
            },
            0 => {
                return lex.token = UnexpectedEndOfProgram;
            },
            _ => lex.bump()
        }
    });
});

// `
const TPL: ByteHandler = Some(|lex| {
    lex.bump();
    lex.read_template_kind();
});

pub struct Lexer<'arena> {
    /// Current `Token` from the source.
    pub token: Token,

    /// Flags whether or not a new line was read before the token
    asi: Asi,

    /// Source to parse, must be a C-style buffer ending with 0 byte
    ptr: *const u8,

    /// Current index
    index: usize,

    /// Position of current token in source
    token_start: usize,

    accessor_start: usize,

    pub quasi: &'arena str,
}


impl<'arena> Lexer<'arena> {
    /// Create a new `Lexer` from source using an existing arena.
    #[inline]
    pub fn new(arena: &'arena Arena, source: &str) -> Self {
        unsafe { Lexer::from_ptr(arena.alloc_str_with_nul(source)) }
    }

    /// Create a new `Lexer` from a raw pointer to byte string.
    ///
    /// **The source must be null terminated!**
    /// Passing a pointer that is not null terminated is undefined behavior!
    ///
    /// **The source must be valid UTF8!**
    /// Passing a pointer to data that is not valid UTF8 will lead
    /// to bugs or undefined behavior.
    #[inline]
    pub unsafe fn from_ptr(ptr: *const u8) -> Self {
        let mut lexer = Lexer {
            token: UnexpectedToken,
            asi: Asi::NoSemicolon,
            ptr,
            index: 0,
            token_start: 0,
            accessor_start: 0,
            quasi: "",
        };

        lexer.consume();

        lexer
    }

    /// Advances the lexer, produces a new `Token` and stores it on `self.token`.
    #[inline]
    pub fn consume(&mut self) {
        self.asi = Asi::NoSemicolon;

        let mut ch;

        unwind_loop!({
            ch = self.read_byte();

            if let Some(handler) = self.handler_from_byte(ch) {
                self.token_start = self.index;
                return handler(self);
            }

            self.bump();

            if ch == b'\n' {
                self.asi = Asi::ImplicitSemicolon;
            }
        })
    }

    /// Create an `&str` slice from source spanning current token.
    #[inline]
    pub fn token_as_str(&self) -> &'arena str {
        let start = self.token_start;
        self.slice_from(start)
    }

    /// Specialized version of `token_as_str` that crates an `&str`
    /// slice for the identifier following an accessor (`.`).
    #[inline]
    pub fn accessor_as_str(&self) -> &'arena str {
        let start = self.accessor_start;
        self.slice_from(start)
    }

    #[inline]
    fn handler_from_byte(&mut self, byte: u8) -> ByteHandler {
        unsafe { *(&BYTE_HANDLERS as *const ByteHandler).offset(byte as isize) }
    }

    /// Get the start and end positions of the current token.
    #[inline]
    pub fn loc(&self) -> (u32, u32) {
        (self.start(), self.end())
    }

    /// Get the start position of the current token.
    #[inline]
    pub fn start(&self) -> u32 {
        self.token_start as u32
    }

    /// Get the end position of the current token.
    #[inline]
    pub fn end(&self) -> u32 {
        self.index as u32
    }

    /// Get the start position of the current token, then advance the lexer.
    #[inline]
    pub fn start_then_consume(&mut self) -> u32 {
        let start = self.start();
        self.consume();
        start
    }

    /// Get the end position of the current token, then advance the lexer.
    #[inline]
    pub fn end_then_consume(&mut self) -> u32 {
        let end = self.end();
        self.consume();
        end
    }

    /// On top of being called when the opening backtick (`) of a template
    /// literal occurs, this method needs to be used by the parser while
    /// parsing a complex template string expression.
    ///
    /// **Note:** Parser needs to expect a BraceClose token before calling
    /// this method to ensure that the tokenizer state is not corrupted.
    #[inline]
    pub fn read_template_kind(&mut self) {
        let start = self.index;

        loop {
            match self.read_byte() {
                b'`' => {
                    let end = self.index;

                    self.bump();
                    self.quasi = self.slice_source(start, end);
                    self.token = TemplateClosed;

                    return;
                },
                b'$' => {
                    let end = self.index;

                    self.bump();

                    match self.read_byte() {
                        b'{' => self.bump(),
                        _    => continue
                    }

                    self.quasi = self.slice_source(start, end);
                    self.token = TemplateOpen;
                    return;
                },
                b'\\' => {
                    self.bump();

                    match self.read_byte() {
                        0 => {
                            self.token = UnexpectedEndOfProgram;
                            return;
                        },
                        _ => self.bump()
                    }
                },
                _ => self.bump()
            }
        }
    }

    /// Get a definition of which ASI rules can be applied.
    #[inline]
    pub fn asi(&self) -> Asi {
        self.asi
    }

    pub fn invalid_token(&mut self) -> Error {
        let start = self.token_start;
        let end = self.index;
        let token = self.token;

        if token != EndOfProgram {
            self.consume();
        }

        Error {
            token,
            start,
            end,
            raw: self.slice_source(start, end).to_owned().into_boxed_str()
        }
    }

    /// Read a byte from the source. Note that this does not increment
    /// the index. In few cases (all of them related to number parsing)
    /// we want to peek at the byte before doing anything. This will,
    /// very very rarely, lead to a situation where the same byte is read
    /// twice, but since this operation is using a raw pointer, the cost
    /// is virtually irrelevant.
    #[inline]
    fn read_byte(&self) -> u8 {
        unsafe { *self.ptr.add(self.index) }
    }

    /// Manually increment the index. Calling `read_byte` and then `bump`
    /// is equivalent to consuming a byte on an iterator.
    #[inline]
    fn bump(&mut self) {
        self.index += 1;
    }

    #[inline]
    fn next_byte(&mut self) -> u8 {
        self.bump();
        self.read_byte()
    }

    #[inline]
    fn read_binary(&mut self) {
        loop {
            match self.read_byte() {
                b'0' => {
                    self.bump();
                },
                b'1' => {
                    self.bump();
                },
                _ => break
            }
        }

        self.token = LiteralBinary;
    }

    /// This is a specialized method that expects the next token to be an identifier,
    /// even if it would otherwise be a keyword.
    ///
    /// This is useful when parsing member expressions such as `foo.function`, where
    /// `function` is actually allowed as a regular identifier, not a keyword.
    ///
    /// The perf gain here comes mainly from avoiding having to first match the `&str`
    /// to a keyword token, and then match that token back to a `&str`.
    #[inline]
    pub fn read_accessor(&mut self) {
        // Look up table that marks which ASCII characters are allowed to start an ident
        const AL: bool = true; // alphabet
        const DO: bool = true; // dollar sign $
        const US: bool = true; // underscore
        const BS: bool = true; // backslash
        const __: bool = false;

        static TABLE: [bool; 128] = [
        // 0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
          __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 0
          __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 1
          __, __, __, __, DO, __, __, __, __, __, __, __, __, __, __, __, // 2
          __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
          __, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, // 4
          AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, __, BS, __, __, US, // 5
          __, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, // 6
          AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, __, __, __, __, __, // 7
        ];

        let mut ch;

        unwind_loop!({
            ch = self.read_byte();

            if ch > 0x20 {
                self.accessor_start = self.index;

                if ch > 127 {
                    unimplemented!();
                    // return unicode(self)
                } else if TABLE[ch as usize] {
                    self.read_label();
                    return self.token = Accessor;
                } else {
                    return self.token = UnexpectedToken;
                }
            }

            self.bump();
        })
    }

    #[inline]
    fn read_label(&mut self) {
        while util::legal_in_label(self.read_byte()) {
            self.bump();
        }
    }

    #[inline]
    fn slice_from(&self, start: usize) -> &'arena str {
        let end = self.index;
        self.slice_source(start, end)
    }

    #[inline]
    fn slice_source(&self, start: usize, end: usize) -> &'arena str {
        use std::str::from_utf8_unchecked;
        use std::slice::from_raw_parts;

        unsafe {
            from_utf8_unchecked(from_raw_parts(
                self.ptr.add(start), end - start
            ))
        }
    }

    #[inline]
    fn read_octal(&mut self) {
        while match self.read_byte() {
            b'0'...b'7' => true,
            _ => false,
        } {
            self.bump();
        }

        self.token = LiteralNumber;
    }

    #[inline]
    fn read_hexadec(&mut self) {
        while match self.read_byte() {
            b'0'...b'9' |
            b'a'...b'f' |
            b'A'...b'F' => true,
            _ => false,
        } {
            self.bump();
        }

        self.token = LiteralNumber;
    }

    #[inline]
    fn read_float(&mut self) {
        loop {
            match self.read_byte() {
                b'0'...b'9'  => self.bump(),
                b'e' | b'E'  => {
                    self.bump();
                    return self.read_scientific();
                },
                _            => break
            }
        }

        self.token = LiteralNumber;
    }

    #[inline]
    fn read_scientific(&mut self) {
        match self.read_byte() {
            b'-' | b'+' => self.bump(),
            _           => {}
        }

        while match self.read_byte() {
            b'0'...b'9' => true,
            _ => false,
        } {
            self.bump();
        }

        self.token = LiteralNumber;
    }

    #[inline]
    pub fn read_regular_expression(&mut self) -> &'arena str {
        let start = self.index - 1;
        let mut in_class = false;
        loop {
            match self.read_byte() {
                b'['  => {
                    self.bump();
                    in_class = true;
                },
                b']'  => {
                    self.bump();
                    in_class = false;
                },
                b'/'  => {
                    self.bump();
                    if !in_class {
                        break;
                    }
                },
                b'\\' => {
                    match self.next_byte() {
                        0 => {
                            self.token = UnexpectedEndOfProgram;
                            return "";
                        },
                        _ => self.bump()
                    }
                },
                b'\n' => {
                    self.bump();
                    self.token = UnexpectedToken;
                    return "";
                },
                _     => self.bump()
            }
        }

        loop {
            match self.read_byte() {
                b'g' | b'i' | b'm' | b'u' | b'y' => {
                    self.bump();
                },
                _                                => {
                    break;
                }
            }
        }

        self.token = LiteralRegEx;
        self.slice_from(start)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn assert_lex<T>(source: &str, tokens: T) where T: AsRef<[(Token, &'static str)]> {
        let arena = Arena::new();
        let mut lex = Lexer::new(&arena, source);

        for &(ref token, slice) in tokens.as_ref() {
            assert_eq!(lex.token, *token);
            assert_eq!(lex.token_as_str(), slice);
            lex.consume();
        }

        assert_eq!(lex.token, EndOfProgram);
    }

    #[test]
    fn empty_lexer() {
        assert_lex("   ", []);
    }

    #[test]
    fn line_comment() {
        assert_lex(" // foo", []);
    }

    #[test]
    fn block_comment() {
        assert_lex(" /* foo */ bar", [(Identifier, "bar")]);
        assert_lex(" /** foo **/ bar", [(Identifier, "bar")]);
        assert_lex(" /*abc foo **/ bar", [(Identifier, "bar")]);
    }

    #[test]
    fn method_call() {
        assert_lex(
            "foo.bar();",
            [
                (Identifier, "foo"),
                (Accessor, ".bar"),
                (ParenOpen, "("),
                (ParenClose, ")"),
                (Semicolon, ";"),
            ]
        );
    }

    #[test]
    fn method_call_with_keyword() {
        assert_lex(
            "foo.function();",
            [
                (Identifier, "foo"),
                (Accessor, ".function"),
                (ParenOpen, "("),
                (ParenClose, ")"),
                (Semicolon, ";"),
            ]
        );
    }

    #[test]
    fn simple_math() {
        assert_lex(
            "let foo = 2 + 2;",
            [
                (DeclarationLet, "let"),
                (Identifier, "foo"),
                (OperatorAssign, "="),
                (LiteralNumber, "2"),
                (OperatorAddition, "+"),
                (LiteralNumber, "2"),
                (Semicolon, ";")
            ]
        );
    }

    #[test]
    fn variable_declaration() {
        assert_lex(
            "var x, y, z = 42;",
            [
                (DeclarationVar, "var"),
                (Identifier, "x"),
                (Comma, ","),
                (Identifier, "y"),
                (Comma, ","),
                (Identifier, "z"),
                (OperatorAssign, "="),
                (LiteralNumber, "42"),
                (Semicolon, ";"),
            ]
        );
    }

    #[test]
    fn function_statement() {
        assert_lex(
            "function foo(bar) { return bar }",
            [
                (Function, "function"),
                (Identifier, "foo"),
                (ParenOpen, "("),
                (Identifier, "bar"),
                (ParenClose, ")"),
                (BraceOpen, "{"),
                (Return, "return"),
                (Identifier, "bar"),
                (BraceClose, "}"),
            ]
        );
    }

    #[test]
    fn unexpected_token() {
        assert_lex("..", [(UnexpectedToken, "..")]);
    }

    #[test]
    fn unexpected_end() {
        assert_lex("'foo", [(UnexpectedEndOfProgram, "'foo")]);
    }

    #[test]
    fn keywords() {
        assert_lex(
            "
                break case class const debugger default delete do else
                export extends false finally for function if implements
                import in instanceof interface let new null package
                protected public return static super switch this throw
                true try undefined typeof var void while with yield
            ",
             &[
                (Break, "break"),
                (Case, "case"),
                (Class, "class"),
                (DeclarationConst, "const"),
                (Debugger, "debugger"),
                (Default, "default"),
                (OperatorDelete, "delete"),
                (Do, "do"),
                (Else, "else"),
                (Export, "export"),
                (Extends, "extends"),
                (LiteralFalse, "false"),
                (Finally, "finally"),
                (For, "for"),
                (Function, "function"),
                (If, "if"),
                (ReservedImplements, "implements"),
                (Import, "import"),
                (OperatorIn, "in"),
                (OperatorInstanceof, "instanceof"),
                (ReservedInterface, "interface"),
                (DeclarationLet, "let"),
                (OperatorNew, "new"),
                (LiteralNull, "null"),
                (ReservedPackage, "package"),
                (ReservedProtected, "protected"),
                (ReservedPublic, "public"),
                (Return, "return"),
                (Static, "static"),
                (Super, "super"),
                (Switch, "switch"),
                (This, "this"),
                (Throw, "throw"),
                (LiteralTrue, "true"),
                (Try, "try"),
                (LiteralUndefined, "undefined"),
                (OperatorTypeof, "typeof"),
                (DeclarationVar, "var"),
                (OperatorVoid, "void"),
                (While, "while"),
                (With, "with"),
                (Yield, "yield"),
            ][..]
        );
    }

    #[test]
    fn operators() {
        assert_lex(
            "
                => new ++ -- ! ~ typeof void delete * / % ** + - << >>
                >>> < <= > >= instanceof in === !== == != & ^ | && ||
                ? = += -= **= *= /= %= <<= >>= >>>= &= ^= |= ...
            ",
             &[
                (OperatorFatArrow, "=>"),
                (OperatorNew, "new"),
                (OperatorIncrement, "++"),
                (OperatorDecrement, "--"),
                (OperatorLogicalNot, "!"),
                (OperatorBitwiseNot, "~"),
                (OperatorTypeof, "typeof"),
                (OperatorVoid, "void"),
                (OperatorDelete, "delete"),
                (OperatorMultiplication, "*"),
                (OperatorDivision, "/"),
                (OperatorRemainder, "%"),
                (OperatorExponent, "**"),
                (OperatorAddition, "+"),
                (OperatorSubtraction, "-"),
                (OperatorBitShiftLeft, "<<"),
                (OperatorBitShiftRight, ">>"),
                (OperatorUBitShiftRight, ">>>"),
                (OperatorLesser, "<"),
                (OperatorLesserEquals, "<="),
                (OperatorGreater, ">"),
                (OperatorGreaterEquals, ">="),
                (OperatorInstanceof, "instanceof"),
                (OperatorIn, "in"),
                (OperatorStrictEquality, "==="),
                (OperatorStrictInequality, "!=="),
                (OperatorEquality, "=="),
                (OperatorInequality, "!="),
                (OperatorBitwiseAnd, "&"),
                (OperatorBitwiseXor, "^"),
                (OperatorBitwiseOr, "|"),
                (OperatorLogicalAnd, "&&"),
                (OperatorLogicalOr, "||"),
                (OperatorConditional, "?"),
                (OperatorAssign, "="),
                (OperatorAddAssign, "+="),
                (OperatorSubtractAssign, "-="),
                (OperatorExponentAssign, "**="),
                (OperatorMultiplyAssign, "*="),
                (OperatorDivideAssign, "/="),
                (OperatorRemainderAssign, "%="),
                (OperatorBSLAssign, "<<="),
                (OperatorBSRAssign, ">>="),
                (OperatorUBSRAssign, ">>>="),
                (OperatorBitAndAssign, "&="),
                (OperatorBitXorAssign, "^="),
                (OperatorBitOrAssign, "|="),
                (OperatorSpread, "..."),
            ][..]
        );
    }
}
