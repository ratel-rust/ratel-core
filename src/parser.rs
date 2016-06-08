use lexicon::Token;
use lexicon::Token::*;
use tokenizer::Tokenizer;
use std::iter::Peekable;
use grammar::*;
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
    fn next(&mut self) -> Option<Token> {
        self.allow_asi = false;
        loop {
            match self.tokenizer.next() {
                Some(LineTermination) => continue,
                token                 => return token
            }
        }
    }

    #[inline(always)]
    fn consume(&mut self) -> Token {
        self.next().expect("Unexpected end of program")
    }

    #[inline(always)]
    fn lookahead(&mut self) -> Option<&Token> {
        while let Some(&LineTermination) = self.tokenizer.peek() {
            self.tokenizer.next();
            self.allow_asi = true;
        }
        self.tokenizer.peek()
    }

    fn array_expression(&mut self) -> Expression {
        Expression::Array(list!(self, self.expression(0), BracketOff))
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
        Expression::Object(list!(self, self.object_member(), BraceOff))
    }

    fn block_or_statement(&mut self) -> Statement {
        if let Some(&BraceOn) = self.lookahead() {
            Statement::Block {
                body: self.block_body()
            }
        } else {
            let token = self.consume();
            self.expression_statement(token)
        }
    }

    fn block_statement(&mut self) -> Statement {
        Statement::Block {
            body: self.block_body_tail(),
        }
    }

    fn block_body_tail(&mut self) -> Vec<Statement> {
        let mut body = Vec::new();
        loop {
            allow!{ self BraceOff => break };

            body.push(
                self.statement().expect("Unexpected end of statements block")
            )
        }

        body
    }

    fn block_body(&mut self) -> Vec<Statement> {
        expect!(self, BraceOn);
        self.block_body_tail()
    }

    fn arrow_function_expression(&mut self, p: Option<Expression>) -> Expression {
        let params: Vec<Parameter> = match p {
            None => Vec::new(),
            Some(Expression::Identifier(name)) => {
                vec![Parameter { name: name }]
            },
            Some(Expression::Sequence(mut list)) => {
                list.drain(..).map(|expression| {
                    match expression {
                        Expression::Identifier(name) => Parameter { name: name },
                        _ => panic!("Cannot cast {:?} to a parameter", expression),
                    }
                }).collect()
            },
            _ =>
                panic!("Cannot cast {:?} to parameters", p),
        };

        let body = if let Some(&BraceOn) = self.lookahead() {
            Statement::Block {
                body: self.block_body()
            }
        } else {
            Statement::Expression { value: self.expression(0) }
        };

        Expression::ArrowFunction {
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

        Expression::Prefix {
            operator: operator,
            operand: Box::new(self.expression(bp)),
        }
    }

    #[inline(always)]
    fn infix_expression(&mut self, left: Expression, bp: u8) -> Expression {
        let operator = expect!(self, Operator(op) => op);

        match operator {
            Increment | Decrement => Expression::Postfix {
                operator: operator,
                operand: Box::new(left),
            },

            Accessor => Expression::Member {
                object: Box::new(left),
                property: Box::new(MemberKey::Literal(
                    expect!(self, Identifier(key) => key)
                )),
            },

            Conditional => Expression::Conditional {
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

                Expression::Binary {
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

        Expression::Function {
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

    fn sequence_or_expression_from_token(&mut self, token: Token) -> Expression {
        let first = self.expression_from_token(token, 0);
        self.sequence_or(first)
    }

    fn sequence_or(&mut self, first: Expression) -> Expression {
        if allow!(self, Comma) {
            let mut list = vec![first, self.expression(0)];

            while allow!(self, Comma) {
                list.push(self.expression(0));
            }

            Expression::Sequence(list)
        } else {
            first
        }
    }

    #[inline(always)]
    fn sequence_or_expression(&mut self) -> Expression {
        let token = self.consume();
        self.sequence_or_expression_from_token(token)
    }

    fn expression(&mut self, lbp: u8) -> Expression {
        let token = self.consume();
        self.expression_from_token(token, lbp)
    }

    #[inline(always)]
    fn expression_from_token(&mut self, token: Token, lbp: u8) -> Expression {
        let left = match token {
            This              => Expression::This,
            Identifier(value) => Expression::Identifier(value),
            Literal(value)    => Expression::Literal(value),
            Operator(optype)  => self.prefix_expression(optype),
            ParenOn           => self.paren_expression(),
            BracketOn         => self.array_expression(),
            BraceOn           => self.object_expression(),
            Function          => self.function_expression(),
            token             => unexpected_token!(self, token)
        };

        self.complex_expression(left, lbp)
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

                Some(&ParenOn)     => Expression::Call {
                    callee: Box::new(left),
                    arguments: list!(self ( self.expression(0) ))
                },

                Some(&BracketOn)   => Expression::Member {
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

    /// Helper for the `for` loops that doesn't consume semicolons
    fn variable_declaration(
        &mut self, kind: VariableDeclarationKind
    ) -> Statement {
        let mut declarators = Vec::new();

        loop {
            let name = expect!(self, Identifier(name) => name);
            declarators.push(VariableDeclarator {
                name: name,
                value: if allow!(self, Operator(Assign)) {
                    Some(self.expression(0))
                } else {
                    None
                },
            });

            allow!{ self Comma => continue };
            break;
        }

        Statement::VariableDeclaration {
            kind: kind,
            declarators: declarators,
        }
    }

    fn variable_declaration_statement(
        &mut self, kind: VariableDeclarationKind
    ) -> Statement {
        statement!(self, self.variable_declaration(kind))
    }

    fn labeled_or_expression_statement(&mut self, label: String) -> Statement {
        if allow!(self, Colon) {
            Statement::Labeled {
                label: label,
                body: Box::new(
                    self.statement().expect("Expected statement")
                ),
            }
        } else {
            let first = self.complex_expression(Expression::Identifier(label), 0);
            statement!(self, Statement::Expression {
                value: self.sequence_or(first)
            })
        }
    }

    fn expression_statement(&mut self, token: Token) -> Statement {
        statement!(self, Statement::Expression {
            value: self.sequence_or_expression_from_token(token)
        })
    }

    fn return_statement(&mut self) -> Statement {
        statement!(self, Statement::Return {
            value: match self.lookahead() {
                None             => None,
                Some(&Semicolon) => None,
                _                => {
                    if self.allow_asi {
                        None
                    } else {
                        Some(self.sequence_or_expression())
                    }
                }
            }
        })
    }

    fn break_statement(&mut self) -> Statement {
        statement!(self, Statement::Break {
            label: match self.lookahead() {
                None             => None,
                Some(&Semicolon) => None,
                _                => {
                    if self.allow_asi {
                        None
                    } else {
                        Some(expect!(self, Identifier(name) => name))
                    }
                }
            }
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

        Statement::If {
            test: test,
            consequent: consequent,
            alternate: alternate,
        }
    }

    fn while_statement(&mut self) -> Statement {
        Statement::While {
            test: surround!(self ( self.expression(0) )),
            body: Box::new(self.block_or_statement()),
        }
    }

    fn for_statement(&mut self) -> Statement {
        expect!(self, ParenOn);

        let init = match self.consume() {
            Semicolon         => None,

            Declaration(kind) => Some(Box::new(
                self.variable_declaration(kind)
            )),

            token             => {
                let expression = self.sequence_or_expression_from_token(token);

                if let Expression::Binary {
                    left,
                    operator: In,
                    right,
                } = expression {
                    return self.for_in_statement_from_expressions(*left, *right);
                }

                Some(Box::new(
                    Statement::Expression {
                        value: expression
                    }
                ))
            },
        };
        if init.is_some() {
            match self.consume() {
                Operator(In)      => return self.for_in_statement(init),
                Identifier(ident) => {
                    if ident != "of" {
                        panic!("Unexpected identifier {}", ident);
                    }
                    return self.for_of_statement(init.unwrap());
                },
                Semicolon         => {},
                token             => unexpected_token!(self, token),
            }
        }

        let test = match self.consume() {
            Semicolon => None,
            token     => Some(self.sequence_or_expression_from_token(token)),
        };
        if !test.is_none() { expect!(self, Semicolon) }

        let update = match self.consume() {
            ParenOff => None,
            token    => Some(self.sequence_or_expression_from_token(token)),
        };
        if !update.is_none() { expect!(self, ParenOff) }

        Statement::For {
            init: init,
            test: test,
            update: update,
            body: Box::new(self.block_or_statement()),
        }
    }

    fn for_in_statement_from_expressions(
        &mut self, left: Expression, right: Expression
    ) -> Statement {
        let left = Box::new(Statement::Expression { value: left });
        expect!(self, ParenOff);

        Statement::ForIn {
            left: left,
            right: right,
            body: Box::new(self.block_or_statement()),
        }
    }

    fn for_in_statement(&mut self, left: Option<Box<Statement>>) -> Statement {
        let left = left.unwrap();
        let right = self.sequence_or_expression();
        expect!(self, ParenOff);

        Statement::ForIn {
            left: left,
            right: right,
            body: Box::new(self.block_or_statement()),
        }
    }

    fn for_of_statement(&mut self, left: Box<Statement>) -> Statement {
        let right = self.sequence_or_expression();
        expect!(self, ParenOff);

        Statement::ForOf {
            left: left,
            right: right,
            body: Box::new(self.block_or_statement()),
        }
    }

    fn parameter(&mut self) -> Parameter {
        Parameter {
            name: expect!(self, Identifier(name) => name)
        }
    }

    fn function_statement(&mut self) -> Statement {
        Statement::Function {
            name: expect!(self, Identifier(name) => name),
            params: list!(self ( self.parameter() )),
            body: self.block_body(),
        }
    }

    fn class_member(&mut self, name: String, is_static: bool) -> ClassMember {
        match self.lookahead() {
            Some(&ParenOn) => {
                if !is_static && name == "constructor" {
                    ClassMember::Constructor {
                        params: list!(self ( self.parameter() )),
                        body: self.block_body(),
                    }
                } else {
                    ClassMember::Method {
                        is_static: is_static,
                        name: name,
                        params: list!(self ( self.parameter())),
                        body: self.block_body(),
                    }
                }
            },
            Some(&Operator(Assign)) => {
                self.consume();
                ClassMember::Property {
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

        Statement::Class {
            name: name,
            extends: super_class,
            body: members,
        }
    }

    fn statement(&mut self) -> Option<Statement> {
        let token = match self.next() {
            Some(token) => token,
            _           => return None,
        };

        Some(match token {
            Semicolon         => return self.statement(),
            BraceOn           => self.block_statement(),
            Declaration(kind) => self.variable_declaration_statement(kind),
            Return            => self.return_statement(),
            Break             => self.break_statement(),
            Function          => self.function_statement(),
            Class             => self.class_statement(),
            If                => self.if_statement(),
            While             => self.while_statement(),
            For               => self.for_statement(),
            Identifier(label) => self.labeled_or_expression_statement(label),
            token             => self.expression_statement(token),
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
