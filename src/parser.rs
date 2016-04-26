use lexicon::Token;
use lexicon::Token::*;
use lexicon::KeywordKind::*;
use lexicon::ReservedKind::*;
use lexicon::CompareKind::*;
use lexicon::OperatorKind::*;
use tokenizer::Tokenizer;
use std::iter::Peekable;
use grammar::*;

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
    ($parser: ident, $p: pat) => {
        if allow!($parser, $p) {
            while allow!($parser, $p) {}
            true
        } else {
            false
        }
    };
}

/// Just a shorthand for allow_many!(self, LineTermination)
macro_rules! ignore_nl {
    ($parser:ident) => {
        allow_many!($parser, LineTermination)
    }
}

/// Expects next token to match `$p`, otherwise panics.
macro_rules! expect {
    ($parser:ident, $p:pat => $value:ident) => ({
        ignore_nl!($parser);
        match $parser.consume() {
            Some($p) => $value,
            None     => panic!("Unexpected end of program"),
            token    => panic!("Unexpected token {:?}", token),
        }
    });
    ($parser:ident, $p:pat) => ({
        ignore_nl!($parser);
        match $parser.consume() {
            Some($p) => {},
            None     => panic!("Unexpected end of program"),
            token    => panic!("Unexpected token {:?}", token),
        }
    })
}

macro_rules! predict {
    ($parser:ident, { $( $p:pat => $then:expr ),* }) => ({
        ignore_nl!($parser);
        match $parser.lookahead() {
            $(
                Some(&$p) => $then,
            )*
            _ => {}
        }
    })
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
                    $then;
                }
            )*
            _ => {}
        }
    })
}

/// Expects a semicolon to end the statement and return `true`. If no
/// semicolon is found, we try to follow the ECMA 262 Automatic
/// Semicolon Insertion (ASI) Rules.
macro_rules! expect_statement_end {
    ($parser:ident, $cont:pat) => ({
        ignore_nl!($parser);

        let asi = $parser.allow_asi;
        let end = match $parser.lookahead() {
            None | Some(&Semicolon) => true,
            Some(&$cont)            => false,
            token                   =>
                asi || panic!("Unexpected token {:?}", token)
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
            None | Some(&Semicolon) => true,
            token                   =>
                asi || panic!("Unexpected token {:?}", token)
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
    [$parser:ident, $item:expr, $start:pat, $separator:pat, $end:pat] => ({
        ignore_nl!($parser);
        expect!($parser, $start);

        let mut list = Vec::new();
        loop {
            ignore_nl!($parser);
            if allow!($parser, $end) {
                break;
            }
            list.push($item);
            if expect_list_end!($parser, $separator, $end) {
                break;
            } else {
                continue;
            }
        }

        list
    });
    [$parser:ident, $item:expr, $separator:pat, $end:pat] => ({
        let mut list = Vec::new();
        loop {
            ignore_nl!($parser);
            if allow!($parser, $end) {
                break;
            }
            list.push($item);
            if expect_list_end!($parser, $separator, $end) {
                break;
            } else {
                continue;
            }
        }

        list
    })
}

macro_rules! expect_wrapped {
    ($parser:ident, $item:expr, $start:pat, $end:pat) => ({
        expect!($parser, $start);
        expect_wrapped!($parser, $item, $end)
    });
    ($parser:ident, $item:expr, $end:pat) => ({
        ignore_nl!($parser);
        let item = $item;
        expect!($parser, $end);
        item
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
    ($parser:ident, $separator:pat, $end:pat) => ({
        ignore_nl!($parser);
        match $parser.consume() {
            Some($separator) => {
                ignore_nl!($parser);
                allow!($parser, $end)
            }
            Some($end)       => true,
            _                => false,
        }
    })
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

    #[inline(always)]
    fn consume(&mut self) -> Option<Token> {
        let token = self.tokenizer.next();

        match token {
            Some(LineTermination) => self.allow_asi = true,
            _                     => self.allow_asi = false,
        }

        // println!("Consume {:?}", token);

        return token;
    }

    #[inline(always)]
    fn lookahead(&mut self) -> Option<&Token> {
        self.tokenizer.peek()
    }

    fn array_expression(&mut self) -> Expression {
        Expression::ArrayExpression(expect_list![self,
            self.expression(),
            BracketOn,
            Comma,
            BracketOff
        ])
    }

    fn object_expression(&mut self) -> Expression {
        Expression::ObjectExpression(expect_list![self,
            expect_key_value_pair!(self,
                self.object_key(),
                Colon,
                self.expression()
            ),
            BlockOn,
            Comma,
            BlockOff
        ])
    }

    fn object_key(&mut self) -> ObjectKey {
        match self.consume() {
            Some(Identifier(key)) | Some(Literal(LiteralString(key))) => {
                ObjectKey::Static(key)
            },
            Some(BracketOn) => ObjectKey::Computed(
                expect_wrapped!(self, self.expression(), BracketOff)
            ),
            token => {
                panic!("Expected object key, got {:?}", token)
            }
        }
    }

    fn optional_block(&mut self) -> OptionalBlock {
        ignore_nl!(self);
        if let Some(&BlockOn) = self.lookahead() {
            self.consume();
            OptionalBlock::Block(self.block())
        } else {
            OptionalBlock::Expression(Box::new(self.expression()))
        }
    }

    fn block(&mut self) -> Vec<Statement> {
        let mut body = Vec::new();
        loop {
            on!(self, {
                BlockOff => break
            });
            match self.statement() {
                Some(statement) => body.push(statement),
                None            => panic!("Unexpected end of function block")
            }
        }

        body
    }

    fn arrow_function_expression(&mut self, p: Expression) -> Expression {
        let params: Vec<Parameter> = match p {
            Expression::Identifier(name) => {
                vec![Parameter { name: name }]
            },
            _ =>
                panic!("Can cast {:?} to parameters", p),
        };

        Expression::ArrowFunctionExpression {
            params: params,
            body: self.optional_block()
        }
    }

    fn expression(&mut self) -> Expression {
        let mut left = match self.lookahead() {
            Some(&Identifier(_)) => {
                Expression::Identifier(expect!(self, Identifier(v) => v))
            },
            Some(&Literal(_))    => {
                Expression::Literal(expect!(self, Literal(v) => v))
            },
            Some(&ParenOn)       => {
                expect_wrapped!(self, self.expression(), ParenOn, ParenOff)
            },
            Some(&BracketOn) => self.array_expression(),
            Some(&BlockOn)   => self.object_expression(),
            Some(_)          => panic!("Unexpected token"),
            _                => panic!("Unexpected end of program")
        };

        'nest: loop {
            left = match self.lookahead() {
                Some(&ParenOn) => {
                    Expression::CallExpression {
                        callee: Box::new(left),
                        arguments: expect_list![self,
                            self.expression(),
                            ParenOn,
                            Comma,
                            ParenOff
                        ]
                    }
                },
                Some(&Accessor) => {
                    self.consume();
                    ignore_nl!(self);

                    Expression::MemberExpression {
                        object: Box::new(left),
                        property: Box::new(ObjectKey::Static(
                            expect!(self, Identifier(key) => key)
                        )),
                    }
                },
                Some(&BracketOn) => Expression::MemberExpression {
                    object: Box::new(left),
                    property: Box::new(ObjectKey::Computed(
                        expect_wrapped!(self,
                            self.expression(),
                            BracketOn,
                            BracketOff
                        )
                    )),
                },
                Some(&Operator(_)) => {
                    match expect!(self, Operator(op) => op) {
                        Increment => {
                            Expression::UpdateExpression {
                                operator: UpdateOperator::Increment,
                                prefix: false,
                                argument: Box::new(left),
                            }
                        },
                        Decrement => {
                            Expression::UpdateExpression {
                                operator: UpdateOperator::Decrement,
                                prefix: false,
                                argument: Box::new(left),
                            }
                        },
                        Add => {
                            ignore_nl!(self);
                            Expression::BinaryExpression {
                                operator: BinaryOperator::Add,
                                left: Box::new(left),
                                right: Box::new(self.expression()),
                            }
                        },
                        Multiply => {
                            ignore_nl!(self);
                            Expression::BinaryExpression {
                                operator: BinaryOperator::Multiply,
                                left: Box::new(left),
                                right: Box::new(self.expression()),
                            }
                        },
                        op => panic!("Unimplemented operator {:?}", op)
                    }
                },
                Some(&FatArrow) => {
                    self.consume();
                    ignore_nl!(self);
                    return self.arrow_function_expression(left);
                }
                _ => break 'nest,
            }
        }

        left
    }

    fn variable_declaration_statement(&mut self, kind: VariableDeclarationKind)
    -> Statement {
        let mut declarations = Vec::new();

        loop {
            declarations.push(expect_key_value_pair!(self,
                expect!(self, Identifier(name) => name),
                Assign,
                self.expression()
            ));

            if expect_statement_end!(self, Comma) {
                break
            }
        }

        Statement::VariableDeclarationStatement {
            kind: kind,
            declarations: declarations,
        }
    }

    fn expression_statement(&mut self) -> Statement {
        let expression = self.expression();
        expect_statement_end!(self);
        Statement::ExpressionStatement(expression)
    }

    fn return_statement(&mut self) -> Statement {
        let expression = self.expression();
        expect_statement_end!(self);
        Statement::ReturnStatement(expression)
    }

    fn while_statement(&mut self) -> Statement {
        expect!(self, ParenOn);
        ignore_nl!(self);
        let condition = self.expression();
        expect!(self, ParenOff);

        Statement::WhileStatement {
            condition: condition,
            body: self.optional_block()
        }
    }

    fn function_statement(&mut self) -> Statement {
        let name = expect!(self, Identifier(name) => name);
        let params = expect_list![self,
            Parameter { name: expect!(self, Identifier(name) => name) },
            ParenOn,
            Comma,
            ParenOff
        ];
        expect!(self, BlockOn);
        Statement::FunctionStatement {
            name: name,
            params: params,
            body: self.block(),
        }
    }

    fn class_member(&mut self, name: String, is_static: bool) -> ClassMember {
        ignore_nl!(self);

        match self.lookahead() {
            Some(&ParenOn) => {
                let params = expect_list![self,
                    Parameter { name: expect!(self, Identifier(name) => name) },
                    ParenOn,
                    Comma,
                    ParenOff
                ];
                expect!(self, BlockOn);
                if !is_static && name == "constructor" {
                    ClassMember::ClassConstructor {
                        params: params,
                        body: self.block(),
                    }
                } else {
                    ClassMember::ClassMethod {
                        is_static: is_static,
                        name: name,
                        params: params,
                        body: self.block(),
                    }
                }
            },
            Some(&Assign) => {
                self.consume();
                ignore_nl!(self);
                ClassMember::ClassProperty {
                    is_static: is_static,
                    name: name,
                    value: self.expression(),
                }
            },
            _ => panic!("Unexpected token"),
        }
    }

    fn class_statement(&mut self) -> Statement {
        let name = expect!(self, Identifier(id) => id);
        ignore_nl!(self);
        let super_class = if allow!(self, Keyword(Extends)) {
            Some(expect!(self, Identifier(name) => name))
        } else {
            None
        };
        expect!(self, BlockOn);
        let mut members = Vec::new();
        'members: loop {
            ignore_nl!(self);
            members.push(match self.consume() {
                Some(Identifier(name)) => self.class_member(name, false),
                Some(Keyword(Static))  => {
                    let name = expect!(self, Identifier(name) => name);
                    self.class_member(name, true)
                },
                Some(Semicolon)        => continue 'members,
                Some(BlockOff)         => break 'members,
                token                  => panic!("Unexpected token {:?}", token),
            });
        }

        Statement::ClassStatement {
            name: name,
            extends: super_class,
            body: members,
        }
    }

    fn statement(&mut self) -> Option<Statement> {
        on!(self, {
            Keyword(Var)      => return Some(self.variable_declaration_statement(
                VariableDeclarationKind::Var
            )),
            Keyword(Let)      => return Some(self.variable_declaration_statement(
                VariableDeclarationKind::Let
            )),
            Keyword(Const)    => return Some(self.variable_declaration_statement(
                VariableDeclarationKind::Const
            )),
            Keyword(Return)   => return Some(self.return_statement()),
            Keyword(Function) => return Some(self.function_statement()),
            Keyword(Class)    => return Some(self.class_statement()),
            Keyword(While)    => return Some(self.while_statement()),
            Semicolon         => return self.statement()
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

    while let Some(statement) = parser.statement() {
        program.body.push(statement);
    }

    return program;
}
