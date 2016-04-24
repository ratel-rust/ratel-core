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
        match $parser.consume() {
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
        match $parser.lookahead() {
            Some(&$p) => {
                $parser.consume();
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
    };
}

macro_rules! ignore_nl {
    ($parser:ident) => {
        allow_many!($parser, LineTermination)
    }
}

/// More robust version of the regular `match`, will peek at the next
/// token, if the token matches `$p` then consume that token, any line
/// breaks after and call the `$then` expression.
macro_rules! on {
    ($parser:ident, { $( $p:pat => $then:expr ),* }) => ({
        ignore_nl!($parser);
        match $parser.lookahead() {
            $(
                Some(&$p) => {
                    $parser.consume();
                    ignore_nl!($parser);
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
        ignore_nl!($parser);

        let asi = $parser.allow_asi;
        let end = match $parser.lookahead() {
            Some(&Semicolon) => true,
            Some(&$cont)     => false,
            token            => asi || panic!("Unexpected token {:?}", token)
        };

        if !asi {
            $parser.consume();
        }

        end
    });

    ($parser:ident) => ({
        ignore_nl!($parser);

        let asi = $parser.allow_asi;
        match $parser.lookahead() {
            Some(&Semicolon) => true,
            token            => asi || panic!("Unexpected token {:?}", token)
        };

        if !asi {
            $parser.consume();
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
            ignore_nl!($parser);
            if allow!($parser, $end) {
                break;
            }
            list.push($item);
            ignore_nl!($parser);
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
        ignore_nl!($parser);
        let key = $key;
        ignore_nl!($parser);
        expect!($parser, $separator);
        ignore_nl!($parser);
        (key, $value)
    })
}

/// Returns true if met with a list closing token `$p`, allows
/// a tailing comma to appear before `$p`.
macro_rules! expect_list_end {
    ($parser:ident, $separator:pat, $end:pat) => {
        match $parser.consume() {
            Some($separator) => {
                ignore_nl!($parser);
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

    fn consume(&mut self) -> Option<Token> {
        let token = self.tokenizer.next();

        match token {
            Some(LineTermination) => self.allow_asi = true,
            _                     => self.allow_asi = false,
        }

        println!("Consume {:?}", token);

        return token;
    }

    fn lookahead(&mut self) -> Option<&Token> {
        self.tokenizer.peek()
    }

    fn array(&mut self) -> Expression {
        Expression::Array(expect_list!(self,
            BracketOn,
            self.expression(),
            Comma,
            BracketOff
        ))
    }

    fn object(&mut self) -> Expression {
        Expression::Object(expect_list!(self,
            BlockOn,
            expect_key_value_pair!(self,
                self.object_key(true),
                Colon,
                self.expression()
            ),
            Comma,
            BlockOff
        ))
    }

    fn object_key(&mut self, allow_string: bool) -> ObjectKey {
        match self.consume() {
            Some(Identifier(key))             => ObjectKey::Static(key),
            Some(Literal(LiteralString(key))) => {
                if !allow_string {
                    panic!("Expected object key, got LiteralString({:?})", key);
                }
                ObjectKey::Static(key)
            },
            Some(BracketOn) => {
                ignore_nl!(self);
                let key = self.expression();
                ignore_nl!(self);
                expect!(self, BracketOff);
                ObjectKey::Computed(key)
            },
            token => {
                panic!("Expected object key, got {:?}", token)
            }
        }
    }

    fn expression(&mut self) -> Expression {
        let left = match self.lookahead() {
            Some(&Literal(_)) => {
                if let Some(Literal(literal)) = self.consume() {
                    Expression::Literal(literal)
                } else {
                    panic!("Failed to read expression")
                }
            }
            Some(&Identifier(_)) => {
                if let Some(Identifier(literal)) = self.consume() {
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
                let property = Box::new(self.object_key(false));
                ignore_nl!(self);

                if let Some(&ParenOn) = self.lookahead() {
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
        match self.consume() {
            Some(Identifier(id)) => id,
            token => panic!("Expected identifier, got {:?}", token),
        }
    }

    fn variable_declaration_statement(&mut self, kind: VariableDeclarationKind)
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

    fn expression_statement(&mut self) -> Statement {
        let expression = self.expression();
        expect_statement_end!(self);
        Statement::Expression(expression)
    }

    fn statement(&mut self) -> Option<Statement> {
        on!(self, {
            Keyword(Var)   => return Some(self.variable_declaration_statement(
                VariableDeclarationKind::Var
            )),
            Keyword(Let)   => return Some(self.variable_declaration_statement(
                VariableDeclarationKind::Let
            )),
            Keyword(Const) => return Some(self.variable_declaration_statement(
                VariableDeclarationKind::Const
            )),
            Semicolon      => return self.statement()
        });

        if self.lookahead().is_some() {
            Some(self.expression_statement())
        } else {
            None
        }
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
