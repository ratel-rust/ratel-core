extern crate ratel;

use ratel::grammar::*;
use ratel::parser::parse;
use ratel::operator::OperatorKind::*;

macro_rules! assert_parse {
    ($string:expr, $body:expr) => {
        assert_eq!(parse($string.into()).expect("Must parse").body, $body);
    }
}

macro_rules! assert_expression {
    ($string:expr, $ex:expr) => {
        match parse($string.into()).expect("Must parse").body[0] {
            Statement::Expression { ref value } => assert_eq!(*value, $ex),
            _                                   => panic!("No expression found"),
        }
    }
}

macro_rules! assert_statement {
    ($string:expr, $ex:expr) => (assert_parse!($string, vec![$ex]))
}

macro_rules! num {
    ($num:expr) => (Expression::Literal(Value::Number($num.into())))
}


macro_rules! expr_bin {
    ($num:expr) => (Expression::Literal(Value::Binary($num)))
}

macro_rules! boxnum {
    ($num:expr) => (Box::new(num!($num)))
}

macro_rules! ident {
    ($name:expr) => (Expression::Identifier($name.into()))
}

macro_rules! param {
    ($name:expr, $expr:expr) => (Parameter {
        name: $name.into(),
        default: $expr.into()
    })
}

#[test]
fn block_statement() {
    assert_statement!("{}", Statement::Block {
        body: Vec::new(),
    });
}

#[test]
fn labeled_statement() {
    assert_statement!("foo: {}", Statement::Labeled {
        label: "foo".into(),
        body: Box::new(Statement::Block {
            body: Vec::new(),
        }),
    });
}

#[test]
fn break_statement() {
    assert_statement!("break;", Statement::Break {
        label: None
    });
}

#[test]
fn break_label_statement() {
    assert_statement!("break foo;", Statement::Break {
        label: Some("foo".into())
    });
}

#[test]
fn break_asi_statement() {
    assert_parse!("

    break
    foo

    ", vec![
        Statement::Break {
            label: None
        },
        Statement::Expression { value: ident!("foo") }
    ]);
}

#[test]
fn return_statement() {
    assert_statement!("return;", Statement::Return {
        value: None,
    });
}

#[test]
fn return_value_statement() {
    assert_statement!("return foo;", Statement::Return {
        value: Some(ident!("foo")),
    });
}

#[test]
fn return_sequence_statement() {
    assert_statement!("return 1, 2, 3;", Statement::Return {
        value: Some(Expression::Sequence(vec![
            num!("1"),
            num!("2"),
            num!("3"),
        ])),
    });
}

#[test]
fn return_asi_statement() {
    assert_parse!("

    return
    foo

    ", vec![
        Statement::Return {
            value: None
        },
        Statement::Expression { value: ident!("foo") }
    ]);
}

#[test]
fn var_declare() {
    assert_statement!("var foo;", Statement::VariableDeclaration {
        kind: VariableDeclarationKind::Var,
        declarators: vec![VariableDeclarator {
            name: "foo".into(),
            value: None,
        }]
    });
}

#[test]
fn var_declare_value() {
    assert_statement!("var foo = 100;", Statement::VariableDeclaration {
        kind: VariableDeclarationKind::Var,
        declarators: vec![VariableDeclarator {
            name: "foo".into(),
            value: Some(num!("100")),
        }]
    });
}

#[test]
fn let_declare() {
    assert_statement!("let foo;", Statement::VariableDeclaration {
        kind: VariableDeclarationKind::Let,
        declarators: vec![VariableDeclarator {
            name: "foo".into(),
            value: None,
        }]
    });
}

#[test]
fn let_declare_value() {
    assert_statement!("let foo = 100;", Statement::VariableDeclaration {
        kind: VariableDeclarationKind::Let,
        declarators: vec![VariableDeclarator {
            name: "foo".into(),
            value: Some(num!("100")),
        }]
    });
}

#[test]
fn const_declare() {
    assert_statement!("const foo;", Statement::VariableDeclaration {
        kind: VariableDeclarationKind::Const,
        declarators: vec![VariableDeclarator {
            name: "foo".into(),
            value: None,
        }]
    });
}

#[test]
fn const_declare_value() {
    assert_statement!("const foo = 100;", Statement::VariableDeclaration {
        kind: VariableDeclarationKind::Const,
        declarators: vec![VariableDeclarator {
            name: "foo".into(),
            value: Some(num!("100")),
        }]
    });
}

#[test]
fn var_muliple_declare() {
    assert_statement!("var foo, bar;", Statement::VariableDeclaration {
        kind: VariableDeclarationKind::Var,
        declarators: vec![VariableDeclarator {
            name: "foo".into(),
            value: None,
        }, VariableDeclarator {
            name: "bar".into(),
            value: None,
        }]
    });
}

#[test]
fn var_muliple_declare_value() {
    assert_statement!("var foo = 100, bar = 200;", Statement::VariableDeclaration {
        kind: VariableDeclarationKind::Var,
        declarators: vec![VariableDeclarator {
            name: "foo".into(),
            value: Some(num!("100")),
        }, VariableDeclarator {
            name: "bar".into(),
            value: Some(num!("200")),
        }]
    });
}

#[test]
fn identifier_expression() {
    assert_expression!("foobar", ident!("foobar"))
}

#[test]
fn null_expression() {
    assert_expression!("null", Expression::Literal(Value::Null));
}

#[test]
fn undefined_expression() {
    assert_expression!("undefined", Expression::Literal(Value::Undefined));
}

#[test]
fn true_expression() {
    assert_expression!("true", Expression::Literal(Value::True));
}

#[test]
fn false_expression() {
    assert_expression!("false", Expression::Literal(Value::False));
}

#[test]
fn number_expression() {
    assert_expression!("100", num!("100"));
}

#[test]
fn binary_number_expression() {
    assert_expression!("0b1100100", expr_bin!(100u64));
}

#[test]
fn octal_number_expression() {
    assert_expression!("0o144", num!("0o144"));
}

#[test]
fn hexdec_number_expression() {
    assert_expression!("0x64", num!("0x64"));
}

#[test]
fn floating_number_expression() {
    assert_expression!("3.14", num!("3.14"));
}

#[test]
fn template_string_plain() {
    assert_expression!("`foobar`", Expression::Template {
        tag: None,
        quasis: vec!["foobar".into()],
        expressions: Vec::new(),
    });
}

#[test]
fn template_string_complex() {
    assert_expression!("`foo${1}bar${2}baz`", Expression::Template {
        tag: None,
        quasis: vec!["foo".into(), "bar".into(), "baz".into()],
        expressions: vec![num!("1"), num!("2")],
    });
}

#[test]
fn binary_expression() {
    assert_expression!("true == 1", Expression::binary(
        Expression::Literal(Value::True),
        Equality,
        num!("1")
    ));
}

#[test]
fn binary_expression_less_than() {
    assert_expression!("i < 10", Expression::binary(
        ident!("i"),
        Lesser,
        num!("10")
    ));
}

#[test]
fn op_precedence_left() {
    assert_expression!("1 + 2 * 3", Expression::binary(
        num!("1"),
        Addition,
        Expression::binary(
            num!("2"),
            Multiplication,
            num!("3")
        ),
    ));
}

#[test]
fn op_precedence_right() {
    assert_expression!("1 * 2 + 3", Expression::binary(
        Expression::binary(
            num!("1"),
            Multiplication,
            num!("2"),
        ),
        Addition,
        num!("3"),
    ));
}

#[test]
fn function_statement() {
    assert_statement!("

    function foo() {
        return bar;
    }

    ", Statement::Function {
        name: "foo".into(),
        params: vec![],
        body: vec![
            Statement::Return {
                value: Some(ident!("bar"))
            }
        ]
    });
}

#[test]
fn function_with_params_statement() {
    assert_statement!("

    function foo(a, b, c) {
        return bar;
    }

    ", Statement::Function {
        name: "foo".into(),
        params: vec![
            param!("a", None),
            param!("b", None),
            param!("c", None),
        ],
        body: vec![
            Statement::Return {
                value: Some(ident!("bar"))
            }
        ]
    });
}

#[test]
fn if_statement() {
    assert_statement!("

    if (true) {
        foo;
    }

    ", Statement::If {
        test: Expression::Literal(Value::True),
        consequent: Box::new(Statement::Block {
            body: vec![Statement::Expression {
                value: ident!("foo")
            }]
        }),
        alternate: None,
    });
}

#[test]
fn if_else_statement() {
    assert_statement!("

    if (true) {
        foo;
    } else {
        bar;
    }

    ", Statement::If {
        test: Expression::Literal(Value::True),
        consequent: Box::new(Statement::Block {
            body: vec![Statement::Expression {
                value: ident!("foo")
            }]
        }),
        alternate: Some(Box::new(Statement::Block {
            body: vec![Statement::Expression {
                value: ident!("bar")
            }]
        })),
    })
}

#[test]
fn if_else_if_else_statement() {
    assert_statement!("

    if (true) {
        foo;
    } else if(false) {
        bar;
    } else {
        baz;
    }

    ", Statement::If {
        test: Expression::Literal(Value::True),
        consequent: Box::new(Statement::Block {
            body: vec![Statement::Expression {
                value: ident!("foo")
            }]
        }),
        alternate: Some(Box::new(Statement::If {
            test: Expression::Literal(Value::False),
            consequent: Box::new(Statement::Block {
                body: vec![Statement::Expression {
                    value: ident!("bar")
                }]
            }),
            alternate: Some(Box::new(Statement::Block {
                body: vec![Statement::Expression {
                    value: ident!("baz")
                }]
            })),
        })),
    });
}

#[test]
fn if_single_statement() {
    assert_statement!("if (true) foo;", Statement::If {
        test: Expression::Literal(Value::True),
        consequent: Box::new(Statement::Expression {
            value: ident!("foo")
        }),
        alternate: None,
    });
}

#[test]
fn if_single_assignment_statement() {
    assert_statement!("if (true) const foo = 100;", Statement::If {
        test: Expression::Literal(Value::True),
        consequent: Box::new(Statement::VariableDeclaration {
            kind: VariableDeclarationKind::Const,
            declarators: vec![VariableDeclarator {
                name: "foo".into(),
                value: Some(num!("100")),
            }]
        }),
        alternate: None,
    });
}

#[test]
fn if_no_block_statement() {
    assert_statement!("

    if (true);

    ", Statement::If {
        test: Expression::Literal(Value::True),
        consequent: Box::new(Statement::Empty),
        alternate: None,
    });
}

#[test]
fn if_else_single_statement() {
    assert_statement!("if (true) foo; else bar;", Statement::If {
        test: Expression::Literal(Value::True),
        consequent: Box::new(Statement::Expression {
            value: ident!("foo")
        }),
        alternate: Some(Box::new(Statement::Expression {
            value: ident!("bar")
        })),
    })
}

#[test]
fn if_else_empty_alternate_statement() {
    assert_statement!("if (true) foo; else;", Statement::If {
        test: Expression::Literal(Value::True),
        consequent: Box::new(Statement::Expression {
            value: ident!("foo")
        }),
        alternate: Some(Box::new(Statement::Empty)),
    })
}

#[test]
fn for_statement() {
    assert_statement!("for (i = 0; i < 10; i++) {}", Statement::For {
        init: Some(Box::new(Statement::Expression {
            value: Expression::binary(
                ident!("i"),
                Assign,
                num!("0"),
            )
        })),
        test: Some(Expression::binary(
            ident!("i"),
            Lesser,
            num!("10"),
        )),
        update: Some(Expression::Postfix {
            operator: Increment,
            operand: Box::new(ident!("i")),
        }),
        body: Box::new(Statement::Block {
            body: Vec::new(),
        }),
    });
}

#[test]
fn for_no_block_statement() {
    assert_statement!("for (i = 0; i < 10; i++);", Statement::For {
        init: Some(Box::new(Statement::Expression {
            value: Expression::binary(
                ident!("i"),
                Assign,
                num!("0"),
            )
        })),
        test: Some(Expression::binary(
            ident!("i"),
            Lesser,
            num!("10"),
        )),
        update: Some(Expression::Postfix {
            operator: Increment,
            operand: Box::new(ident!("i")),
        }),
        body: Box::new(Statement::Empty),
    });
}

#[test]
fn for_declare_statement() {
    assert_statement!("for (let i = 0; i < 10; i++) {}", Statement::For {
        init: Some(Box::new(Statement::VariableDeclaration {
            kind: VariableDeclarationKind::Let,
            declarators: vec![
                VariableDeclarator {
                    name: "i".into(),
                    value: Some(num!("0")),
                }
            ],
        })),
        test: Some(Expression::binary(
            ident!("i"),
            Lesser,
            num!("10"),
        )),
        update: Some(Expression::Postfix {
            operator: Increment,
            operand: Box::new(ident!("i")),
        }),
        body: Box::new(Statement::Block {
            body: Vec::new(),
        }),
    });
}

#[test]
fn for_declare_no_block_statement() {
    assert_statement!("for (let i = 0; i < 10; i++);", Statement::For {
        init: Some(Box::new(Statement::VariableDeclaration {
            kind: VariableDeclarationKind::Let,
            declarators: vec![
                VariableDeclarator {
                    name: "i".into(),
                    value: Some(num!("0")),
                }
            ],
        })),
        test: Some(Expression::binary(
            ident!("i"),
            Lesser,
            num!("10"),
        )),
        update: Some(Expression::Postfix {
            operator: Increment,
            operand: Box::new(ident!("i")),
        }),
        body: Box::new(Statement::Empty),
    });
}

#[test]
fn for_empty_statement() {
    assert_statement!("for (;;) {}", Statement::For {
        init: None,
        test: None,
        update: None,
        body: Box::new(Statement::Block {
            body: Vec::new(),
        }),
    });
}

#[test]
fn for_empty_no_block_statement() {
    assert_statement!("for (;;);", Statement::For {
        init: None,
        test: None,
        update: None,
        body: Box::new(Statement::Empty),
    });
}

#[test]
fn for_in_statement() {
    assert_statement!("for (item in object) {}", Statement::ForIn {
        left: Box::new(Statement::Expression {
            value: ident!("item")
        }),
        right: ident!("object"),
        body: Box::new(Statement::Block {
            body: Vec::new(),
        }),
    });
}

#[test]
fn for_in_no_block_statement() {
    assert_statement!("for (item in object);", Statement::ForIn {
        left: Box::new(Statement::Expression {
            value: ident!("item")
        }),
        right: ident!("object"),
        body: Box::new(Statement::Empty),
    });
}

#[test]
fn for_in_declare_statement() {
    assert_statement!("for (let item in object) {}", Statement::ForIn {
        left: Box::new(Statement::VariableDeclaration {
            kind: VariableDeclarationKind::Let,
            declarators: vec![
                VariableDeclarator {
                    name: "item".into(),
                    value: None,
                }
            ],
        }),
        right: ident!("object"),
        body: Box::new(Statement::Block {
            body: Vec::new(),
        }),
    });
}

#[test]
fn for_in_declare_statement_initial_value() {
    assert_statement!("for (let item = 0 in object) {}", Statement::ForIn {
        left: Box::new(Statement::VariableDeclaration {
            kind: VariableDeclarationKind::Let,
            declarators: vec![
                VariableDeclarator {
                    name: "item".into(),
                    value: Some(num!("0")),
                }
            ],
        }),
        right: ident!("object"),
        body: Box::new(Statement::Block {
            body: Vec::new(),
        }),
    });
}

#[test]
fn for_in_declare_no_block_statement() {
    assert_statement!("for (let item in object);", Statement::ForIn {
        left: Box::new(Statement::VariableDeclaration {
            kind: VariableDeclarationKind::Let,
            declarators: vec![
                VariableDeclarator {
                    name: "item".into(),
                    value: None,
                }
            ],
        }),
        right: ident!("object"),
        body: Box::new(Statement::Empty),
    });
}

#[test]
fn for_of_statement() {
    assert_statement!("for (item of array) {}", Statement::ForOf {
        left: Box::new(Statement::Expression {
            value: ident!("item")
        }),
        right: ident!("array"),
        body: Box::new(Statement::Block {
            body: Vec::new(),
        }),
    });
}

#[test]
fn for_of_no_block_statement() {
    assert_statement!("for (item of array);", Statement::ForOf {
        left: Box::new(Statement::Expression {
            value: ident!("item")
        }),
        right: ident!("array"),
        body: Box::new(Statement::Empty),
    });
}

#[test]
fn for_of_declare_statement() {
    assert_statement!("for (let item of array) {}", Statement::ForOf {
        left: Box::new(Statement::VariableDeclaration {
            kind: VariableDeclarationKind::Let,
            declarators: vec![
                VariableDeclarator {
                    name: "item".into(),
                    value: None,
                }
            ],
        }),
        right: ident!("array"),
        body: Box::new(Statement::Block {
            body: Vec::new(),
        }),
    });
}

#[test]
fn for_of_declare_statement_initial_value() {
    assert_statement!("for (let item = 0 of array) {}", Statement::ForOf {
        left: Box::new(Statement::VariableDeclaration {
            kind: VariableDeclarationKind::Let,
            declarators: vec![
                VariableDeclarator {
                    name: "item".into(),
                    value: Some(num!("0")),
                }
            ],
        }),
        right: ident!("array"),
        body: Box::new(Statement::Block {
            body: Vec::new(),
        }),
    });
}

#[test]
fn for_of_declare_no_block_statement() {
    assert_statement!("for (let item of array);", Statement::ForOf {
        left: Box::new(Statement::VariableDeclaration {
            kind: VariableDeclarationKind::Let,
            declarators: vec![
                VariableDeclarator {
                    name: "item".into(),
                    value: None,
                }
            ],
        }),
        right: ident!("array"),
        body: Box::new(Statement::Empty),
    });
}

#[test]
fn while_statement() {
    assert_statement!("

    while (true) {
        foo;
    }

    ", Statement::While {
        test: Expression::Literal(Value::True),
        body: Box::new(Statement::Block {
            body: vec![Statement::Expression {
                value: ident!("foo")
            }]
        }),
    });
}

#[test]
fn while_single_statement() {
    assert_statement!("while (true) foo;", Statement::While {
        test: Expression::Literal(Value::True),
        body: Box::new(Statement::Expression {
            value: ident!("foo")
        }),
    });
}

#[test]
fn while_no_block_statement() {
    assert_statement!("while (true);", Statement::While {
        test: Expression::Literal(Value::True),
        body: Box::new(Statement::Empty),
    });
}

#[test]
fn arrow_function() {
    assert_expression!("

    () => {
        bar;
    }

    ", Expression::ArrowFunction {
        params: vec![],
        body: Box::new(Statement::Block {
            body: vec![Statement::Expression {
                value: ident!("bar")
            }]
        })
    });
}

#[test]
fn arrow_function_shorthand() {
    assert_expression!("n => n * n", Expression::ArrowFunction {
        params: vec![
            param!("n", None)
        ],
        body: Box::new(Statement::Expression {
            value: Expression::binary(
                ident!("n"),
                Multiplication,
                ident!("n")
            )
        }),
    });
}

#[test]
fn arrow_function_with_params() {
    assert_expression!("

    (a, b, c) => {
        bar;
    }

    ", Expression::ArrowFunction {
        params: vec![
            param!("a", None),
            param!("b", None),
            param!("c", None),
        ],
        body: Box::new(Statement::Block {
            body: vec![Statement::Expression {
                value: ident!("bar")
            }]
        })
    });
}

#[test]
fn arrow_function_with_default_params() {
    assert_expression!("

    (a, b, c = 2) => bar;

    ", Expression::ArrowFunction {
        params: vec![
            param!("a", None),
            param!("b", None),
            param!("c", Some(Box::new(num!("2")))),
        ],
        body: Box::new(Statement::Expression {
            value: ident!("bar")
        })
    });
}

#[test]
fn function_expression() {
    assert_expression!("

    (function () {
        return bar;
    })

    ", Expression::Function {
        name: None,
        params: vec![],
        body: vec![
            Statement::Return {
                value: Some(ident!("bar"))
            }
        ]
    });
}

#[test]
fn named_function_expression() {
    assert_expression!("

    (function foo() {
        return bar;
    })

    ", Expression::Function {
        name: Some("foo".into()),
        params: vec![],
        body: vec![
            Statement::Return {
                value: Some(ident!("bar"))
            }
        ]
    });
}

#[test]
fn expression_statement() {
    assert_statement!("foo", Statement::Expression {
        value: ident!("foo")
    });
}

#[test]
fn sequence_expression_statement() {
    assert_statement!("foo, bar, baz", Statement::Expression {
        value: Expression::Sequence(vec![
            ident!("foo"),
            ident!("bar"),
            ident!("baz"),
        ])
    });
}

#[test]
fn sequence_in_accessor() {
    assert_expression!("foo[1, 2, 3]", Expression::ComputedMember {
        object: Box::new(ident!("foo")),
        property: Box::new(
            Expression::Sequence(vec![
                num!("1"),
                num!("2"),
                num!("3"),
            ])
        )
    });
}

#[test]
fn object_string_literal_member() {
    assert_expression!("({foo:100})", Expression::Object(vec![
        ObjectMember::Value {
            key: ObjectKey::Literal("foo".into()),
            value: num!("100"),
        }
    ]));
}

#[test]
fn object_number_literal_member() {
    assert_expression!("({100:100})", Expression::Object(vec![
        ObjectMember::Value {
            key: ObjectKey::Literal("100".into()),
            value: num!("100"),
        }
    ]));
}

#[test]
fn object_binary_literal_member() {
    assert_expression!("({ 0b1100100 : 100})", Expression::Object(vec![
        ObjectMember::Value {
            key: ObjectKey::Binary(100),
            value: num!("100"),
        }
    ]));
}

#[test]
fn object_hex_literal_member() {
    assert_expression!("({ 0x64 : 100})", Expression::Object(vec![
        ObjectMember::Value {
            key: ObjectKey::Literal("0x64".into()),
            value: num!("100"),
        }
    ]));
}

#[test]
fn object_keyword_literal_member() {
    assert_expression!("({ function : 100})", Expression::Object(vec![
        ObjectMember::Value {
            key: ObjectKey::Literal("function".into()),
            value: num!("100"),
        }
    ]));
}

#[test]
fn object_value_word_literal_member() {
    assert_expression!("({ true : 100})", Expression::Object(vec![
        ObjectMember::Value {
            key: ObjectKey::Literal("true".into()),
            value: num!("100"),
        }
    ]));
}

#[test]
fn object_operator_word_literal_member() {
    assert_expression!("({ new : 100})", Expression::Object(vec![
        ObjectMember::Value {
            key: ObjectKey::Literal("new".into()),
            value: num!("100"),
        }
    ]));
}

#[test]
fn object_computed_member() {
    assert_expression!("({[100]:100})", Expression::Object(vec![
        ObjectMember::Value {
            key: ObjectKey::Computed(num!("100")),
            value: num!("100"),
        }
    ]));
}

#[test]
fn object_shorthand_member() {
    assert_expression!("({foo})", Expression::Object(vec![
        ObjectMember::Shorthand {
            key: "foo".into(),
        }
    ]));
}

#[test]
fn object_method_member() {
    assert_expression!("({foo() {} })", Expression::Object(vec![
        ObjectMember::Method {
            key: ObjectKey::Literal("foo".into()),
            params: vec![],
            body: vec![],
        }
    ]));
}

#[test]
fn object_computed_method_member() {
    assert_expression!("({[100]() {} })", Expression::Object(vec![
        ObjectMember::Method {
            key: ObjectKey::Computed(num!("100")),
            params: vec![],
            body: vec![],
        }
    ]));
}

#[test]
fn object_number_method_member() {
    assert_expression!("({100 () {} })", Expression::Object(vec![
        ObjectMember::Method {
            key: ObjectKey::Literal("100".into()),
            params: vec![],
            body: vec![],
        }
    ]));
}

#[test]
fn accesss_member_identifier() {
    assert_expression!("this.foo", Expression::Member {
        object: Box::new(Expression::This),
        property: "foo".into()
    })
}

#[test]
fn accesss_member_keyword() {
    assert_expression!("this.function", Expression::Member {
        object: Box::new(Expression::This),
        property: "function".into()
    })
}

#[test]
fn accesss_member_value_word() {
    assert_expression!("this.true", Expression::Member {
        object: Box::new(Expression::This),
        property: "true".into()
    })
}

#[test]
fn accesss_member_operator_word() {
    assert_expression!("this.new", Expression::Member {
        object: Box::new(Expression::This),
        property: "new".into()
    })
}

#[test]
fn class_statement() {
    assert_statement!("class Foo {}", Statement::Class {
        name: "Foo".into(),
        extends: None,
        body: Vec::new(),
    });
}

#[test]
fn class_extends_statement() {
    assert_statement!("class Foo extends Bar {}", Statement::Class {
        name: "Foo".into(),
        extends: Some("Bar".into()),
        body: Vec::new(),
    });
}

#[test]
fn class_with_constructor_statement() {
    assert_statement!("

    class Foo {
        constructor() {}
    }

    ", Statement::Class {
        name: "Foo".into(),
        extends: None,
        body: vec![
            ClassMember::Constructor {
                params: Vec::new(),
                body: Vec::new(),
            }
        ],
    });
}

#[test]
fn class_with_method_statement() {
    assert_statement!("

    class Foo {
        bar() {}
    }

    ", Statement::Class {
        name: "Foo".into(),
        extends: None,
        body: vec![
            ClassMember::Method {
                is_static: false,
                key: ClassKey::Literal("bar".into()),
                params: Vec::new(),
                body: Vec::new(),
            }
        ],
    });
}

#[test]
fn class_with_keyword_method_statement() {
    assert_statement!("

    class Foo {
        undefined() {}
    }

    ", Statement::Class {
        name: "Foo".into(),
        extends: None,
        body: vec![
            ClassMember::Method {
                is_static: false,
                key: ClassKey::Literal("undefined".into()),
                params: Vec::new(),
                body: Vec::new(),
            }
        ],
    });
}

#[test]
fn class_with_number_method_statement() {
    assert_statement!("

    class Foo {
        100() {}
    }

    ", Statement::Class {
        name: "Foo".into(),
        extends: None,
        body: vec![
            ClassMember::Method {
                is_static: false,
                key: ClassKey::Number("100".into()),
                params: Vec::new(),
                body: Vec::new(),
            }
        ],
    });
}

#[test]
fn class_with_binary_method_statement() {
    assert_statement!("

    class Foo {
        0b100() {}
    }

    ", Statement::Class {
        name: "Foo".into(),
        extends: None,
        body: vec![
            ClassMember::Method {
                is_static: false,
                key: ClassKey::Binary(4),
                params: Vec::new(),
                body: Vec::new(),
            }
        ],
    });
}

#[test]
fn class_with_computed_method_statement() {
    assert_statement!(r#"

    class Foo {
        [300]() {}
    }

    "#, Statement::Class {
        name: "Foo".into(),
        extends: None,
        body: vec![
            ClassMember::Method {
                is_static: false,
                key: ClassKey::Computed(
                    Expression::Literal(Value::Number("300".into()))
                ),
                params: Vec::new(),
                body: Vec::new(),
            }
        ],
    });
}

#[test]
fn class_with_static_method_statement() {
    assert_statement!("

    class Foo {
        static bar() {}
    }

    ", Statement::Class {
        name: "Foo".into(),
        extends: None,
        body: vec![
            ClassMember::Method {
                is_static: true,
                key: ClassKey::Literal("bar".into()),
                params: Vec::new(),
                body: Vec::new(),
            }
        ],
    });
}

#[test]
fn class_with_property_statement() {
    assert_statement!("

    class Foo {
        bar = 100;
    }

    ", Statement::Class {
        name: "Foo".into(),
        extends: None,
        body: vec![
            ClassMember::Property {
                is_static: false,
                key: ClassKey::Literal("bar".into()),
                value: num!("100"),
            }
        ],
    });
}

#[test]
fn class_with_static_property_statement() {
    assert_statement!("

    class Foo {
        static bar = 100;
    }

    ", Statement::Class {
        name: "Foo".into(),
        extends: None,
        body: vec![
            ClassMember::Property {
                is_static: true,
                key: ClassKey::Literal("bar".into()),
                value: num!("100"),
            }
        ],
    });
}

#[test]
fn regular_expression() {
    assert_expression!("/[\\w.]+/i", Expression::RegEx {
        pattern: "[\\w.]+".into(),
        flags: "i".into()
    });
}

#[test]
fn regular_expression_escaping() {
    assert_expression!("/^\\a/i", Expression::RegEx {
        pattern: "^\\a".into(),
        flags: "i".into()
    });
}

#[test]
fn try_catch_statement() {
    assert_statement!("

    try {
        true;
    } catch (err) {
        false;
    }

    ", Statement::Try {
        body: Box::new(Statement::Block {
            body: vec![
                Statement::Expression {
                    value: Expression::Literal(Value::True)
                }
            ]
        }),
        error: "err".into(),
        handler: Box::new(Statement::Block {
            body: vec![
                Statement::Expression {
                    value: Expression::Literal(Value::False)
                }
            ]
        }),
    });
}
