use std::str;
use std::str::Bytes;
use std::iter::{ Peekable, Iterator };
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
        match $tokenizer.source.peek() {
            $(
                Some(&$secondary) => {
                    $tokenizer.source.next();
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
    source: Peekable<Bytes<'a>>,
    buffer: Vec<u8>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Tokenizer {
            source: source.bytes().peekable(),
            buffer: Vec::with_capacity(128),
        }
    }

    fn read_string(&mut self, first: u8) -> String {
        self.buffer.clear();
        let mut escape = false;

        while let Some(ch) = self.source.next() {
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

        while let Some(&peek) = self.source.peek() {
            match peek {
                b'0' | b'1' => {
                    value <<= 1;
                    if peek == b'1' {
                        value += 1;
                    }
                    self.source.next();
                },
                _ => return LiteralInteger(value),
            }
        }

        return LiteralInteger(value);
    }

    fn read_octal(&mut self) -> LiteralValue {
        let mut value = 0;

        while let Some(&peek) = self.source.peek() {
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
                self.source.next();
            }
        }

        return LiteralInteger(value);
    }

    fn read_hexadec(&mut self) -> LiteralValue {
        let mut value = 0;

        while let Some(&peek) = self.source.peek() {
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
                self.source.next();
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
            if let Some(&peek) = self.source.peek() {
                if peek == b'b' {
                    self.source.next();
                    return self.read_binary();
                } else if peek == b'o' {
                    self.source.next();
                    return self.read_octal();
                } else if peek == b'x' {
                    self.source.next();
                    return self.read_hexadec();
                }
            }
        }

        while let Some(&ch) = self.source.peek() {
            match ch {
                b'0'...b'9' => {
                    self.buffer.push(ch);
                    self.source.next();
                },
                b'.' => {
                    if !period {
                        period = true;
                        self.buffer.push(ch);
                        self.source.next();
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
        while let Some(&ch) = self.source.peek() {
            if ch == b'\n' {
                return;
            }
            self.source.next();
        }
    }

    fn read_block_comment(&mut self) {
        let mut asterisk = false;

        while let Some(ch) = self.source.next() {
            if ch == b'/' && asterisk {
                return;
            }
            if ch == b'*' {
                asterisk = true;
            }
        }
    }

    fn read_label(&mut self, first: u8) -> Token {
        self.buffer.clear();
        self.buffer.push(first);

        while let Some(&ch) = self.source.peek() {
            match ch {
                b'a'...b'z' |
                b'A'...b'Z' |
                b'0'...b'9' |
                b'_' | b'$' => {
                    self.buffer.push(ch);
                    self.source.next();
                }
                _ => break
            }
        }

        let slice: &[u8] = &self.buffer;

        match slice {
            b"new"        => Operator(New),
            b"typeof"     => Operator(Typeof),
            b"delete"     => Operator(Delete),
            b"void"       => Operator(Void),
            b"in"         => Operator(In),
            b"instanceof" => Operator(Instanceof),
            b"var"        => Declaration(Var),
            b"let"        => Declaration(Let),
            b"const"      => Declaration(Const),
            b"break"      => Break,
            b"do"         => Do,
            b"case"       => Case,
            b"else"       => Else,
            b"catch"      => Catch,
            b"export"     => Export,
            b"class"      => Class,
            b"extends"    => Extends,
            b"return"     => Return,
            b"while"      => While,
            b"finally"    => Finally,
            b"super"      => Super,
            b"with"       => With,
            b"continue"   => Continue,
            b"for"        => For,
            b"switch"     => Switch,
            b"yield"      => Yield,
            b"debugger"   => Debugger,
            b"function"   => Function,
            b"this"       => This,
            b"default"    => Default,
            b"if"         => If,
            b"throw"      => Throw,
            b"import"     => Import,
            b"try"        => Try,
            b"await"      => Await,
            b"static"     => Static,
            b"true"       => Literal(LiteralTrue),
            b"false"      => Literal(LiteralFalse),
            b"undefined"  => Literal(LiteralUndefined),
            b"null"       => Literal(LiteralNull),
            b"enum"       => Reserved(Enum),
            b"implements" => Reserved(Implements),
            b"package"    => Reserved(Package),
            b"protected"  => Reserved(Protected),
            b"interface"  => Reserved(Interface),
            b"private"    => Reserved(Private),
            b"public"     => Reserved(Public),
            _             => Identifier(unsafe { str::from_utf8_unchecked(slice) }.to_owned()),
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        'lex: while let Some(ch) = self.source.next() {

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
                    match self.source.peek() {
                        Some(&b'/') => {
                            self.source.next();
                            self.read_comment();
                            continue 'lex
                        },
                        Some(&b'*') => {
                            self.source.next();
                            self.read_block_comment();
                            continue 'lex
                        },
                        Some(&b'=') => {
                            self.source.next();
                            Operator(DivideAssign)
                        },
                        _ => Operator(Division)
                    }
                },
                b'.' => {
                    match self.source.peek() {
                        Some(&b'0'...b'9') => {
                            Literal(self.read_number(b'.'))
                        },
                        Some(&b'.') => {
                            self.source.next();
                            match self.source.next() {
                                Some(b'.') => Operator(Spread),
                                ch        => {
                                    panic!("Invalid character `{:?}`", ch);
                                }
                            }
                        },
                        _ => Operator(Accessor)
                    }
                },
                b'0'...b'9' => Literal(self.read_number(ch)),
                _ => {
                    match ch {
                        b'a'...b'z' |
                        b'A'...b'Z' |
                        b'$' | b'_' => {
                            self.read_label(ch)
                        },

                        b' ' | b'\t' => continue 'lex,

                        _ => {
                            panic!("Invalid character `{:?}`", ch);
                        }
                    }
                }
            });
        }
        return None;
    }
}
