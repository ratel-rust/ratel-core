use lexicon::Token;
use lexicon::Token::*;
use lexicon::KeywordKind::*;
use lexicon::ReservedKind::*;
use lexicon::CompareKind::*;
use lexicon::OperatorKind::*;
use tokenizer::Tokenizer;
use std::iter::Peekable;
use literals::*;

/// Expects next token to match `$p`, otherwise panics.
macro_rules! expect {
    ($parser:ident, $p:pat) => {
        match $parser.next() {
            Some($p) => {},
            None     => panic!("Unexpected end of program"),
            token    => panic!("Unexpected token {:?}", token),
        }
    }
}

/// If the next token matches `$p`, consume that token and return
/// true, else do nothing and return false
macro_rules! allow {
    ($parser:ident, $p:pat) => {
        match $parser.tokenizer.peek() {
            Some(&$p) => {
                $parser.next();
                true
            },
            _ => false
        }
    }
}

/// Allow multiple occurences of `$p` in a row, consuming them all,
/// returns true if at least one was found
macro_rules! allow_many {
    ($parser:ident, $p:pat) => {
        if allow!($parser, $p) {
            while allow!($parser, $p) {}
            true
        } else {
            false
        }
    }
}

/// More robust version of the regular `match`, will peek at the next
/// token, if the token matches `$p` then consume that token, any line
/// breaks after and call the `$then` expression.
macro_rules! on {
    ($parser:ident, { $( $p:pat => $then:expr ),* }) => ({
        allow_many!($parser, LineTermination);
        match $parser.tokenizer.peek() {
            $(
                Some(&$p) => {
                    $parser.next();
                    allow_many!($parser, LineTermination);
                    $then
                }
            )*
            _ => {}
        }
    })
}

/// Expects a semicolon to end the statement and return `true`. If no
/// semicolon is found, but `LimeTermination` occured, invoke
/// automatic semicolon injection. Additionally a `$cont` token can
/// be specified, if met the macro will stop and return `false`.
macro_rules! expect_statement_end {
    ($parser:ident, $cont:pat) => ({
        allow_many!($parser, LineTermination);

        let asi = $parser.allow_asi;
        let end = match $parser.tokenizer.peek() {
            Some(&Semicolon) => true,
            Some(&$cont)     => false,
            token            => asi || panic!("Unexpected token {:?}", token)
        };

        if !asi {
            $parser.next();
        }

        end
    });

    ($parser:ident) => ({
        allow_many!($parser, LineTermination);

        let asi = $parser.allow_asi;
        match $parser.tokenizer.peek() {
            Some(&Semicolon) => true,
            token            => asi || panic!("Unexpected token {:?}", token)
        };

        if !asi {
            $parser.next();
        }

        true
    })
}

/// Read a list of items with predefined `$start`, `$end` and
/// `$separator` tokens and an `$item` expression that is then
/// pushed onto a vector.
macro_rules! expect_list {
    ($parser:ident, $start:pat, $item:expr, $separator:pat, $end:pat) => ({
        let mut list = Vec::new();

        expect!($parser, $start);
        loop {
            allow_many!($parser, LineTermination);
            if allow!($parser, $end) {
                break;
            }
            list.push($item);
            allow_many!($parser, LineTermination);
            if expect_list_end!($parser, $separator, $end) {
                break;
            } else {
                continue;
            }
        }

        list
    })
}

/// Shorthand for reading a key expression, separator token and
/// value expression in that order.
macro_rules! expect_key_value_pair {
    ($parser:ident, $key:expr, $separator:pat, $value:expr) => ({
        allow_many!($parser, LineTermination);
        let key = $key;
        allow_many!($parser, LineTermination);
        expect!($parser, $separator);
        allow_many!($parser, LineTermination);
        (key, $value)
    })
}

/// Returns true if met with a list closing token `$p`, allows
/// a tailing comma to appear before `$p`.
macro_rules! expect_list_end {
    ($parser:ident, $separator:pat, $end:pat) => {
        match $parser.next() {
            Some($separator) => {
                allow_many!($parser, LineTermination);
                allow!($parser, $end)
            }
            Some($end)       => true,
            _                => false,
        }
    }
}

type PT<'a> = Peekable<Tokenizer<'a>>;

#[derive(Debug)]
pub enum VariableDeclarationKind {
    Var,
    Let,
    Const,
}

#[derive(Debug)]
pub enum ObjectKey {
    Static(String),
    Computed(Expression),
}

#[derive(Debug)]
pub enum Expression {
    Variable(String),
    Literal(LiteralValue),
    Array(Vec<Expression>),
    Object(Vec<(ObjectKey, Expression)>),
    Member {
        object: Box<Expression>,
        property: Box<ObjectKey>,
    },
    MethodCall {
        object: Box<Expression>,
        method: Box<ObjectKey>,
        arguments: Vec<Expression>,
    },
    Addition {
        left: Box<Expression>,
        right: Box<Expression>,
    }
}

#[derive(Debug)]
pub struct Program {
    body: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    VariableDeclaration {
        kind: VariableDeclarationKind,
        declarations: Vec<(String, Expression)>,
    },
    Expression(Expression),
    Comment(String),
    BlockComment(String),
}

pub struct Parser<'a> {
    tokenizer: Peekable<Tokenizer<'a>>,
    allow_asi: bool,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a String) -> Self {
        Parser {
            tokenizer: Tokenizer::new(source).peekable(),
            allow_asi: false,
        }
    }

    pub fn next(&mut self) -> Option<Token> {
        let token = self.tokenizer.next();

        match token {
            Some(LineTermination) => self.allow_asi = true,
            _                     => self.allow_asi = false,
        }

        println!("Consume {:?}", token);

        return token;
    }

    pub fn array(&mut self) -> Expression {
        Expression::Array(expect_list!(self,
            BracketOn,
            self.expression(),
            Comma,
            BracketOff
        ))
    }

    pub fn object(&mut self) -> Expression {
        Expression::Object(expect_list!(self,
            BlockOn,
            expect_key_value_pair!(self,
                self.object_key(),
                Colon,
                self.expression()
            ),
            Comma,
            BlockOff
        ))
    }

    pub fn object_key(&mut self) -> ObjectKey {
        match self.next() {
            Some(Identifier(key)) | Some(Literal(LiteralString(key))) => {
                ObjectKey::Static(key)
            },
            Some(BracketOn) => {
                allow_many!(self, LineTermination);
                let key = self.expression();
                allow_many!(self, LineTermination);
                expect!(self, BracketOff);
                ObjectKey::Computed(key)
            },
            token => {
                panic!("Expected object key, got {:?}", token)
            }
        }
    }

    pub fn object_property(&mut self) -> ObjectKey {
        match self.next() {
            Some(Identifier(property)) => {
                ObjectKey::Static(property)
            },
            Some(BracketOn) => {
                allow_many!(self, LineTermination);
                let property = self.expression();
                allow_many!(self, LineTermination);
                expect!(self, BracketOff);
                ObjectKey::Computed(property)
            },
            token => {
                panic!("Expected object property, got {:?}", token)
            }
        }
    }

    pub fn expression(&mut self) -> Expression {
        let left = match self.tokenizer.peek() {
            Some(&Literal(_)) => {
                if let Some(Literal(literal)) = self.next() {
                    Expression::Literal(literal)
                } else {
                    panic!("Failed to read expression")
                }
            }
            Some(&Identifier(_)) => {
                if let Some(Identifier(literal)) = self.next() {
                    Expression::Variable(literal)
                } else {
                    panic!("Failed to read expression")
                }
            }
            Some(&BracketOn) => self.array(),
            Some(&BlockOn)   => self.object(),
            Some(_)          => panic!("Unexpected token"),
            _                => panic!("Unexpected end of program")
        };

        on!(self, {
            Accessor => {
                let object = Box::new(left);
                let property = Box::new(self.object_property());
                allow_many!(self, LineTermination);

                if let Some(&ParenOn) = self.tokenizer.peek() {
                    return Expression::MethodCall {
                        object: object,
                        method: property,
                        arguments: expect_list!(self,
                            ParenOn,
                            self.expression(),
                            Comma,
                            ParenOff
                        )
                    };
                }

                return Expression::Member {
                    object: object,
                    property: property,
                }
            },
            Operator(Add) => return Expression::Addition {
                left: Box::new(left),
                right: Box::new(self.expression()),
            }
        });

        left
    }

    fn identifier(&mut self) -> String {
        match self.next() {
            Some(Identifier(id)) => id,
            token => panic!("Expected identifier, got {:?}", token),
        }
    }

    fn variable_declaration(&mut self, kind: VariableDeclarationKind)
    -> Statement {
        let mut declarations = Vec::new();

        loop {
            declarations.push(expect_key_value_pair!(self,
                self.identifier(),
                Assign,
                self.expression()
            ));

            if expect_statement_end!(self, Comma) {
                break
            }
        }

        Statement::VariableDeclaration {
            kind: kind,
            declarations: declarations,
        }
    }

    fn statement(&mut self) -> Option<Statement> {
        on!(self, {
            Keyword(Var)   => return Some(self.variable_declaration(
                VariableDeclarationKind::Var
            )),
            Keyword(Let)   => return Some(self.variable_declaration(
                VariableDeclarationKind::Let
            )),
            Keyword(Const) => return Some(self.variable_declaration(
                VariableDeclarationKind::Const
            )),
            Semicolon      => return self.statement()
        });

        // println!("Not a declaration {:?}", self.tokenizer.peek());

        if self.tokenizer.peek().is_some() {
            let expression = self.expression();
            expect_statement_end!(self);
            Some(Statement::Expression(expression))
        } else {
            None
        }
        // if let Some(token) = self.next() {
        //     return match token {
        //         LineTermination       => self.statement(),
        //         Comment(comment)      => Some(
        //             Statement::Comment(comment)
        //         ),
        //         BlockComment(comment) => Some (
        //             Statement::BlockComment(comment)
        //         ),
        //         Keyword(Var)          => Some(self.variable_declaration(
        //             VariableDeclarationKind::Var
        //         )),
        //         Keyword(Let)          => Some(self.variable_declaration(
        //             VariableDeclarationKind::Let
        //         )),
        //         Keyword(Const)        => Some(self.variable_declaration(
        //             VariableDeclarationKind::Const
        //         )),
        //         _ => None,
        //     }
        // }
    }
}

pub fn parse(source: String) -> Program {
    let mut parser = Parser::new(&source);
    let mut program = Program { body: Vec::new() };

    // for token in tokenizer {
    //     println!("{:?}", token);
    // }
    while let Some(statement) = parser.statement() {
        program.body.push(statement);
    }

    println!("{:#?}", program);

    return program;
}
