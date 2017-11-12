use parser::Parser;
use lexer::Token::*;
use lexer::{Token, TemplateKind};
use ast::{Ptr, Loc, List, ListBuilder, Expression, ExpressionPtr, ExpressionList, ObjectMember, Property, OperatorKind, Value, Parameter, ParameterKey, ParameterList};
use ast::OperatorKind::*;

impl<'ast> Parser<'ast> {
    #[inline]
    pub fn expression(&mut self, lbp: u8) -> ExpressionPtr<'ast> {
        let left = match self.lexer.token {
            This               => {
                self.lexer.consume();
                self.alloc_in_loc(Expression::This)
            },
            Literal(value)     => {
                self.lexer.consume();
                self.alloc_in_loc(Expression::Value(value))
            },
            Identifier(value)  => {
                self.lexer.consume();
                self.alloc_in_loc(Expression::Identifier(value))
            },
            Operator(Division) => {
                self.lexer.consume();
                self.regular_expression()
            },
            Operator(optype)   => {
                self.lexer.consume();
                self.prefix_expression(optype)
            },
            ParenOpen          => {
                self.lexer.consume();
                self.paren_expression()
            },
            BracketOpen        => {
                self.lexer.consume();
                self.array_expression()
            },
            BraceOpen          => {
                self.lexer.consume();
                self.object_expression()
            },
            Function           => {
                self.lexer.consume();
                self.function_expression()
            },
            Class              => {
                self.lexer.consume();
                self.class_expression()
            },
            Template(kind)     => {
                self.lexer.consume();
                self.template_expression(None, kind)
            },
            _                  => unexpected_token!(self)
        };

        self.complex_expression(left, lbp)
    }

    #[inline]
    pub fn complex_expression(&mut self, mut left: ExpressionPtr<'ast>, lbp: u8) -> ExpressionPtr<'ast> {
        loop {
            left = match self.lexer.token {
                Operator(op @ Increment) |
                Operator(op @ Decrement) => {
                    self.lexer.consume();

                    // TODO: op.end
                    self.alloc(Loc::new(left.start, left.end, Expression::Postfix {
                        operator: op,
                        operand: left,
                    }))
                }

                Operator(op @ Conditional) => {
                    self.lexer.consume();

                    let consequent = self.expression(op.binding_power());
                    expect!(self, Colon);
                    let alternate = self.expression(op.binding_power());

                    self.alloc(Expression::Conditional {
                        test: left,
                        consequent: consequent,
                        alternate: alternate,
                    }.at(0, 0))
                }

                Operator(FatArrow) => {
                    self.lexer.consume();

                    let params = match left.item {
                        Expression::Sequence { body } => body,
                        _                             => List::from(self.arena, left)
                    };

                    return self.arrow_function_expression(params);
                }

                Operator(op) => {
                    let rbp = op.binding_power();

                    if lbp > rbp {
                        break;
                    }

                    self.lexer.consume();

                    if !op.infix() {
                        unexpected_token!(self);
                    }

                    if op.assignment() {
                        // TODO: verify that left is assignable
                    }

                    let right = self.expression(rbp);

                    self.alloc(Loc::new(left.start, right.end, Expression::Binary {
                        operator: op,
                        left: left,
                        right: right,
                    }))
                },

                Accessor(member) => {
                    self.lexer.consume();

                    let right = self.alloc_in_loc(member);

                    self.alloc(Loc::new(left.start, right.end, Expression::Member {
                        object: left,
                        property: right,
                    }))
                },

                ParenOpen => {
                    if lbp > 18 {
                        break;
                    }

                    self.lexer.consume();

                    let arguments = self.expression_list();

                    self.alloc(Expression::Call {
                        callee: left,
                        arguments,
                    }.at(0, 0))
                },

                BracketOpen => {
                    if lbp > 19 {
                        break;
                    }

                    self.lexer.consume();

                    let property = self.sequence_or_expression();

                    expect!(self, BracketClose);

                    self.alloc(Expression::ComputedMember {
                        object: left,
                        property: property,
                    }.at(0, 0))
                },

                Template(kind) => {
                    if lbp > 0 {
                        break;
                    }

                    self.lexer.consume();

                    self.template_expression(Some(left), kind)
                },

                _ => break
            }
        }

        left
    }

    #[inline]
    pub fn arrow_function_expression(&mut self, params: ExpressionList<'ast>) -> ExpressionPtr<'ast> {
        let params = self.params_from_expressions(params);

        let body = match self.lexer.token {
            BraceOpen => {
                self.lexer.consume();
                self.block_statement()
            },
            _ => self.expression_statement(),
        };

        self.alloc(Expression::Arrow {
            params,
            body,
        }.at(0, 0))
    }

    #[inline]
    pub fn sequence_or_expression(&mut self) -> ExpressionPtr<'ast> {
        let first = self.expression(0);
        self.sequence_or(first)
    }

    #[inline]
    pub fn sequence_or(&mut self, first: ExpressionPtr<'ast>) -> ExpressionPtr<'ast> {
        match self.lexer.token {
            Comma => {
                self.lexer.consume();

                let mut builder = ListBuilder::new(self.arena, first);
                builder.push(self.expression(0));

                while let Comma = self.lexer.token {
                    self.lexer.consume();
                    builder.push(self.expression(0));
                }

                self.alloc(Expression::Sequence {
                    body: builder.into_list()
                }.at(0, 0))
            },
            _ => first
        }
    }

    #[inline]
    pub fn expression_list(&mut self) -> ExpressionList<'ast> {
        if self.lexer.token == ParenClose {
            self.lexer.consume();
            return List::empty();
        }

        let expression = self.expression(0);
        let mut builder = ListBuilder::new(self.arena, expression);

        loop {
            let expression = match self.lexer.token {
                ParenClose => {
                    self.lexer.consume();
                    break;
                },
                Comma      => {
                    self.lexer.consume();
                    self.expression(0)
                }
                _          => unexpected_token!(self),
            };

            builder.push(expression);
        }

        builder.into_list()
    }

    #[inline]
    pub fn paren_expression(&mut self) -> ExpressionPtr<'ast> {
        match self.lexer.token {
            ParenClose => {
                self.lexer.consume();
                expect!(self, Operator(FatArrow));
                self.arrow_function_expression(List::empty())
            },
            _ => {
                let expression = self.sequence_or_expression();

                expect!(self, ParenClose);

                expression
            }
        }
    }

    #[inline]
    fn prefix_expression(&mut self, operator: OperatorKind) -> ExpressionPtr<'ast> {
        if !operator.prefix() {
            unexpected_token!(self);
        }

        let operand = self.expression(15);

        self.alloc(Expression::Prefix {
            operator: operator,
            operand: operand,
        }.at(0, 0))
    }

    #[inline]
    pub fn object_expression(&mut self) -> ExpressionPtr<'ast> {
        if self.lexer.token == BraceClose {
            self.lexer.consume();
            return self.alloc_in_loc(Expression::Object {
                body: List::empty()
            });
        }

        let member = self.object_member();

        let mut builder = ListBuilder::new(self.arena, member);

        loop {
            match self.lexer.token {
                BraceClose => {
                    self.lexer.consume();
                    break;
                },
                Comma      => {
                    self.lexer.consume();
                },
                _          => unexpected_token!(self)
            }

            match self.lexer.token {
                BraceClose => {
                    self.lexer.consume();
                    break;
                },
                _ => builder.push(self.object_member()),
            }
        }

        self.alloc(Expression::Object {
            body: builder.into_list()
        }.at(0, 0))
    }

    pub fn object_member(&mut self) -> Ptr<'ast, Loc<ObjectMember<'ast>>> {
        let property = match self.lexer.token {
            Identifier(label) => {
                self.lexer.consume();

                match self.lexer.token {
                    Colon | ParenOpen => self.in_loc(Property::Literal(label)),

                    _ => return self.alloc_in_loc(ObjectMember::Shorthand(label)),
                }
            },
            Literal(Value::String(key)) |
            Literal(Value::Number(key)) => {
                self.lexer.consume();
                self.in_loc(Property::Literal(key))
            },
            Literal(Value::Binary(num)) => {
                self.lexer.consume();
                self.in_loc(Property::Binary(num))
            },
            BracketOpen => {
                self.lexer.consume();

                let expression = self.sequence_or_expression();
                let property = Loc::new(0, 0, Property::Computed(expression));

                expect!(self, BracketClose);

                property
            },
            _ => {
                // Allow word tokens such as "null" and "typeof" as identifiers
                match self.lexer.token.as_word() {
                    Some(label) => {
                        self.lexer.consume();
                        self.in_loc(Property::Literal(label))
                    }
                    None        => unexpected_token!(self)
                }
            }
        };

        let property = self.alloc(property);

        match self.lexer.token {
            Colon => {
                self.lexer.consume();

                let value = self.expression(0);

                self.alloc(Loc::new(0, 0, ObjectMember::Value {
                    property,
                    value,
                }))
            },
            ParenOpen => {
                self.lexer.consume();

                let params = self.parameter_list();
                let body = self.block_body();

                self.alloc(Loc::new(0, 0, ObjectMember::Method {
                    property,
                    params,
                    body,
                }))
            },
            _ => unexpected_token!(self)
        }
    }

    #[inline]
    pub fn array_expression(&mut self) -> ExpressionPtr<'ast> {
        let expression = match self.lexer.token {
            Comma        => {
                self.lexer.consume();
                self.alloc_in_loc(Expression::Void)
            },
            BracketClose => {
                self.lexer.consume();
                return self.alloc(Expression::Array { body: List::empty() }.at(0,0))
            },
            _            => {
                let expression = self.expression(0);

                match self.lexer.token {
                    BracketClose => {
                        self.lexer.consume();

                        let body = List::from(self.arena, expression);

                        return self.alloc(Expression::Array { body }.at(0, 0));
                    },
                    Comma        => {
                        self.lexer.consume();
                        expression
                    },
                    _            => unexpected_token!(self),
                }
            }
        };

        let mut builder = ListBuilder::new(self.arena, expression);

        loop {
            match self.lexer.token {
                Comma => {
                    self.lexer.consume();

                    builder.push(self.alloc_in_loc(Expression::Void));

                    continue;
                },
                BracketClose => {
                    self.lexer.consume();

                    builder.push(self.alloc_in_loc(Expression::Void));

                    break;
                },
                _ => {
                    let expression = self.expression(0);

                    builder.push(expression);
                }
            }

            match self.lexer.token {
                BracketClose => {
                    self.lexer.consume();
                    break;
                }
                Comma        => self.lexer.consume(),
                _            => unexpected_token!(self),
            }
        }

        self.alloc(Expression::Array {
            body: builder.into_list()
        }.at(0,0))
    }

    #[inline]
    pub fn regular_expression(&mut self) -> ExpressionPtr<'ast> {
        self.lexer.read_regular_expression();

        let value = match self.lexer.token {
            Literal(value) => {
                self.lexer.consume();

                value
            },
            _ => unexpected_token!(self),
        };

        self.alloc(Expression::Value(value).at(0, 0))
    }

    fn template_expression(&mut self, tag: Option<ExpressionPtr<'ast>>, kind: TemplateKind<'ast>) -> ExpressionPtr<'ast> {
        let (quasi, expression) = match kind {
            TemplateKind::Open(quasi) => {
                let quasi = self.alloc_in_loc(quasi);

                let expression = self.sequence_or_expression();

                expect!(self, BraceClose);

                (quasi, expression)
            },

            TemplateKind::Closed(quasi) => {
                let quasi = self.alloc_in_loc(quasi);

                let template = Expression::Template {
                    tag,
                    expressions: List::empty(),
                    quasis: List::from(self.arena, quasi),
                };

                return self.alloc_in_loc(template);
            }
        };

        let mut quasis = ListBuilder::new(self.arena, quasi);
        let mut expressions = ListBuilder::new(self.arena, expression);

        loop {
            self.lexer.read_template_kind();

            match self.lexer.token {
                Template(TemplateKind::Open(quasi)) => {
                    self.lexer.consume();
                    quasis.push(self.alloc_in_loc(quasi));
                    expressions.push(self.sequence_or_expression());

                    expect!(self, BraceClose);
                },
                Template(TemplateKind::Closed(quasi)) => {
                    self.lexer.consume();
                    quasis.push(self.alloc_in_loc(quasi));
                    break;
                },
                _ => unexpected_token!(self)
            }
        }

        self.alloc(Expression::Template {
            tag,
            expressions: expressions.into_list(),
            quasis: quasis.into_list(),
        }.at(0, 0))
    }

    #[inline]
    pub fn function_expression(&mut self) -> ExpressionPtr<'ast> {
        let name = match self.lexer.token {
            Identifier(name) => {
                self.lexer.consume();
                Some(self.alloc_in_loc(name))
            },
            _ => None
        };

        let function = self.function(name);

        self.alloc(Expression::Function { function }.at(0, 0))
    }

    #[inline]
    pub fn class_expression(&mut self) -> ExpressionPtr<'ast> {
        let name = match self.lexer.token {
            Identifier(name) => {
                self.lexer.consume();
                Some(self.alloc_in_loc(name))
            },
            _ => None
        };

        let class = self.class(name);

        self.alloc(Expression::Class { class }.at(0, 0))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ast::{OperatorKind, Value, Statement, Function, Class};
    use parser::parse;
    use parser::mock::Mock;

    #[test]
    fn ident_expression() {
        let module = parse("foobar;").unwrap();

        let expected = Expression::Identifier("foobar");

        assert_expr!(module, expected);
    }

    #[test]
    fn value_expression() {
        let module_a = parse(r#""foobar";"#).unwrap();
        let module_b = parse("100;").unwrap();
        let module_c = parse("true;").unwrap();

        let expected_a = Expression::Value(Value::String(r#""foobar""#));
        let expected_b = Expression::Value(Value::Number("100"));
        let expected_c = Expression::Value(Value::True);

        assert_expr!(module_a, expected_a);
        assert_expr!(module_b, expected_b);
        assert_expr!(module_c, expected_c);
    }

    #[test]
    fn template_expression() {
        let src = "`foobar`;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Template {
            tag: None,
            expressions: List::empty(),
            quasis: mock.list(["foobar"]),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn tagged_template_expression() {
        let src = "foo`bar`;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Template {
            tag: Some(mock.ident("foo")),
            expressions: List::empty(),
            quasis: mock.list(["bar"]),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn complex_template_expression() {
        let src = "`foo${ 10 }bar${ 20 }baz`;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Template {
            tag: None,
            expressions: mock.list([
                Expression::Value(Value::Number("10")),
                Expression::Value(Value::Number("20")),
            ]),
            quasis: mock.list(["foo", "bar", "baz"]),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn tagged_complex_template_expression() {
        let src = "foo`bar${ 42 }baz`;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Template {
            tag: Some(mock.ident("foo")),
            expressions: mock.list([
                Expression::Value(Value::Number("42")),
            ]),
            quasis: mock.list(["bar", "baz"]),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn sequence_expression() {
        let src = "foo, bar, baz;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Sequence {
            body: mock.list([
                Expression::Identifier("foo"),
                Expression::Identifier("bar"),
                Expression::Identifier("baz"),
            ])
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn binary_expression() {
        let src = "foo + bar;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Binary {
            operator: OperatorKind::Addition,
            left: mock.ident("foo"),
            right: mock.ident("bar"),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn parenthesized_binary_expression() {
        let src = "(2 + 2);";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Binary {
            operator: OperatorKind::Addition,
            left: mock.number("2"),
            right: mock.number("2"),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn conditional_expression() {
        let src = "true ? foo : bar";

        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Conditional {
            test: mock.ptr(Expression::Value(Value::True)),
            consequent: mock.ident("foo"),
            alternate: mock.ident("bar"),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn postfix_expression() {
        let src = "baz++;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Postfix {
            operator: OperatorKind::Increment,
            operand: mock.ident("baz"),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn call_expression() {
        let src = "foo();";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Call {
            callee: mock.ident("foo"),
            arguments: List::empty(),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn member_expression() {
        let src = "foo.bar";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Member {
            object: mock.ident("foo"),
            property: mock.ptr("bar"),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn keyword_member_expression() {
        let src = "foo.function";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Member {
            object: mock.ident("foo"),
            property: mock.ptr("function"),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn computed_member_expression() {
        let src = "foo[10]";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::ComputedMember {
            object: mock.ident("foo"),
            property: mock.number("10"),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn regular_expression() {
        let src = r#"/^[A-Z]+\/[\d]+/g"#;
        let module = parse(src).unwrap();

        let expected = Expression::Value(Value::RegEx("/^[A-Z]+\\/[\\d]+/g"));

        assert_expr!(module, expected);
    }

    #[test]
    fn array_expression() {
        let src = "[0, 1, 2]";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Array {
            body: mock.list([
                Expression::Value(Value::Number("0")),
                Expression::Value(Value::Number("1")),
                Expression::Value(Value::Number("2")),
            ])
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn sparse_array_expression() {
        let src = "[,,foo,bar,,]";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Array {
            body: mock.list([
                Expression::Void,
                Expression::Void,
                Expression::Identifier("foo"),
                Expression::Identifier("bar"),
                Expression::Void,
                Expression::Void,
            ])
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn function_expression() {
        let src = "(function () {})";
        let module = parse(src).unwrap();

        let expected = Expression::Function {
            function: Function {
                name: None.into(),
                params: List::empty(),
                body: List::empty()
            }
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn named_function_expression() {
        let src = "(function foo () {})";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Function {
            function: Function {
                name: mock.ptr("foo").into(),
                params: List::empty(),
                body: List::empty()
            }
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn arrow_function_expression() {
        let src = "() => bar";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Arrow {
            params: ParameterList::empty(),
            body: mock.ptr(Statement::Expression {
                expression: mock.ident("bar")
            })
        };
        assert_expr!(module, expected);
    }

    #[test]
    fn arrow_function_shorthand() {
        let src = "n => n* n";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Arrow {
            params: mock.list([
                Parameter {
                    key: ParameterKey::Identifier("n"),
                    value: None,
                },
            ]),

            body: mock.ptr(Statement::Expression {
                expression: mock.ptr(Expression::Binary {
                    operator: OperatorKind::Multiplication,
                    left: mock.ident("n"),
                    right: mock.ident("n"),
                })
            })

        };
        assert_expr!(module, expected);
    }

    #[test]
    fn arrow_function_with_params() {
        let src = "(a, b, c) => bar";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Arrow {
            params: mock.list([
                Parameter {
                    key: ParameterKey::Identifier("a"),
                    value: None,
                },
                Parameter {
                    key: ParameterKey::Identifier("b"),
                    value: None,
                },
                Parameter {
                    key: ParameterKey::Identifier("c"),
                    value: None,
                }
            ]),

            body: mock.ptr(Statement::Expression {
                expression: mock.ident("bar")
            })

        };
        assert_expr!(module, expected);
    }


    #[test]
    #[should_panic]
    fn arrow_function_invalid_params_throws() {
        parse("(a, b, c * 2) => bar").unwrap();
    }

    #[test]
    fn arrow_function_with_default_params() {
        let src = "(a, b, c = 2) => bar";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Arrow {
            params: mock.list([
                Parameter {
                    key: ParameterKey::Identifier("a"),
                    value: None,
                },
                Parameter {
                    key: ParameterKey::Identifier("b"),
                    value: None,
                },
                Parameter {
                    key: ParameterKey::Identifier("c"),
                    value: Some(mock.ptr(Expression::Value(Value::Number("2"))))
                }
            ]),
            body: mock.ptr(Statement::Expression {
                expression: mock.ident("bar")
            })
        };
        assert_expr!(module, expected);
    }

    #[test]
    fn class_expression() {
        let src = "(class {})";
        let module = parse(src).unwrap();

        let expected = Expression::Class {
            class: Class {
                name: None.into(),
                extends: None,
                body: List::empty()
            }
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn named_class_expression() {
        let src = "(class Foo {})";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Class {
            class: Class {
                name: mock.ptr("Foo").into(),
                extends: None,
                body: List::empty()
            }
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn named_child_class_expression() {
        let src = "(class Foo extends Bar {})";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Class {
            class: Class {
                name: mock.ptr("Foo").into(),
                extends: mock.ptr("Bar").into(),
                body: List::empty()
            }
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn regression_operator_precedence() {
        let src = "true === true && false === false";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Binary {
            operator: OperatorKind::LogicalAnd,
            left: mock.ptr(Expression::Binary {
                operator: OperatorKind::StrictEquality,
                left: mock.ptr(Expression::Value(Value::True)),
                right: mock.ptr(Expression::Value(Value::True)),
            }),
            right: mock.ptr(Expression::Binary {
                operator: OperatorKind::StrictEquality,
                left: mock.ptr(Expression::Value(Value::False)),
                right: mock.ptr(Expression::Value(Value::False)),
            }),
        };

        assert_expr!(module, expected);
    }

}
