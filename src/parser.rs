use lexicon::Token;
use lexicon::Token::*;
use tokenizer::Tokenizer;
use std::iter::Peekable;
use grammar::*;
use grammar::Statement::*;
use grammar::Expression::*;
use grammar::ClassMember::*;
use grammar::OperatorType::*;

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
    };
    {$parser:ident $( $p:pat => $then:expr ),* } => ({
        match $parser.lookahead() {
            $(
                Some(&$p) => {
                    $parser.consume();
                    $then
                }
            )*
            _ => {}
        }
    });
}

/// Expects next token to match `$p`, otherwise panics.
macro_rules! expect {
    ($parser:ident, $p:pat => $value:ident) => (
        match $parser.consume() {
            $p    => $value,
            token => unexpected_token!($parser, token),
        }
    );
    ($parser:ident, $p:pat) => (
        match $parser.consume() {
            $p    => {},
            token => unexpected_token!($parser, token),
        }
    )
}

macro_rules! unexpected_token {
    ($parser:ident) => ({
        unexpected_token!($parser, $parser.consume());
    });
    ($parser:ident, $token:expr) => {
        panic!("Unexpected token {:?}", $token);
    }
}

/// Evaluates the `$eval` expression, then expects a semicolon or
/// end of program. If neither is found, but a LineTermination
/// occured on previous token, parsing will continue as if a
/// semicolon was present. In other cases cause a panic.
macro_rules! statement {
    ($parser:ident, $eval:expr) => ({
        let value = $eval;

        let is_end = match $parser.lookahead() {
            Some(&Semicolon) => {
                $parser.consume();
                true
            },
            Some(&ParenOff)  => true,
            Some(&BraceOff)  => true,
            None             => true,
            _                => false
        };

        if !is_end && !$parser.allow_asi {
            unexpected_token!($parser);
        };

        value
    })
}

/// Read a list of items with predefined `$start`, `$end` and
/// `$separator` tokens and an `$item` expression that is then
/// pushed onto a vector.
macro_rules! list {
    ($parser:ident, $item:expr, $end:pat) => ({
        let mut list = Vec::new();
        loop {
            if allow!($parser, $end) {
                break;
            }
            list.push($item);

            match $parser.consume() {
                Comma => allow!{ $parser $end => break },
                $end  => break,
                _     => {},
            }
        }

        list
    });
    ($parser:ident, $item:expr, $start:pat, $end:pat) => ({
        expect!($parser, $start);
        list!($parser, $item, $end)
    });
    ($parser:ident ( $item:expr )) => {
        list!($parser, $item, ParenOn, ParenOff)
    };
    ($parser:ident [ $item:expr ]) => {
        list!($parser, $item, BracketOn, BracketOff)
    };
    ($parser:ident { $item:expr }) => {
        list!($parser, $item, BraceOn, BraceOff)
    };
}

macro_rules! surround {
    ($parser:ident ( $eval:expr )) => ({
        expect!($parser, ParenOn);
        let value = $eval;
        expect!($parser, ParenOff);
        value
    });
    ($parser:ident [ $eval:expr ]) => ({
        expect!($parser, BracketOn);
        let value = $eval;
        expect!($parser, BracketOff);
        value
    });
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
    fn consume(&mut self) -> Token {
        self.handle_line_termination();
        let token = self.tokenizer.next().expect("Unexpected end of program");

        // println!("Consume {:?}", token);

        self.allow_asi = false;
        token
    }

    #[inline(always)]
    fn lookahead(&mut self) -> Option<&Token> {
        self.handle_line_termination();
        self.tokenizer.peek()
    }

    fn array_expression(&mut self) -> Expression {
        ArrayExpression(list!(self, self.expression(0), BracketOff))
    }

    #[inline(always)]
    fn object_member(&mut self) -> ObjectMember {
        match self.consume() {
            Identifier(key) | Literal(LiteralString(key)) => {
                match self.lookahead() {
                    Some(&Colon)   => {
                        self.consume();
                        ObjectMember::Literal {
                            key: key,
                            value: self.expression(0),
                        }
                    },
                    Some(&ParenOn) => ObjectMember::Method {
                        name: key,
                        params: list!(self ( self.parameter() )),
                        body: self.block_body()
                    },
                    _ => ObjectMember::Shorthand {
                        key: key,
                    }
                }
            },
            BracketOn => {
                let key = self.expression(0);
                expect!(self, BracketOff);
                match self.consume() {
                    Colon => ObjectMember::Computed {
                        key: key,
                        value: self.expression(0),
                    },
                    ParenOn => ObjectMember::ComputedMethod {
                        name: key,
                        params: list!(self, self.parameter(), ParenOff),
                        body: self.block_body(),
                    },
                    token => unexpected_token!(self, token),
                }
            },
            token => {
                panic!("Expected object key, got {:?}", token)
            }
        }
    }

    fn object_expression(&mut self) -> Expression {
        ObjectExpression(list!(self, self.object_member(), BraceOff))
    }

    fn block_or_statement(&mut self) -> Statement {
        if let Some(&BraceOn) = self.lookahead() {
            BlockStatement {
                body: self.block_body()
            }
        } else {
            self.expression_statement()
        }
    }

    fn block_statement(&mut self) -> Statement {
        BlockStatement {
            body: self.block_body(),
        }
    }

    fn block_body(&mut self) -> Vec<Statement> {
        expect!(self, BraceOn);

        let mut body = Vec::new();
        loop {
            allow!{ self BraceOff => break };

            body.push(
                self.statement().expect("Unexpected end of statements block")
            )
        }

        body
    }

    fn arrow_function_expression(&mut self, p: Option<Expression>) -> Expression {
        let params: Vec<Parameter> = match p {
            None => Vec::new(),
            Some(IdentifierExpression(name)) => {
                vec![Parameter { name: name }]
            },
            Some(SequenceExpression(mut list)) => {
                list.drain(..).map(|expression| {
                    match expression {
                        IdentifierExpression(name) => Parameter { name: name },
                        _ => panic!("Cannot cast {:?} to a parameter", expression),
                    }
                }).collect()
            },
            _ =>
                panic!("Cannot cast {:?} to parameters", p),
        };

        let body = if let Some(&BraceOn) = self.lookahead() {
            BlockStatement {
                body: self.block_body()
            }
        } else {
            ExpressionStatement(self.expression(0))
        };

        ArrowFunctionExpression {
            params: params,
            body: Box::new(body)
        }
    }

    #[inline(always)]
    fn prefix_expression(&mut self, operator: OperatorType) -> Expression {
        let bp = operator.binding_power(true);

        if !operator.prefix() {
            panic!("Unexpected operator {:?}", operator);
        }

        PrefixExpression {
            operator: operator,
            operand: Box::new(self.expression(bp)),
        }
    }

    #[inline(always)]
    fn infix_expression(&mut self, left: Expression, bp: u8) -> Expression {
        let operator = expect!(self, Operator(op) => op);

        match operator {
            Increment | Decrement => PostfixExpression {
                operator: operator,
                operand: Box::new(left),
            },

            Accessor => MemberExpression {
                object: Box::new(left),
                property: Box::new(MemberKey::Literal(
                    expect!(self, Identifier(key) => key)
                )),
            },

            Conditional => ConditionalExpression {
                test: Box::new(left),
                consequent: Box::new(self.expression(bp)),
                alternate: {
                    expect!(self, Colon);
                    Box::new(self.expression(bp))
                }
            },

            FatArrow => self.arrow_function_expression(Some(left)),

            _ => {
                if !operator.infix() {
                    panic!("Unexpected operator {:?}", operator);
                }

                if operator.assignment() {
                    // TODO: verify that left is assignable
                }

                BinaryExpression {
                    left: Box::new(left),
                    operator: operator,
                    right: Box::new(
                        self.expression(bp)
                    ),
                }
            }
        }
    }

    fn function_expression(&mut self) -> Expression {
        let name = if let Some(&Identifier(_)) = self.lookahead() {
            Some(expect!(self, Identifier(name) => name))
        } else {
            None
        };

        FunctionExpression {
            name: name,
            params: list!(self ( self.parameter() )),
            body: self.block_body(),
        }
    }

    fn paren_expression(&mut self) -> Expression {
        if allow!(self, ParenOff) {
            expect!(self, Operator(FatArrow));
            return self.arrow_function_expression(None);
        }

        let expression = self.sequence_or_expression();
        expect!(self, ParenOff);

        expression
    }

    fn sequence_or_expression_with(&mut self, first: Expression) -> Expression {
        if allow!(self, Comma) {
            let mut list = vec![first, self.expression(0)];

            while allow!(self, Comma) {
                list.push(self.expression(0));
            }

            SequenceExpression(list)
        } else {
            first
        }
    }

    fn sequence_or_expression(&mut self) -> Expression {
        let first = self.expression(0);
        self.sequence_or_expression_with(first)
    }

    fn complex_expression(&mut self, mut left: Expression, lbp: u8) -> Expression {
        'right: loop {
            let rbp = match self.lookahead() {
                Some(&Operator(ref op)) => op.binding_power(false),
                _                       => 0,
            };

            if lbp > rbp {
                break 'right;
            }

            left = match self.lookahead() {
                Some(&Operator(_)) => self.infix_expression(left, rbp),

                Some(&ParenOn)     => CallExpression {
                    callee: Box::new(left),
                    arguments: list!(self ( self.expression(0) ))
                },

                Some(&BracketOn)   => MemberExpression {
                    object: Box::new(left),
                    property: Box::new(MemberKey::Computed(
                        surround!(self [ self.sequence_or_expression() ])
                    ))
                },

                _                  => break 'right,
            }
        }

        left
    }

    fn expression(&mut self, lbp: u8) -> Expression {
        let left = match self.consume() {
            This              => ThisExpression,
            Identifier(value) => IdentifierExpression(value),
            Literal(value)    => LiteralExpression(value),
            Operator(optype)  => self.prefix_expression(optype),
            ParenOn           => self.paren_expression(),
            BracketOn         => self.array_expression(),
            BraceOn           => self.object_expression(),
            Function          => self.function_expression(),
            token             => unexpected_token!(self, token)
        };

        self.complex_expression(left, lbp)
    }

    fn variable_declaration_statement(&mut self) -> Statement {
        let kind = match self.consume() {
            Var   => VariableDeclarationKind::Var,
            Let   => VariableDeclarationKind::Let,
            Const => VariableDeclarationKind::Const,
            token => unexpected_token!(self, token),
        };

        let mut declarators = Vec::new();

        loop {
            let name = expect!(self, Identifier(name) => name);
            expect!(self, Operator(Assign));
            declarators.push(VariableDeclarator {
                name: name,
                value: self.expression(0),
            });

            allow!{ self Comma => continue };
            break;
        }

        statement!(self, VariableDeclarationStatement {
            kind: kind,
            declarators: declarators,
        })
    }

    fn labeled_or_expression_statement(&mut self) -> Statement {
        let label = expect!(self, Identifier(label) => label);

        if allow!(self, Colon) {
            LabeledStatement {
                label: label,
                body: Box::new(
                    self.statement().expect("Expected statement")
                ),
            }
        } else {
            let first = self.complex_expression(IdentifierExpression(label), 0);
            statement!(self, ExpressionStatement(
                self.sequence_or_expression_with(first)
            ))
        }
    }

    fn expression_statement(&mut self) -> Statement {
        statement!(self, ExpressionStatement(
            self.sequence_or_expression()
        ))
    }

    fn return_statement(&mut self) -> Statement {
        self.handle_line_termination();

        let value = if self.allow_asi {
            None
        } else {
            if let Some(&Semicolon) = self.lookahead() {
                None
            } else {
                Some(self.expression(0))
            }
        };

        statement!(self, ReturnStatement {
            value: value
        })
    }

    fn break_statement(&mut self) -> Statement {
        self.handle_line_termination();

        let label = if self.allow_asi {
            None
        } else {
            match self.lookahead() {
                Some(&Identifier(_)) => Some(
                    expect!(self, Identifier(name) => name)
                ),
                _                    => None,
            }
        };

        statement!(self, BreakStatement {
            label: label
        })
    }

    fn if_statement(&mut self) -> Statement {
        let test = surround!(self ( self.expression(0) ));
        let consequent = Box::new(self.block_or_statement());
        let alternate = if allow!(self, Else) {
            if allow!(self, If) {
                Some(Box::new(self.if_statement()))
            } else {
                Some(Box::new(self.block_or_statement()))
            }
        } else {
            None
        };

        statement!(self, IfStatement {
            test: test,
            consequent: consequent,
            alternate: alternate,
        })
    }

    fn while_statement(&mut self) -> Statement {
        statement!(self, WhileStatement {
            test: surround!(self ( self.expression(0) )),
            body: Box::new(self.block_or_statement()),
        })
    }

    fn parameter(&mut self) -> Parameter {
        Parameter {
            name: expect!(self, Identifier(name) => name)
        }
    }

    fn function_statement(&mut self) -> Statement {
        FunctionStatement {
            name: expect!(self, Identifier(name) => name),
            params: list!(self ( self.parameter() )),
            body: self.block_body(),
        }
    }

    fn class_member(&mut self, name: String, is_static: bool) -> ClassMember {
        match self.lookahead() {
            Some(&ParenOn) => {
                if !is_static && name == "constructor" {
                    ClassConstructor {
                        params: list!(self ( self.parameter() )),
                        body: self.block_body(),
                    }
                } else {
                    ClassMethod {
                        is_static: is_static,
                        name: name,
                        params: list!(self ( self.parameter())),
                        body: self.block_body(),
                    }
                }
            },
            Some(&Operator(Assign)) => {
                self.consume();
                ClassProperty {
                    is_static: is_static,
                    name: name,
                    value: self.expression(0),
                }
            },
            _ => unexpected_token!(self),
        }
    }

    fn class_statement(&mut self) -> Statement {
        let name = expect!(self, Identifier(id) => id);
        let super_class = if allow!(self, Extends) {
            Some(expect!(self, Identifier(name) => name))
        } else {
            None
        };
        expect!(self, BraceOn);
        let mut members = Vec::new();
        'members: loop {
            members.push(match self.consume() {
                Identifier(name) => self.class_member(name, false),
                Static           => {
                    let name = expect!(self, Identifier(name) => name);
                    self.class_member(name, true)
                },
                Semicolon        => continue 'members,
                BraceOff         => break 'members,
                token            => unexpected_token!(self, token)
            });
        }

        ClassStatement {
            name: name,
            extends: super_class,
            body: members,
        }
    }

    fn statement(&mut self) -> Option<Statement> {
        Some(match self.lookahead() {
            // intentional returns here!
            None                 => return None,
            Some(&Semicolon)     => return self.statement(),

            Some(&BraceOn)       => self.block_statement(),

            Some(&Var)           |
            Some(&Let)           |
            Some(&Const)         => self.variable_declaration_statement(),

            Some(&Return)        => {
                self.consume();
                self.return_statement()
            },

            Some(&Break)         => {
                self.consume();
                self.break_statement()
            },

            Some(&Function)      => {
                self.consume();
                self.function_statement()
            },

            Some(&Class)         => {
                self.consume();
                self.class_statement()
            },

            Some(&If)            => {
                self.consume();
                self.if_statement()
            },

            Some(&While)         => {
                self.consume();
                self.while_statement()
            },

            Some(&Identifier(_)) => self.labeled_or_expression_statement(),

            Some(_)              => self.expression_statement(),
        })
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
