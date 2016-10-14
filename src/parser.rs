use lexicon::Token;
use lexicon::Token::*;
use tokenizer::Tokenizer;
use grammar::*;
use grammar::OperatorType::*;
use error::Result;

/// Peek on the next token. Return with an error if tokenizer fails.
macro_rules! peek {
    ($parser:ident) => {
        match $parser.token {
            Some(token) => token,

            None => {
                let token = try!($parser.tokenizer.get_token());

                $parser.token = Some(token);

                token
            }
        }
    }
}

/// Get the next token. Return with an error if tokenizer fails.
macro_rules! next {
    ($parser:ident) => {
        match $parser.token {
            Some(token) => {
                $parser.consume();

                token
            },
            None => try!($parser.tokenizer.get_token())
        }
    }
}

/// If the next token matches `$p`, consume that token and execute `$eval`.
macro_rules! allow {
    ($parser:ident, $p:pat => $eval:expr) => {
        match peek!($parser) {
            $p => {
                $parser.consume();
                $eval;
            },
            _ => {}
        }
    }
}

/// Return an error if the next token doesn't match $p.
macro_rules! expect {
    ($parser:ident, $p:pat) => {
        match next!($parser) {
            $p => {},
            _  => unexpected_token!($parser)
        }
    }
}

/// Expect the next token to be an Identifier, extracting the OwnedSlice
/// out of it. Returns an error otherwise.
macro_rules! expect_identifier {
    ($parser:ident) => {
        match next!($parser) {
            Identifier(ident) => ident,
            _                 => unexpected_token!($parser)
        }
    }
}

/// Expecta semicolon to terminate a statement. Will assume a semicolon
/// following the ASI rules.
macro_rules! expect_semicolon {
    ($parser:ident) => {
        // TODO: Tokenizer needs to flag when a new line character has been
        //       consumed to satisfy all ASI rules
        match peek!($parser) {
            Control(b';') => $parser.consume(),
            Control(b')') |
            Control(b'}') |
            EndOfProgram  => {},
            _             => unexpected_token!($parser)
        }
    }
}

/// Return an error for current token.
macro_rules! unexpected_token {
    ($parser:ident) => {
        return Err($parser.tokenizer.invalid_token())
    };
}


pub struct Parser<'a> {
    // Tokenizer will produce tokens from the source
    tokenizer: Tokenizer<'a>,

    // TODO: Move to tokenizer
    allow_asi: bool,

    // Current token, to be used by peek! and next! macros
    token: Option<Token>,
}

impl<'a> Parser<'a> {
    #[inline]
    pub fn new(source: &'a str) -> Self {
        Parser {
            tokenizer: Tokenizer::new(source),
            allow_asi: false,
            token: None,
        }
    }

    #[inline]
    fn consume(&mut self) {
        self.token = None;
    }

    #[inline]
    fn array_expression(&mut self) -> Result<Expression> {
        let mut list = Vec::new();

        loop {
            match next!(self) {
                Control(b')') => break,
                token         => {
                    let expression = try!(self.expression_from_token(token, 0));
                    list.push(expression);
                }
            }

            match next!(self) {
                Control(b']') => break,
                Control(b',') => continue,
                _             => unexpected_token!(self)
            }
        }

        Ok(Expression::Array(list))
    }

    #[inline]
    fn object_member_list(&mut self) -> Result<Vec<ObjectMember>> {
        let mut list = Vec::new();

        loop {
            match next!(self) {
                Control(b'}') => break,
                token         => {
                    list.push(try!(self.object_member(token)));
                }
            }

            match next!(self) {
                Control(b'}') => break,
                Control(b',') => continue,
                _             => unexpected_token!(self)
            }
        }

        Ok(list)
    }

    #[inline]
    fn object_member(&mut self, token: Token) -> Result<ObjectMember> {
        Ok(match token {
            Identifier(key) | Literal(LiteralString(key)) => {
                match peek!(self) {
                    Control(b':') => {
                        self.consume();

                        ObjectMember::Literal {
                            key: key,
                            value: try!(self.expression(0)),
                        }
                    },

                    Control(b'(') => {
                        self.consume();

                        ObjectMember::Method {
                            name: key,
                            params: try!(self.parameter_list()),
                            body: try!(self.block_body())
                        }
                    },

                    _ => ObjectMember::Shorthand {
                        key: key,
                    }
                }
            },
            Control(b'[') => {
                let key = try!(self.expression(0));

                expect!(self, Control(b']'));

                match next!(self) {
                    Control(b':') => ObjectMember::Computed {
                        key: key,
                        value: try!(self.expression(0)),
                    },
                    Control(b'(') => ObjectMember::ComputedMethod {
                        name: key,
                        params: try!(self.parameter_list()),
                        body: try!(self.block_body()),
                    },
                    _ => unexpected_token!(self)
                }
            },
            _ => unexpected_token!(self)
        })
    }

    #[inline]
    fn object_expression(&mut self) -> Result<Expression> {
        Ok(Expression::Object(try!(self.object_member_list())))
    }

    #[inline]
    fn block_or_statement(&mut self) -> Result<Statement> {
        match peek!(self) {
            Control(b'{') => {
                self.consume();

                Ok(Statement::Block {
                    body: try!(self.block_body_tail())
                })
            },
            _ => {
                let token = next!(self);
                self.expression_statement(token)
            }
        }
    }

    #[inline]
    fn block_statement(&mut self) -> Result<Statement> {
        Ok(Statement::Block {
            body: try!(self.block_body_tail()),
        })
    }

    #[inline]
    fn block_body_tail(&mut self) -> Result<Vec<Statement>> {
        let mut body = Vec::new();

        loop {
            body.push(match next!(self) {
                Control(b'}') => break,
                token         => try!(self.statement(token))
            });
        }

        Ok(body)
    }

    #[inline]
    fn block_body(&mut self) -> Result<Vec<Statement>> {
        expect!(self, Control(b'{'));
        self.block_body_tail()
    }

    fn arrow_function_expression(&mut self, p: Option<Expression>) -> Result<Expression> {
        let params: Vec<Parameter> = match p {
            None => Vec::new(),
            Some(Expression::Identifier(name)) => {
                vec![Parameter { name: name }]
            },
            Some(Expression::Sequence(mut list)) => {
                let mut params = Vec::with_capacity(list.len());

                for expression in list.drain(..) {
                    match expression {
                        Expression::Identifier(name) => {
                            params.push(Parameter { name: name });
                        },
                        _ => unexpected_token!(self)
                    }
                }

                params
            },
            _ => unexpected_token!(self)
        };

        let body = match next!(self) {
            Control(b'{') => {
                Statement::Block {
                    body: try!(self.block_body_tail())
                }
            }
            token => try!(self.expression_from_token(token, 0)).into()
        };

        Ok(Expression::ArrowFunction {
            params: params,
            body: Box::new(body)
        })
    }

    #[inline]
    fn prefix_expression(&mut self, operator: OperatorType) -> Result<Expression> {
        if !operator.prefix() {
            unexpected_token!(self);
        }

        Ok(Expression::Prefix {
            operator: operator,
            operand: Box::new(try!(self.expression(15))),
        })
    }

    #[inline]
    fn infix_expression(&mut self, left: Expression, bp: u8, op: OperatorType) -> Result<Expression> {
        Ok(match op {
            Increment | Decrement => Expression::Postfix {
                operator: op,
                operand: Box::new(left),
            },

            Accessor => Expression::member(left, expect_identifier!(self)),

            Conditional => Expression::Conditional {
                test: Box::new(left),
                consequent: Box::new(try!(self.expression(bp))),
                alternate: {
                    expect!(self, Control(b':'));
                    Box::new(try!(self.expression(bp)))
                }
            },

            FatArrow => return self.arrow_function_expression(Some(left)),

            _ => {
                if !op.infix() {
                    unexpected_token!(self);
                }

                if op.assignment() {
                    // TODO: verify that left is assignable
                }

                Expression::binary(left, op, try!(self.expression(bp)))
            }
        })
    }

    fn function_expression(&mut self) -> Result<Expression> {
        let name = match peek!(self) {
            Identifier(name) => {
                self.consume();

                Some(name)
            },
            _ => None
        };

        Ok(Expression::Function {
            name: name,
            params: try!(self.parameter_list()),
            body: try!(self.block_body()),
        })
    }

    #[inline]
    fn paren_expression(&mut self) -> Result<Expression> {
        match next!(self) {
            Control(b')') => {
                expect!(self, Operator(FatArrow));

                self.arrow_function_expression(None)
            },
            token => {
                let expression = try!(self.expression_from_token(token, 0));
                let expression = try!(self.sequence_or(expression));

                expect!(self, Control(b')'));

                Ok(expression)
            }
        }
    }

    #[inline]
    fn sequence_or_expression(&mut self) -> Result<Expression> {
        let token = next!(self);
        self.sequence_or_expression_from_token(token)
    }

    #[inline]
    fn sequence_or_expression_from_token(&mut self, token: Token) -> Result<Expression> {
        let first = try!(self.expression_from_token(token, 0));
        self.sequence_or(first)
    }

    #[inline]
    fn sequence_or(&mut self, first: Expression) -> Result<Expression> {
        Ok(match peek!(self) {
            Control(b',') => {
                self.consume();

                let mut list = vec![first, try!(self.expression(0))];

                loop {
                    match peek!(self) {
                        Control(b',') => {
                            self.consume();

                            list.push(try!(self.expression(0)));
                        },
                        _ => break,
                    }
                }

                Expression::Sequence(list)
            },
            _ => first
        })
    }

    fn expression_list(&mut self) -> Result<Vec<Expression>> {
        let mut list = Vec::new();

        loop {
            match next!(self) {
                Control(b')') => break,
                token         => {
                    let expression = try!(self.expression_from_token(token, 0));
                    list.push(expression);
                }
            }

            match next!(self) {
                Control(b')') => break,
                Control(b',') => continue,
                _             => unexpected_token!(self)
            }
        }

        Ok(list)
    }

    #[inline]
    fn expression(&mut self, lbp: u8) -> Result<Expression> {
        let token = next!(self);
        self.expression_from_token(token, lbp)
    }

    #[inline]
    fn expression_from_token(&mut self, token: Token, lbp: u8) -> Result<Expression> {
        let left = match token {
            This              => Expression::This,
            Literal(value)    => Expression::Literal(value),
            Identifier(value) => Expression::from(value),
            Operator(optype)  => try!(self.prefix_expression(optype)),
            Control(b'(')     => try!(self.paren_expression()),
            Control(b'[')     => try!(self.array_expression()),
            Control(b'{')     => try!(self.object_expression()),
            Function          => try!(self.function_expression()),
            _                 => unexpected_token!(self)
        };

        self.complex_expression(left, lbp)
    }

    #[inline]
    fn complex_expression(&mut self, mut left: Expression, lbp: u8) -> Result<Expression> {
        loop {
            left = match peek!(self) {
                Operator(op) => {
                    let rbp = op.binding_power();

                    if lbp > rbp {
                        break;
                    }

                    self.consume();

                    try!(self.infix_expression(left, rbp, op))
                },

                Control(b'(') => {
                    if lbp > 0 {
                        break;
                    }

                    self.consume();

                    Expression::Call {
                        callee: Box::new(left),
                        arguments: try!(self.expression_list()),
                    }
                },

                Control(b'[') => {
                    if lbp > 0 {
                        break;
                    }

                    self.consume();

                    let property = try!(self.sequence_or_expression());

                    expect!(self, Control(b']'));

                    Expression::ComputedMember {
                        object: Box::new(left),
                        property: Box::new(property),
                    }
                },

                _ => break
            }
        }

        Ok(left)
    }

    /// Helper for the `for` loops that doesn't consume semicolons
    fn variable_declaration(&mut self, kind: VariableDeclarationKind) -> Result<Statement> {
        let mut declarators = Vec::new();

        loop {
            declarators.push(VariableDeclarator {
                name: expect_identifier!(self),
                value: match peek!(self) {
                    Operator(Assign) => {
                        self.consume();

                        Some(try!(self.expression(0)))
                    },
                    _ => None
                }
            });

            allow!(self, Control(b',') => continue);

            break;
        }

        Ok(Statement::VariableDeclaration {
            kind: kind,
            declarators: declarators,
        })
    }

    #[inline]
    fn variable_declaration_statement(&mut self, kind: VariableDeclarationKind) -> Result<Statement> {
        let statement = try!(self.variable_declaration(kind));

        expect_semicolon!(self);

        Ok(statement)
    }

    #[inline]
    fn labeled_or_expression_statement(&mut self, label: OwnedSlice) -> Result<Statement> {
        allow!(self, Control(b':') => {
            let token = next!(self);

            return Ok(Statement::Labeled {
                label: label,
                body: Box::new(try!(self.statement(token)))
            })
        });

        let first = try!(self.complex_expression(label.into(), 0));

        let expression = self.sequence_or(first);

        expect_semicolon!(self);

        expression.map(|expr| Statement::from(expr))
    }

    #[inline]
    fn expression_statement(&mut self, token: Token) -> Result<Statement> {
        let statement = try!(self.sequence_or_expression_from_token(token)).into();

        expect_semicolon!(self);

        Ok(statement)
    }

    #[inline]
    fn return_statement(&mut self) -> Result<Statement> {
        let statement = Statement::Return {
            value: match peek!(self) {
                EndOfProgram  => None,
                Control(b';') => None,
                _             => {
                    if self.allow_asi {
                        None
                    } else {
                        Some(try!(self.sequence_or_expression()))
                    }
                }
            }
        };

        expect_semicolon!(self);

        Ok(statement)
    }

    #[inline]
    fn throw_statement(&mut self) -> Result<Statement> {
        let statement = Statement::Throw {
            value: try!(self.sequence_or_expression())
        };

        expect_semicolon!(self);

        Ok(statement)
    }

    #[inline]
    fn break_statement(&mut self) -> Result<Statement> {
        let statement = Statement::Break {
            label: match peek!(self) {
                EndOfProgram  => None,
                Control(b';') => None,
                _             => {
                    if self.allow_asi {
                        None
                    } else {
                        Some(expect_identifier!(self))
                    }
                }
            }
        };

        expect_semicolon!(self);

        Ok(statement)
    }

    fn if_statement(&mut self) -> Result<Statement> {
        expect!(self, Control(b'('));

        let test = try!(self.expression(0));

        expect!(self, Control(b')'));

        let consequent = Box::new(try!(self.block_or_statement()));

        let alternate = match peek!(self) {
            Else => {
                self.consume();

                match peek!(self) {
                    If => {
                        self.consume();

                        Some(Box::new(try!(self.if_statement())))
                    },

                    _ => Some(Box::new(try!(self.block_or_statement())))
                }
            },

            _ => None
        };

        Ok(Statement::If {
            test: test,
            consequent: consequent,
            alternate: alternate,
        })
    }

    #[inline]
    fn while_statement(&mut self) -> Result<Statement> {
        expect!(self, Control(b'('));

        let test = try!(self.expression(0));

        expect!(self, Control(b')'));

        Ok(Statement::While {
            test: test,
            body: Box::new(try!(self.block_or_statement())),
        })
    }

    #[inline]
    fn for_statement(&mut self) -> Result<Statement> {
        expect!(self, Control(b'('));

        let init = match next!(self) {
            Control(b';')     => None,

            Declaration(kind) => Some(Box::new(try!(self.variable_declaration(kind)))),

            token             => {
                let expression = try!(self.sequence_or_expression_from_token(token));

                if let Expression::Binary {
                    left,
                    operator: In,
                    right,
                } = expression {
                    return self.for_in_statement_from_expressions(*left, *right);
                }

                Some(Box::new(expression.into()))
            },
        };

        if init.is_some() {
            match next!(self) {
                Operator(In)      => return self.for_in_statement(init.unwrap()),
                Identifier(ident) => {
                    if ident.as_str() != "of" {
                        unexpected_token!(self);
                    }
                    return self.for_of_statement(init.unwrap());
                },
                Control(b';')     => {},
                _                 => unexpected_token!(self),
            }
        }

        let test = match next!(self) {
            Control(b';') => None,
            token         => Some(try!(self.sequence_or_expression_from_token(token))),
        };

        if !test.is_none() {
            expect!(self, Control(b';'));
        }

        let update = match next!(self) {
            Control(b')') => None,
            token         => Some(try!(self.sequence_or_expression_from_token(token))),
        };
        if !update.is_none() {
            expect!(self, Control(b')'));
        }

        Ok(Statement::For {
            init: init,
            test: test,
            update: update,
            body: Box::new(try!(self.block_or_statement())),
        })
    }

    fn for_in_statement_from_expressions(&mut self, left: Expression, right: Expression)
    -> Result<Statement> {
        let left = Box::new(left.into());

        expect!(self, Control(b')'));

        Ok(Statement::ForIn {
            left: left,
            right: right,
            body: Box::new(try!(self.block_or_statement())),
        })
    }

    fn for_in_statement(&mut self, left: Box<Statement>) -> Result<Statement> {
        let right = try!(self.sequence_or_expression());

        expect!(self, Control(b')'));

        Ok(Statement::ForIn {
            left: left,
            right: right,
            body: Box::new(try!(self.block_or_statement())),
        })
    }

    fn for_of_statement(&mut self, left: Box<Statement>) -> Result<Statement> {
        let right = try!(self.sequence_or_expression());

        expect!(self, Control(b')'));

        Ok(Statement::ForOf {
            left: left,
            right: right,
            body: Box::new(try!(self.block_or_statement())),
        })
    }

    fn parameter_list(&mut self) -> Result<Vec<Parameter>> {
        let mut list = Vec::new();

        loop {
            match next!(self) {
                Control(b')')    => break,
                Identifier(name) => {
                    list.push(Parameter {
                        name: name
                    });
                },
                _ => unexpected_token!(self)
            }

            match next!(self) {
                Control(b')') => break,
                Control(b',') => {},
                _             => unexpected_token!(self)
            }
        }

        Ok(list)
    }

    #[inline]
    fn function_statement(&mut self) -> Result<Statement> {
        let name = expect_identifier!(self);

        expect!(self, Control(b'('));

        Ok(Statement::Function {
            name: name,
            params: try!(self.parameter_list()),
            body: try!(self.block_body()),
        })
    }

    fn class_member(&mut self, name: OwnedSlice, is_static: bool) -> Result<ClassMember> {
        Ok(match next!(self) {
            Control(b'(') => {
                if !is_static && name.as_str() == "constructor" {
                    ClassMember::Constructor {
                        params: try!(self.parameter_list()),
                        body: try!(self.block_body()),
                    }
                } else {
                    ClassMember::Method {
                        is_static: is_static,
                        name: name,
                        params: try!(self.parameter_list()),
                        body: try!(self.block_body()),
                    }
                }
            },
            Operator(Assign) => {
                ClassMember::Property {
                    is_static: is_static,
                    name: name,
                    value: try!(self.expression(0)),
                }
            },
            _ => unexpected_token!(self),
        })
    }

    #[inline]
    fn class_statement(&mut self) -> Result<Statement> {
        let name = expect_identifier!(self);
        let super_class = match next!(self) {
            Extends => {
                let name = expect_identifier!(self);

                expect!(self, Control(b'{'));

                Some(name)
            },
            Control(b'{') => None,
            _             => unexpected_token!(self)
        };

        let mut members = Vec::new();

        loop {
            members.push(match next!(self) {
                Identifier(name) => try!(self.class_member(name, false)),
                Static           => {
                    let name = expect_identifier!(self);

                    try!(self.class_member(name, true))
                },
                Control(b';')    => continue,
                Control(b'}')    => break,
                _                => unexpected_token!(self)
            });
        }

        Ok(Statement::Class {
            name: name,
            extends: super_class,
            body: members,
        })
    }

    #[inline]
    fn statement(&mut self, token: Token) -> Result<Statement> {
        match token {
            Control(b';')     => Ok(Statement::Transparent { body: Vec::new() }),
            Control(b'{')     => self.block_statement(),
            Declaration(kind) => self.variable_declaration_statement(kind),
            Return            => self.return_statement(),
            Break             => self.break_statement(),
            Function          => self.function_statement(),
            Class             => self.class_statement(),
            If                => self.if_statement(),
            While             => self.while_statement(),
            For               => self.for_statement(),
            Identifier(label) => self.labeled_or_expression_statement(label),
            Throw             => self.throw_statement(),
            _                 => self.expression_statement(token),
        }
    }

    #[inline]
    pub fn parse(&mut self) -> Result<Vec<Statement>> {
        let mut body = Vec::new();

        loop {
            body.push(match next!(self) {
                EndOfProgram => break,
                token        => try!(self.statement(token))
            })
        }

        Ok(body)
    }
}

pub fn parse(source: String) -> Program {
    let (error, body) = {
        let mut parser = Parser::new(&source);

        match parser.parse() {
            Ok(body) => (None, body),
            Err(err) => (Some(err), Vec::new())
        }
    };

    Program {
        source: source,
        body: body,
        error: error
    }
}
