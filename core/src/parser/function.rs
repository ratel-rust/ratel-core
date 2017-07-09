use parser::Parser;
use lexer::Token;
use lexer::Token::*;
use ast::{Loc, EmptyListBuilder, Name, Function, Class, ClassMember, Property, Value, OperatorKind};

impl<'ast> Parser<'ast> {
    #[inline]
    pub fn function<N, I>(&mut self, name: I) -> Function<'ast, N> where
        N: Name<'ast>,
        I: Into<N>,
    {
        expect!(self, ParenOpen);

        Function {
            name: name.into(),
            params: self.parameter_list(),
            body: self.block_body(),
        }
    }

    #[inline]
    pub fn class<N, I>(&mut self, name: I) -> Class<'ast, N> where
        N: Name<'ast>,
        I: Into<N>,
    {
        let super_class = match self.next() {
            Extends   => {
                let name = expect_identifier!(self);
                let name = self.alloc_in_loc(name);

                expect!(self, BraceOpen);

                Some(name)
            },
            BraceOpen => None,
            _         => unexpected_token!(self)
        };

        let mut body = EmptyListBuilder::new(self.arena);

        loop {
            match self.next() {
                Semicolon  => continue,
                BraceClose => break,
                Static     => {
                    let token = self.next();
                    body.push(self.class_member(token, true))
                },
                token      => body.push(self.class_member(token, false)),
            }
        }

        Class {
            name: name.into(),
            extends: super_class,
            body: body.into_list(),
        }
    }

    fn class_member(&mut self, token: Token<'ast>, is_static: bool) -> Loc<ClassMember<'ast>> {
        let property = match token {
            Identifier(label) => Property::Literal(label),
            Literal(Value::Number(num)) => Property::Literal(num),
            Literal(Value::Binary(num)) => Property::Binary(num),
            BracketOpen => {
                let expression = self.sequence_or_expression();

                expect!(self, BracketClose);

                Property::Computed(self.alloc(expression))
            },
            _ => {
                // Allow word tokens such as "null" and "typeof" as identifiers
                match token.as_word() {
                    Some(label) => Property::Literal(label),
                    _           => unexpected_token!(self)
                }
            }
        };

        match self.next() {
            ParenOpen => {
                let params = self.parameter_list();
                let body = self.block_body();

                Loc::new(0, 0, if !is_static && property.is_constructor() {
                    ClassMember::Constructor {
                        params,
                        body,
                    }
                } else {
                    ClassMember::Method {
                        is_static: is_static,
                        property,
                        params,
                        body,
                    }
                })
            },
            Operator(OperatorKind::Assign) => {
                let expression = self.expression(0);

                Loc::new(0, 0, ClassMember::Value {
                    is_static,
                    property,
                    value: self.alloc(expression),
                })
            },
            _ => unexpected_token!(self),
        }
    }
}

#[cfg(test)]
mod test {
    use parser::parse;
    use parser::mock::Mock;
    use ast::{List, Statement, Function, Class, ClassMember, Property, Parameter};

    #[test]
    fn function_empty() {
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
    fn function_params() {
        let src = "function foo(bar, baz) {}";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Function {
                function: Function {
                    name: mock.ptr("foo").into(),
                    params: mock.list([
                        Parameter::Identifier {
                            label: "bar",
                            value: None,
                        },
                        Parameter::Identifier {
                            label: "baz",
                            value: None,
                        },
                    ]),
                    body: List::empty(),
                }
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn function_body() {
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
    fn class_empty() {
        let src = "class Foo {}";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Class {
                class: Class {
                    name: mock.ptr("Foo").into(),
                    extends: None,
                    body: List::empty(),
                }
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn child_class_empty() {
        let src = "class Foo extends Bar {}";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Class {
                class: Class {
                    name: mock.ptr("Foo").into(),
                    extends: Some(mock.ptr("Bar")),
                    body: List::empty(),
                }
            }
        ]);

        assert_eq!(module.body(), expected);
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
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Class {
                class: Class {
                    name: mock.ptr("Foo").into(),
                    extends: None,
                    body: mock.list([
                        ClassMember::Constructor {
                            params: mock.list([
                                Parameter::Identifier {
                                    label: "bar",
                                    value: None,
                                },
                                Parameter::Identifier {
                                    label: "baz",
                                    value: None,
                                },
                            ]),
                            body: mock.list([
                                Statement::Expression {
                                    expression: mock.ident("debug")
                                }
                            ])
                        }
                    ])
                }
            }
        ]);

        assert_eq!(module.body(), expected);
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
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Class {
                class: Class {
                    name: mock.ptr("Foo").into(),
                    extends: None,
                    body: mock.list([
                        ClassMember::Method {
                            is_static: false,
                            property: Property::Literal("doge"),
                            params: mock.list([
                                Parameter::Identifier {
                                    label: "bar",
                                    value: None,
                                },
                                Parameter::Identifier {
                                    label: "baz",
                                    value: None,
                                },
                            ]),
                            body: mock.list([
                                Statement::Expression {
                                    expression: mock.ident("debug")
                                }
                            ])
                        },
                        ClassMember::Method {
                            is_static: true,
                            property: Property::Literal("toThe"),
                            params: mock.list([
                                Parameter::Identifier {
                                    label: "moon",
                                    value: None,
                                },
                            ]),
                            body: mock.list([
                                Statement::Expression {
                                    expression: mock.ident("debug")
                                }
                            ])
                        },
                        ClassMember::Method {
                            is_static: false,
                            property: Property::Literal("function"),
                            params: List::empty(),
                            body: List::empty()
                        },
                        ClassMember::Method {
                            is_static: true,
                            property: Property::Literal("function"),
                            params: List::empty(),
                            body: List::empty()
                        },
                        ClassMember::Method {
                            is_static: true,
                            property: Property::Literal("constructor"),
                            params: List::empty(),
                            body: List::empty()
                        },
                    ])
                }
            }
        ]);

        assert_eq!(module.body(), expected);
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
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Class {
                class: Class {
                    name: mock.ptr("Foo").into(),
                    extends: None,
                    body: mock.list([
                        ClassMember::Value {
                            is_static: false,
                            property: Property::Literal("doge"),
                            value: mock.number("10")
                        },
                        ClassMember::Value {
                            is_static: false,
                            property: Property::Literal("to"),
                            value: mock.number("20")
                        },
                        ClassMember::Value {
                            is_static: false,
                            property: Property::Literal("the"),
                            value: mock.number("30")
                        },
                        ClassMember::Value {
                            is_static: true,
                            property: Property::Literal("moon"),
                            value: mock.number("42")
                        },
                    ])
                }
            }
        ]);

        assert_eq!(module.body(), expected);
    }
}
