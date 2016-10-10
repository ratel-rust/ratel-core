use std::str;
use lexicon::Token;
use lexicon::Token::*;
use lexicon::ReservedKind::*;
use grammar::OwnedSlice;
use grammar::OperatorType::*;
use grammar::VariableDeclarationKind::*;
use grammar::LiteralValue;
use grammar::LiteralValue::*;
use error::{ Error, Result };

/// Helper macro for declaring byte-handler functions with correlating constants.
/// This becomes handy due to a lookup table present below.
macro_rules! define_handlers {
    { $(const $static_name:ident: $name:ident |$tok:pat, $byte:pat| $code:block)* } => {
        $(
            fn $name($tok: &mut Tokenizer, $byte: u8) -> Result<Token> $code

            const $static_name: fn(&mut Tokenizer, u8) -> Result<Token> = $name;
        )*
    }
}

/// Lookup table mapping any incoming byte to a handler function defined below.
static BYTE_HANDLERS: [fn(&mut Tokenizer, u8) -> Result<Token>; 256] = [
//   0    1    2    3    4    5    6    7    8    9    A    B    C    D    E    F   //
    ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, // 0
    ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, // 1
    ___, EXL, QOT, ___, IDT, PRC, AMP, QOT, CTL, CTL, ATR, PLS, CTL, MIN, PRD, SLH, // 2
    ZER, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, CTL, CTL, LSS, EQL, MOR, QST, // 3
    ___, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, // 4
    IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, CTL, ___, CTL, CRT, IDT, // 5
    ___, IDT, L_B, L_C, L_D, L_E, L_F, IDT, IDT, L_I, IDT, IDT, L_L, IDT, L_N, IDT, // 6
    L_P, IDT, L_R, L_S, L_T, L_U, L_V, L_W, IDT, L_Y, IDT, CTL, PIP, CTL, TLD, ___, // 7
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // 8
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // 9
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // A
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // B
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // C
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // D
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // E
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // F
];

// Handler function definitions:
define_handlers! {
    const ___: invalid_byte |tok, _| {
        Err(tok.invalid_character())
    }

    // =
    const EQL: equal_sign |tok, _| {
        tok.bump();

        let op = match tok.peek_byte() {
            b'=' => {
                tok.bump();

                match tok.peek_byte() {
                    b'=' => {
                        tok.bump();

                        StrictEquality
                    },

                    _ => Equality
                }
            },

            b'>' => {
                tok.bump();

                FatArrow
            },

            _ => Assign
        };

        Ok(Operator(op))
    }

    // !
    const EXL: exclamation_mark |tok, _| {
        tok.bump();

        let op = match tok.peek_byte() {
            b'=' => {
                tok.bump();

                match tok.peek_byte() {
                    b'=' => {
                        tok.bump();

                        StrictInequality
                    },

                    _ => Inequality
                }
            },

            _ => LogicalNot
        };

        Ok(Operator(op))
    }

    // <
    const LSS: less_sign |tok, _| {
        tok.bump();

        let op = match tok.peek_byte() {
            b'<' => {
                tok.bump();

                match tok.peek_byte() {
                    b'=' => {
                        tok.bump();

                        BSLAssign
                    },

                    _ => BitShiftLeft
                }
            },

            b'=' => {
                tok.bump();

                LesserEquals
            },

            _ => LogicalNot
        };

        Ok(Operator(op))
    }

    // >
    const MOR: more_sign |tok, _| {
        tok.bump();

        let op = match tok.peek_byte() {
            b'>' => {
                tok.bump();

                match tok.peek_byte() {
                    b'>' => {
                        tok.bump();

                        match tok.peek_byte() {
                            b'=' => {
                                tok.bump();

                                UBSRAssign
                            }

                            _ => UBitShiftRight
                        }
                    },

                    _ => BSRAssign
                }
            },

            b'=' => {
                tok.bump();

                GreaterEquals
            },

            _ => Greater
        };

        Ok(Operator(op))
    }

    // ?
    const QST: question_mark |tok, _| {
        tok.bump();

        Ok(Operator(Conditional))
    }

    // ~
    const TLD: tilde |tok, _| {
        tok.bump();

        Ok(Operator(BitwiseNot))
    }

    // ^
    const CRT: caret |tok, _| {
        tok.bump();

        let op = match tok.peek_byte() {
            b'=' => {
                tok.bump();

                BitXorAssign
            },

            _ => BitwiseXor
        };

        Ok(Operator(op))
    }

    // &
    const AMP: ampersand |tok, _| {
        tok.bump();

        let op = match tok.peek_byte() {
            b'&' => {
                tok.bump();

                LogicalAnd
            },

            b'=' => {
                tok.bump();

                BitAndAssign
            },

            _ => BitwiseAnd
        };

        Ok(Operator(op))
    }

    // |
    const PIP: pipe |tok, _| {
        tok.bump();

        let op = match tok.peek_byte() {
            b'|' => {
                tok.bump();

                LogicalOr
            },

            b'=' => {
                tok.bump();

                BitOrAssign
            },

            _ => BitwiseOr
        };

        Ok(Operator(op))
    }

    // +
    const PLS: plus_sign |tok, _| {
        tok.bump();

        let op = match tok.peek_byte() {
            b'+' => {
                tok.bump();

                Increment
            },

            b'=' => {
                tok.bump();

                AddAssign
            },

            _ => Addition
        };

        Ok(Operator(op))
    }

    // -
    const MIN: minus_sign |tok, _| {
        tok.bump();

        let op = match tok.peek_byte() {
            b'-' => {
                tok.bump();

                Decrement
            },

            b'=' => {
                tok.bump();

                SubstractAssign
            },

            _ => Substraction
        };

        Ok(Operator(op))
    }

    // *
    const ATR: asterisk |tok, _| {
        tok.bump();

        let op = match tok.peek_byte() {
            b'*' => {
                tok.bump();

                match tok.peek_byte() {
                    b'=' => {
                        tok.bump();

                        ExponentAssign
                    },

                    _ => Exponent
                }
            },

            b'=' => {
                tok.bump();

                MultiplyAssign
            },

            _ => Multiplication
        };

        Ok(Operator(op))
    }

    // /
    const SLH: slash |tok, _| {
        tok.bump();

        let op = match tok.peek_byte() {
            b'/' => {
                tok.bump();
                tok.read_comment();

                return tok.get_token();
            },

            b'*' => {
                tok.bump();
                tok.read_block_comment();

                return tok.get_token();
            },

            b'=' => {
                tok.bump();

                DivideAssign
            }

            _ => Division
        };

        Ok(Operator(op))
    }

    // %
    const PRC: percent |tok, _| {
        tok.bump();

        let op = match tok.peek_byte() {
            b'=' => {
                tok.bump();

                RemainderAssign
            },

            _ => Remainder
        };

        Ok(Operator(op))
    }

    // Non-keyword Identifier: starting with a letter, _ or $
    const IDT: identifier |tok, _| {
        Ok(Identifier(unsafe {
            OwnedSlice::from_str(tok.consume_label_characters())
        }))
    }

    // Identifier or keyword starting with a letter `b`
    const L_B: label_b |tok, _| {
        Ok(match tok.consume_label_characters() {
            "break"      => Break,
            slice        => Identifier(unsafe { OwnedSlice::from_str(slice) }),
        })
    }

    // Identifier or keyword starting with a letter `c`
    const L_C: label_c |tok, _| {
        Ok(match tok.consume_label_characters() {
            "const"      => Declaration(Const),
            "case"       => Case,
            "class"      => Class,
            "catch"      => Catch,
            "continue"   => Continue,
            slice        => Identifier(unsafe { OwnedSlice::from_str(slice) }),
        })
    }

    // Identifier or keyword starting with a letter `d`
    const L_D: label_d |tok, _| {
        Ok(match tok.consume_label_characters() {
            "delete"     => Operator(Delete),
            "do"         => Do,
            "debugger"   => Debugger,
            "default"    => Default,
            slice        => Identifier(unsafe { OwnedSlice::from_str(slice) }),
        })
    }

    // Identifier or keyword starting with a letter `e`
    const L_E: label_e |tok, _| {
        Ok(match tok.consume_label_characters() {
            "else"       => Else,
            "export"     => Export,
            "extends"    => Extends,
            "enum"       => Reserved(Enum),
            slice        => Identifier(unsafe { OwnedSlice::from_str(slice) }),
        })
    }

    // Identifier or keyword starting with a letter `f`
    const L_F: label_f |tok, _| {
        Ok(match tok.consume_label_characters() {
            "finally"    => Finally,
            "for"        => For,
            "function"   => Function,
            "false"      => Literal(LiteralFalse),
            slice        => Identifier(unsafe { OwnedSlice::from_str(slice) }),
        })
    }

    // Identifier or keyword starting with a letter `i`
    const L_I: label_i |tok, _| {
        Ok(match tok.consume_label_characters() {
            "in"         => Operator(In),
            "instanceof" => Operator(Instanceof),
            "if"         => If,
            "import"     => Import,
            "implements" => Reserved(Implements),
            "interface"  => Reserved(Interface),
            slice        => Identifier(unsafe { OwnedSlice::from_str(slice) }),
        })
    }

    // Identifier or keyword starting with a letter `l`
    const L_L: label_l |tok, _| {
        Ok(match tok.consume_label_characters() {
            "let"        => Declaration(Let),
            slice        => Identifier(unsafe { OwnedSlice::from_str(slice) }),
        })
    }

    // Identifier or keyword starting with a letter `n`
    const L_N: label_n |tok, _| {
        Ok(match tok.consume_label_characters() {
            "new"        => Operator(New),
            "null"       => Literal(LiteralNull),
            slice        => Identifier(unsafe { OwnedSlice::from_str(slice) }),
        })
    }

    // Identifier or keyword starting with a letter `p`
    const L_P: label_p |tok, _| {
        Ok(match tok.consume_label_characters() {
            "package"    => Reserved(Package),
            "protected"  => Reserved(Protected),
            "private"    => Reserved(Private),
            "public"     => Reserved(Public),
            slice        => Identifier(unsafe { OwnedSlice::from_str(slice) }),
        })
    }

    // Identifier or keyword starting with a letter `r`
    const L_R: label_r |tok, _| {
        Ok(match tok.consume_label_characters() {
            "return"     => Return,
            slice        => Identifier(unsafe { OwnedSlice::from_str(slice) }),
        })
    }

    // Identifier or keyword starting with a letter `s`
    const L_S: label_s |tok, _| {
        Ok(match tok.consume_label_characters() {
            "super"      => Super,
            "switch"     => Switch,
            "static"     => Static,
            slice        => Identifier(unsafe { OwnedSlice::from_str(slice) }),
        })
    }

    // Identifier or keyword starting with a letter `t`
    const L_T: label_t |tok, _| {
        Ok(match tok.consume_label_characters() {
            "typeof"     => Operator(Typeof),
            "this"       => This,
            "throw"      => Throw,
            "try"        => Try,
            "true"       => Literal(LiteralTrue),
            slice        => Identifier(unsafe { OwnedSlice::from_str(slice) }),
        })
    }

    // Identifier or keyword starting with a letter `u`
    const L_U: label_u |tok, _| {
        Ok(match tok.consume_label_characters() {
            "undefined"  => Literal(LiteralUndefined),
            slice        => Identifier(unsafe { OwnedSlice::from_str(slice) }),
        })
    }

    // Identifier or keyword starting with a letter `v`
    const L_V: label_v |tok, _| {
        Ok(match tok.consume_label_characters() {
            "void"       => Operator(Void),
            "var"        => Declaration(Var),
            slice        => Identifier(unsafe { OwnedSlice::from_str(slice) }),
        })
    }

    // Identifier or keyword starting with a letter `w`
    const L_W: label_w |tok, _| {
        Ok(match tok.consume_label_characters() {
            "while"      => While,
            "with"       => With,
            slice        => Identifier(unsafe { OwnedSlice::from_str(slice) }),
        })
    }

    // Identifier or keyword starting with a letter `y`
    const L_Y: label_y |tok, _| {
        Ok(match tok.consume_label_characters() {
            "yield"      => Yield,
            slice        => Identifier(unsafe { OwnedSlice::from_str(slice) }),
        })
    }

    // Unicode character
    const UNI: unicode |tok, _| {
        let start = tok.index;

        let first = tok.source[start..].chars().next().expect("Has to have one");

        // TODO: check first.is_alphanumeric();

        tok.index += first.len_utf8();

        tok.consume_label_characters();

        Ok(Identifier(unsafe {
            let slice = tok.source.slice_unchecked(start, tok.index);
            OwnedSlice::from_str(slice)
        }))
    }

    // 0
    const ZER: zero |tok, _| {
        let start = tok.index;

        tok.bump();

        match tok.peek_byte() {
            b'b' => {
                tok.bump();

                return Ok(Literal(tok.read_binary()));
            },

            b'o' => {
                tok.bump();

                return Ok(Literal(tok.read_octal()));
            },

            b'x' => {
                tok.bump();

                return Ok(Literal(tok.read_hexadec()));
            },

            _ => {}
        }

        while !tok.is_eof() {
            match tok.read_byte() {
                b'0'...b'9' => {
                    tok.bump();
                },
                b'.' => {
                    tok.bump();

                    return Ok(Literal(tok.read_float(start)));
                },
                _ => break,
            }
        }

        let value = unsafe {
            let slice = tok.source.slice_unchecked(start, tok.index);
            OwnedSlice::from_str(slice)
        };

        Ok(Literal(LiteralFloat(value)))
    }

    // 1 to 9
    const DIG: digit |tok, _| {
        let start = tok.index;

        tok.bump();

        while !tok.is_eof() {
            match tok.read_byte() {
                b'0'...b'9' => {
                    tok.bump();
                },
                b'.' => {
                    tok.bump();

                    return Ok(Literal(tok.read_float(start)));
                },
                _ => break,
            }
        }

        let value = unsafe {
            let slice = tok.source.slice_unchecked(start, tok.index);
            OwnedSlice::from_str(slice)
        };

        Ok(Literal(LiteralFloat(value)))
    }

    // .
    const PRD: period |tok, _| {
        let start = tok.index;

        tok.bump();

        match tok.peek_byte() {
            b'0'...b'9' => {
                tok.bump();

                Ok(Literal(tok.read_float(start)))
            },

            b'.' => {
                tok.bump();

                match tok.peek_byte() {
                    b'.' => {
                        tok.bump();

                        Ok(Operator(Spread))
                    },

                    _ => Err(tok.invalid_character())
                }
            },

            _ => Ok(Operator(Accessor))
        }
    }

    // " or '
    const QOT: quote |tok, byte| {
        let start = tok.index;

        tok.bump();

        loop {
            let ch = tok.expect_byte();

            if ch == byte {
                break;
            }

            if ch == b'\\' {
                tok.expect_byte();
            }
        }

        let value = unsafe {
            let slice = tok.source.slice_unchecked(start, tok.index);
            OwnedSlice::from_str(slice)
        };

        Ok(Literal(LiteralString(value)))
    }

    // One of: ( ) [ ] { } : ; ,
    const CTL: control_sign |tok, byte| {
        tok.bump();

        Ok(Control(byte))
    }
}

mod whitespace {
    const __: bool = false;
    const WH: bool = true;

    pub static TABLE: [bool; 256] = [
    // 0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
      __, __, __, __, __, __, __, __, __, WH, WH, __, __, WH, __, __, // 0
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 1
      WH, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 5
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
    ];
}

mod ident_lookup {
    // Look up table that marks which ASCII characters are allowed in identifiers
    pub const NU: bool = true; // digit
    pub const AL: bool = true; // alphabet
    pub const DO: bool = true; // dollar sign $
    pub const US: bool = true; // underscore
    pub const UN: bool = true; // unicode
    pub const __: bool = false;

    pub static TABLE: [bool; 256] = [
    // 0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 0
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 1
      __, __, __, __, DO, __, __, __, __, __, __, __, __, __, __, __, // 2
      NU, NU, NU, NU, NU, NU, NU, NU, NU, NU, __, __, __, __, __, __, // 3
      __, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, // 4
      AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, __, __, __, __, US, // 5
      __, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, // 6
      AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, __, __, __, __, __, // 7
      UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, // 8
      UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, // 9
      UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, // A
      UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, // B
      UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, // C
      UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, // D
      UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, // E
      UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, UN, // F
    ];
}

pub struct Tokenizer<'a> {
    // String slice to parse
    pub source: &'a str,

    // Byte pointer to the slice above
    byte_ptr: *const u8,

    // Current index
    index: usize,

    // Lenght of the source
    length: usize,

    // Current token
    token: Option<Token>,

    // Index of current token in source
    pub token_start: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Tokenizer {
            source: source,
            byte_ptr: source.as_ptr(),
            index: 0,
            length: source.len(),
            token: None,
            token_start: 0,
        }
    }

    // Check if we are at the end of the source.
    #[inline]
    fn is_eof(&self) -> bool {
        self.index == self.length
    }

    // Read a byte from the source. Note that this does not increment
    // the index. In few cases (all of them related to number parsing)
    // we want to peek at the byte before doing anything. This will,
    // very very rarely, lead to a situation where the same byte is read
    // twice, but since this operation is using a raw pointer, the cost
    // is virtually irrelevant.
    #[inline]
    fn read_byte(&self) -> u8 {
        unsafe { *self.byte_ptr.offset(self.index as isize) }
    }

    #[inline]
    fn peek_byte(&self) -> u8 {
        if self.is_eof() {
            return 0;
        }

        self.read_byte()
    }

    // Manually increment the index. Calling `read_byte` and then `bump`
    // is equivalent to consuming a byte on an iterator.
    #[inline]
    fn bump(&mut self) {
        self.index += 1;
    }

    fn invalid_character(&self) -> Error {
        if self.is_eof() {
            return Error::UnexpectedEndOfProgram;
        }

        let len = self.source[self.index..]
                      .chars()
                      .next()
                      .expect("Must have a character")
                      .len_utf8();

        Error::UnexpectedToken {
            start: self.index,
            end: self.index + len,
        }
    }

    // fn invalid_token(&self) -> Error {
    //     Error::UnexpectedToken {
    //         start: self.token_start,
    //         end: self.index,
    //     }
    // }

    #[inline]
    fn expect_byte(&mut self) -> u8 {
        if self.is_eof() {
            panic!("Unexpected end of source");
        }

        let ch = self.read_byte();
        self.bump();
        ch
    }

    #[inline]
    fn read_binary(&mut self) -> LiteralValue {
        let mut value = 0;

        while !self.is_eof() {
            let peek = self.read_byte();
            match peek {
                b'0' => {
                    value <<= 1;
                    self.bump();
                },
                b'1' => {
                    value = (value << 1) + 1;
                    self.bump();
                },
                _ => break
            }
        }

        LiteralInteger(value)
    }

    #[inline]
    fn read_octal(&mut self) -> LiteralValue {
        let mut value = 0;

        while !self.is_eof() {
            let peek = self.read_byte();
            let digit = match peek {
                b'0'...b'7' => peek - b'0',
                _           => break
            };

            value = (value << 3) + digit as u64;
            self.bump();
        }

        LiteralInteger(value)
    }

    #[inline]
    fn read_hexadec(&mut self) -> LiteralValue {
        let mut value = 0;

        while !self.is_eof() {
            let peek = self.read_byte();
            let digit = match peek {
                b'0'...b'9' => peek - b'0',
                b'a'...b'f' => peek - b'a' + 10,
                b'A'...b'F' => peek - b'A' + 10,
                _           => break
            };

            value = (value << 4) + digit as u64;
            self.bump();
        }

        return LiteralInteger(value);
    }

    #[inline]
    fn consume_label_characters(&mut self) -> &str {
        let start = self.index;

        self.bump();

        while !self.is_eof() && ident_lookup::TABLE[self.read_byte() as usize] {
            self.bump();
        }

        unsafe {
            self.source.slice_unchecked(start, self.index)
        }
    }

    #[inline]
    fn read_float(&mut self, start: usize) -> LiteralValue {
        while !self.is_eof() {
            let ch = self.read_byte();
            match ch {
                b'0'...b'9' => self.bump(),
                _           => break
            }
        }

        LiteralValue::LiteralFloat(unsafe {
            OwnedSlice::from_str(self.source.slice_unchecked(start, self.index))
        })
    }

    #[inline]
    fn read_comment(&mut self) {
        while !self.is_eof() {
            if self.read_byte() == b'\n' {
                return;
            }
            self.bump();
        }
    }

    #[inline]
    fn read_block_comment(&mut self) {
        loop {
            if self.expect_byte() == b'*' && self.expect_byte() == b'/' {
                return;
            }
        }
    }

    #[inline]
    pub fn peek(&mut self) -> Token {
        match self.token {
            Some(token) => token,

            None => {
                let token = self.get_token().unwrap();

                self.token = Some(token);

                token
            }
        }
    }

    #[inline]
    pub fn next(&mut self) -> Token {
        match self.token {
            Some(token) => {
                self.consume();

                token
            },
            None => self.get_token().unwrap()
        }
    }

    #[inline]
    pub fn consume(&mut self) {
        self.token = None;
    }

    #[inline]
    fn get_token(&mut self) -> Result<Token> {
        self.consume_whitespace();

        self.token_start = self.index;

        if self.is_eof() {
            return Ok(EndOfProgram);
        }

        let ch = self.read_byte();

        BYTE_HANDLERS[ch as usize](self, ch)
    }

    #[inline]
    fn consume_whitespace(&mut self) {
        while !self.is_eof() {
            let ch = self.read_byte();

            // if ch <= 0x20 {
            if whitespace::TABLE[ch as usize] {
                self.bump();
                continue;
            }

            return;
        }
    }

    #[inline]
    pub fn expect_identifier(&mut self) -> OwnedSlice {
        match self.next() {
            Identifier(ident) => ident,
            token             => panic!("Unexpected token `{:?}` {}", token, self.index)
        }
    }

    #[inline]
    pub fn expect_semicolon(&mut self) {
        match self.next() {
            Control(b';') => self.bump(),
            Control(b')') |
            Control(b'}') => return,
            token         => panic!("Unexpected token `{:?}` {}", token, self.index)
        }
    }

    #[inline]
    pub fn expect_control(&mut self, expected: u8) {
        let token = self.next();

        if token != Control(expected) {
            panic!("Unexpected token `{:?}` {}", token, self.index);
        }
    }

    #[inline]
    pub fn allow_control(&mut self) -> u8 {
        match self.peek() {
            Control(byte) => byte,
            _             => 0
        }
    }
}
