use std::str;
use std::iter::Iterator;
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

    // Auxiliary index informing when last token begun
    pub token_start: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Tokenizer {
            buffer: Vec::with_capacity(30),
            source: source,
            byte_ptr: source.as_ptr(),
            index: 0,
            token_start: 0,
            length: source.len(),
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

    fn read_string(&mut self, first: u8) -> String {
        self.buffer.clear();
        let mut escape = false;

        while !self.is_eof() {
            let ch = self.expect_byte();
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
                b'0' | b'1' => {
                    value <<= 1;
                    if peek == b'1' {
                        value += 1;
                    }
                    self.bump();
                },
                _ => return LiteralInteger(value),
            }
        }

        return LiteralInteger(value);
    }

    fn read_octal(&mut self) -> LiteralValue {
        let mut value = 0;

        while !self.is_eof() {
            let peek = self.read_byte();

            let digit = match peek {
                b'0' => 0,
                b'1' => 1,
                b'2' => 2,
                b'3' => 3,
                b'4' => 4,
                b'5' => 5,
                b'6' => 6,
                b'7' => 7,
                _    => -1,
            };
            if digit == -1 {
                return LiteralInteger(value);
            } else {
                value <<= 3;
                value += digit;
                self.bump();
            }
        }

        return LiteralInteger(value);
    }

    fn read_hexadec(&mut self) -> LiteralValue {
        let mut value = 0;

        while !self.is_eof() {
            let peek = self.read_byte();
            let digit = match peek {
                b'0'        => 0,
                b'1'        => 1,
                b'2'        => 2,
                b'3'        => 3,
                b'4'        => 4,
                b'5'        => 5,
                b'6'        => 6,
                b'7'        => 7,
                b'8'        => 8,
                b'9'        => 9,
                b'a' | b'A' => 10,
                b'b' | b'B' => 11,
                b'c' | b'C' => 12,
                b'd' | b'D' => 13,
                b'e' | b'E' => 14,
                b'f' | b'F' => 15,
                _           => -1,
            };
            if digit == -1 {
                return LiteralInteger(value);
            } else {
                value <<= 4;
                value += digit;
                self.bump();
            }
        }

        return LiteralInteger(value);
    }

    fn read_number(&mut self, first: u8) -> LiteralValue {
        self.buffer.clear();
        self.buffer.push(first);
        let mut period = false;

        if first == b'.' {
            period = true;
        } else if first == b'0' {
            if !self.is_eof() {
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
        }

        while !self.is_eof() {
            let ch = self.read_byte();
            match ch {
                b'0'...b'9' => {
                    self.buffer.push(ch);
                    self.bump();
                },
                b'.' => {
                    if !period {
                        period = true;
                        self.buffer.push(ch);
                        self.bump();
                    } else {
                        return LiteralValue::float_from_string(&self.buffer);
                    }
                },
                _ => return LiteralValue::float_from_string(&self.buffer),
            }
        }

        return LiteralValue::float_from_string(&self.buffer);
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
            match self.read_byte() {
                b'a'...b'z' |
                b'A'...b'Z' |
                b'0'...b'9' |
                b'_' | b'$' => self.bump(),
                _ => break
            }
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
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        while !self.is_eof() {
            self.token_start = self.index;

            let ch = self.expect_byte();

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
                b'a'...b'z' |
                b'A'...b'Z' |
                b'$' | b'_' => self.read_label(),

                b' ' | b'\t' => continue,

                _ => {
                    panic!("Invalid character `{:?}`", ch);
                }
            });
        }
        return None;
    }
}
