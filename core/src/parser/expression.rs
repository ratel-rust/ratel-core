use parser::Parser;
use lexer::Token::*;
use lexer::{Token, TemplateKind};
use ast::{Loc, List, ListBuilder, Expression, ExpressionPtr, ObjectMember, Property, OperatorKind, Value};
use ast::OperatorKind::*;

impl<'ast> Parser<'ast> {
    #[inline]
    pub fn expression(&mut self, lbp: u8) -> Loc<Expression<'ast>> {
        let token = self.next();
        self.expression_from(token, lbp)
    }

    #[inline]
    pub fn expression_from(&mut self, token: Token<'ast>, lbp: u8) -> Loc<Expression<'ast>> {
        let left = match token {
            This               => self.in_loc(Expression::This),
            Literal(value)     => self.in_loc(Expression::Value(value)),
            Identifier(value)  => self.in_loc(Expression::Identifier(value)),
            Operator(Division) => self.regular_expression(),
            Operator(optype)   => self.prefix_expression(optype),
            ParenOpen          => self.paren_expression(),
            BracketOpen        => self.array_expression(),
            BraceOpen          => self.object_expression(),
            Function           => self.function_expression(),
            // Class              => self.class_expression(),
            Template(kind)     => self.template_expression(None, kind),
            _                  => unexpected_token!(self)
        };

        self.complex_expression(left, lbp)
    }

    #[inline]
    pub fn complex_expression(&mut self, mut left: Loc<Expression<'ast>>, lbp: u8) -> Loc<Expression<'ast>> {
        loop {
            left = match self.peek() {
                Operator(op @ Increment) |
                Operator(op @ Decrement) => {
                    self.consume();

                    // TODO: op.end
                    Loc::new(left.start, left.end, Expression::Postfix {
                        operator: op,
                        operand: self.alloc(left),
                    })
                }

                Operator(op @ Conditional) => {
                    self.consume();

                    let consequent = self.expression(op.binding_power());
                    expect!(self, Colon);
                    let alternate = self.expression(op.binding_power());

                    Expression::Conditional {
                        test: self.alloc(left),
                        consequent: self.alloc(consequent),
                        alternate: self.alloc(alternate),
                    }.at(0, 0)
                }

                Operator(op) => {
                    self.consume();

                    let rbp = op.binding_power();

                    if lbp > rbp {
                        break;
                    }

                    if !op.infix() {
                        unexpected_token!(self);
                    }

                    if op.assignment() {
                        // TODO: verify that left is assignable
                    }

                    let right = self.expression(rbp);

                    Loc::new(left.start, right.end, Expression::Binary {
                        operator: op,
                        left: self.alloc(left),
                        right: self.alloc(right),
                    })
                },

                Accessor(member) => {
                    self.consume();

                    let right = self.in_loc(member);

                    Loc::new(left.start, right.end, Expression::Member {
                        object: self.alloc(left),
                        property: self.alloc(right),
                    })
                },

                ParenOpen => {
                    if lbp > 18 {
                        break;
                    }

                    self.consume();

                    Expression::Call {
                        callee: self.alloc(left),
                        arguments: self.expression_list(),
                    }.at(0, 0)
                },

                BracketOpen => {
                    if lbp > 19 {
                        break;
                    }

                    self.consume();

                    let property = self.sequence_or_expression();

                    expect!(self, BracketClose);

                    Expression::ComputedMember {
                        object: self.alloc(left),
                        property: self.alloc(property),
                    }.at(0, 0)
                },

                Template(kind) => {
                    if lbp > 0 {
                        break;
                    }

                    self.consume();

                    let tag = Some(self.alloc(left));

                    self.template_expression(tag, kind)
                },

                _ => break
            }
        }

        left
    }

    #[inline]
    pub fn sequence_or_expression(&mut self) -> Loc<Expression<'ast>> {
        let token = self.next();
        self.sequence_or_expression_from(token)
    }

    #[inline]
    pub fn sequence_or_expression_from(&mut self, token: Token<'ast>) -> Loc<Expression<'ast>> {
        let first = self.expression_from(token, 0);
        self.sequence_or(first)
    }

    #[inline]
    pub fn sequence_or(&mut self, first: Loc<Expression<'ast>>) -> Loc<Expression<'ast>> {
        match self.peek() {
            Comma => {
                self.consume();

                let mut builder = ListBuilder::new(self.arena, first);
                builder.push(self.expression(0));

                loop {
                    match self.peek() {
                        Comma => {
                            self.consume();

                            builder.push(self.expression(0));
                        },
                        _ => break,
                    }
                }

                Expression::Sequence {
                    body: builder.into_list()
                }.at(0, 0)
            },
            _ => first
        }
    }

    #[inline]
    pub fn expression_list(&mut self) -> List<'ast, Loc<Expression<'ast>>> {
        let expression = match self.next() {
            ParenClose => return List::empty(),
            token      => self.expression_from(token, 0),
        };

        let mut builder = ListBuilder::new(self.arena, expression);

        loop {
            let expression = match self.next() {
                ParenClose => break,
                Comma      => self.expression(0),
                _          => unexpected_token!(self),
            };

            builder.push(expression);
        }

        builder.into_list()
    }

    #[inline]
    pub fn paren_expression(&mut self) -> Loc<Expression<'ast>> {
        match self.next() {
            // ParenClose => {
            //     expect!(self, Operator(FatArrow));

            //     self.arrow_function_expression(None)
            // },
            token => {
                let expression = self.sequence_or_expression_from(token);

                expect!(self, ParenClose);

                expression
            }
        }
    }

    #[inline]
    fn prefix_expression(&mut self, operator: OperatorKind) -> Loc<Expression<'ast>> {
        if !operator.prefix() {
            unexpected_token!(self);
        }

        let operand = self.expression(15);

        Expression::Prefix {
            operator: operator,
            operand: self.alloc(operand),
        }.at(0, 0)
    }

    #[inline]
    pub fn object_expression(&mut self) -> Loc<Expression<'ast>> {
        let member = match self.next() {
            BraceClose => {
                return self.in_loc(Expression::Object {
                    body: List::empty()
                });
            },
            token => self.object_member(token),
        };

        let mut builder = ListBuilder::new(self.arena, member);

        loop {
            match self.next() {
                BraceClose => break,
                Comma      => {},
                _          => unexpected_token!(self)
            }

            match self.next() {
                BraceClose => break,
                token => builder.push(self.object_member(token)),
            }
        }

        Expression::Object {
            body: builder.into_list()
        }.at(0, 0)
    }

    pub fn object_member(&mut self, token: Token<'ast>) -> Loc<ObjectMember<'ast>> {
        let property = match token {
            Identifier(label) => {
                match self.peek() {
                    Colon | ParenOpen => self.in_loc(Property::Literal(label)),

                    _ => return self.in_loc(ObjectMember::Shorthand(label)),
                }
            },

            Literal(Value::String(key)) |
            Literal(Value::Number(key)) => self.in_loc(Property::Literal(key)),
            Literal(Value::Binary(num)) => self.in_loc(Property::Binary(num)),

            BracketOpen => {
                let expression = self.sequence_or_expression();
                let property = Loc::new(0, 0, Property::Computed(self.alloc(expression)));

                expect!(self, BracketClose);

                property
            },

            _ => {
                // Allow word tokens such as "null" and "typeof" as identifiers
                match token.as_word() {
                    Some(label) => self.in_loc(Property::Literal(label)),
                    None        => unexpected_token!(self)
                }
            }
        };

        let property = self.alloc(property);

        match self.next() {
            Colon => {
                let value = self.expression(0);

                Loc::new(0, 0, ObjectMember::Value {
                    property,
                    value: self.alloc(value),
                })
            },
            ParenOpen => {
                Loc::new(0, 0, ObjectMember::Method {
                    property,
                    params: self.parameter_list(),
                    body: self.block_body(),
                })
            },
            _ => unexpected_token!(self)
        }
    }

    #[inline]
    pub fn array_expression(&mut self) -> Loc<Expression<'ast>> {
        let expression = match self.next() {
            Comma        => self.in_loc(Expression::Void),
            BracketClose => return Expression::Array { body: List::empty() }.at(0,0),
            token        => {
                let expression = self.expression_from(token, 0);

                match self.next() {
                    BracketClose => {
                        let body = List::from(self.arena, expression);

                        return Expression::Array { body }.at(0, 0);
                    },
                    Comma        => expression,
                    _            => unexpected_token!(self),
                }
            }
        };

        let mut builder = ListBuilder::new(self.arena, expression);

        loop {
            match self.next() {
                Comma => {
                    builder.push(self.in_loc(Expression::Void));

                    continue;
                },
                BracketClose => {
                    builder.push(self.in_loc(Expression::Void));

                    break;
                },
                token => {
                    let expression = self.expression_from(token, 0);

                    builder.push(expression);
                }
            }

            match self.next() {
                BracketClose => break,
                Comma        => {},
                _            => unexpected_token!(self),
            }
        }

        Expression::Array {
            body: builder.into_list()
        }.at(0,0)
    }

    #[inline]
    pub fn regular_expression(&mut self) -> Loc<Expression<'ast>> {
        let value = match self.lexer.read_regular_expression() {
            Literal(value) => value,
            _              => unexpected_token!(self),
        };

        Expression::Value(value).at(0, 0)
    }

    fn template_expression(&mut self, tag: Option<ExpressionPtr<'ast>>, kind: TemplateKind<'ast>) -> Loc<Expression<'ast>> {
        let (quasi, expression) = match kind {
            TemplateKind::Open(quasi) => {
                let quasi = self.in_loc(quasi);

                let expression = self.sequence_or_expression();

                expect!(self, BraceClose);

                (quasi, expression)
            },

            TemplateKind::Closed(quasi) => {
                let quasi = self.in_loc(quasi);

                let template = Expression::Template {
                    tag,
                    expressions: List::empty(),
                    quasis: List::from(self.arena, quasi),
                };

                return self.in_loc(template);
            }
        };

        let mut quasis = ListBuilder::new(self.arena, quasi);
        let mut expressions = ListBuilder::new(self.arena, expression);

        loop {
            match self.lexer.read_template_kind() {
                Template(TemplateKind::Open(quasi)) => {
                    quasis.push(self.in_loc(quasi));
                    expressions.push(self.sequence_or_expression());

                    expect!(self, BraceClose);
                },
                Template(TemplateKind::Closed(quasi)) => {
                    quasis.push(self.in_loc(quasi));
                    break;
                },
                _ => unexpected_token!(self)
            }
        }

        Expression::Template {
            tag,
            expressions: expressions.into_list(),
            quasis: quasis.into_list(),
        }.at(0, 0)
    }

    #[inline]
    pub fn function_expression(&mut self) -> Loc<Expression<'ast>> {
        let name = match self.peek() {
            Identifier(name) => {
                self.consume();
                Some(self.alloc_in_loc(name))
            },
            _ => None
        };

        Expression::Function {
            function: self.function(name)
        }.at(0, 0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ast::{OperatorKind, Value, Statement, Function};
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
}
