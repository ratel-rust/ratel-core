use std::io::prelude::*;
use std::io::Error;
use std::fs::File;
use std::iter::{ Peekable, Iterator };
use std::string::Drain;

#[derive(Debug)]
enum KeywordType {
    Var,
    Let,
    Const,
    Return,
    Function,
    If,
    Else,
    Import,
    Export,
    Default,
    Delete,
    Class,
    Extends,
}

#[derive(Debug)]
enum CompareType {
    Is,             // ===
    Isnt,           // !==
    Equals,         // ==
    NotEquals,      // !=
    Lesser,         // <
    LesserEquals,   // <=
    Greater,        // >
    GreaterEquals,  // >=
}

#[derive(Debug)]
enum OperatorType {
    Add,        // +
    Substract,  // -
    Multiply,   // *
    Divide,     // /
    Modulo,     // %
    Exponent,   // **
    Void,       // void
    Not         // !
}

#[derive(Debug)]
enum Token {
    Semicolon,
    Comma,
    Colon,
    Accessor,  // .
    Compare(CompareType),
    Operator(OperatorType),
    Assign,
    ParenOn,
    ParenOff,
    BracketOn,
    BracketOff,
    BlockOn,
    BlockOff,
    FatArrow, // =>
    Keyword(KeywordType),
    Identifier(String),
    ValueTrue,
    ValueFalse,
    ValueUndefined,
    ValueNull,
    ValueNumber(f64),
    ValueString(String),
    Comment(String),
    BlockComment(String),
}

use CompareType::*;
use OperatorType::*;
use KeywordType::*;
use Token::*;

struct Tokenizer<'a> {
    source: Peekable<Drain<'a>>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a mut String) -> Self {
        Tokenizer {
            source: source.drain(..).peekable(),
        }
    }

    fn read_label(&mut self, first: char) -> String {
        let mut label = first.to_string();

        while let Some(&ch) = self.source.peek() {
            match ch {
                'a' ... 'z' | 'A' ... 'Z' | '0' ... '9' | '_' | '$' => {
                    label.push(ch);
                    self.source.next();
                }
                _ => {
                    return label;
                }
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
                }
            }
        }

        return value;
    }

    fn read_binary(&mut self) -> f64 {
        return 0.0;
    }

    fn read_octal(&mut self) -> f64 {
        return 0.0;
    }

    fn read_hexadec(&mut self) -> f64 {
        return 0.0;
    }

    fn read_number(&mut self, first: char) -> f64 {
        let mut strvalue = first.to_string();
        let mut period = false;

        if first == '0' {
            if let Some(&peek) = self.source.peek() {
                if peek == 'b' {
                    return self.read_binary();
                } else if peek == 'o' {
                    return self.read_octal();
                } else if peek == 'x' {
                    return self.read_hexadec();
                }
            }
        }

        while let Some(&ch) = self.source.peek() {
            match ch {
                '0' ... '9' => {
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
                ';' => return Some(Semicolon),
                ',' => return Some(Comma),
                ':' => return Some(Colon),
                '.' => return Some(Accessor),
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
                '"' => return Some(ValueString( self.read_string(ch) )),
                '\'' => return Some(ValueString( self.read_string(ch) )),
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
                'a' ... 'z' | 'A' ... 'Z' | '$' | '_' => {
                    let label = self.read_label(ch);
                    return Some(match label.as_ref() {
                        "var"       => Keyword(Var),
                        "let"       => Keyword(Let),
                        "const"     => Keyword(Const),
                        "return"    => Keyword(Return),
                        "function"  => Keyword(Function),
                        "if"        => Keyword(If),
                        "else"      => Keyword(Else),
                        "import"    => Keyword(Import),
                        "export"    => Keyword(Export),
                        "default"   => Keyword(Default),
                        "delete"    => Keyword(Delete),
                        "class"     => Keyword(Class),
                        "extends"   => Keyword(Extends),
                        "true"      => ValueTrue,
                        "false"     => ValueFalse,
                        "undefined" => ValueUndefined,
                        "null"      => ValueNull,
                        "void"      => Operator(Void),
                        _           => Identifier(label),
                    })
                },
                '0' ... '9' => {
                    let number = self.read_number(ch);
                    return Some(ValueNumber( number ));
                },
                _ => {},
            }
        }
        return None;
    }
}

fn parse_file(path: &str) -> Result<(), Error> {
    let mut f = try!(File::open(path));
    let mut s = String::new();
    try!(f.read_to_string(&mut s));
    let mut tokenizer = Tokenizer::new(&mut s);
    while let Some(token) = tokenizer.next() {
        println!("{:?}", token);
    }
    Ok(())
}

fn main() {
    parse_file("test.js").unwrap();
}
