use toolshed::list::ListBuilder;
use parser::{Parser, Parse, ANY, B0};
use lexer::Token::*;
use ast::{Node, NodeList, EmptyName, OptionalName, MandatoryName, Name};
use ast::{MethodKind, Pattern, Function, Class, ClassMember, PropertyKey};

impl<'ast> Parse<'ast> for EmptyName {
    type Output = Self;

    #[inline]
    fn parse(_: &mut Parser<'ast>) -> Self::Output {
        EmptyName
    }
}

impl<'ast> Parse<'ast> for OptionalName<'ast> {
    type Output = Self;

    #[inline]
    fn parse(par: &mut Parser<'ast>) -> Self::Output {
        if par.lexer.token != Identifier {
            return OptionalName(None);
        }

        let name = par.lexer.token_as_str();
        let name = OptionalName(Some(par.alloc_in_loc(name)));
        par.lexer.consume();
        name
    }
}

impl<'ast> Parse<'ast> for MandatoryName<'ast> {
    type Output = Self;

    #[inline]
    fn parse(par: &mut Parser<'ast>) -> Self::Output {
        if par.lexer.token != Identifier {
            return par.error();
        }

        let name = par.lexer.token_as_str();
        let name = MandatoryName(par.alloc_in_loc(name));
        par.lexer.consume();
        name
    }
}

impl<'ast> Parse<'ast> for Pattern<'ast> {
    type Output = Node<'ast, Self>;

    #[inline]
    fn parse(par: &mut Parser<'ast>) -> Self::Output {
        match par.lexer.token {
            Identifier  => par.pattern_identifier(),
            BracketOpen => par.pattern_array(),
            BraceOpen   => par.pattern_object(),
            _           => par.error()
        }
    }
}

impl<'ast, N> Parse<'ast> for Function<'ast, N> where
    N: Name<'ast> + Parse<'ast, Output = N>,
{
    type Output = Self;

    #[inline]
    fn parse(par: &mut Parser<'ast>) -> Self::Output {
        let generator: bool = if par.lexer.token == OperatorMultiplication {
            par.lexer.consume();
            true
        } else {
            false
        };

        let name = N::parse(par);

        Function {
            name,
            generator,
            params: par.params(),
            body: par.block(),
        }
    }
}

impl<'ast, N> Parse<'ast> for Node<'ast, Function<'ast, N>> where
    N: Name<'ast> + Parse<'ast, Output = N>,
{
    type Output = Self;

    #[inline]
    fn parse(par: &mut Parser<'ast>) -> Self::Output {
        let start = par.lexer.start();
        let function = Function::parse(par);

        par.alloc_at_loc(start, function.body.end, function)
    }
}

impl<'ast> Parse<'ast> for ClassMember<'ast> {
    type Output = Node<'ast, ClassMember<'ast>>;

    #[inline]
    fn parse(par: &mut Parser<'ast>) -> Self::Output {
        let start = par.lexer.start();

        let is_static = match par.lexer.token {
            Static => {
                par.lexer.consume();
                true
            }
            _ => false
        };

        let mut kind = MethodKind::Method;

        let token_start = par.lexer.start();
        let token_end;

        let key = match par.lexer.token {
            _ if par.lexer.token.is_word() => {
                let mut label = par.lexer.token_as_str();
                token_end = par.lexer.end_then_consume();

                if par.lexer.token.is_word() {
                    kind = match label {
                        "get" => MethodKind::Get,
                        "set" => MethodKind::Set,
                        _     => return par.error()
                    };
                    label = par.lexer.token_as_str();
                    par.lexer.consume();
                } else if !is_static && label == "constructor" {
                    kind = MethodKind::Constructor;
                }

                PropertyKey::Literal(label)
            },
            LiteralNumber => {
                let num = par.lexer.token_as_str();
                token_end = par.lexer.end_then_consume();
                PropertyKey::Literal(num)
            },
            LiteralBinary => {
                let num = par.lexer.token_as_str();
                token_end = par.lexer.end_then_consume();
                PropertyKey::Binary(num)
            },
            BracketOpen => {
                par.lexer.consume();

                let expression = par.expression::<ANY>();
                token_end = par.lexer.end();

                expect!(par, BracketClose);

                PropertyKey::Computed(expression)
            },
            _ => return par.error()
        };

        let key = par.alloc_at_loc(token_start, token_end, key);
        let end;
        let member = match par.lexer.token {
            ParenOpen => {
                let value = Node::parse(par);

                end = value.end;

                ClassMember::Method {
                    is_static,
                    key,
                    kind,
                    value,
                }
            },
            OperatorAssign => {
                par.lexer.consume();

                let expression = par.expression::<B0>();

                end = expression.end;

                ClassMember::Literal {
                    is_static,
                    key,
                    value: expression,
                }
            },
            _ => return par.error(),
        };

        if par.lexer.token == Semicolon {
            par.lexer.consume();
        }

        par.alloc_at_loc(start, end, member)
    }
}

impl<'ast, N> Parse<'ast> for Class<'ast, N> where
    N: Name<'ast> + Parse<'ast, Output = N>,
{
    type Output = Self;

    #[inline]
    fn parse(par: &mut Parser<'ast>) -> Self::Output {
        let name = N::parse(par);

        let super_class = match par.lexer.token {
            Extends => {
                par.lexer.consume();

                Some(par.expression::<B0>())
            },
            _ => None
        };

        Class {
            name,
            extends: super_class,
            body: par.block(),
        }
    }
}

impl<'ast> Parser<'ast> {
    #[inline]
    fn pattern_void(&mut self) -> Node<'ast, Pattern<'ast>> {
        let loc = self.lexer.start();
        self.alloc_at_loc(loc, loc, Pattern::Void)
    }

    #[inline]
    fn pattern_identifier(&mut self) -> Node<'ast, Pattern<'ast>> {
        let ident = Pattern::Identifier(self.lexer.token_as_str());
        let ident = self.alloc_in_loc(ident);

        self.lexer.consume();

        ident
    }

    #[inline]
    fn pattern_array(&mut self) -> Node<'ast, Pattern<'ast>> {
        let start = self.lexer.start_then_consume();
        let elements = self.array_elements(Parser::pattern_array_element);
        let end = self.lexer.end_then_consume();

        self.alloc_at_loc(start, end, Pattern::ArrayPattern {
            elements
        })
    }

    #[inline]
    fn pattern_object(&mut self) -> Node<'ast, Pattern<'ast>> {
        let start = self.lexer.start_then_consume();
        let properties = self.property_list();
        let end = self.lexer.end_then_consume();

        self.alloc_at_loc(start, end, Pattern::ObjectPattern {
            properties,
        })
    }

    #[inline]
    fn pattern_assign(&mut self, left: Node<'ast, Pattern<'ast>>) -> Node<'ast, Pattern<'ast>> {
        match self.lexer.token {
            OperatorAssign => {
                self.lexer.consume();

                let right = self.expression::<B0>();

                self.alloc_at_loc(left.start, right.end, Pattern::AssignmentPattern {
                    left,
                    right,
                })
            },
            _ => left
        }
    }

    #[inline]
    fn pattern_array_element(&mut self) -> Node<'ast, Pattern<'ast>> {
        let left = match self.lexer.token {
            Identifier           => self.pattern_identifier(),
            BracketOpen          => self.pattern_array(),
            BraceOpen            => self.pattern_object(),
            Comma | BracketClose => return self.pattern_void(),
            _                    => self.error()
        };

        self.pattern_assign(left)
    }

    #[inline]
    fn pattern_param(&mut self) -> Node<'ast, Pattern<'ast>> {
        let left = match self.lexer.token {
            Identifier           => self.pattern_identifier(),
            BracketOpen          => self.pattern_array(),
            BraceOpen            => self.pattern_object(),
            _                    => self.error()
        };

        self.pattern_assign(left)
    }

    #[inline]
    fn rest_element(&mut self) -> Node<'ast, Pattern<'ast>> {
        let start = self.lexer.start_then_consume();
        let argument = match self.lexer.token {
            Identifier => {
                let ident = self.lexer.token_as_str();
                let ident = self.alloc_in_loc(ident);

                self.lexer.consume();

                ident
            },
            _ => self.error()
        };

        expect!(self, ParenClose);

        self.alloc_at_loc(start, argument.end, Pattern::RestElement {
            argument
        })
    }

    #[inline]
    fn params(&mut self) -> NodeList<'ast, Pattern<'ast>> {
        expect!(self, ParenOpen);

        let item = match self.lexer.token {
            ParenClose     => {
                self.lexer.consume();

                return NodeList::empty();
            },
            OperatorSpread => return NodeList::from(self.arena, self.rest_element()),
            _              => self.pattern_param()
        };

        let builder = ListBuilder::new(self.arena, item);

        loop {
            match self.lexer.token {
                Comma => {
                    self.lexer.consume();
                },
                ParenClose => {
                    self.lexer.consume();

                    break;
                },
                _ => {
                    self.error::<()>();

                    break;
                }
            }

            match self.lexer.token {
                ParenClose => {
                    self.lexer.consume();

                    break;
                },
                OperatorSpread => {
                    builder.push(self.arena, self.rest_element());

                    break;
                },
                _ => {
                    builder.push(self.arena, self.pattern_param());
                }
            }
        }

        builder.as_list()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use parser::parse;
    use parser::mock::Mock;
    use ast::{NodeList, Literal, Expression, Function, Class};
    use ast::{ClassMember, Pattern};
    use ast::statement::*;

    #[test]
    fn function_empty() {
        let src = "function foo() {}";
        let mock = Mock::new();

        let expected = mock.list([
            Function {
                name: mock.name("foo"),
                generator: false,
                params: NodeList::empty(),
                body: mock.empty_block(),
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn function_with_generator_flag() {
        {
            let src = "function* foo() {}";
            let mock = Mock::new();

            let expected = mock.list([
                Function {
                    name: mock.name("foo"),
                    generator: true,
                    params: NodeList::empty(),
                    body: mock.empty_block(),
                }
            ]);

            assert_eq!(parse(src).unwrap().body(), expected);
        }
        
        {
            let src = "function * foo() {}";
            let mock = Mock::new();

            let expected = mock.list([
                Function {
                    name: mock.name("foo"),
                    generator: true,
                    params: NodeList::empty(),
                    body: mock.empty_block(),
                }
            ]);

            assert_eq!(parse(src).unwrap().body(), expected);
        }

        {
            let src = "function *foo() {}";
            let mock = Mock::new();

            let expected = mock.list([
                Function {
                    name: mock.name("foo"),
                    generator: true,
                    params: NodeList::empty(),
                    body: mock.empty_block(),
                }
            ]);

            assert_eq!(parse(src).unwrap().body(), expected);
        }
    }

    #[test]
    fn function_params() {
        let src = "function foo(bar, baz) {}";
        let mock = Mock::new();

        let expected = mock.list([
            Function {
                name: mock.name("foo"),
                generator: false,
                params: mock.list([
                    Pattern::Identifier("bar"),
                    Pattern::Identifier("baz"),
                ]),
                body: mock.empty_block(),
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn function_body() {
        let src = "function foo() { bar; baz; }";
        let mock = Mock::new();

        let expected = mock.list([
            Function {
                name: mock.name("foo"),
                generator: false,
                params: NodeList::empty(),
                body: mock.block([
                    mock.ptr("bar"),
                    mock.ptr("baz"),
                ])
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn function_with_default_params() {
        let src = "function foo (a = 0, b = 1, c = 2) { return 2 }";
        let mock = Mock::new();

        let expected = mock.list([
            Function {
                name: mock.name("foo"),
                generator: false,
                params: mock.list([
                    Pattern::AssignmentPattern {
                        left: mock.ptr(Pattern::Identifier("a")),
                        right: mock.number("0")
                    },
                    Pattern::AssignmentPattern {
                        left: mock.ptr(Pattern::Identifier("b")),
                        right: mock.number("1")
                    },
                    Pattern::AssignmentPattern {
                        left: mock.ptr(Pattern::Identifier("c")),
                        right: mock.number("2")
                    }
                ]),
                body: mock.block([
                    ReturnStatement {
                        value: Some(mock.number("2"))
                    }
                ])
            }
        ]);
        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn function_with_non_trailing_default_params() {
        let src = "function foo (a, b, c = 2, d) { return 2 }";
        let mock = Mock::new();

        let expected = mock.list([
            Function {
                name: mock.name("foo"),
                generator: false,
                params: mock.list([
                    Pattern::Identifier("a"),
                    Pattern::Identifier("b"),
                    Pattern::AssignmentPattern {
                        left: mock.ptr(Pattern::Identifier("c")),
                        right: mock.number("2")
                    },
                    Pattern::Identifier("d")
                ]),
                body: mock.block([
                    ReturnStatement {
                        value: Some(mock.number("2"))
                    }
                ])
            }
        ]);
        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn function_with_rest_element() {
        let src = "function foo(...rest) {}";
        let mock = Mock::new();

        let expected = mock.list([
            Function {
                name: mock.name("foo"),
                generator: false,
                params: mock.list([
                    Pattern::RestElement {
                        argument: mock.ptr("rest"),
                    }
                ]),
                body: mock.empty_block()
            }
        ]);
        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn function_with_tailing_rest_element() {
        let src = "function foo(a, b = 10, ...rest) {}";
        let mock = Mock::new();

        let expected = mock.list([
            Function {
                name: mock.name("foo"),
                generator: false,
                params: mock.list([
                    Pattern::Identifier("a"),
                    Pattern::AssignmentPattern {
                        left: mock.ptr(Pattern::Identifier("b")),
                        right: mock.number("10")
                    },
                    Pattern::RestElement {
                        argument: mock.ptr("rest"),
                    }
                ]),
                body: mock.empty_block()
            }
        ]);
        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn function_with_non_trailing_rest_element() {
        assert!(parse("function foo(...rest, a) {}").is_err());
    }

    #[test]
    fn class_empty() {
        let src = "class Foo {}";
        let mock = Mock::new();

        let expected = mock.list([
            Class {
                name: mock.name("Foo"),
                extends: None,
                body: mock.empty_block(),
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn child_class_empty() {
        let src = "class Foo extends Bar {}";
        let mock = Mock::new();

        let expected = mock.list([
            Class {
                name: mock.name("Foo"),
                extends: Some(mock.ptr("Bar")),
                body: mock.empty_block(),
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn class_constructor() {
        let src = r#"

        class Foo {
            constructor(bar, baz) {
                debug;
            }
        }

        "#;
        let mock = Mock::new();

        let expected = mock.list([
            Class {
                name: mock.name("Foo"),
                extends: None,
                body: mock.block([
                    ClassMember::Method {
                        is_static: false,
                        key: mock.ptr(PropertyKey::Literal("constructor")),
                        kind: MethodKind::Constructor,
                        value: mock.ptr(Function {
                            name: EmptyName,
                            generator: false,
                            params: mock.list([
                                Pattern::Identifier("bar"),
                                Pattern::Identifier("baz")
                            ]),
                            body: mock.block([
                                mock.ptr("debug")
                            ])
                        })
                    }
                ])
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn class_method() {
        let src = r#"

        class Foo {
            doge(bar, baz) {
                debug;
            }

            static toThe(moon) {
                debug;
            }

            function() {}
            static function() {}
            static constructor() {}
        }

        "#;
        let mock = Mock::new();

        let expected = mock.list([
            Class {
                name: mock.name("Foo"),
                extends: None,
                body: mock.block([
                    ClassMember::Method {
                        is_static: false,
                        key: mock.ptr(PropertyKey::Literal("doge")),
                        kind: MethodKind::Method,
                        value: mock.ptr(Function {
                            name: EmptyName,
                            generator: false,
                            params: mock.list([
                                Pattern::Identifier("bar"),
                                Pattern::Identifier("baz")
                            ]),
                            body: mock.block([
                                mock.ptr("debug")
                            ])
                        })
                    },
                    ClassMember::Method {
                        is_static: true,
                        key: mock.ptr(PropertyKey::Literal("toThe")),
                        kind: MethodKind::Method,
                        value: mock.ptr(Function {
                            name: EmptyName,
                            generator: false,
                            params: mock.list([
                                Pattern::Identifier("moon")
                            ]),
                            body: mock.block([
                                mock.ptr("debug")
                            ])
                        })
                    },
                    ClassMember::Method {
                        is_static: false,
                        key: mock.ptr(PropertyKey::Literal("function")),
                        kind: MethodKind::Method,
                        value: mock.ptr(Function {
                            name: EmptyName,
                            generator: false,
                            params: NodeList::empty(),
                            body: mock.empty_block()
                        })
                    },
                    ClassMember::Method {
                        is_static: true,
                        key: mock.ptr(PropertyKey::Literal("function")),
                        kind: MethodKind::Method,
                        value: mock.ptr(Function {
                            name: EmptyName,
                            generator: false,
                            params: NodeList::empty(),
                            body: mock.empty_block()
                        })
                    },
                    ClassMember::Method {
                        is_static: true,
                        key: mock.ptr(PropertyKey::Literal("constructor")),
                        kind: MethodKind::Method,
                        value: mock.ptr(Function {
                            name: EmptyName,
                            generator: false,
                            params: NodeList::empty(),
                            body: mock.empty_block()
                        })
                    },
                ])
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn class_value() {
        let src = r#"

        class Foo {
            doge = 10;
            to = 20;
            the = 30;
            static moon = 42;
        }

        "#;
        let mock = Mock::new();

        let expected = mock.list([
            Class {
                name: mock.name("Foo"),
                extends: None,
                body: mock.block([
                    ClassMember::Literal {
                        is_static: false,
                        key: mock.ptr(PropertyKey::Literal("doge")),
                        value: mock.number("10")
                    },
                    ClassMember::Literal {
                        is_static: false,
                        key: mock.ptr(PropertyKey::Literal("to")),
                        value: mock.number("20")
                    },
                    ClassMember::Literal {
                        is_static: false,
                        key: mock.ptr(PropertyKey::Literal("the")),
                        value: mock.number("30")
                    },
                    ClassMember::Literal {
                        is_static: true,
                        key: mock.ptr(PropertyKey::Literal("moon")),
                        value: mock.number("42")
                    },
                ])
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }


    #[test]
    fn class_extends_null() {
        let src = "class Foo extends null {}";
        let mock = Mock::new();

        let expected = mock.list([
            Class {
                name: mock.name("Foo"),
                extends: Some(mock.ptr(Expression::Literal(Literal::Null))),
                body: mock.empty_block()
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn class_methods() {
        let src = r#"

        class Foo {
            get length (foo) { }
            set length (bar) { }
        }

        "#;
        let mock = Mock::new();

        let expected = mock.list([
            Class {
                name: mock.name("Foo"),
                extends: None,
                body: mock.block([
                    ClassMember::Method {
                        is_static: false,
                        key: mock.ptr(PropertyKey::Literal("length")),
                        kind: MethodKind::Get,
                        value: mock.ptr(Function {
                            name: EmptyName,
                            generator: false,
                            params: mock.list([
                                Pattern::Identifier("foo")
                            ]),
                            body: mock.empty_block()
                        })
                    },
                    ClassMember::Method {
                        is_static: false,
                        key: mock.ptr(PropertyKey::Literal("length")),
                        kind: MethodKind::Set,
                        value: mock.ptr(Function {
                            name: EmptyName,
                            generator: false,
                            params: mock.list([
                                Pattern::Identifier("bar")
                            ]),
                            body: mock.empty_block()
                        })
                    },
                ])
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }
}
