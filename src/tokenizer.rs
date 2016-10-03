use std::str;
use lexicon::Token;
use lexicon::Token::*;
use lexicon::ReservedKind::*;
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

// Look up table that marks which ASCII characters are allowed in identifiers
const NU: bool = true; // digit
const AL: bool = true; // alphabet
const DO: bool = true; // dollar sign $
const UN: bool = true; // underscore
const __: bool = false;

static IDENT_ALLOWED: [bool; 256] = [
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

static IDENT_START_ALLOWED: [bool; 256] = [
// 0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 0
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 1
  __, __, __, __, DO, __, __, __, __, __, __, __, __, __, __, __, // 2
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
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

pub struct Tokenizer<'a> {
    // Helper buffer for parsing strings that can't be just memcopied from
    // the original source (escaped characters)
    buffer: Vec<u8>,

    // String slice to parse
    source: &'a str,

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
            buffer: Vec::with_capacity(30),
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
    fn read_byte_bump(&mut self) -> u8 {
        if self.is_eof() {
            panic!("Unexpected end of source");
        }

        let ch = self.read_byte();
        self.bump();
        ch
    }

    fn read_string(&mut self, first: u8) -> String {
        self.buffer.clear();
        let mut escape = false;

        while !self.is_eof() {
            let ch = self.read_byte_bump();
            if ch == first && escape == false {
                break;
            }
            match ch {
                b'\\' => {
                    if escape {
                        escape = false;
                        self.buffer.push(ch);
                    } else {
                        escape = true;
                    }
                },
                _ => {
                    self.buffer.push(ch);
                    escape = false;
                },
            }
        }

        unsafe { str::from_utf8_unchecked(self.buffer.as_ref()) }.to_owned()
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

        LiteralValue::float_from_string(&self.source[start .. self.index])
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
            if self.read_byte_bump() == b'*' && self.read_byte_bump() == b'/' {
                return;
            }
        }
    }

    fn read_label(&mut self) -> Token {
        let start = self.index - 1;

        while !self.is_eof() {
            if !IDENT_ALLOWED[self.read_byte() as usize] {
                break;
            }

            self.bump();
        }

        let slice = &self.source[start..self.index];

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
            _            => Identifier(slice.to_owned()),
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

    fn get_token(&mut self) -> Option<Token> {
        while !self.is_eof() {
            self.token_start = self.index;

            let ch = self.read_byte_bump();

            if IDENT_START_ALLOWED[ch as usize] {
                return Some(self.read_label());
            }

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

            return Some(match ch {
                b'\n' => LineTermination,
                b';' => Semicolon,
                b',' => Comma,
                b':' => Colon,
                b'(' => ParenOn,
                b')' => ParenOff,
                b'[' => BracketOn,
                b']' => BracketOff,
                b'{' => BraceOn,
                b'}' => BraceOff,
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
                                match self.read_byte_bump() {
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
                b' ' | b'\t' => continue,

                _ => {
                    panic!("Invalid character `{:?}`", ch);
                }
            });
        }
        return None;
    }

    #[inline]
    fn read_peeked_byte(&mut self) -> u8 {
        unsafe { *self.byte_ptr.offset(self.token_start as isize) }
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

    pub fn expect_identifier(&mut self) -> String {
        if self.token.is_some() {
            self.token = None;
            self.index = self.token_start;
        } else {
            self.consume_whitespace();
        }

        let ch = self.read_byte_bump();

        if !IDENT_START_ALLOWED[ch as usize] {
            panic!("Invalid character `{:?}`", ch);
        }

        match self.read_label() {
            Identifier(string) => string,
            token              => panic!("Unexpected token `{:?}`", token)
        }
    }

    #[inline]
    pub fn expect_str(&mut self, bytes: &str) {
        if self.token.is_some() {
            self.token = None;
            self.index = self.token_start;
        } else {
            self.consume_whitespace();
        }

        let end = self.index + bytes.len();

        if &self.source[self.index..end] != bytes {
            panic!("Invalid character `{:?}`", self.read_peeked_byte());
        }

        self.index = end;
    }

    #[inline]
    pub fn expect_byte(&mut self, byte: u8) {
        if self.token.is_some() {
            self.token = None;
            self.index = self.token_start;
        } else {
            self.consume_whitespace();
        }

        let ch = self.read_byte_bump();

        if ch != byte {
            panic!("Invalid character `{:?}`", ch);
        }
    }

    #[inline]
    pub fn allow_byte(&mut self, byte: u8) -> bool {
        if self.token.is_some() {
            if self.read_peeked_byte() == byte {
                self.token = None;
                true
            } else {
                false
            }
        } else {
            self.consume_whitespace();

            if self.read_byte() == byte {
                self.bump();
                true
            } else {
                false
            }
        }
    }
}
