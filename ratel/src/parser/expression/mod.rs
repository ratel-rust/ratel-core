use toolshed::list::ListBuilder;
use parser::{Parser, Parse, BindingPower, ANY, B0, B15};
use lexer::TokenTable;
use lexer::Token::*;
use ast::{Node, NodeList, Expression, ExpressionNode, IdentifierNode, ExpressionList};
use ast::{Property, PropertyKey, OperatorKind, Literal, Function, Class, StatementNode};
use ast::expression::*;

mod prefix;
mod literal;
mod array;
mod object;
mod template;

pub use self::prefix::*;
pub use self::literal::*;
pub use self::object::ObjectExpressionHandler;
pub use self::array::ArrayExpressionHandler;
pub use self::template::{TemplateStringLiteralHandler, TemplateExpressionHandler};

pub type ExpressionHandlerFn = for<'ast> fn(&mut Parser<'ast>) -> ExpressionNode<'ast>;

pub type Context = &'static TokenTable<ExpressionHandlerFn>;

macro_rules! create_handlers {
    ($( const $name:ident = |$par:ident| $code:expr; )*) => {
        $(
            pub struct $name;

            impl ExpressionHandler for $name {
                fn expression<'ast>($par: &mut Parser<'ast>) -> ExpressionNode<'ast> {
                    $code
                }
            }
        )*
    };
}

pub trait ExpressionHandler {
    fn expression<'ast>(par: &mut Parser<'ast>) -> ExpressionNode<'ast>;
}

impl<E> crate::parser::statement::StatementHandler for E
where
    E: ExpressionHandler
{
    fn statement<'ast>(par: &mut Parser<'ast>) -> StatementNode<'ast> {
        let expression = Self::expression(par);
        par.expression_statement(expression)
    }
}

pub fn error<'ast>(par: &mut Parser<'ast>) -> ExpressionNode<'ast> {
    let loc = par.lexer.start();
    par.error::<()>();
    par.node_at(loc, loc, Expression::Void)
}

pub fn void<'ast>(par: &mut Parser<'ast>) -> ExpressionNode<'ast> {
    let loc = par.lexer.start();
    par.node_at(loc, loc, Expression::Void)
}

create_handlers! {
    const SpreadExpressionHandler = |par| {
        let start = par.lexer.start_then_consume();
        let argument = par.expression::<B0>();

        par.node_at(start, argument.end, SpreadExpression { argument })
    };

    const ThisHandler = |par| {
        par.node_consume(ThisExpression)
    };

    const NewHandler = |par| {
        let (start, op_end) = par.lexer.loc();

        par.lexer.consume();

        if par.lexer.token == Accessor {
            par.lexer.consume();

            let meta = par.node_at(start, op_end, "new");
            let expression = par.meta_property_expression(meta);
            let end = par.lexer.end_then_consume();

            par.node_at(start, end, expression)
        } else {
            let operand = par.expression::<B15>();
            let end = operand.end;

            par.node_at(start, end, PrefixExpression {
                operator: OperatorKind::New,
                operand,
            })
        }
    };

    const ParenHandler = |par| {
        let start = par.lexer.start_then_consume();
        match par.lexer.token {
            ParenClose => {
                par.lexer.consume();
                expect!(par, OperatorFatArrow);
                let expression = par.arrow_function_expression(NodeList::empty());
                let end = par.lexer.end();
                par.node_at(start, end, expression)
            },
            _ => {
                let expression = par.expression::<ANY>();

                expect!(par, ParenClose);

                expression
            }
        }
    };

    const RegExHandler = |par| {
        let start = par.lexer.start();
        let value = par.lexer.read_regular_expression();
        let end = par.lexer.end();

        expect!(par, LiteralRegEx);

        par.node_at(start, end, Literal::RegEx(value))
    };
}

impl<'ast> Parser<'ast> {
    fn bound_expression(&mut self) -> ExpressionNode<'ast> {
        self.set.expressions.default.get(self.lexer.token)(self)
    }

    fn context_bound_expression(&mut self, context: Context) -> ExpressionNode<'ast> {
        context.get(self.lexer.token)(self)
    }

    pub fn expression<B>(&mut self) -> ExpressionNode<'ast>
    where
        B: BindingPower
    {
        let left = self.bound_expression();

        self.nested_expression::<B>(left)
    }

    pub fn expression_in_array_context<B>(&mut self) -> ExpressionNode<'ast>
    where
        B: BindingPower
    {
        let left = self.context_bound_expression(&self.set.expressions.array);

        self.nested_expression::<B>(left)
    }

    pub fn expression_in_call_context<B>(&mut self) -> ExpressionNode<'ast>
    where
        B: BindingPower
    {
        let left = self.context_bound_expression(&self.set.expressions.call);

        self.nested_expression::<B>(left)
    }

    pub fn arrow_function_expression(&mut self, params: ExpressionList<'ast>) -> ArrowExpression<'ast> {
        let params = self.params_from_expressions(params);

        let body = match self.lexer.token {
            BraceOpen => ArrowBody::Block(self.unchecked_block()),
            _         => ArrowBody::Expression(self.expression::<B0>()),
        };

        ArrowExpression {
            params,
            body,
        }
    }

    pub fn call_arguments(&mut self) -> ExpressionList<'ast> {
        if self.lexer.token == ParenClose {
            return NodeList::empty();
        }

        let expression = self.expression_in_call_context::<B0>();
        let builder = ListBuilder::new(self.arena, expression);

        loop {
            let expression = match self.lexer.token {
                ParenClose => break,
                Comma      => {
                    self.lexer.consume();

                    if self.lexer.token == ParenClose {
                        break
                    }

                    self.expression_in_call_context::<B0>()
                }
                _ => {
                    self.error::<()>();
                    break;
                }
            };

            builder.push(self.arena, expression);
        }

        builder.as_list()
    }

    pub fn meta_property_expression(&mut self, meta: IdentifierNode<'ast>) -> MetaPropertyExpression<'ast> {
        let property = self.lexer.token_as_str();

        // Only `NewTarget` is a valid MetaProperty.
        if !self.lexer.token.is_word() || property != "target" {
            self.error::<()>();
        }

        let property = self.node(property);

        MetaPropertyExpression {
            meta,
            property,
        }
    }

    pub fn property_list(&mut self) -> NodeList<'ast, Property<'ast>> {
        if self.lexer.token == BraceClose {
            return NodeList::empty();
        }

        let builder = ListBuilder::new(self.arena, self.property());

        loop {
            match self.lexer.token {
                BraceClose => break,
                Comma      => self.lexer.consume(),
                _          => {
                    self.error::<()>();
                    break;
                }
            }

            match self.lexer.token {
                BraceClose => break,
                _          => builder.push(self.arena, self.property()),
            }
        }

        builder.as_list()
    }

    pub fn property(&mut self) -> Node<'ast, Property<'ast>> {
        let start = self.lexer.start();

        let key = match self.lexer.token {
            _ if self.lexer.token.is_word() => {
                let (start, end) = self.lexer.loc();
                let label = self.lexer.token_as_str();

                self.lexer.consume();

                match self.lexer.token {
                    Colon | ParenOpen => self.node_at(start, end, PropertyKey::Literal(label)),

                    _ => return self.node_at(start, end, Property::Shorthand(label)),
                }
            },
            LiteralString |
            LiteralNumber => self.node_consume_str(|num| PropertyKey::Literal(num)),
            LiteralBinary => self.node_consume_str(|num| PropertyKey::Binary(num)),
            BracketOpen => {
                let start = self.lexer.start_then_consume();
                let expression = self.expression::<ANY>();
                let end = self.lexer.end();

                expect!(self, BracketClose);

                self.node_at(start, end, PropertyKey::Computed(expression))
            },
            _ => return self.error(),
        };

        match self.lexer.token {
            Colon => {
                self.lexer.consume();

                let value = self.expression::<B0>();

                self.node_at(start, value.end, Property::Literal {
                    key,
                    value,
                })
            },
            ParenOpen => {
                let value = Node::parse(self);

                self.node_at(start, value.end, Property::Method {
                    key,
                    value,
                })
            },
            _ => return self.error()
        }
    }

    pub fn array_elements<F, I>(&mut self, get: F) -> NodeList<'ast, I> where
        F: Fn(&mut Parser<'ast>) -> Node<'ast, I>,
        I: 'ast + Copy,
    {
        let item = match self.lexer.token {
            BracketClose => return NodeList::empty(),
            _            => get(self),
        };

        let builder = ListBuilder::new(self.arena, item);

        loop {
            match self.lexer.token {
                Comma        => self.lexer.consume(),
                BracketClose => break,
                _            => {
                    self.error::<()>();
                    break;
                }
            }

            builder.push(self.arena, get(self))
        }

        builder.as_list()
    }

    pub fn function_expression(&mut self) -> ExpressionNode<'ast> {
        let start = self.lexer.start_then_consume();
        let function = Function::parse(self);

        self.node_at(start, function.body.end, function)
    }

    pub fn class_expression(&mut self) -> ExpressionNode<'ast> {
        let start = self.lexer.start_then_consume();
        let class = Class::parse(self);

        self.node_at(start, class.body.end, class)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ast::{OperatorKind, Literal, Statement, Function, Pattern, Class};
    use ast::expression::*;
    use ast::statement::*;
    use parser::parse;
    use parser::mock::Mock;

    #[test]
    fn ident_expression() {
        let expected = Expression::Identifier("foobar");

        assert_expr!("foobar;", expected);
    }

    #[test]
    fn value_expression() {
        let expected_a = Literal::String(r#""foobar""#);
        let expected_b = Literal::Number("100");
        let expected_c = Literal::True;

        assert_expr!(r#""foobar";"#, expected_a);
        assert_expr!("100;", expected_b);
        assert_expr!("true;", expected_c);
    }

    #[test]
    fn template_expression() {
        let src = "`foobar`;";
        let mock = Mock::new();

        let expected = TemplateLiteral {
            expressions: NodeList::empty(),
            quasis: mock.list(["foobar"]),
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn tagged_template_expression() {
        let src = "foo`bar`;";
        let mock = Mock::new();

        let expected = TaggedTemplateExpression {
            tag: mock.ptr("foo"),
            quasi: mock.ptr(TemplateLiteral {
                expressions: NodeList::empty(),
                quasis: mock.list(["bar"]),
            })
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn complex_template_expression() {
        let src = "`foo${ 10 }bar${ 20 }baz`;";
        let mock = Mock::new();

        let expected = TemplateLiteral {
            expressions: mock.list([
                Literal::Number("10"),
                Literal::Number("20"),
            ]),
            quasis: mock.list(["foo", "bar", "baz" ]),
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn tagged_complex_template_expression() {
        let src = "foo`bar${ 42 }baz`;";
        let mock = Mock::new();

        let expected = TaggedTemplateExpression {
            tag: mock.ptr("foo"),
            quasi: mock.ptr(TemplateLiteral {
                expressions: mock.list([
                    Literal::Number("42"),
                ]),
                quasis: mock.list(["bar", "baz"]),
            })
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn sequence_expression() {
        let src = "foo, bar, baz;";
        let mock = Mock::new();

        let expected = SequenceExpression {
            body: mock.list(["foo", "bar", "baz"]),
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn binary_expression() {
        let src = "foo + bar;";
        let mock = Mock::new();

        let expected = BinaryExpression {
            operator: OperatorKind::Addition,
            left: mock.ptr("foo"),
            right: mock.ptr("bar"),
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn parenthesized_binary_expression() {
        let src = "(2 + 2);";
        let mock = Mock::new();

        let expected = BinaryExpression {
            operator: OperatorKind::Addition,
            left: mock.number("2"),
            right: mock.number("2"),
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn conditional_expression() {
        let src = "true ? foo : bar";

        let mock = Mock::new();

        let expected = ConditionalExpression {
            test: mock.ptr(Expression::Literal(Literal::True)),
            consequent: mock.ptr("foo"),
            alternate: mock.ptr("bar"),
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn postfix_expression() {
        let src = "baz++;";
        let mock = Mock::new();

        let expected = PostfixExpression {
            operator: OperatorKind::Increment,
            operand: mock.ptr("baz"),
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn call_expression() {
        {
            let src = "foo();";
            let mock = Mock::new();

            let expected = CallExpression {
                callee: mock.ptr("foo"),
                arguments: NodeList::empty(),
            };

            assert_expr!(src, expected);
        }

        {
            let src = "foo(1);";
            let mock = Mock::new();

            let expected = CallExpression {
                callee: mock.ptr("foo"),
                arguments: mock.list([
                    Literal::Number("1"),
                ]),
            };

            assert_expr!(src, expected);
        }

        {
            let src = "foo(1,2);";
            let mock = Mock::new();

            let expected = CallExpression {
                callee: mock.ptr("foo"),
                arguments: mock.list([
                    Literal::Number("1"),
                    Literal::Number("2"),
                ]),
            };

            assert_expr!(src, expected);
        }

        {
            let src = "foo(1,);";
            let mock = Mock::new();

            let expected = CallExpression {
                callee: mock.ptr("foo"),
                arguments: mock.list([
                    Literal::Number("1"),
                ]),
            };

            assert_expr!(src, expected);
        }

        {
            let src = "foo(1,2,);";
            let mock = Mock::new();

            let expected = CallExpression {
                callee: mock.ptr("foo"),
                arguments: mock.list([
                    Literal::Number("1"),
                    Literal::Number("2"),
                ]),
            };

            assert_expr!(src, expected);
        }
    }

    #[test]
    fn member_expression() {
        let src = "foo.bar";
        let mock = Mock::new();

        let expected = MemberExpression {
            object: mock.ptr("foo"),
            property: mock.ptr("bar"),
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn keyword_member_expression() {
        let src = "foo.function";
        let mock = Mock::new();

        let expected = MemberExpression {
            object: mock.ptr("foo"),
            property: mock.ptr("function"),
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn computed_member_expression() {
        let src = "foo[10]";
        let mock = Mock::new();

        let expected = ComputedMemberExpression {
            object: mock.ptr("foo"),
            property: mock.number("10"),
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn meta_property_expression() {
        let src = "new.target";
        let mock = Mock::new();
        let expected = MetaPropertyExpression {
            meta: mock.ptr("new"),
            property: mock.ptr("target"),
        };
        assert_expr!(src, expected);
    }

    #[test]
    fn meta_property_expression_throws() {
        assert!(parse("new.callee").is_err());
    }

    #[test]
    fn regular_expression() {
        let src = r#"/^[A-Z]+\/[\d]+/g"#;

        let expected = Literal::RegEx("/^[A-Z]+\\/[\\d]+/g");

        assert_expr!(src, expected);
    }

    #[test]
    fn array_expression() {
        let src = "[0, 1, 2]";
        let mock = Mock::new();

        let expected = ArrayExpression {
            body: mock.list([
                Literal::Number("0"),
                Literal::Number("1"),
                Literal::Number("2"),
            ])
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn sparse_array_expression() {
        let src = "[,,foo,bar,,]";
        let mock = Mock::new();

        let expected = ArrayExpression {
            body: mock.list([
                Expression::Void,
                Expression::Void,
                Expression::Identifier("foo"),
                Expression::Identifier("bar"),
                Expression::Void,
                Expression::Void,
            ])
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn spread_expression_in_array() {
        let src = "[a, b, ...c]";
        let mock = Mock::new();

        let expected = ArrayExpression {
            body: mock.list([
                Expression::Identifier("a"),
                Expression::Identifier("b"),
                Expression::Spread(SpreadExpression {
                    argument: mock.ptr("c")
                })
            ])
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn spread_expression_in_call() {
        let src = "foo(a, b, ...c)";
        let mock = Mock::new();

        let expected = CallExpression {
            callee: mock.ptr("foo"),
            arguments: mock.list([
                Expression::Identifier("a"),
                Expression::Identifier("b"),
                Expression::Spread(SpreadExpression {
                    argument: mock.ptr("c")
                })
            ])
        };

        assert_expr!(src, expected);
    }


    #[test]
    fn spread_expression_illegal_bare() {
        assert!(parse("let foo = ...c;").is_err());
    }

    #[test]
    fn function_expression() {
        let src = "(function () {})";
        let mock = Mock::new();

        let expected = Function {
            name: None.into(),
            generator: false,
            params: NodeList::empty(),
            body: mock.empty_block()
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn named_function_expression() {
        let src = "(function foo () {})";
        let mock = Mock::new();

        let expected = Function {
            name: mock.name("foo"),
            generator: false,
            params: NodeList::empty(),
            body: mock.empty_block()
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn arrow_function_expression() {
        let src = "() => bar";
        let mock = Mock::new();

        let expected = ArrowExpression {
            params: NodeList::empty(),
            body: ArrowBody::Expression(mock.ptr("bar")),
        };
        assert_expr!(src, expected);
    }

    #[test]
    fn arrow_function_shorthand() {
        let src = "n => n * n";
        let mock = Mock::new();

        let expected = ArrowExpression {
            params: mock.list([
                Pattern::Identifier("n")
            ]),

            body: ArrowBody::Expression(mock.ptr(BinaryExpression {
                operator: OperatorKind::Multiplication,
                left: mock.ptr("n"),
                right: mock.ptr("n"),
            }))

        };
        assert_expr!(src, expected);
    }

    #[test]
    fn arrow_function_with_params() {
        let src = "(a, b, c) => bar";
        let mock = Mock::new();

        let expected = ArrowExpression {
            params: mock.list([
                Pattern::Identifier("a"),
                Pattern::Identifier("b"),
                Pattern::Identifier("c")
            ]),
            body: ArrowBody::Expression(mock.ptr("bar"))
        };
        assert_expr!(src, expected);
    }

    #[test]
    fn arrow_function_invalid_params_throws() {
        assert!(parse("(a, b, c * 2) => bar").is_err());
    }

    #[test]
    fn arrow_function_with_default_params() {
        let src = "(a, b, c = 2) => bar";
        let mock = Mock::new();

        let expected = ArrowExpression {
            params: mock.list([
                Pattern::Identifier("a"),
                Pattern::Identifier("b"),
                Pattern::AssignmentPattern {
                    left: mock.ptr(Pattern::Identifier("c")),
                    right: mock.number("2")
                }
            ]),
            body: ArrowBody::Expression(mock.ptr("bar"))
        };
        assert_expr!(src, expected);
    }

    #[test]
    fn class_expression() {
        let src = "(class {})";
        let mock = Mock::new();

        let expected = Class {
            name: None.into(),
            extends: None,
            body: mock.empty_block()
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn named_class_expression() {
        let src = "(class Foo {})";
        let mock = Mock::new();

        let expected = Class {
            name: mock.name("Foo"),
            extends: None,
            body: mock.empty_block()
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn named_child_class_expression() {
        let src = "(class Foo extends Bar {})";
        let mock = Mock::new();

        let expected = Class {
            name: mock.name("Foo"),
            extends: Some(mock.ptr("Bar")),
            body: mock.empty_block()
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn regression_operator_precedence() {
        let src = "true === true && false === false";
        let mock = Mock::new();

        let expected = BinaryExpression {
            operator: OperatorKind::LogicalAnd,
            left: mock.ptr(BinaryExpression {
                operator: OperatorKind::StrictEquality,
                left: mock.ptr(Literal::True),
                right: mock.ptr(Literal::True),
            }),
            right: mock.ptr(BinaryExpression {
                operator: OperatorKind::StrictEquality,
                left: mock.ptr(Literal::False),
                right: mock.ptr(Literal::False),
            }),
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn arrow_function_in_sequence() {
        let src = "(() => {}, foo)";
        let mock = Mock::new();

        let expected = SequenceExpression {
            body: mock.list([
                Expression::Arrow(ArrowExpression {
                    params: NodeList::empty(),
                    body: ArrowBody::Block(mock.ptr(BlockStatement {
                        body: NodeList::empty()
                    }))
                }),
                Expression::Identifier("foo"),
            ])
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn regression_increments() {
        let src = "x++ + ++y";
        let mock = Mock::new();

        let expected = BinaryExpression {
            operator: OperatorKind::Addition,
            left: mock.ptr(PostfixExpression {
                operator: OperatorKind::Increment,
                operand: mock.ptr("x"),
            }),
            right: mock.ptr(PrefixExpression {
                operator: OperatorKind::Increment,
                operand: mock.ptr("y"),
            })
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn regression_decrements() {
        let src = "x-- - --y";
        let mock = Mock::new();

        let expected = BinaryExpression {
            operator: OperatorKind::Subtraction,
            left: mock.ptr(PostfixExpression {
                operator: OperatorKind::Decrement,
                operand: mock.ptr("x"),
            }),
            right: mock.ptr(PrefixExpression {
                operator: OperatorKind::Decrement,
                operand: mock.ptr("y"),
            })
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn assignment_to_lvalue() {
        assert!(parse("(x++)++").is_err());
        assert!(parse("x+++++y").is_err());
    }

    #[test]
    fn regression_asi_increments() {
        let src = r#"x
        ++
        y"#;
        let mock = Mock::new();

        let expected = mock.list([
            mock.ptr(Expression::Identifier("x")),
            mock.ptr(PrefixExpression {
                operator: OperatorKind::Increment,
                operand: mock.ptr("y"),
            }),
        ]);
        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn regression_asi_decrements() {
        let src = r#"x
        --
        y"#;
        let mock = Mock::new();

        let expected = mock.list([
            mock.ptr(Expression::Identifier("x")),
            mock.ptr(PrefixExpression {
                operator: OperatorKind::Decrement,
                operand: mock.ptr("y"),
            }),
        ]);
        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn regression_asi_safe() {
        let src = r#"foo
        .bar"#;
        let mock = Mock::new();

        let expected = MemberExpression {
            object: mock.ptr("foo"),
            property: mock.ptr("bar"),
        };

        assert_expr!(src, expected);
    }
}
