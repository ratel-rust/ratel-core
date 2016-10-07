use std::str;
use lexicon::Token;
use lexicon::Token::*;
use lexicon::ReservedKind::*;
use grammar::OwnedSlice;
use grammar::OperatorType::*;
use grammar::VariableDeclarationKind::*;
use grammar::LiteralValue;
use grammar::LiteralValue::*;



macro_rules! match_operators {
    { $tokenizer:ident $ch:ident
        $(
            $prime:pat => $tok:ident $cascade:tt
        )*
    } => ({
        match $ch {
            $(
                $prime => match_descend!( $tokenizer $tok $cascade )
            ),*
            ,
            _ => {}
        }
    })
}

macro_rules! match_descend {
    ( $tokenizer:ident $matched:ident { $(
        $secondary:pat => $tok:ident $cascade:tt
    )* } ) => ({
        if $tokenizer.is_eof() {
            return Some(Operator($matched));
        }
        match $tokenizer.read_byte() {
            $(
                $secondary => {
                    $tokenizer.bump();
                    match_descend!( $tokenizer $tok $cascade )
                },
            )*
            _ => return Some(Operator($matched))
        }
    });
    ( $tokenizer:ident return $matched:expr ) => {
        return Some($matched)
    };
    ( $tokenizer:ident $matched:expr , ) => {
        return Some(Operator($matched))
    }
}

mod ident_lookup {
    // Look up table that marks which ASCII characters are allowed in identifiers
    pub const NU: bool = true; // digit
    pub const AL: bool = true; // alphabet
    pub const DO: bool = true; // dollar sign $
    pub const UN: bool = true; // underscore
    pub const __: bool = false;

    pub static TABLE: [bool; 256] = [
    // 0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 0
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 1
      __, __, __, __, DO, __, __, __, __, __, __, __, __, __, __, __, // 2
      NU, NU, NU, NU, NU, NU, NU, NU, NU, NU, __, __, __, __, __, __, // 3
      __, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, // 4
      AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, __, __, __, __, UN, // 5
      __, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, // 6
      AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, __, __, __, __, __, // 7
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

mod byte_category {
    pub const __: u8 = 0; // invalid token
    pub const WH: u8 = 1; // whitespace
    pub const OP: u8 = 2; // guaranteed operator
    pub const ID: u8 = 3; // guaranteed identifier | operator
    pub const CT: u8 = 4; // control bytes ( ) [ ] { } ; : ,
    pub const UN: u8 = 5; // potential unicode identifier

    pub static TABLE: [u8; 256] = [
    // 0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
      __, __, __, __, __, __, __, __, __, WH, WH, __, __, WH, __, __, // 0
      __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 1
      WH, OP, __, __, ID, OP, OP, __, CT, CT, OP, OP, CT, OP, __, __, // 2
      __, __, __, __, __, __, __, __, __, __, CT, CT, OP, OP, OP, OP, // 3
      __, ID, ID, ID, ID, ID, ID, ID, ID, ID, ID, ID, ID, ID, ID, ID, // 4
      ID, ID, ID, ID, ID, ID, ID, ID, ID, ID, ID, CT, __, CT, OP, ID, // 5
      __, ID, ID, ID, ID, ID, ID, ID, ID, ID, ID, ID, ID, ID, ID, ID, // 6
      ID, ID, ID, ID, ID, ID, ID, ID, ID, ID, ID, CT, OP, CT, OP, __, // 7
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
    fn is_eof(&mut self) -> bool {
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

    fn read_string(&mut self, first: u8) -> OwnedSlice {
        let start = self.index - 1;

        loop {
            let ch = self.expect_byte();

            if ch == first {
                break;
            }

            if ch == b'\\' {
                self.expect_byte();
            }
        }

        unsafe {
            OwnedSlice::from_str(&self.source.slice_unchecked(start, self.index))
        }
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

    fn read_number(&mut self, first: u8) -> LiteralValue {
        let start = self.index - 1;

        if first == b'.' {
            return self.read_float(start);
        } else if first == b'0' {
            if self.is_eof() {
                return LiteralValue::LiteralInteger(0);
            }
            match self.read_byte() {
                b'b' => {
                    self.bump();
                    return self.read_binary();
                },
                b'o' => {
                    self.bump();
                    return self.read_octal();
                },
                b'x' => {
                    self.bump();
                    return self.read_hexadec();
                },
                _ => {}
            }
        }

        let mut value = (first - b'0') as u64;

        while !self.is_eof() {
            let ch = self.read_byte();
            match ch {
                b'0'...b'9' => {
                    value = value * 10 + (ch - b'0') as u64;
                    self.bump();
                },
                b'.' => {
                    return self.read_float(start);
                },
                _ => break,
            }
        }

        LiteralValue::LiteralInteger(value)
    }

    fn read_comment(&mut self) {
        while !self.is_eof() {
            if self.read_byte() == b'\n' {
                return;
            }
            self.bump();
        }
    }

    fn read_block_comment(&mut self) {
        loop {
            if self.expect_byte() == b'*' && self.expect_byte() == b'/' {
                return;
            }
        }
    }

    fn read_label(&mut self) -> Token {
        let start = self.index - 1;

        while !self.is_eof() {
            if !ident_lookup::TABLE[self.read_byte() as usize] {
                break;
            }

            self.bump();
        }

        let slice = unsafe {
            self.source.slice_unchecked(start, self.index)
        };

        match slice {
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
        }
    }

    #[inline]
    pub fn peek(&mut self) -> Option<&Token> {
        if self.token.is_none() {
            self.token = self.get_token();
        }

        self.token.as_ref()
    }

    #[inline]
    pub fn next(&mut self) -> Option<Token> {
        if self.token.is_some() {
            return self.token.take();
        }

        self.get_token()
    }

    #[inline]
    fn read_operator(&mut self, ch: u8) -> Option<Token> {
        match_operators! { self ch
            b'=' => Assign {
                b'=' => Equality {
                    b'=' => StrictEquality,
                }
                b'>' => FatArrow,
            }
            b'!' => LogicalNot {
                b'=' => Inequality {
                    b'=' => StrictInequality,
                }
            }
            b'<' => Lesser {
                b'<' => BitShiftLeft {
                    b'=' => BSLAssign,
                }
                b'=' => LesserEquals,
            }
            b'>' => Greater {
                b'>' => BitShiftRight {
                    b'>' => UBitShiftRight {
                        b'=' => UBSRAssign,
                    }
                    b'=' => BSRAssign,
                }
                b'=' => GreaterEquals,
            }
            b'?' => Conditional,
            b'~' => BitwiseNot,
            b'^' => BitwiseXor {
                b'=' => BitXorAssign,
            }
            b'&' => BitwiseAnd {
                b'&' => LogicalAnd,
                b'=' => BitAndAssign,
            }
            b'|' => BitwiseOr {
                b'|' => LogicalOr,
                b'=' => BitOrAssign,
            }
            b'+' => Addition {
                b'+' => Increment,
                b'=' => AddAssign,
            }
            b'-' => Substraction {
                b'-' => Decrement,
                b'=' => SubstractAssign,
            }
            b'*' => Multiplication {
                b'*' => Exponent {
                    b'=' => ExponentAssign,
                }
                b'=' => MultiplyAssign,
            }
            b'%' => Remainder {
                b'=' => RemainderAssign,
            }
        }

        None
    }

    fn get_token(&mut self) -> Option<Token> {
        while !self.is_eof() {
            self.token_start = self.index;

            use self::byte_category::*;

            let ch = self.expect_byte();

            match TABLE[ch as usize] {
                WH => continue,
                OP => return self.read_operator(ch),
                ID => return Some(self.read_label()),
                CT => return Some(Control(ch)),
                _  => {},
            }

            return Some(match ch {
                b'"' | b'\'' => {
                    Literal(LiteralString( self.read_string(ch) ))
                },
                b'/' => {
                    if self.is_eof() {
                        Operator(Division)
                    } else {
                        match self.read_byte() {
                            b'/' => {
                                self.bump();
                                self.read_comment();
                                continue;
                            },
                            b'*' => {
                                self.bump();
                                self.read_block_comment();
                                continue;
                            },
                            b'=' => {
                                self.bump();
                                Operator(DivideAssign)
                            },
                            _ => Operator(Division)
                        }
                    }
                },
                b'.' => {
                    if self.is_eof() {
                        Operator(Accessor)
                    } else {
                        match self.read_byte() {
                            b'0'...b'9' => {
                                Literal(self.read_number(b'.'))
                            },
                            b'.' => {
                                self.bump();
                                match self.expect_byte() {
                                    b'.' => Operator(Spread),
                                    ch   => {
                                        panic!("Invalid character `{:?}`", ch);
                                    }
                                }
                            },
                            _ => Operator(Accessor)
                        }
                    }
                },
                b'0'...b'9' => Literal(self.read_number(ch)),
                b' ' | b'\t' | b'\n' | b'\r' => continue,

                _ => {
                    panic!("Invalid character `{:?}`", ch);
                }
            });
        }

        None
    }

    #[inline]
    pub fn consume_whitespace(&mut self) {
        while !self.is_eof() {
            match self.read_byte() {
                b' '  |
                b'\t' |
                b'\n' => self.bump(),
                _     => break,
            }
        }
    }

    #[inline]
    pub fn expect_identifier(&mut self) -> OwnedSlice {
        if self.token.is_some() {
            return match self.token.take() {
                Some(Identifier(ident)) => ident,
                Some(token)             => panic!("Unexpected token `{:?}`", token),
                _                       => unreachable!(),
            };
        }

        self.consume_whitespace();

        let ch = self.expect_byte();

        if byte_category::TABLE[ch as usize] != byte_category::ID {
            panic!("Invalid character `{:?}`", ch);
        }

        match self.read_label() {
            Identifier(ident) => ident,
            token             => panic!("Unexpected token `{:?}`", token)
        }
    }

    #[inline]
    pub fn expect_control(&mut self, byte: u8) {
        let ct = match self.token {
            Some(Control(ct)) => {
                self.token = None;
                ct
            },
            None => {
                self.consume_whitespace();
                self.expect_byte()
            },
            Some(ref token) => panic!("Unexpected token `{:?}`", token)
        };

        if ct != byte {
            panic!("Invalid character `{:?}`", ct);
        }
    }

    #[inline]
    pub fn expect_semicolon(&mut self) {
        if self.token.is_some() {
            self.token = None;
            self.index = self.token_start;
        } else {
            self.consume_whitespace();
        }

        if self.is_eof() {
            return;
        }

        match self.read_byte() {
            b';' => self.bump(),
            b')' |
            b'}' => return,
            ch   => panic!("Unexpected character {:?} {}", ch, self.index)
        }
    }

    #[inline]
    pub fn allow_control(&mut self, byte: u8) -> bool {
        match self.token {
            Some(Control(ct)) => {
                if ct == byte {
                    self.token = None;
                    true
                } else {
                    false
                }
            },
            None => {
                self.consume_whitespace();

                if self.read_byte() == byte {
                    self.bump();
                    true
                } else {
                    false
                }
            },
            _ => false
        }
    }
}
