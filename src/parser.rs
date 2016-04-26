use lexicon::Token;
use lexicon::Token::*;
use lexicon::KeywordKind::*;
use tokenizer::Tokenizer;
use std::iter::Peekable;
use grammar::*;
use grammar::OperatorType::*;

/// If the next token matches `$p`, consume that token and return
/// true, else do nothing and return false
macro_rules! allow {
    ($parser:ident, { $( $p:pat => $then:expr ),* }) => ({
        match $parser.lookahead() {
            $(
                Some(&$p) => {
                    $parser.consume();
                    $then;
                }
            )*
            _ => {}
        }
    });
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

/// Expects next token to match `$p`, otherwise panics.
macro_rules! expect {
    ($parser:ident, $p:pat => $value:ident) => ({
        match $parser.consume() {
            Some($p) => $value,
            None     => panic!("Unexpected end of program"),
            token    => unexpected_token!($parser, token),
        }
    });
    ($parser:ident, $p:pat) => ({
        match $parser.consume() {
            Some($p) => {},
            None     => panic!("Unexpected end of program"),
            token    => unexpected_token!($parser, token),
        }
    })
}

macro_rules! unexpected_token {
    ($parser:ident) => ({
        if let Some(token) = $parser.consume() {
            unexpected_token!($parser, token);
        } else {
            panic!("Unexpected end of program");
        }
    });
    ($parser:ident, $token:expr) => {
        panic!("Unexpected token {:?}", $token)
    }
}

/// Expects a semicolon or end of program. If neither is found,
/// but a LineTermination occured on previous token, parsing
/// will continue as if a semicolon was present. In other cases
/// cause a panic.
macro_rules! expect_statement_end {
    ($parser:ident) => ({
        let is_end = match $parser.lookahead() {
            Some(&Semicolon) => {
                $parser.consume();
                true
            },
            None => true,
            _    => false
        };

        if !is_end && !$parser.allow_asi {
            panic!("Expected semicolon, found {:?}", $parser.consume());
        };
    })
}

/// Read a list of items with predefined `$start`, `$end` and
/// `$separator` tokens and an `$item` expression that is then
/// pushed onto a vector.
macro_rules! expect_list {
    [$parser:ident, $item:expr, $start:pat, $separator:pat, $end:pat] => ({
        expect!($parser, $start);

        let mut list = Vec::new();
        loop {
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
        let item = $item;
        expect!($parser, $end);
        item
    })
}

/// Shorthand for reading a key expression, separator token and
/// value expression in that order.
macro_rules! expect_key_value_pair {
    ($parser:ident, $key:expr, $separator:pat, $value:expr) => ({
        let key = $key;
        expect!($parser, $separator);
        (key, $value)
    })
}

/// Returns true if met with a list closing token `$p`, allows
/// a tailing comma to appear before `$p`.
macro_rules! expect_list_end {
    ($parser:ident, $separator:pat, $end:pat) => ({
        match $parser.consume() {
            Some($separator) => {
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
    fn handle_line_termination(&mut self) {
        while let Some(&LineTermination) = self.tokenizer.peek() {
            self.tokenizer.next();
            self.allow_asi = true;
        }
    }

    #[inline(always)]
    fn consume(&mut self) -> Option<Token> {
        self.handle_line_termination();
        let token = self.tokenizer.next();
        self.allow_asi = false;
        token
    }

    #[inline(always)]
    fn lookahead(&mut self) -> Option<&Token> {
        self.handle_line_termination();
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
            Some(BracketOn) => {
                let expression = self.expression();
                expect!(self, BracketOff);
                ObjectKey::Computed(expression)
            },
            token => {
                panic!("Expected object key, got {:?}", token)
            }
        }
    }

    fn optional_block(&mut self) -> OptionalBlock {
        if let Some(&BlockOn) = self.lookahead() {
            OptionalBlock::Block(self.block())
        } else {
            OptionalBlock::Expression(Box::new(self.expression()))
        }
    }

    fn block(&mut self) -> Vec<Statement> {
        expect!(self, BlockOn);
        let mut body = Vec::new();
        loop {
            allow!(self, {
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
            Some(&Operator(_))   => {
                let operator = expect!(self, Operator(op) => op);
                match operator {
                    Increment | Decrement => {
                        Expression::PrefixExpression {
                            operator: operator,
                            argument: Box::new(self.expression()),
                        }
                    },
                    _ => panic!("Unexpected operator {:?}", operator)
                }
            }
            Some(&ParenOn)       => {
                self.consume();
                let expression = self.expression();
                expect!(self, ParenOff);
                expression
            },
            Some(&BracketOn) => self.array_expression(),
            Some(&BlockOn)   => self.object_expression(),
            _                => unexpected_token!(self)
        };

        'nest: loop {
            left = match self.lookahead() {
                Some(&Operator(_)) => {
                    let operator = expect!(self, Operator(op) => op);
                    match operator {
                        Increment | Decrement => {
                            Expression::PostfixExpression {
                                operator: operator,
                                argument: Box::new(left),
                            }
                        },
                        Add | Substract | Multiply | Divide => {
                            Expression::BinaryExpression {
                                operator: operator,
                                left: Box::new(left),
                                right: Box::new(self.expression()),
                            }
                        },
                        Accessor => {
                            Expression::MemberExpression {
                                object: Box::new(left),
                                property: Box::new(ObjectKey::Static(
                                    expect!(self, Identifier(key) => key)
                                )),
                            }
                        },
                        op => panic!("Unimplemented operator {:?}", op)
                    }
                },
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
                Some(&BracketOn) => {
                    self.consume();
                    let expression = self.expression();
                    expect!(self, BracketOff);

                    Expression::MemberExpression {
                        object: Box::new(left),
                        property: Box::new(ObjectKey::Computed(expression))
                    }
                },
                Some(&FatArrow) => {
                    self.consume();
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
                Operator(Assign),
                self.expression()
            ));

            if allow!(self, Comma) {
                continue;
            }

            expect_statement_end!(self);
            break;
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
        let condition = self.expression();
        expect!(self, ParenOff);
        let body = self.optional_block();
        expect_statement_end!(self);

        Statement::WhileStatement {
            condition: condition,
            body: body,
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
        Statement::FunctionStatement {
            name: name,
            params: params,
            body: self.block(),
        }
    }

    fn class_member(&mut self, name: String, is_static: bool) -> ClassMember {
        match self.lookahead() {
            Some(&ParenOn) => {
                let params = expect_list![self,
                    Parameter { name: expect!(self, Identifier(name) => name) },
                    ParenOn,
                    Comma,
                    ParenOff
                ];
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
            Some(&Operator(Assign)) => {
                self.consume();
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
        let super_class = if allow!(self, Keyword(Extends)) {
            Some(expect!(self, Identifier(name) => name))
        } else {
            None
        };
        expect!(self, BlockOn);
        let mut members = Vec::new();
        'members: loop {
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
        allow!(self, {
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
