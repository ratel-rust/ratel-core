use parser::{Parser, Parse, B0, B1};
use lexer::Token::*;
use ast::{NoName, OptionalName, MandatoryName, MethodKind};
use ast::{Ptr, Loc, EmptyListBuilder, Name, Function, Class, ClassMember, Property};

impl<'ast> Parse<'ast> for NoName {
    type Output = Self;

    #[inline]
    fn parse(par: &mut Parser<'ast>) -> Self::Output {
        NoName
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

impl<'ast, N> Parse<'ast> for Function<'ast, N> where
    N: Name<'ast> + Parse<'ast, Output = N>,
{
    type Output = Self;

    #[inline]
    fn parse(par: &mut Parser<'ast>) -> Self::Output {
        let name = N::parse(par);

        expect!(par, ParenOpen);

        Function {
            name,
            params: par.parameter_list(),
            body: par.block(),
        }
    }
}

impl<'ast> Parse<'ast> for ClassMember<'ast> {
    type Output = Ptr<'ast, Loc<ClassMember<'ast>>>;

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

        let key = match par.lexer.token {
            _ if par.lexer.token.is_word() => {
                let mut label = par.lexer.token_as_str();
                par.lexer.consume();

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

                Property::Literal(label)
            },
            LiteralNumber => {
                let num = par.lexer.token_as_str();
                par.lexer.consume();
                Property::Literal(num)
            },
            LiteralBinary => {
                let num = par.lexer.token_as_str();
                par.lexer.consume();
                Property::Binary(num)
            },
            BracketOpen => {
                par.lexer.consume();

                let expression = par.expression(B0);

                expect!(par, BracketClose);

                Property::Computed(expression)
            },
            _ => return par.error()
        };

        let end;
        let member = match par.lexer.token {
            ParenOpen => {
                par.lexer.consume();

                let params = par.parameter_list();
                let body = par.block();

                end = body.end;

                ClassMember::Method {
                    is_static: is_static,
                    key,
                    kind,
                    params,
                    body,
                }
            },
            OperatorAssign => {
                par.lexer.consume();

                let expression = par.expression(B1);

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

                Some(par.expression(B1))
            },
            _ => None
        };

        Class {
            name: name.into(),
            extends: super_class,
            body: par.block(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use parser::parse;
    use parser::mock::Mock;
    use ast::{List, Literal, Expression, Statement, Function, Class};
    use ast::{ClassMember, Property, Parameter, ParameterKey};
    use ast::statement::*;

    #[test]
    fn function_empty() {
        let src = "function foo() {}";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Function(Function {
                name: mock.name("foo"),
                params: List::empty(),
                body: mock.empty_block(),
            })
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn function_params() {
        let src = "function foo(bar, baz) {}";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Function(Function {
                name: mock.name("foo"),
                params: mock.list([
                    Parameter {
                        key: ParameterKey::Identifier("bar"),
                        value: None,
                    },
                    Parameter {
                        key: ParameterKey::Identifier("baz"),
                        value: None,
                    },
                ]),
                body: mock.empty_block(),
            })
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn function_body() {
        let src = "function foo() { bar; baz; }";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Function(Function {
                name: mock.name("foo"),
                params: List::empty(),
                body: mock.block([
                    mock.ptr("bar"),
                    mock.ptr("baz"),
                ])
            })
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn function_with_default_params() {
        let src = "function foo (a = 0, b = 1, c = 2) { return 2 }";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Function(Function {
                name: mock.name("foo"),
                params: mock.list([
                    Parameter {
                        key: ParameterKey::Identifier("a"),
                        value: Some(mock.number("0")),
                    },
                    Parameter {
                        key: ParameterKey::Identifier("b"),
                        value: Some(mock.number("1")),
                    },
                    Parameter {
                        key: ParameterKey::Identifier("c"),
                        value: Some(mock.number("2")),
                    }
                ]),
                body: mock.block([
                    ReturnStatement {
                        value: Some(mock.number("2"))
                    }
                ])
            })
        ]);
        assert_eq!(module.body(), expected);
    }

    #[test]
    #[should_panic]
    fn function_with_non_trailing_default_params() {
        let src = "function foo (a, b, c = 2, d) { return 2 }";
        parse(src).unwrap();
    }

    #[test]
    fn class_empty() {
        let src = "class Foo {}";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Class(Class {
                name: mock.name("Foo"),
                extends: None,
                body: mock.empty_block(),
            })
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn child_class_empty() {
        let src = "class Foo extends Bar {}";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Class(Class {
                name: mock.name("Foo"),
                extends: Some(mock.ptr("Bar")),
                body: mock.empty_block(),
            })
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
            Statement::Class(Class {
                name: mock.name("Foo"),
                extends: None,
                body: mock.block([
                    ClassMember::Method {
                        is_static: false,
                        key: Property::Literal("constructor"),
                        kind: MethodKind::Constructor,
                        params: mock.list([
                            Parameter {
                                key: ParameterKey::Identifier("bar"),
                                value: None,
                            },
                            Parameter {
                                key: ParameterKey::Identifier("baz"),
                                value: None,
                            },
                        ]),
                        body: mock.block([
                            mock.ptr("debug")
                        ])
                    }
                ])
            })
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
            Statement::Class(Class {
                name: mock.name("Foo"),
                extends: None,
                body: mock.block([
                    ClassMember::Method {
                        is_static: false,
                        key: Property::Literal("doge"),
                        kind: MethodKind::Method,
                        params: mock.list([
                            Parameter {
                                key: ParameterKey::Identifier("bar"),
                                value: None,
                            },
                            Parameter {
                                key: ParameterKey::Identifier("baz"),
                                value: None,
                            },
                        ]),
                        body: mock.block([
                            mock.ptr("debug")
                        ])
                    },
                    ClassMember::Method {
                        is_static: true,
                        key: Property::Literal("toThe"),
                        kind: MethodKind::Method,
                        params: mock.list([
                            Parameter {
                                key: ParameterKey::Identifier("moon"),
                                value: None,
                            },
                        ]),
                        body: mock.block([
                            mock.ptr("debug")
                        ])
                    },
                    ClassMember::Method {
                        is_static: false,
                        key: Property::Literal("function"),
                        kind: MethodKind::Method,
                        params: List::empty(),
                        body: mock.empty_block()
                    },
                    ClassMember::Method {
                        is_static: true,
                        key: Property::Literal("function"),
                        kind: MethodKind::Method,
                        params: List::empty(),
                        body: mock.empty_block()
                    },
                    ClassMember::Method {
                        is_static: true,
                        key: Property::Literal("constructor"),
                        kind: MethodKind::Method,
                        params: List::empty(),
                        body: mock.empty_block()
                    },
                ])
            })
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
            Statement::Class(Class {
                name: mock.name("Foo"),
                extends: None,
                body: mock.block([
                    ClassMember::Literal {
                        is_static: false,
                        key: Property::Literal("doge"),
                        value: mock.number("10")
                    },
                    ClassMember::Literal {
                        is_static: false,
                        key: Property::Literal("to"),
                        value: mock.number("20")
                    },
                    ClassMember::Literal {
                        is_static: false,
                        key: Property::Literal("the"),
                        value: mock.number("30")
                    },
                    ClassMember::Literal {
                        is_static: true,
                        key: Property::Literal("moon"),
                        value: mock.number("42")
                    },
                ])
            })
        ]);

        assert_eq!(module.body(), expected);
    }


    #[test]
    fn class_extends_null() {
        let src = "class Foo extends null {}";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Class(Class {
                name: mock.name("Foo"),
                extends: Some(mock.ptr(Expression::Literal(Literal::Null))),
                body: mock.empty_block()
            })
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn class_methods() {
        let src = r#"

        class Foo {
            get length (foo) { }
            set length (bar) { }
        }

        "#;
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Class(Class {
                name: mock.name("Foo"),
                extends: None,
                body: mock.block([
                    ClassMember::Method {
                        is_static: false,
                        key: Property::Literal("length"),
                        kind: MethodKind::Get,
                        params: mock.list([
                            Parameter {
                                key: ParameterKey::Identifier("foo"),
                                value: None,
                            },
                        ]),
                        body: mock.empty_block()
                    },
                    ClassMember::Method {
                        is_static: false,
                        key: Property::Literal("length"),
                        kind: MethodKind::Set,
                        params: mock.list([
                            Parameter {
                                key: ParameterKey::Identifier("bar"),
                                value: None,
                            },
                        ]),
                        body: mock.empty_block()
                    },
                ])
            })
        ]);

        assert_eq!(module.body(), expected);
    }
}
