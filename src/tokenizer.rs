use std::str::Chars;
use std::iter::{ Peekable, Iterator };
use lexicon::Token;
use lexicon::Token::*;
use lexicon::KeywordKind::*;
use lexicon::ReservedKind::*;
use grammar::OperatorType;
use grammar::OperatorType::*;
use grammar::LiteralValue;
use grammar::LiteralValue::*;

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
            match ch {
                'a'...'z' | 'A'...'Z' | '0'...'9' | '_' | '$' => {
                    label.push(ch);
                    self.source.next();
                },
                _ => {
                    return label;
                },
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

        if first == '0' {
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

        while let Some(&ch) = self.source.peek() {
            if ch == '/' && asterisk {
                self.source.next();
                return;
            }
            if ch == '*' {
                asterisk = true;
                self.source.next();
                continue;
            }
            self.source.next();
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        'lex: while let Some(ch) = self.source.next() {
            return Some(match ch {
                '\n' => LineTermination,
                ';' => Semicolon,
                ',' => Comma,
                ':' => Colon,
                '(' => ParenOn,
                ')' => ParenOff,
                '[' => BracketOn,
                ']' => BracketOff,
                '{' => BlockOn,
                '}' => BlockOff,
                '<' => {
                    let comp_type = if Some(&'=') == self.source.peek() {
                        self.source.next();
                        LesserEquals
                    } else {
                        Lesser
                    };
                    Operator(comp_type)
                },
                '>' => {
                    let comp_type = if Some(&'=') == self.source.peek() {
                        self.source.next();
                        GreaterEquals
                    } else {
                        Greater
                    };
                    Operator(comp_type)
                },
                '.' => Operator(Accessor),
                '"' | '\'' => {
                    Literal(LiteralString( self.read_string(ch) ))
                },
                '=' => {
                    if Some(&'>') == self.source.peek() {
                        self.source.next();
                        FatArrow
                    } else {
                        if Some(&'=') == self.source.peek() {
                            self.source.next();
                            if Some(&'=') == self.source.peek() {
                                self.source.next();
                                Operator(StrictEquality)
                            } else {
                                Operator(Equality)
                            }
                        } else {
                            Operator(Assign)
                        }
                    }
                },
                '!' => {
                    if Some(&'=') == self.source.peek() {
                        self.source.next();
                        if Some(&'=') == self.source.peek() {
                            self.source.next();
                            Operator(StrictInequality)
                        } else {
                            Operator(Inequality)
                        }
                    } else {
                        Operator(LogicalNot)
                    }
                },
                '+' => match self.source.peek() {
                    Some(&'+') => {
                        self.source.next();
                        Operator(Increment)
                    },
                    _          => Operator(Add)
                },
                '-' => match self.source.peek() {
                    Some(&'-') => {
                        self.source.next();
                        Operator(Decrement)
                    }
                    _          => Operator(Substract)
                },
                '/' => {
                    if Some(&'/') == self.source.peek() {
                        self.source.next();
                        self.read_comment();
                        continue 'lex;
                    }
                    if Some(&'*') == self.source.peek() {
                        self.source.next();
                        self.read_block_comment();
                        continue 'lex;
                    }
                    Operator(Divide)
                }
                '*' => {
                    if Some(&'*') == self.source.peek() {
                        self.source.next();
                        Operator(Exponent)
                    } else {
                        Operator(Multiply)
                    }
                },
                '%' => return Some(Operator(Modulo)),
                '0'...'9' => Literal(self.read_number(ch)),
                'a'...'z' | 'A'...'Z' | '$' | '_' => {
                    let label = self.read_label(ch);
                    match label.as_ref() {
                        "new"        => Operator(New),
                        "typeof"     => Operator(Typeof),
                        "delete"     => Operator(Delete),
                        "void"       => Operator(Void),
                        "in"         => Operator(In),
                        "instanceof" => Operator(Instanceof),
                        "break"      => Keyword(Break),
                        "do"         => Keyword(Do),
                        "case"       => Keyword(Case),
                        "else"       => Keyword(Else),
                        "var"        => Keyword(Var),
                        "let"        => Keyword(Let),
                        "catch"      => Keyword(Catch),
                        "export"     => Keyword(Export),
                        "class"      => Keyword(Class),
                        "extends"    => Keyword(Extends),
                        "return"     => Keyword(Return),
                        "while"      => Keyword(While),
                        "const"      => Keyword(Const),
                        "finally"    => Keyword(Finally),
                        "super"      => Keyword(Super),
                        "with"       => Keyword(With),
                        "continue"   => Keyword(Continue),
                        "for"        => Keyword(For),
                        "switch"     => Keyword(Switch),
                        "yield"      => Keyword(Yield),
                        "debugger"   => Keyword(Debugger),
                        "function"   => Keyword(Function),
                        "this"       => Keyword(This),
                        "default"    => Keyword(Default),
                        "if"         => Keyword(If),
                        "throw"      => Keyword(Throw),
                        "import"     => Keyword(Import),
                        "try"        => Keyword(Try),
                        "await"      => Keyword(Await),
                        "static"     => Keyword(Static),
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
                },
                _         => continue 'lex,
            });
        }
        return None;
    }
}
