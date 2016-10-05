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
            Some(&$p) => {
                $parser.tokenizer.next();
                true
            },
            _ => false
        }
    };
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

        $parser.tokenizer.expect_semicolon();

        value
    })
}

macro_rules! surround {
    ($parser:ident, $b1:expr, $eval:expr, $b2:expr) => ({
        $parser.tokenizer.expect_byte($b1);
        let value = $eval;
        $parser.tokenizer.expect_byte($b2);
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
    fn consume(&mut self) -> Token {
        self.tokenizer.next().expect("Unexpected end of program")
    }

    #[inline]
    fn array_expression(&mut self) -> Expression {
        Expression::Array(self.expression_list(b']'))
    }

    fn object_member_list(&mut self) -> Vec<ObjectMember> {
        let mut list = Vec::new();

        loop {
            if self.tokenizer.allow_byte(b'}') {
                break;
            }

            list.push(self.object_member());

            if self.tokenizer.allow_byte(b'}') {
                break;
            }

            self.tokenizer.expect_byte(b',');
        }

        list
    }

    #[inline]
    fn object_member(&mut self) -> ObjectMember {
        match self.consume() {
            Identifier(key) | Literal(LiteralString(key)) => {
                match self.tokenizer.peek() {
                    Some(&Colon)   => {
                        self.consume();

                        ObjectMember::Literal {
                            key: key,
                            value: self.expression(0),
                        }
                    },
                    Some(&ParenOn) => {
                        self.consume();

                        ObjectMember::Method {
                            name: key,
                            params: self.parameter_list(),
                            body: self.block_body(),
                        }
                    },
                    _ => ObjectMember::Shorthand {
                        key: key,
                    }
                }
            },
            BracketOn => {
                let key = self.expression(0);
                self.tokenizer.expect_byte(b']');
                match self.consume() {
                    Colon => ObjectMember::Computed {
                        key: key,
                        value: self.expression(0),
                    },
                    ParenOn => ObjectMember::ComputedMethod {
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

    fn block_or_statement(&mut self) -> Statement {
        if let Some(&BraceOn) = self.tokenizer.peek() {
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
            if self.tokenizer.allow_byte(b'}') {
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
        self.tokenizer.expect_byte(b'{');
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

        let body = if let Some(&BraceOn) = self.tokenizer.peek() {
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
    fn infix_expression(&mut self, left: Expression, bp: u8) -> Expression {
        let operator = match self.consume() {
            Operator(op) => op,
            token        => unexpected_token!(self, token)
        };

        match operator {
            Increment | Decrement => Expression::Postfix {
                operator: operator,
                operand: Box::new(left),
            },

            Accessor => Expression::Member {
                object: Box::new(left),
                property: self.tokenizer.expect_identifier(),
            },

            Conditional => Expression::Conditional {
                test: Box::new(left),
                consequent: Box::new(self.expression(bp)),
                alternate: {
                    self.tokenizer.expect_byte(b':');
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
        let name = if self.tokenizer.allow_byte(b'{') {
            None
        } else {
            let name = self.tokenizer.expect_identifier();
            self.tokenizer.expect_byte(b'{');
            Some(name)
        };

        Expression::Function {
            name: name,
            params: self.parameter_list(),
            body: self.block_body(),
        }
    }

    #[inline]
    fn paren_expression(&mut self) -> Expression {
        if self.tokenizer.allow_byte(b')') {
            match self.consume() {
                Operator(FatArrow) => {},
                token              => unexpected_token!(self, token)
            }
            return self.arrow_function_expression(None);
        }

        let expression = self.sequence_or_expression();
        self.tokenizer.expect_byte(b')');

        expression
    }

    #[inline]
    fn sequence_or_expression_from_token(&mut self, token: Token) -> Expression {
        let first = self.expression_from_token(token, 0);
        self.sequence_or(first)
    }

    fn sequence_or(&mut self, first: Expression) -> Expression {
        if self.tokenizer.allow_byte(b',') {
            let mut list = vec![first, self.expression(0)];

            while self.tokenizer.allow_byte(b',') {
                list.push(self.expression(0));
            }

            Expression::Sequence(list)
        } else {
            first
        }
    }

    #[inline]
    fn sequence_or_expression(&mut self) -> Expression {
        let token = self.consume();
        self.sequence_or_expression_from_token(token)
    }

    fn expression_list(&mut self, terminator: u8) -> Vec<Expression> {
        let mut list = Vec::new();

        loop {
            if self.tokenizer.allow_byte(terminator) {
                break;
            }

            list.push(self.expression(0));

            if self.tokenizer.allow_byte(terminator) {
                break;
            }

            self.tokenizer.expect_byte(b',');
        }

        list
    }

    #[inline]
    fn expression(&mut self, lbp: u8) -> Expression {
        let token = self.consume();
        self.expression_from_token(token, lbp)
    }

    #[inline]
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
        loop {
            let rbp = match self.tokenizer.peek() {
                Some(&Operator(ref op)) => op.binding_power(),
                _                       => 0,
            };

            if lbp > rbp {
                break;
            }

            left = match self.tokenizer.peek() {
                Some(&Operator(_)) => self.infix_expression(left, rbp),

                Some(&ParenOn)     => {
                    self.tokenizer.next();

                    Expression::Call {
                        callee: Box::new(left),
                        arguments: self.expression_list(b')'),
                    }
                },

                Some(&BracketOn)   => Expression::ComputedMember {
                    object: Box::new(left),
                    property: Box::new(
                        surround!(self, b'[', self.sequence_or_expression(), b']')
                    )
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
                value: if self.tokenizer.allow_byte(b'=') {
                    Some(self.expression(0))
                } else {
                    None
                },
            });

            if self.tokenizer.allow_byte(b',') {
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

    fn labeled_or_expression_statement(&mut self, label: OwnedSlice) -> Statement {
        if self.tokenizer.allow_byte(b':') {
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

    #[inline]
    fn expression_statement(&mut self, token: Token) -> Statement {
        statement!(self, Statement::Expression {
            value: self.sequence_or_expression_from_token(token)
        })
    }

    fn return_statement(&mut self) -> Statement {
        statement!(self, Statement::Return {
            value: match self.tokenizer.peek() {
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
            label: match self.tokenizer.peek() {
                None             => None,
                Some(&Semicolon) => None,
                _                => {
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

    fn while_statement(&mut self) -> Statement {
        Statement::While {
            test: surround!(self, b'(', self.expression(0), b')'),
            body: Box::new(self.block_or_statement()),
        }
    }

    fn for_statement(&mut self) -> Statement {
        self.tokenizer.expect_byte(b'(');

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
                    let slice = ident.as_str();
                    if slice != "of" {
                        panic!("Unexpected identifier {}", slice);
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
        if !test.is_none() {
            self.tokenizer.expect_byte(b';')
        }

        let update = match self.consume() {
            ParenOff => None,
            token    => Some(self.sequence_or_expression_from_token(token)),
        };
        if !update.is_none() {
            self.tokenizer.expect_byte(b')');
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
        let left = Box::new(Statement::Expression { value: left });
        self.tokenizer.expect_byte(b')');

        Statement::ForIn {
            left: left,
            right: right,
            body: Box::new(self.block_or_statement()),
        }
    }

    fn for_in_statement(&mut self, left: Option<Box<Statement>>) -> Statement {
        let left = left.unwrap();
        let right = self.sequence_or_expression();
        self.tokenizer.expect_byte(b')');

        Statement::ForIn {
            left: left,
            right: right,
            body: Box::new(self.block_or_statement()),
        }
    }

    fn for_of_statement(&mut self, left: Box<Statement>) -> Statement {
        let right = self.sequence_or_expression();
        self.tokenizer.expect_byte(b')');

        Statement::ForOf {
            left: left,
            right: right,
            body: Box::new(self.block_or_statement()),
        }
    }

    fn parameter_list(&mut self) -> Vec<Parameter> {
        let mut list = Vec::new();

        loop {
            if self.tokenizer.allow_byte(b')') {
                break;
            }

            list.push(self.parameter());

            if self.tokenizer.allow_byte(b')') {
                break;
            }

            self.tokenizer.expect_byte(b',');
        }

        list
    }

    #[inline]
    fn parameter(&mut self) -> Parameter {
        Parameter {
            name: self.tokenizer.expect_identifier()
        }
    }

    fn function_statement(&mut self) -> Statement {
        let name = self.tokenizer.expect_identifier();

        self.tokenizer.expect_byte(b'(');

        Statement::Function {
            name: name,
            params: self.parameter_list(),
            body: self.block_body(),
        }
    }

    fn class_member(&mut self, name: OwnedSlice, is_static: bool) -> ClassMember {
        match self.tokenizer.peek() {
            Some(&ParenOn) => {
                self.tokenizer.next();
                let slice = name.as_str();

                if !is_static && slice == "constructor" {
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
        let name = self.tokenizer.expect_identifier();
        let super_class = match self.consume() {
            Extends => {
                let name = self.tokenizer.expect_identifier();
                self.tokenizer.expect_byte(b'{');
                Some(name)
            },
            BraceOn => None,
            token   => unexpected_token!(self, token)
        };

        let mut members = Vec::new();

        loop {
            members.push(match self.consume() {
                Identifier(name) => self.class_member(name, false),
                Static           => {
                    let name = self.tokenizer.expect_identifier();
                    self.class_member(name, true)
                },
                Semicolon        => continue,
                BraceOff         => break,
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
        let token = match self.tokenizer.next() {
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
    let mut body = Vec::new();

    {
        let mut parser = Parser::new(&source);

        while let Some(statement) = parser.statement() {
            body.push(statement);
        }
    }

    Program::new(source, body)
}
