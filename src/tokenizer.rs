use std::str::Chars;
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
    source: Peekable<Chars<'a>>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a String) -> Self {
        Tokenizer {
            source: source.chars().peekable(),
        }
    }

    fn read_label(&mut self, first: char) -> String {
        let mut label = first.to_string();

        while let Some(&ch) = self.source.peek() {
            if ch.is_alphanumeric() || ch == '_' || ch == '$' {
                label.push(ch);
                self.source.next();
            } else {
                return label;
            }
        }

        return label;
    }

    fn read_string(&mut self, first: char) -> String {
        let mut value = String::new();
        let mut escape = false;

        while let Some(ch) = self.source.next() {
            if ch == first && escape == false {
                return value;
            }
            match ch {
                '\\' => {
                    if escape {
                        escape = false;
                        value.push(ch);
                    } else {
                        escape = true;
                    }
                },
                _ => {
                    value.push(ch);
                    escape = false;
                },
            }
        }

        return value;
    }

    fn read_binary(&mut self) -> LiteralValue {
        let mut value = 0;

        while let Some(&peek) = self.source.peek() {
            match peek {
                '0' | '1' => {
                    value <<= 1;
                    if peek == '1' {
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
                '0' => 0,
                '1' => 1,
                '2' => 2,
                '3' => 3,
                '4' => 4,
                '5' => 5,
                '6' => 6,
                '7' => 7,
                _   => -1,
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
                '0'       => 0,
                '1'       => 1,
                '2'       => 2,
                '3'       => 3,
                '4'       => 4,
                '5'       => 5,
                '6'       => 6,
                '7'       => 7,
                '8'       => 8,
                '9'       => 9,
                'a' | 'A' => 10,
                'b' | 'B' => 11,
                'c' | 'C' => 12,
                'd' | 'D' => 13,
                'e' | 'E' => 14,
                'f' | 'F' => 15,
                _         => -1,
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

    fn read_number(&mut self, first: char) -> LiteralValue {
        let mut value = first.to_string();
        let mut period = false;

        if first == '.' {
            period = true;
        } else if first == '0' {
            if let Some(&peek) = self.source.peek() {
                if peek == 'b' {
                    self.source.next();
                    return self.read_binary();
                } else if peek == 'o' {
                    self.source.next();
                    return self.read_octal();
                } else if peek == 'x' {
                    self.source.next();
                    return self.read_hexadec();
                }
            }
        }

        while let Some(&ch) = self.source.peek() {
            match ch {
                '0'...'9' => {
                    value.push(ch);
                    self.source.next();
                },
                '.' => {
                    if !period {
                        period = true;
                        value.push(ch);
                        self.source.next();
                    } else {
                        return LiteralValue::float_from_string(value);
                    }
                },
                _ => return LiteralValue::float_from_string(value),
            }
        }

        return LiteralValue::float_from_string(value);
    }

    fn read_comment(&mut self) {
        while let Some(&ch) = self.source.peek() {
            if ch == '\n' {
                return;
            }
            self.source.next();
        }
    }

    fn read_block_comment(&mut self) {
        let mut asterisk = false;

        while let Some(ch) = self.source.next() {
            if ch == '/' && asterisk {
                return;
            }
            if ch == '*' {
                asterisk = true;
            }
        }
    }

    fn label_to_token(&mut self, label: String) -> Token {
        match label.as_ref() {
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
            _            => Identifier(label),
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        'lex: while let Some(ch) = self.source.next() {

            match_operators! { self ch
                '=' => Assign {
                    '=' => Equality {
                        '=' => StrictEquality,
                    }
                    '>' => FatArrow,
                }
                '!' => LogicalNot {
                    '=' => Inequality {
                        '=' => StrictInequality,
                    }
                }
                '<' => Lesser {
                    '<' => BitShiftLeft {
                        '=' => BSLAssign,
                    }
                    '=' => LesserEquals,
                }
                '>' => Greater {
                    '>' => BitShiftRight {
                        '>' => UBitShiftRight {
                            '=' => UBSRAssign,
                        }
                        '=' => BSRAssign,
                    }
                    '=' => GreaterEquals,
                }
                '?' => Conditional,
                '~' => BitwiseNot,
                '^' => BitwiseXor {
                    '=' => BitXorAssign,
                }
                '&' => BitwiseAnd {
                    '&' => LogicalAnd,
                    '=' => BitAndAssign,
                }
                '|' => BitwiseOr {
                    '|' => LogicalOr,
                    '=' => BitOrAssign,
                }
                '+' => Addition {
                    '+' => Increment,
                    '=' => AddAssign,
                }
                '-' => Substraction {
                    '-' => Decrement,
                    '=' => SubstractAssign,
                }
                '*' => Multiplication {
                    '*' => Exponent {
                        '=' => ExponentAssign,
                    }
                    '=' => MultiplyAssign,
                }
                '%' => Remainder {
                    '=' => RemainderAssign,
                }
            }

            return Some(match ch {
                '\n' => LineTermination,
                ';' => Semicolon,
                ',' => Comma,
                ':' => Colon,
                '(' => ParenOn,
                ')' => ParenOff,
                '[' => BracketOn,
                ']' => BracketOff,
                '{' => BraceOn,
                '}' => BraceOff,
                '"' | '\'' => {
                    Literal(LiteralString( self.read_string(ch) ))
                },
                '/' => {
                    match self.source.peek() {
                        Some(&'/') => {
                            self.source.next();
                            self.read_comment();
                            continue 'lex
                        },
                        Some(&'*') => {
                            self.source.next();
                            self.read_block_comment();
                            continue 'lex
                        },
                        Some(&'=') => {
                            self.source.next();
                            Operator(DivideAssign)
                        },
                        _ => Operator(Division)
                    }
                },
                '.' => {
                    match self.source.peek() {
                        Some(&'0'...'9') => {
                            Literal(self.read_number('.'))
                        },
                        Some(&'.') => {
                            self.source.next();
                            match self.source.next() {
                                Some('.') => Operator(Spread),
                                ch        => {
                                    panic!("Invalid character `{:?}`", ch);
                                }
                            }
                        },
                        _ => Operator(Accessor)
                    }
                },
                '0'...'9' => Literal(self.read_number(ch)),
                _ => {
                    if ch.is_alphabetic() || ch == '$' || ch == '_' {
                       let label = self.read_label(ch);
                        self.label_to_token(label)
                    } else if ch.is_whitespace() {
                        continue 'lex
                    } else {
                        panic!("Invalid character `{:?}`", ch);
                    }
                }
            });
        }
        return None;
    }
}
