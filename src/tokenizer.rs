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


trait OnByte: Sync {
    fn execute(&self, tok: &mut Tokenizer, byte: u8) -> Result<Token>;
}

macro_rules! on_byte {
    { $(const $static_name:ident: $name:ident |$tok:pat, $byte:pat| $code:expr)* } => {
        $(
            struct $name;

            impl OnByte for $name {
                fn execute(&self, $tok: &mut Tokenizer, $byte: u8) -> Result<Token> {
                    $code
                }
            }

            const $static_name: &'static $name = &$name;
        )*
    }
}

on_byte! {
    const ___: InvalidByte |tok, _| {
        Err(Error {
            line: 0,
            column: tok.index,
        })
    }

    // =
    const EQL: EqualSign |tok, _| {
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
    const EXL: ExclamationMark |tok, _| {
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
    const LSS: LessSign |tok, _| {
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
    const MOR: MoreSign |tok, _| {
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
    const QST: QuestionMark |tok, _| {
        tok.bump();

        Ok(Operator(Conditional))
    }

    // ~
    const TLD: Tilde |tok, _| {
        tok.bump();

        Ok(Operator(BitwiseNot))
    }

    // ^
    const CRT: Caret |tok, _| {
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
    const AMP: Ampersand |tok, _| {
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
    const PIP: Pipe |tok, _| {
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
    const PLS: PlusSign |tok, _| {
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
    const MIN: MinusSign |tok, _| {
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
    const ATR: Asterisk |tok, _| {
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
    const SLH: Slash |tok, _| {
        tok.bump();

        let op = match tok.peek_byte() {
            b'/' => {
                tok.bump();
                tok.read_comment();
                tok.consume_whitespace();

                return tok.get_token();
            },

            b'*' => {
                tok.bump();
                tok.read_block_comment();
                tok.consume_whitespace();

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
    const PRC: Percent |tok, _| {
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

    // Label starting with a letter, _ or $
    const LBL: Label |tok, _| {
        let start = tok.index;

        tok.bump();

        while !tok.is_eof() {
            if !ident_lookup::TABLE[tok.read_byte() as usize] {
                break;
            }
            tok.bump();
        }

        let slice = unsafe {
            tok.source.slice_unchecked(start, tok.index)
        };

        Ok(match slice {
            "new"        => Operator(New),
            "typeof"     => Operator(Typeof),
            "delete"     => Operator(Delete),
            "void"       => Operator(Void),
            "in"         => Operator(In),
            "instanceof" => Operator(Instanceof),
            "var"        => Declaration(Var),
            "let"        => Declaration(Let),
            "const"      => Declaration(Const),
            "break"      => Break,
            "do"         => Do,
            "case"       => Case,
            "else"       => Else,
            "catch"      => Catch,
            "export"     => Export,
            "class"      => Class,
            "extends"    => Extends,
            "return"     => Return,
            "while"      => While,
            "finally"    => Finally,
            "super"      => Super,
            "with"       => With,
            "continue"   => Continue,
            "for"        => For,
            "switch"     => Switch,
            "yield"      => Yield,
            "debugger"   => Debugger,
            "function"   => Function,
            "this"       => This,
            "default"    => Default,
            "if"         => If,
            "throw"      => Throw,
            "import"     => Import,
            "try"        => Try,
            "await"      => Await,
            "static"     => Static,
            "true"       => Literal(LiteralTrue),
            "false"      => Literal(LiteralFalse),
            "undefined"  => Literal(LiteralUndefined),
            "null"       => Literal(LiteralNull),
            "enum"       => Reserved(Enum),
            "implements" => Reserved(Implements),
            "package"    => Reserved(Package),
            "protected"  => Reserved(Protected),
            "interface"  => Reserved(Interface),
            "private"    => Reserved(Private),
            "public"     => Reserved(Public),
            _            => Identifier(unsafe { OwnedSlice::from_str(slice) }),
        })
    }

    // 0 to 9
    const DIG: Digit |tok, first| {
        let start = tok.index;

        tok.bump();

        if first == b'0' {
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

    // .
    const PRD: Period |tok, _| {
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

                    _ => Err(Error {
                        line: 0,
                        column: tok.index
                    })
                }
            },

            _ => Ok(Operator(Accessor))
        }
    }

    // " or '
    const QOT: Quote |tok, byte| {
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

    // space, tab, carriage return, new line
    const WHT: Whitespace |tok, _| {
        tok.bump();

        tok.consume_whitespace();

        tok.get_token()
    }

    // One of: ( ) [ ] { } : ; ,
    const CTL: ControlSign |tok, byte| {
        tok.bump();

        Ok(Control(byte))
    }

    const UNI: Unicode |_, _| {
        unimplemented!()
    }
}

#[allow(dead_code)]
static ON_BYTES: [&'static OnByte; 256] = [
//   0    1    2    3    4    5    6    7    8    9    A    B    C    D    E    F   //
    ___, ___, ___, ___, ___, ___, ___, ___, ___, WHT, WHT, ___, ___, WHT, ___, ___, // 0
    ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, // 1
    WHT, EXL, QOT, ___, LBL, PRC, AMP, QOT, CTL, CTL, ATR, PLS, CTL, MIN, PRD, SLH, // 2
    DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, CTL, CTL, LSS, EQL, MOR, QST, // 3
    ___, LBL, LBL, LBL, LBL, LBL, LBL, LBL, LBL, LBL, LBL, LBL, LBL, LBL, LBL, LBL, // 4
    LBL, LBL, LBL, LBL, LBL, LBL, LBL, LBL, LBL, LBL, LBL, CTL, ___, CTL, CRT, LBL, // 5
    ___, LBL, LBL, LBL, LBL, LBL, LBL, LBL, LBL, LBL, LBL, LBL, LBL, LBL, LBL, LBL, // 6
    LBL, LBL, LBL, LBL, LBL, LBL, LBL, LBL, LBL, LBL, LBL, CTL, PIP, CTL, TLD, ___, // 7
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // 8
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // 9
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // A
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // B
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // C
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // D
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // E
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // F
];

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

    #[inline]
    fn expect_byte(&mut self) -> u8 {
        if self.is_eof() {
            panic!("Unexpected end of source");
        }

        let ch = self.read_byte();
        self.bump();
        ch
    }

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
                self.token = None;
                token
            },
            None => self.get_token().unwrap()
        }
    }

    #[inline]
    fn get_token(&mut self) -> Result<Token> {
        if self.is_eof() {
            return Ok(EndOfProgram);
        }

        let ch = self.read_byte();

        ON_BYTES[ch as usize].execute(self, ch)
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

            if ch == b'/' && self.index + 1 < self.length {
                let slice = unsafe {
                    self.source.slice_unchecked(self.index, self.index + 2)
                };

                match slice {
                    "//" => {
                        self.index += 2;
                        self.read_comment();
                        continue;
                    },
                    "/*" => {
                        self.index += 2;
                        self.read_block_comment();
                        continue;
                    },
                    _ => return
                }
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
