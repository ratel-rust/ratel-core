use std::str::Chars;
use std::iter::{ Peekable, Iterator };
use lexicon::Token;
use lexicon::Token::*;
use lexicon::KeywordKind::*;
use lexicon::ReservedKind::*;
use lexicon::CompareKind::*;
use lexicon::OperatorKind::*;

pub struct Tokenizer<'a> {
    source: Peekable<Chars<'a>>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a mut String) -> Self {
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

    fn read_binary(&mut self) -> f64 {
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
                _ => return value as f64,
            }
        }

        return value as f64;
    }

    fn read_octal(&mut self) -> f64 {
        let mut value = 0;

        while let Some(&peek) = self.source.peek() {
            match peek {
                '0' ... '7' => {
                    value <<= 3;
                    value += match peek {
                        '1' => 1,
                        '2' => 2,
                        '3' => 3,
                        '4' => 4,
                        '5' => 5,
                        '6' => 6,
                        '7' => 7,
                        _   => 0,
                    };
                    self.source.next();
                },
                _ => return value as f64,
            }
        }

        return value as f64;
    }

    fn read_hexadec(&mut self) -> f64 {
        let mut value = 0;

        while let Some(&peek) = self.source.peek() {
            match peek {
                '0' ... '9' | 'a'...'f' | 'A'...'F' => {
                    value <<= 4;
                    value += match peek {
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
                        _         => 0,
                    };
                    self.source.next();
                },
                _ => return value as f64,
            }
        }

        return value as f64;
    }

    fn read_number(&mut self, first: char) -> f64 {
        let mut strvalue = first.to_string();
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
                    strvalue.push(ch);
                    self.source.next();
                },
                '.' => {
                    if !period {
                        period = true;
                        strvalue.push(ch);
                        self.source.next();
                    } else {
                        return strvalue.parse().unwrap_or(0.0);
                    }
                },
                _ => return strvalue.parse().unwrap_or(0.0),
            }
        }

        return 0.0;
    }

    fn read_comment(&mut self) -> String {
        let mut comment = String::new();

        while let Some(&ch) = self.source.peek() {
            if ch == '\n' {
                return comment;
            }
            comment.push(ch);
            self.source.next();
        }

        return comment;
    }

    fn read_block_comment(&mut self) -> String {
        let mut comment = String::new();
        let mut asterisk = false;

        while let Some(&ch) = self.source.peek() {
            if ch == '/' && asterisk {
                self.source.next();
                return comment;
            }
            if ch == '*' {
                asterisk = true;
                self.source.next();
                continue;
            }
            if asterisk {
                comment.push('*');
            }
            comment.push(ch);
            self.source.next();
        }

        return comment;
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        while let Some(ch) = self.source.next() {
            match ch {
                '\n' => return Some(LineTermination),
                ';' => return Some(Semicolon),
                ',' => return Some(Comma),
                ':' => return Some(Colon),
                '(' => return Some(ParenOn),
                ')' => return Some(ParenOff),
                '[' => return Some(BracketOn),
                ']' => return Some(BracketOff),
                '{' => return Some(BlockOn),
                '}' => return Some(BlockOff),
                '<' => {
                    let comp_type = if Some(&'=') == self.source.peek() {
                        self.source.next();
                        LesserEquals
                    } else {
                        Lesser
                    };
                    return Some(Compare(comp_type));
                },
                '>' => {
                    let comp_type = if Some(&'=') == self.source.peek() {
                        self.source.next();
                        GreaterEquals
                    } else {
                        Greater
                    };
                    return Some(Compare(comp_type));
                },
                '.' => return Some(Accessor),
                '"' | '\'' => {
                    return Some(LiteralString( self.read_string(ch) ));
                },
                '=' => {
                    if Some(&'>') == self.source.peek() {
                        self.source.next();
                        return Some(FatArrow);
                    }
                    if Some(&'=') == self.source.peek() {
                        self.source.next();
                        if Some(&'=') == self.source.peek() {
                            self.source.next();
                            return Some(Compare(Is));
                        }
                        return Some(Compare(Equals));
                    }
                    return Some(Assign);
                },
                '!' => {
                    if Some(&'=') == self.source.peek() {
                        self.source.next();
                        if Some(&'=') == self.source.peek() {
                            self.source.next();
                            return Some(Compare(Isnt));
                        }
                        return Some(Compare(NotEquals));
                    }
                    return Some(Operator(Not));
                },
                '+' => return Some(Operator(Add)),
                '-' => return Some(Operator(Substract)),
                '/' => {
                    if Some(&'/') == self.source.peek() {
                        self.source.next();
                        return Some(Comment(self.read_comment()));
                    }
                    if Some(&'*') == self.source.peek() {
                        self.source.next();
                        return Some(BlockComment(self.read_block_comment()));
                    }
                    return Some(Operator(Divide));
                }
                '*' => {
                    if Some(&'*') == self.source.peek() {
                        self.source.next();
                        return Some(Operator(Exponent));
                    }
                    return Some(Operator(Multiply));
                },
                '%' => return Some(Operator(Modulo)),
                'a'...'z' | 'A'...'Z' | '$' | '_' => {
                    let label = self.read_label(ch);
                    return Some(match label.as_ref() {
                        "break"      => Keyword(Break),
                        "do"         => Keyword(Do),
                        "in"         => Keyword(In),
                        "typeof"     => Keyword(Typeof),
                        "case"       => Keyword(Case),
                        "else"       => Keyword(Else),
                        "instanceof" => Keyword(Instanceof),
                        "var"        => Keyword(Var),
                        "let"        => Keyword(Let),
                        "catch"      => Keyword(Catch),
                        "export"     => Keyword(Export),
                        "new"        => Keyword(New),
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
                        "delete"     => Keyword(Delete),
                        "import"     => Keyword(Import),
                        "try"        => Keyword(Try),
                        "void"       => Keyword(Void),
                        "async"      => Keyword(Async),
                        "await"      => Keyword(Await),
                        "true"       => LiteralTrue,
                        "false"      => LiteralFalse,
                        "undefined"  => LiteralUndefined,
                        "null"       => LiteralNull,
                        "enum"       => Reserved(Enum),
                        "implements" => Reserved(Implements),
                        "package"    => Reserved(Package),
                        "protected"  => Reserved(Protected),
                        "interface"  => Reserved(Interface),
                        "private"    => Reserved(Private),
                        "public"     => Reserved(Public),
                        _            => Identifier(label),
                    })
                },
                '0'...'9' => {
                    let number = self.read_number(ch);
                    return Some(LiteralNumber(number));
                },
                _ => {},
            }
        }
        return None;
    }
}
