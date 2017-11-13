use std::marker::PhantomData;

mod token;
mod labels;
mod util;

pub use lexer::token::*;

use lexer::labels::*;
use lexer::token::Token::*;

use std::str;
use ast::Value;
use ast::OperatorKind::*;
use error::Error;
use arena::Arena;

macro_rules! expect_byte {
    ($lex:ident) => ({
        match $lex.read_byte() {
            0 => return $lex.token = UnexpectedEndOfProgram,
            _ => $lex.bump()
        }
    });
}

macro_rules! repeat8 {
    ($step:expr) => {
        $step
        $step
        $step
        $step
        $step
        $step
        $step
        $step
    }
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

/// Describes the state of Automatic Semicolon Insertion
///
/// * `ExplicitSemicolon` - a `Semicolon` token has been issued.
/// * `ImplicitSemicolon` - issued a token that permits semicolon insertion.
/// * `NoSemicolon`       - issued a token that does not permit semicolon insertion.
#[derive(Clone, Copy)]
pub enum Asi {
    ExplicitSemicolon,
    ImplicitSemicolon,
    NoSemicolon,
}

type ByteHandler = Option<for<'src> fn(&mut Lexer<'src>)>;

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
    let op = match lex.next_byte() {
        b'=' => {
            match lex.next_byte() {
                b'=' => {
                    lex.bump();

                    StrictEquality
                },

                _ => Equality
            }
        },

        b'>' => {
            lex.bump();

            FatArrow
        },

        _ => Assign
    };

    lex.token = Operator(op);
});

// !
const EXL: ByteHandler = Some(|lex| {
    let op = match lex.next_byte() {
        b'=' => {
            match lex.next_byte() {
                b'=' => {
                    lex.bump();

                    StrictInequality
                },

                _ => Inequality
            }
        },

        _ => LogicalNot
    };

    lex.token = Operator(op);
});

// <
const LSS: ByteHandler = Some(|lex| {
    let op = match lex.next_byte() {
        b'<' => {
            match lex.next_byte() {
                b'=' => {
                    lex.bump();

                    BSLAssign
                },

                _ => BitShiftLeft
            }
        },

        b'=' => {
            lex.bump();

            LesserEquals
        },

        _ => Lesser
    };

    lex.token = Operator(op);
});

// >
const MOR: ByteHandler = Some(|lex| {
    let op = match lex.next_byte() {
        b'>' => {
            match lex.next_byte() {
                b'>' => {
                    match lex.next_byte() {
                        b'=' => {
                            lex.bump();

                            UBSRAssign
                        }

                        _ => UBitShiftRight
                    }
                },

                b'=' => {
                    lex.bump();

                    BSRAssign
                },

                _ => BitShiftRight
            }
        },

        b'=' => {
            lex.bump();

            GreaterEquals
        },

        _ => Greater
    };

    lex.token = Operator(op);
});

// ?
const QST: ByteHandler = Some(|lex| {
    lex.bump();

    lex.token = Operator(Conditional);
});

// ~
const TLD: ByteHandler = Some(|lex| {
    lex.bump();

    lex.token = Operator(BitwiseNot);
});

// ^
const CRT: ByteHandler = Some(|lex| {
    let op = match lex.next_byte() {
        b'=' => {
            lex.bump();

            BitXorAssign
        },

        _ => BitwiseXor
    };

    lex.token = Operator(op);
});

// &
const AMP: ByteHandler = Some(|lex| {
    let op = match lex.next_byte() {
        b'&' => {
            lex.bump();

            LogicalAnd
        },

        b'=' => {
            lex.bump();

            BitAndAssign
        },

        _ => BitwiseAnd
    };

    lex.token = Operator(op);
});

// |
const PIP: ByteHandler = Some(|lex| {
    let op = match lex.next_byte() {
        b'|' => {
            lex.bump();

            LogicalOr
        },

        b'=' => {
            lex.bump();

            BitOrAssign
        },

        _ => BitwiseOr
    };

    lex.token = Operator(op);
});

// +
const PLS: ByteHandler = Some(|lex| {
    let op = match lex.next_byte() {
        b'+' => {
            lex.bump();

            Increment
        },

        b'=' => {
            lex.bump();

            AddAssign
        },

        _ => Addition
    };

    lex.token = Operator(op);
});

// -
const MIN: ByteHandler = Some(|lex| {
    let op = match lex.next_byte() {
        b'-' => {
            lex.bump();

            Decrement
        },

        b'=' => {
            lex.bump();

            SubstractAssign
        },

        _ => Substraction
    };

    lex.token = Operator(op);
});

// *
const ATR: ByteHandler = Some(|lex| {
    let op = match lex.next_byte() {
        b'*' => {
            match lex.next_byte() {
                b'=' => {
                    lex.bump();

                    ExponentAssign
                },

                _ => Exponent
            }
        },

        b'=' => {
            lex.bump();

            MultiplyAssign
        },

        _ => Multiplication
    };

    lex.token = Operator(op);
});

// /
const SLH: ByteHandler = Some(|lex| {
    let op = match lex.next_byte() {
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
            // Keep consuming bytes until */ happens in a row
            unwind_loop!({
                match lex.next_byte() {
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
                    _ => {}
                }
            });
        },

        b'=' => {
            lex.bump();

            DivideAssign
        }

        _ => Division
    };

    lex.token = Operator(op);
});

// %
const PRC: ByteHandler = Some(|lex| {
    let op = match lex.next_byte() {
        b'=' => {
            lex.bump();

            RemainderAssign
        },

        _ => Remainder
    };

    lex.token = Operator(op);
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

                    Operator(Spread)
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

pub struct Lexer<'src> {
    pub token: Token,

    /// Flags whether or not a new line was read before the token
    asi: Asi,

    /// Source to parse, must be a C-style buffer ending with 0 byte
    ptr: *const u8,

    /// Current index
    index: usize,

    /// Index of current token in source
    token_start: usize,

    accessor_start: usize,

    pub quasi: &'src str,
}


impl<'src> Lexer<'src> {
    #[inline]
    pub fn new(arena: &'src Arena, source: &str) -> Self {
        unsafe { Lexer::from_ptr(arena.alloc_str_zero_end(source)) }
    }

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

    #[inline]
    pub fn token_as_str(&self) -> &'src str {
        let start = self.token_start;
        self.slice_from(start)
    }

    #[inline]
    pub fn accessor_as_str(&self) -> &'src str {
        let start = self.accessor_start;
        self.slice_from(start)
    }

    #[inline]
    fn handler_from_byte(&mut self, byte: u8) -> ByteHandler {
        unsafe { *(&BYTE_HANDLERS as *const ByteHandler).offset(byte as isize) }
    }

    #[inline]
    pub fn loc(&self) -> (u32, u32) {
        (self.token_start as u32, self.index as u32)
    }

    #[inline]
    pub fn loc_start(&self) -> usize {
        self.token_start
    }

    #[inline]
    pub fn loc_end(&self) -> usize {
        self.index
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

    /// Check if Automatic Semicolon Insertion rules can be applied
    #[inline]
    pub fn asi(&self) -> Asi {
        self.asi
    }

    pub fn invalid_token(&mut self) -> Error {
        let start = self.token_start;
        let end = self.index;

        if self.token != EndOfProgram {
            self.consume();
        }

        Error::UnexpectedToken {
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
        unsafe { *self.ptr.offset(self.index as isize) }
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
    fn read_label(&mut self) -> &'src str {
        let start = self.token_start;

        unwind_loop!({
            if util::legal_in_label(self.read_byte()) {
                self.bump();
            } else {
                return self.slice_from(start)
            }
        })
    }

    #[inline]
    fn slice_from(&self, start: usize) -> &'src str {
        let end = self.index;
        self.slice_source(start, end)
    }

    #[inline]
    fn slice_source(&self, start: usize, end: usize) -> &'src str {
        use std::str::from_utf8_unchecked;
        use std::slice::from_raw_parts;

        unsafe {
            from_utf8_unchecked(from_raw_parts(
                self.ptr.offset(start as isize), end - start
            ))
        }
    }

    #[inline]
    fn read_octal(&mut self) {
        loop {
            match self.read_byte() {
                b'0'...b'7' => self.bump(),
                _           => break
            };
        }

        self.token = LiteralNumber;
    }

    #[inline]
    fn read_hexadec(&mut self) {
        loop {
            match self.read_byte() {
                b'0'...b'9' |
                b'a'...b'f' |
                b'A'...b'F' => self.bump(),
                _           => break
            };
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

        loop {
            match self.read_byte() {
                b'0'...b'9' => self.bump(),
                _           => break
            }
        }

        self.token = LiteralNumber;
    }

    #[inline]
    pub fn read_regular_expression(&mut self) -> &'src str {
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
                (Operator(Assign), "="),
                (LiteralNumber, "2"),
                (Operator(Addition), "+"),
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
                (Operator(Assign), "="),
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
                (Operator(Delete), "delete"),
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
                (Operator(In), "in"),
                (Operator(Instanceof), "instanceof"),
                (ReservedInterface, "interface"),
                (DeclarationLet, "let"),
                (Operator(New), "new"),
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
                (Operator(Typeof), "typeof"),
                (DeclarationVar, "var"),
                (Operator(Void), "void"),
                (While, "while"),
                (With, "with"),
                (Yield, "yield"),
            ][..]
        );
    }
}
