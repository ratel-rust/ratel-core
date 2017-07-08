use parser::Parser;
use lexer::Token::*;
use lexer::{Asi, Token};
use ast::{Loc, List, ListBuilder, Statement, Declarator, Expression, DeclarationKind};
use ast::OperatorKind::*;

impl<'ast> Parser<'ast> {
    #[inline]
    pub fn statement(&mut self, token: Token<'ast>) -> Loc<Statement<'ast>> {
        match token {
            Semicolon          => self.in_loc(Statement::Empty),
            BraceOpen          => self.block_statement(),
            Declaration(kind)  => self.variable_declaration_statement(kind),
            Return             => self.return_statement(),
            Break              => self.break_statement(),
            Function           => self.function_statement(),
            // Class              => self.class_statement(),
            If                 => self.if_statement(),
            While              => self.while_statement(),
            Do                 => self.do_statement(),
            // For                => self.for_statement(),
            // Identifier(label)  => self.labeled_or_expression_statement(label),
            Throw              => self.throw_statement(),
            Try                => self.try_statement(),
            _                  => self.expression_statement(token),
        }
    }

    #[inline]
    fn expect_statement(&mut self) -> Loc<Statement<'ast>> {
        let token = self.next();
        self.statement(token)
    }

    #[inline]
    pub fn block_statement(&mut self) -> Loc<Statement<'ast>> {
        Statement::Block {
            body: self.block_body_tail()
        }.at(0, 0)
    }

    #[inline]
    pub fn expression_statement(&mut self, token: Token<'ast>) -> Loc<Statement<'ast>> {
        let expression = self.expression_from(token, 0);

        let start = expression.start;
        let end = expression.end;
        let expression = self.alloc(expression);

        expect_semicolon!(self);

        Statement::Expression { expression }.at(start, end)
    }

    #[inline]
    pub fn function_statement(&mut self) -> Loc<Statement<'ast>> {
        let name = expect_identifier!(self);
        let name = self.alloc_in_loc(name);

        Statement::Function {
            function: self.function(name)
        }.at(0, 0)
    }

    #[inline]
    pub fn return_statement(&mut self) -> Loc<Statement<'ast>> {
        let value = match self.asi() {
            Asi::NoSemicolon => {
                let expression = self.expression(0);

                expect_semicolon!(self);

                Some(self.alloc(expression))
            }

            Asi::ImplicitSemicolon => None,
            Asi::ExplicitSemicolon => {
                self.consume();

                None
            }
        };

        Statement::Return {
            value,
        }.at(0, 0)
    }

    #[inline]
    pub fn variable_declaration_statement(&mut self, kind: DeclarationKind) -> Loc<Statement<'ast>> {
        let declaration = Statement::Declaration {
            kind: kind,
            declarators: self.variable_declarators()
        }.at(0, 0);

        expect_semicolon!(self);

        declaration
    }

    #[inline]
    pub fn variable_declarator(&mut self) -> Loc<Declarator<'ast>> {
        let name = match self.next() {
            BraceOpen        => self.object_expression(),
            BracketOpen      => self.array_expression(),
            Identifier(name) => self.in_loc(Expression::Identifier(name)),
            _                => unexpected_token!(self),
        };
        let name = self.alloc(name);

        let value = match self.peek() {
            Operator(Assign) => {
                self.consume();
                let value = self.expression(0);

                Some(self.alloc(value))
            },
            _ => None
        };

        Loc::new(0, 0, Declarator {
            name,
            value,
        })
    }

    #[inline]
    pub fn variable_declarators(&mut self) -> List<'ast, Loc<Declarator<'ast>>> {
        let mut builder = ListBuilder::new(self.arena, self.variable_declarator());

        match self.peek() {
            Comma => self.consume(),
            _     => return builder.into_list(),
        }

        loop {
            builder.push(self.variable_declarator());

            match self.peek() {
                Comma => self.consume(),
                _     => return builder.into_list(),
            }
        }
    }

    #[inline]
    pub fn break_statement(&mut self) -> Loc<Statement<'ast>> {
        let label = match self.asi() {
            Asi::ExplicitSemicolon => {
                self.consume();
                None
            },
            Asi::ImplicitSemicolon => None,
            Asi::NoSemicolon => {
                let label = expect_identifier!(self);

                expect_semicolon!(self);

                Some(self.alloc_in_loc(Expression::Identifier(label)))
            }
        };

        Statement::Break {
            label
        }.at(0, 0)
    }

    #[inline]
    pub fn throw_statement(&mut self) -> Loc<Statement<'ast>> {
        let value = self.expression(0);
        expect_semicolon!(self);

        Statement::Throw {
            value: self.alloc(value)
        }.at(0, 0)
    }

    #[inline]
    pub fn try_statement(&mut self) -> Loc<Statement<'ast>> {
        let body = self.block_body();
        expect!(self, Catch);
        expect!(self, ParenOpen);

        let error = expect_identifier!(self);
        let error = self.alloc_in_loc(Expression::Identifier(error));
        expect!(self, ParenClose);

        let handler = self.block_body();
        expect_semicolon!(self);

        Statement::Try {
            body: body,
            error: error,
            handler: handler
        }.at(0, 0)
    }

    #[inline]
    pub fn if_statement(&mut self) -> Loc<Statement<'ast>> {
        expect!(self, ParenOpen);

        let test = self.expression(0);
        let test = self.alloc(test);
        expect!(self, ParenClose);

        let consequent = self.expect_statement();
        let consequent = self.alloc(consequent);

        let alternate = match self.peek() {
            Else => {
                self.consume();
                let statement = self.expect_statement();
                Some(self.alloc(statement))
            },
            _ => None
        };

        Statement::If {
            test,
            consequent,
            alternate,
        }.at(0, 0)
    }

    #[inline]
    pub fn while_statement(&mut self) -> Loc<Statement<'ast>> {
        expect!(self, ParenOpen);

        let test = self.expression(0);
        let test = self.alloc(test);
        expect!(self, ParenClose);

        let body = self.expect_statement();

        Statement::While {
            test,
            body: self.alloc(body)
        }.at(0, 0)
    }

    #[inline]
    pub fn do_statement(&mut self) -> Loc<Statement<'ast>> {
        let body = self.expect_statement();
        expect!(self, While);

        let test = self.expression(0);

        Statement::Do {
            body: self.alloc(body),
            test: self.alloc(test),
        }.at(0, 0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use parser::parse;
    use parser::mock::Mock;
    use ast::{Value, ObjectMember, Function};

    #[test]
    fn function_statement_empty() {
        let src = "function foo() {}";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Function {
                function: Function {
                    name: mock.ptr("foo").into(),
                    params: List::empty(),
                    body: List::empty(),
                }
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn function_statement_params() {
        let src = "function foo(bar, baz) {}";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Function {
                function: Function {
                    name: mock.ptr("foo").into(),
                    params: mock.list([
                        "bar",
                        "baz",
                    ]),
                    body: List::empty(),
                }
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn function_statement_body() {
        let src = "function foo() { bar; baz; }";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Function {
                function: Function {
                    name: mock.ptr("foo").into(),
                    params: List::empty(),
                    body: mock.list([
                        Statement::Expression {
                            expression: mock.ident("bar")
                        },
                        Statement::Expression {
                            expression: mock.ident("baz")
                        },
                    ])
                }
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    #[should_panic]
    fn function_statement_must_have_name() {
        parse("function() {}").unwrap();
    }

    #[test]
    fn block_statement() {
        let src = "{ true }";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Block {
                body: mock.list([
                    Statement::Expression {
                        expression: mock.ptr(Expression::Value(Value::True))
                    }
                ])
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn if_statement() {
        let src = "if (true) foo;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::If {
                test: mock.ptr(Expression::Value(Value::True)),
                consequent: mock.ptr(Statement::Expression {
                    expression: mock.ident("foo")
                }),
                alternate: None
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn if_else_statement() {
        let src = "if (true) foo; else { bar; }";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::If {
                test: mock.ptr(Expression::Value(Value::True)),
                consequent: mock.ptr(Statement::Expression {
                    expression: mock.ident("foo")
                }),
                alternate: Some(mock.ptr(Statement::Block {
                    body: mock.list([
                        Statement::Expression {
                            expression: mock.ident("bar")
                        }
                    ])
                }))
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn while_statement() {
        let src = "while (true) foo;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::While {
                test: mock.ptr(Expression::Value(Value::True)),
                body: mock.ptr(Statement::Expression {
                    expression: mock.ident("foo")
                })
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn while_statement_block() {
        let src = "while (true) { foo; }";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::While {
                test: mock.ptr(Expression::Value(Value::True)),
                body: mock.ptr(Statement::Block {
                    body: mock.list([
                        Statement::Expression {
                            expression: mock.ident("foo")
                        }
                    ])
                })
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn do_statement() {
        let src = "do foo; while (true)";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Do {
                body: mock.ptr(Statement::Expression {
                    expression: mock.ident("foo")
                }),
                test: mock.ptr(Expression::Value(Value::True))
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn break_statement() {
        let src = "break;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Break {
                label: None,
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn break_statement_label() {
        let src = "break foo;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Break {
                label: Some(mock.ident("foo")),
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn throw_statement() {
        let src = "throw '3'";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Throw {
                value: mock.ptr(Expression::Value(Value::String("'3'"))),
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn try_statement_empty() {
        let src = "try {} catch (err) {}";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Try {
                body: List::empty(),
                error: mock.ident("err"),
                handler: List::empty()
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn try_statement() {
        let src = "try { foo; } catch (err) { bar; }";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Try {
                body: mock.list([
                    Statement::Expression {
                        expression: mock.ident("foo")
                    }
                ]),
                error: mock.ident("err"),
                handler: mock.list([
                    Statement::Expression {
                        expression: mock.ident("bar")
                    }
                ]),
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn variable_declaration_statement() {
        let src = "var x, y, z = 42;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Declaration {
                kind: DeclarationKind::Var,
                declarators: mock.list([
                    Declarator {
                        name: mock.ident("x"),
                        value: None,
                    },
                    Declarator {
                        name: mock.ident("y"),
                        value: None,
                    },
                    Declarator {
                        name: mock.ident("z"),
                        value: Some(mock.number("42"))
                    }
                ])
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn variable_declaration_statement_destructuring_array() {
        let src = "let [x, y] = [1, 2];";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Declaration {
                kind: DeclarationKind::Let,
                declarators: mock.list([
                    Declarator {
                        name: mock.ptr(Expression::Array {
                            body: mock.list([
                                Expression::Identifier("x"),
                                Expression::Identifier("y"),
                            ])
                        }),
                        value: Some(mock.ptr(Expression::Array {
                            body: mock.list([
                                Expression::Value(Value::Number("1")),
                                Expression::Value(Value::Number("2")),
                            ])
                        })),
                    },
                ])
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn variable_declaration_statement_destructuring_object() {
        let src = "const { x, y } = { a, b };";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Declaration {
                kind: DeclarationKind::Const,
                declarators: mock.list([
                    Declarator {
                        name: mock.ptr(Expression::Object {
                            body: mock.list([
                                ObjectMember::Shorthand("x"),
                                ObjectMember::Shorthand("y"),
                            ])
                        }),
                        value: Some(mock.ptr(Expression::Object {
                            body: mock.list([
                                ObjectMember::Shorthand("a"),
                                ObjectMember::Shorthand("b"),
                            ])
                        })),
                    },
                ])
            }
        ]);

        assert_eq!(module.body(), expected);
    }
}
