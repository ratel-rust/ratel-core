use lexicon::Token;
use lexicon::Token::*;
use tokenizer::Tokenizer;
use grammar::*;
use grammar::OperatorType::*;

/// If the next token matches `$p`, consume that token and return
/// true, else do nothing and return false
macro_rules! allow {
    ($parser:ident, $p:pat) => {
        match $parser.tokenizer.peek() {
            $p => {
                $parser.tokenizer.consume();
                true
            },
            _ => false
        }
    };
}

macro_rules! unexpected_token {
    ($parser:ident) => ({
        unexpected_token!($parser, $parser.tokenizer.next());
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

        $parser.tokenizer.expect_semicolon();

        value
    })
}

macro_rules! surround {
    ($parser:ident, $b1:expr, $eval:expr, $b2:expr) => ({
        $parser.tokenizer.expect_control($b1);
        let value = $eval;
        $parser.tokenizer.expect_control($b2);
        value
    });
}

pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
    allow_asi: bool,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Parser {
            tokenizer: Tokenizer::new(source),
            allow_asi: false,
        }
    }

    #[inline]
    fn array_expression(&mut self) -> Expression {
        Expression::Array(self.expression_list(b']'))
    }

    #[inline]
    fn object_member_list(&mut self) -> Vec<ObjectMember> {
        let mut list = Vec::new();

        loop {
            if self.tokenizer.allow_control() == b'}' {
                self.tokenizer.next();
                break;
            }

            list.push(self.object_member());

            if self.tokenizer.allow_control() == b'}' {
                self.tokenizer.next();
                break;
            }

            self.tokenizer.expect_control(b',');
        }

        list
    }

    #[inline]
    fn object_member(&mut self) -> ObjectMember {
        match self.tokenizer.next() {
            Identifier(key) | Literal(LiteralString(key)) => {
                match self.tokenizer.peek() {
                    Control(b':') => {
                        self.tokenizer.consume();

                        ObjectMember::Literal {
                            key: key,
                            value: self.expression(0),
                        }
                    },

                    Control(b'(') => {
                        self.tokenizer.consume();

                        ObjectMember::Method {
                            name: key,
                            params: self.parameter_list(),
                            body: self.block_body()
                        }
                    },

                    _ => ObjectMember::Shorthand {
                        key: key,
                    }
                }
            },
            Control(b'[') => {
                let key = self.expression(0);

                self.tokenizer.expect_control(b']');

                match self.tokenizer.next() {
                    Control(b':') => ObjectMember::Computed {
                        key: key,
                        value: self.expression(0),
                    },
                    Control(b'(') => ObjectMember::ComputedMethod {
                        name: key,
                        params: self.parameter_list(),
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

    #[inline]
    fn object_expression(&mut self) -> Expression {
        Expression::Object(self.object_member_list())
    }

    #[inline]
    fn block_or_statement(&mut self) -> Statement {
        match self.tokenizer.allow_control() {
            b'{' => {
                self.tokenizer.next();

                Statement::Block {
                    body: self.block_body_tail()
                }
            },
            _ => {
                let token = self.tokenizer.next();
                self.expression_statement(token)
            }
        }
    }

    #[inline]
    fn block_statement(&mut self) -> Statement {
        Statement::Block {
            body: self.block_body_tail(),
        }
    }

    #[inline]
    fn block_body_tail(&mut self) -> Vec<Statement> {
        let mut body = Vec::new();

        loop {
            if self.tokenizer.allow_control() == b'}' {
                self.tokenizer.next();

                break;
            }

            body.push(
                self.statement().expect("Unexpected end of statements block")
            )
        }

        body
    }

    #[inline]
    fn block_body(&mut self) -> Vec<Statement> {
        self.tokenizer.expect_control(b'{');
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

        let body = match self.tokenizer.allow_control() {
            b'{' => {
                self.tokenizer.next();

                Statement::Block {
                    body: self.block_body_tail()
                }
            }
            _    => self.expression(0).into()
        };

        Expression::ArrowFunction {
            params: params,
            body: Box::new(body)
        }
    }

    #[inline]
    fn prefix_expression(&mut self, operator: OperatorType) -> Expression {
        if !operator.prefix() {
            panic!("Unexpected operator {:?}", operator);
        }

        Expression::Prefix {
            operator: operator,
            operand: Box::new(self.expression(15)),
        }
    }

    #[inline]
    fn infix_expression(&mut self, left: Expression, bp: u8, op: OperatorType) -> Expression {
        match op {
            Increment | Decrement => Expression::Postfix {
                operator: op,
                operand: Box::new(left),
            },

            Accessor => Expression::member(left, self.tokenizer.expect_identifier()),

            Conditional => Expression::Conditional {
                test: Box::new(left),
                consequent: Box::new(self.expression(bp)),
                alternate: {
                    self.tokenizer.expect_control(b':');
                    Box::new(self.expression(bp))
                }
            },

            FatArrow => self.arrow_function_expression(Some(left)),

            _ => {
                if !op.infix() {
                    panic!("Unexpected operator {:?}", op);
                }

                if op.assignment() {
                    // TODO: verify that left is assignable
                }

                Expression::binary(left, op, self.expression(bp))
            }
        }
    }

    fn function_expression(&mut self) -> Expression {
        let name = match self.tokenizer.peek() {
            Identifier(name) => {
                self.tokenizer.consume();

                Some(name)
            },
            _                => None
        };

        Expression::Function {
            name: name,
            params: self.parameter_list(),
            body: self.block_body(),
        }
    }

    #[inline]
    fn paren_expression(&mut self) -> Expression {
        if self.tokenizer.allow_control() == b')' {
            self.tokenizer.next();

            match self.tokenizer.next() {
                Operator(FatArrow) => {},
                token              => unexpected_token!(self, token)
            }

            return self.arrow_function_expression(None);
        }

        let expression = self.sequence_or_expression();
        self.tokenizer.expect_control(b')');

        expression
    }

    #[inline]
    fn sequence_or_expression_from_token(&mut self, token: Token) -> Expression {
        let first = self.expression_from_token(token, 0);
        self.sequence_or(first)
    }

    #[inline]
    fn sequence_or(&mut self, first: Expression) -> Expression {
        match self.tokenizer.allow_control() {
            b',' => {
                self.tokenizer.next();

                let mut list = vec![first, self.expression(0)];

                while self.tokenizer.allow_control() == b',' {
                    self.tokenizer.next();

                    list.push(self.expression(0));
                }

                Expression::Sequence(list)
            },
            _ => first
        }
    }

    #[inline]
    fn sequence_or_expression(&mut self) -> Expression {
        let token = self.tokenizer.next();
        self.sequence_or_expression_from_token(token)
    }

    fn expression_list(&mut self, terminator: u8) -> Vec<Expression> {
        let mut list = Vec::new();

        loop {
            if self.tokenizer.allow_control() == terminator {
                self.tokenizer.next();
                break;
            }

            list.push(self.expression(0));

            if self.tokenizer.allow_control() == terminator {
                self.tokenizer.next();
                break;
            }

            self.tokenizer.expect_control(b',');
        }

        list
    }

    #[inline]
    fn expression(&mut self, lbp: u8) -> Expression {
        let token = self.tokenizer.next();
        self.expression_from_token(token, lbp)
    }

    #[inline]
    fn expression_from_token(&mut self, token: Token, lbp: u8) -> Expression {
        let left = match token {
            This              => Expression::This,
            Literal(value)    => Expression::Literal(value),
            Identifier(value) => value.into(),
            Operator(optype)  => self.prefix_expression(optype),
            Control(b'(')     => self.paren_expression(),
            Control(b'[')     => self.array_expression(),
            Control(b'{')     => self.object_expression(),
            Function          => self.function_expression(),
            token             => unexpected_token!(self, token)
        };

        self.complex_expression(left, lbp)
    }

    fn complex_expression(&mut self, mut left: Expression, lbp: u8) -> Expression {
        loop {
            left = match self.tokenizer.peek() {
                Operator(op) => {
                    let rbp = op.binding_power();

                    if lbp > rbp {
                        break;
                    }

                    self.tokenizer.consume();

                    self.infix_expression(left, rbp, op)
                },

                Control(b'(') => {
                    if lbp > 0 {
                        break;
                    }

                    self.tokenizer.consume();

                    Expression::Call {
                        callee: Box::new(left),
                        arguments: self.expression_list(b')'),
                    }
                },

                Control(b'[') => {
                    if lbp > 0 {
                        break;
                    }

                    self.tokenizer.consume();

                    let property = self.sequence_or_expression();

                    self.tokenizer.expect_control(b']');

                    Expression::ComputedMember {
                        object: Box::new(left),
                        property: Box::new(property),
                    }
                },

                _ => break
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
            declarators.push(VariableDeclarator {
                name: self.tokenizer.expect_identifier(),
                value: match self.tokenizer.peek() {
                    Operator(Assign) => {
                        self.tokenizer.consume();

                        Some(self.expression(0))
                    },
                    _ => None
                }
            });

            if self.tokenizer.allow_control() == b',' {
                self.tokenizer.next();

                continue;
            }

            break;
        }

        Statement::VariableDeclaration {
            kind: kind,
            declarators: declarators,
        }
    }

    #[inline]
    fn variable_declaration_statement(
        &mut self, kind: VariableDeclarationKind
    ) -> Statement {
        statement!(self, self.variable_declaration(kind))
    }

    #[inline]
    fn labeled_or_expression_statement(&mut self, label: OwnedSlice) -> Statement {
        match self.tokenizer.allow_control() {
            b':' => {
                self.tokenizer.next();

                Statement::Labeled {
                    label: label,
                    body: Box::new(self.statement().expect("Expected statement")),
                }
            },
            _ => {
                let first = self.complex_expression(label.into(), 0);

                statement!(self, self.sequence_or(first).into())
            }
        }
    }

    #[inline]
    fn expression_statement(&mut self, token: Token) -> Statement {
        statement!(self, self.sequence_or_expression_from_token(token).into())
    }

    #[inline]
    fn return_statement(&mut self) -> Statement {
        statement!(self, Statement::Return {
            value: match self.tokenizer.peek() {
                EndOfProgram  => None,
                Control(b';') => None,
                _             => {
                    if self.allow_asi {
                        None
                    } else {
                        Some(self.sequence_or_expression())
                    }
                }
            }
        })
    }

    #[inline]
    fn throw_statement(&mut self) -> Statement {
        statement!(self, Statement::Throw {
            value: self.sequence_or_expression()
        })
    }

    #[inline]
    fn break_statement(&mut self) -> Statement {
        statement!(self, Statement::Break {
            label: match self.tokenizer.peek() {
                EndOfProgram  => None,
                Control(b';') => None,
                _             => {
                    if self.allow_asi {
                        None
                    } else {
                        Some(self.tokenizer.expect_identifier())
                    }
                }
            }
        })
    }

    fn if_statement(&mut self) -> Statement {
        let test = surround!(self, b'(', self.expression(0), b')');
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

    #[inline]
    fn while_statement(&mut self) -> Statement {
        Statement::While {
            test: surround!(self, b'(', self.expression(0), b')'),
            body: Box::new(self.block_or_statement()),
        }
    }

    #[inline]
    fn for_statement(&mut self) -> Statement {
        self.tokenizer.expect_control(b'(');

        let init = match self.tokenizer.next() {
            Control(b';')     => None,

            Declaration(kind) => Some(Box::new(self.variable_declaration(kind))),

            token             => {
                let expression = self.sequence_or_expression_from_token(token);

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
            match self.tokenizer.next() {
                Operator(In)      => return self.for_in_statement(init),
                Identifier(ident) => {
                    let slice = ident.as_str();
                    if slice != "of" {
                        panic!("Unexpected identifier {}", slice);
                    }
                    return self.for_of_statement(init.unwrap());
                },
                Control(b';')     => {},
                token             => unexpected_token!(self, token),
            }
        }

        let test = match self.tokenizer.next() {
            Control(b';') => None,
            token         => Some(self.sequence_or_expression_from_token(token)),
        };
        if !test.is_none() {
            self.tokenizer.expect_control(b';')
        }

        let update = match self.tokenizer.next() {
            Control(b')') => None,
            token         => Some(self.sequence_or_expression_from_token(token)),
        };
        if !update.is_none() {
            self.tokenizer.expect_control(b')');
        }

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
        let left = Box::new(left.into());
        self.tokenizer.expect_control(b')');

        Statement::ForIn {
            left: left,
            right: right,
            body: Box::new(self.block_or_statement()),
        }
    }

    fn for_in_statement(&mut self, left: Option<Box<Statement>>) -> Statement {
        let left = left.unwrap();
        let right = self.sequence_or_expression();
        self.tokenizer.expect_control(b')');

        Statement::ForIn {
            left: left,
            right: right,
            body: Box::new(self.block_or_statement()),
        }
    }

    fn for_of_statement(&mut self, left: Box<Statement>) -> Statement {
        let right = self.sequence_or_expression();
        self.tokenizer.expect_control(b')');

        Statement::ForOf {
            left: left,
            right: right,
            body: Box::new(self.block_or_statement()),
        }
    }

    fn parameter_list(&mut self) -> Vec<Parameter> {
        let mut list = Vec::new();

        loop {
            if self.tokenizer.allow_control() == b')' {
                self.tokenizer.next();
                break;
            }

            list.push(self.parameter());

            if self.tokenizer.allow_control() == b')' {
                self.tokenizer.next();
                break;
            }

            self.tokenizer.expect_control(b',');
        }

        list
    }

    #[inline]
    fn parameter(&mut self) -> Parameter {
        Parameter {
            name: self.tokenizer.expect_identifier()
        }
    }

    #[inline]
    fn function_statement(&mut self) -> Statement {
        let name = self.tokenizer.expect_identifier();

        self.tokenizer.expect_control(b'(');

        Statement::Function {
            name: name,
            params: self.parameter_list(),
            body: self.block_body(),
        }
    }

    fn class_member(&mut self, name: OwnedSlice, is_static: bool) -> ClassMember {
        match self.tokenizer.peek() {
            Control(b'(') => {
                self.tokenizer.consume();

                if !is_static && name.as_str() == "constructor" {
                    ClassMember::Constructor {
                        params: self.parameter_list(),
                        body: self.block_body(),
                    }
                } else {
                    ClassMember::Method {
                        is_static: is_static,
                        name: name,
                        params: self.parameter_list(),
                        body: self.block_body(),
                    }
                }
            },
            Operator(Assign) => {
                self.tokenizer.consume();

                ClassMember::Property {
                    is_static: is_static,
                    name: name,
                    value: self.expression(0),
                }
            },
            _ => unexpected_token!(self),
        }
    }

    #[inline]
    fn class_statement(&mut self) -> Statement {
        let name = self.tokenizer.expect_identifier();
        let super_class = match self.tokenizer.next() {
            Extends => {
                let name = self.tokenizer.expect_identifier();
                self.tokenizer.expect_control(b'{');
                Some(name)
            },
            Control(b'{') => None,
            token         => unexpected_token!(self, token)
        };

        let mut members = Vec::new();

        loop {
            members.push(match self.tokenizer.next() {
                Identifier(name) => self.class_member(name, false),
                Static           => {
                    let name = self.tokenizer.expect_identifier();
                    self.class_member(name, true)
                },
                Control(b';')    => continue,
                Control(b'}')    => break,
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
        let token = self.tokenizer.next();

        Some(match token {
            EndOfProgram      => return None,
            Control(b';')     => Statement::Transparent { body: Vec::new() },
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
            token             => self.expression_statement(token),
        })
    }
}

pub fn parse(source: String) -> Program {
    let mut body = Vec::new();

    {
        let mut parser = Parser::new(&source);

        while let Some(statement) = parser.statement() {
            body.push(statement);
        }
    }

    Program::new(source, body)
}
