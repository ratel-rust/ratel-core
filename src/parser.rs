use lexicon::Token;
use lexicon::Token::*;
use tokenizer::Tokenizer;
use grammar::*;
use grammar::OperatorType::*;
use error::Result;

/// If the next token matches `$p`, consume that token and return
/// true, else do nothing and return false
macro_rules! allow {
    ($parser:ident, $p:pat) => {
        match try!($parser.tokenizer.peek()) {
            $p => {
                $parser.tokenizer.consume();
                true
            },
            _ => false
        }
    };
}

macro_rules! expect {
    ($parser:ident, $p:pat) => {
        match try!($parser.tokenizer.next()) {
            $p => {},
            _  => return Err($parser.tokenizer.invalid_token())
        }
    }
}

macro_rules! unexpected_token {
    ($parser:ident) => {
        return Err($parser.tokenizer.invalid_token())
    };
}

macro_rules! surround {
    ($parser:ident, $b1:pat, $eval:expr, $b2:pat) => ({
        expect!($parser, Control($b1));
        let value = $eval;
        expect!($parser, Control($b2));
        value
    });
}

pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
    allow_asi: bool,
}

impl<'a> Parser<'a> {
    #[inline]
    pub fn new(source: &'a str) -> Self {
        Parser {
            tokenizer: Tokenizer::new(source),
            allow_asi: false,
        }
    }

    #[inline]
    fn array_expression(&mut self) -> Result<Expression> {
        Ok(Expression::Array(try!(self.expression_list(Control(b']')))))
    }

    #[inline]
    fn object_member_list(&mut self) -> Result<Vec<ObjectMember>> {
        let mut list = Vec::new();

        loop {
            if allow!(self, Control(b'}')) {
                break;
            }

            list.push(try!(self.object_member()));

            if allow!(self, Control(b'}')) {
                break;
            }

            expect!(self, Control(b','));
        }

        Ok(list)
    }

    #[inline]
    fn object_member(&mut self) -> Result<ObjectMember> {
        Ok(match try!(self.tokenizer.next()) {
            Identifier(key) | Literal(LiteralString(key)) => {
                match try!(self.tokenizer.peek()) {
                    Control(b':') => {
                        self.tokenizer.consume();

                        ObjectMember::Literal {
                            key: key,
                            value: try!(self.expression(0)),
                        }
                    },

                    Control(b'(') => {
                        self.tokenizer.consume();

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

                match try!(self.tokenizer.next()) {
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
        match try!(self.tokenizer.peek()) {
            Control(b'{') => {
                self.tokenizer.consume();

                Ok(Statement::Block {
                    body: try!(self.block_body_tail())
                })
            },
            _ => {
                let token = try!(self.tokenizer.next());
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
            if allow!(self, Control(b'}')) {
                break;
            }

            body.push(
                try!(self.statement()).expect("Unexpected end of statements block")
            )
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

        let body = match try!(self.tokenizer.peek()) {
            Control(b'{') => {
                self.tokenizer.consume();

                Statement::Block {
                    body: try!(self.block_body_tail())
                }
            }
            _ => try!(self.expression(0)).into()
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

            Accessor => Expression::member(left, try!(self.tokenizer.expect_identifier())),

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
        let name = match try!(self.tokenizer.peek()) {
            Identifier(name) => {
                self.tokenizer.consume();

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
        if allow!(self, Control(b')')) {
            match try!(self.tokenizer.next()) {
                Operator(FatArrow) => {},
                _                  => unexpected_token!(self)
            }

            return self.arrow_function_expression(None);
        }

        let expression = try!(self.sequence_or_expression());

        expect!(self, Control(b')'));

        Ok(expression)
    }

    #[inline]
    fn sequence_or_expression_from_token(&mut self, token: Token) -> Result<Expression> {
        let first = try!(self.expression_from_token(token, 0));
        self.sequence_or(first)
    }

    #[inline]
    fn sequence_or(&mut self, first: Expression) -> Result<Expression> {
        Ok(match try!(self.tokenizer.peek()) {
            Control(b',') => {
                self.tokenizer.consume();

                let mut list = vec![first, try!(self.expression(0))];

                while allow!(self, Control(b',')) {
                    self.tokenizer.consume();

                    list.push(try!(self.expression(0)));
                }

                Expression::Sequence(list)
            },
            _ => first
        })
    }

    #[inline]
    fn sequence_or_expression(&mut self) -> Result<Expression> {
        let token = try!(self.tokenizer.next());
        self.sequence_or_expression_from_token(token)
    }

    fn expression_list(&mut self, terminator: Token) -> Result<Vec<Expression>> {
        let mut list = Vec::new();

        loop {
            if try!(self.tokenizer.peek()) == terminator {
                self.tokenizer.consume();
                break;
            }

            list.push(try!(self.expression(0)));

            if try!(self.tokenizer.peek()) == terminator {
                self.tokenizer.consume();
                break;
            }

            expect!(self, Control(b','));
        }

        Ok(list)
    }

    #[inline]
    fn expression(&mut self, lbp: u8) -> Result<Expression> {
        let token = try!(self.tokenizer.next());
        self.expression_from_token(token, lbp)
    }

    #[inline]
    fn expression_from_token(&mut self, token: Token, lbp: u8) -> Result<Expression> {
        let left = match token {
            This              => Expression::This,
            Literal(value)    => Expression::Literal(value),
            Identifier(value) => value.into(),
            Operator(optype)  => try!(self.prefix_expression(optype)),
            Control(b'(')     => try!(self.paren_expression()),
            Control(b'[')     => try!(self.array_expression()),
            Control(b'{')     => try!(self.object_expression()),
            Function          => try!(self.function_expression()),
            _                 => unexpected_token!(self)
        };

        self.complex_expression(left, lbp)
    }

    fn complex_expression(&mut self, mut left: Expression, lbp: u8) -> Result<Expression> {
        loop {
            left = match try!(self.tokenizer.peek()) {
                Operator(op) => {
                    let rbp = op.binding_power();

                    if lbp > rbp {
                        break;
                    }

                    self.tokenizer.consume();

                    try!(self.infix_expression(left, rbp, op))
                },

                Control(b'(') => {
                    if lbp > 0 {
                        break;
                    }

                    self.tokenizer.consume();

                    Expression::Call {
                        callee: Box::new(left),
                        arguments: try!(self.expression_list(Control(b')'))),
                    }
                },

                Control(b'[') => {
                    if lbp > 0 {
                        break;
                    }

                    self.tokenizer.consume();

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
                name: try!(self.tokenizer.expect_identifier()),
                value: match try!(self.tokenizer.peek()) {
                    Operator(Assign) => {
                        self.tokenizer.consume();

                        Some(try!(self.expression(0)))
                    },
                    _ => None
                }
            });

            if allow!(self, Control(b',')) {
                continue;
            }

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

        try!(self.tokenizer.expect_semicolon());

        Ok(statement)
    }

    #[inline]
    fn labeled_or_expression_statement(&mut self, label: OwnedSlice) -> Result<Statement> {
        Ok(match try!(self.tokenizer.peek()) {
            Control(b':') => {
                self.tokenizer.consume();

                Statement::Labeled {
                    label: label,
                    body: Box::new(self.statement().unwrap().expect("Expected statement")),
                }
            },
            _ => {
                let first = try!(self.complex_expression(label.into(), 0));

                let statement = try!(self.sequence_or(first)).into();

                try!(self.tokenizer.expect_semicolon());

                statement
            }
        })
    }

    #[inline]
    fn expression_statement(&mut self, token: Token) -> Result<Statement> {
        let statement = try!(self.sequence_or_expression_from_token(token)).into();

        try!(self.tokenizer.expect_semicolon());

        Ok(statement)
    }

    #[inline]
    fn return_statement(&mut self) -> Result<Statement> {
        let statement = Statement::Return {
            value: match try!(self.tokenizer.peek()) {
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

        try!(self.tokenizer.expect_semicolon());

        Ok(statement)
    }

    #[inline]
    fn throw_statement(&mut self) -> Result<Statement> {
        let statement = Statement::Throw {
            value: try!(self.sequence_or_expression())
        };

        try!(self.tokenizer.expect_semicolon());

        Ok(statement)
    }

    #[inline]
    fn break_statement(&mut self) -> Result<Statement> {
        let statement = Statement::Break {
            label: match try!(self.tokenizer.peek()) {
                EndOfProgram  => None,
                Control(b';') => None,
                _             => {
                    if self.allow_asi {
                        None
                    } else {
                        Some(try!(self.tokenizer.expect_identifier()))
                    }
                }
            }
        };

        try!(self.tokenizer.expect_semicolon());

        Ok(statement)
    }

    fn if_statement(&mut self) -> Result<Statement> {
        let test = surround!(self, b'(', try!(self.expression(0)), b')');
        let consequent = Box::new(try!(self.block_or_statement()));
        let alternate = if allow!(self, Else) {
            if allow!(self, If) {
                Some(Box::new(try!(self.if_statement())))
            } else {
                Some(Box::new(try!(self.block_or_statement())))
            }
        } else {
            None
        };

        Ok(Statement::If {
            test: test,
            consequent: consequent,
            alternate: alternate,
        })
    }

    #[inline]
    fn while_statement(&mut self) -> Result<Statement> {
        Ok(Statement::While {
            test: surround!(self, b'(', try!(self.expression(0)), b')'),
            body: Box::new(try!(self.block_or_statement())),
        })
    }

    #[inline]
    fn for_statement(&mut self) -> Result<Statement> {
        expect!(self, Control(b'('));

        let init = match try!(self.tokenizer.next()) {
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
            match try!(self.tokenizer.next()) {
                Operator(In)      => return self.for_in_statement(init),
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

        let test = match try!(self.tokenizer.next()) {
            Control(b';') => None,
            token         => Some(try!(self.sequence_or_expression_from_token(token))),
        };

        if !test.is_none() {
            expect!(self, Control(b';'));
        }

        let update = match try!(self.tokenizer.next()) {
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

    fn for_in_statement(&mut self, left: Option<Box<Statement>>) -> Result<Statement> {
        let left = left.unwrap();
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
            if allow!(self, Control(b')')) {
                break;
            }

            list.push(try!(self.parameter()));

            if allow!(self, Control(b')')) {
                break;
            }

            expect!(self, Control(b','));
        }

        Ok(list)
    }

    #[inline]
    fn parameter(&mut self) -> Result<Parameter> {
        Ok(Parameter {
            name: try!(self.tokenizer.expect_identifier())
        })
    }

    #[inline]
    fn function_statement(&mut self) -> Result<Statement> {
        let name = try!(self.tokenizer.expect_identifier());

        expect!(self, Control(b'('));

        Ok(Statement::Function {
            name: name,
            params: try!(self.parameter_list()),
            body: try!(self.block_body()),
        })
    }

    fn class_member(&mut self, name: OwnedSlice, is_static: bool) -> Result<ClassMember> {
        Ok(match try!(self.tokenizer.peek()) {
            Control(b'(') => {
                self.tokenizer.consume();

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
                self.tokenizer.consume();

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
        let name = try!(self.tokenizer.expect_identifier());
        let super_class = match try!(self.tokenizer.next()) {
            Extends => {
                let name = try!(self.tokenizer.expect_identifier());
                expect!(self, Control(b'{'));
                Some(name)
            },
            Control(b'{') => None,
            _             => unexpected_token!(self)
        };

        let mut members = Vec::new();

        loop {
            members.push(match try!(self.tokenizer.next()) {
                Identifier(name) => try!(self.class_member(name, false)),
                Static           => {
                    let name = try!(self.tokenizer.expect_identifier());

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

    fn statement(&mut self) -> Result<Option<Statement>> {
        let token = try!(self.tokenizer.next());

        Ok(Some(match token {
            EndOfProgram      => return Ok(None),
            Control(b';')     => Statement::Transparent { body: Vec::new() },
            Control(b'{')     => try!(self.block_statement()),
            Declaration(kind) => try!(self.variable_declaration_statement(kind)),
            Return            => try!(self.return_statement()),
            Break             => try!(self.break_statement()),
            Function          => try!(self.function_statement()),
            Class             => try!(self.class_statement()),
            If                => try!(self.if_statement()),
            While             => try!(self.while_statement()),
            For               => try!(self.for_statement()),
            Identifier(label) => try!(self.labeled_or_expression_statement(label)),
            Throw             => try!(self.throw_statement()),
            token             => try!(self.expression_statement(token)),
        }))
    }
}

pub fn parse(source: String) -> Program {
    let mut body = Vec::new();
    let mut error = None;

    {
        let mut parser = Parser::new(&source);

        loop {
            match parser.statement() {
                Ok(Some(statement)) => body.push(statement),
                Ok(None)            => break,
                Err(err)            => {
                    error = Some(err);
                    break;
                }
            }
        }
    }

    Program {
        source: source,
        body: body,
        error: error
    }
}
