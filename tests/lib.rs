extern crate badger;

pub use badger::*;
pub use badger::grammar::*;
pub use badger::parser::parse;
pub use badger::grammar::Statement::*;
pub use badger::grammar::Expression::*;
pub use badger::grammar::ClassMember::*;
pub use badger::grammar::OperatorType::*;

macro_rules! assert_parse {
    ($string:expr, $body:expr) => {
        assert_eq!(parse($string.to_string()).body, $body);
    }
}

macro_rules! assert_expression {
    ($string:expr, $ex:expr) => {
        match parse($string.to_string()).body[0] {
            ExpressionStatement(ref expression) => assert_eq!(*expression, $ex),
            _                                   => panic!("No expression found"),
        }
    }
}

macro_rules! assert_statement {
    ($string:expr, $ex:expr) => (assert_parse!($string, vec![$ex]))
}

macro_rules! num {
    ($num:expr) => (LiteralExpression(LiteralFloat($num)))
}

macro_rules! boxnum {
    ($num:expr) => (Box::new(num!($num)))
}

macro_rules! ident {
    ($name:expr) => (IdentifierExpression($name.to_string()))
}

macro_rules! param {
    ($name:expr) => (Parameter {
        name: $name.to_string()
    })
}

#[test]
fn block_statement() {
    assert_statement!("{}", BlockStatement {
        body: Vec::new(),
    });
}

#[test]
fn labeled_statement() {
    assert_statement!("foo: {}", LabeledStatement {
        label: "foo".to_string(),
        body: Box::new(BlockStatement {
            body: Vec::new(),
        }),
    });
}

#[test]
fn break_statement() {
    assert_statement!("break;", BreakStatement {
        label: None
    });
}

#[test]
fn break_label_statement() {
    assert_statement!("break foo;", BreakStatement {
        label: Some("foo".to_string())
    });
}

#[test]
fn break_asi_statement() {
    assert_parse!("

    break
    foo

    ", vec![
        BreakStatement {
            label: None
        },
        ExpressionStatement(ident!("foo"))
    ]);
}

#[test]
fn return_statement() {
    assert_statement!("return;", ReturnStatement {
        value: None,
    });
}

#[test]
fn return_value_statement() {
    assert_statement!("return foo;", ReturnStatement {
        value: Some(ident!("foo")),
    });
}

#[test]
fn return_sequence_statement() {
    assert_statement!("return 1, 2, 3;", ReturnStatement {
        value: Some(SequenceExpression(vec![
            num!(1.0),
            num!(2.0),
            num!(3.0),
        ])),
    });
}

#[test]
fn return_asi_statement() {
    assert_parse!("

    return
    foo

    ", vec![
        ReturnStatement {
            value: None
        },
        ExpressionStatement(ident!("foo"))
    ]);
}

#[test]
fn var_declare() {
    assert_statement!("var foo;", VariableDeclarationStatement {
        kind: VariableDeclarationKind::Var,
        declarators: vec![VariableDeclarator {
            name: "foo".to_string(),
            value: None,
        }]
    });
}

#[test]
fn var_declare_value() {
    assert_statement!("var foo = 100;", VariableDeclarationStatement {
        kind: VariableDeclarationKind::Var,
        declarators: vec![VariableDeclarator {
            name: "foo".to_string(),
            value: Some(num!(100.0)),
        }]
    });
}

#[test]
fn let_declare() {
    assert_statement!("let foo;", VariableDeclarationStatement {
        kind: VariableDeclarationKind::Let,
        declarators: vec![VariableDeclarator {
            name: "foo".to_string(),
            value: None,
        }]
    });
}

#[test]
fn let_declare_value() {
    assert_statement!("let foo = 100;", VariableDeclarationStatement {
        kind: VariableDeclarationKind::Let,
        declarators: vec![VariableDeclarator {
            name: "foo".to_string(),
            value: Some(num!(100.0)),
        }]
    });
}

#[test]
fn const_declare() {
    assert_statement!("const foo;", VariableDeclarationStatement {
        kind: VariableDeclarationKind::Const,
        declarators: vec![VariableDeclarator {
            name: "foo".to_string(),
            value: None,
        }]
    });
}

#[test]
fn const_declare_value() {
    assert_statement!("const foo = 100;", VariableDeclarationStatement {
        kind: VariableDeclarationKind::Const,
        declarators: vec![VariableDeclarator {
            name: "foo".to_string(),
            value: Some(num!(100.0)),
        }]
    });
}

#[test]
fn var_muliple_declare() {
    assert_statement!("var foo, bar;", VariableDeclarationStatement {
        kind: VariableDeclarationKind::Var,
        declarators: vec![VariableDeclarator {
            name: "foo".to_string(),
            value: None,
        }, VariableDeclarator {
            name: "bar".to_string(),
            value: None,
        }]
    });
}


#[test]
fn var_muliple_declare_value() {
    assert_statement!("var foo = 100, bar = 200;", VariableDeclarationStatement {
        kind: VariableDeclarationKind::Var,
        declarators: vec![VariableDeclarator {
            name: "foo".to_string(),
            value: Some(num!(100.0)),
        }, VariableDeclarator {
            name: "bar".to_string(),
            value: Some(num!(200.0)),
        }]
    });
}

#[test]
fn identifier_expression() {
    assert_expression!("foobar", ident!("foobar"))
}

#[test]
fn null_expression() {
    assert_expression!("null", LiteralExpression(LiteralNull));
}

#[test]
fn undefined_expression() {
    assert_expression!("undefined", LiteralExpression(LiteralUndefined));
}

#[test]
fn true_expression() {
    assert_expression!("true", LiteralExpression(LiteralTrue));
}

#[test]
fn false_expression() {
    assert_expression!("false", LiteralExpression(LiteralFalse));
}

#[test]
fn number_expression() {
    assert_expression!("100", num!(100.0));
}

#[test]
fn binary_number_expression() {
    assert_expression!("0b1100100", LiteralExpression(LiteralInteger(100)));
}

#[test]
fn octal_number_expression() {
    assert_expression!("0o144", LiteralExpression(LiteralInteger(100)));
}

#[test]
fn hexdec_number_expression() {
    assert_expression!("0x64", LiteralExpression(LiteralInteger(100)));
}

#[test]
fn floating_number_expression() {
    assert_expression!("3.14", num!(3.14));
}

#[test]
fn binary_expression() {
    assert_expression!("true == 1", BinaryExpression {
        left: Box::new(LiteralExpression(LiteralTrue)),
        operator: Equality,
        right: boxnum!(1.0)
    });
}

#[test]
fn op_precedence_left() {
    assert_expression!("1 + 2 * 3", BinaryExpression {
        left: boxnum!(1.0),
        operator: Addition,
        right: Box::new(BinaryExpression {
            left: boxnum!(2.0),
            operator: Multiplication,
            right: boxnum!(3.0),
        }),
    });
}

#[test]
fn op_precedence_right() {
    assert_expression!("1 * 2 + 3", BinaryExpression {
        left: Box::new(BinaryExpression {
            left: boxnum!(1.0),
            operator: Multiplication,
            right: boxnum!(2.0),
        }),
        operator: Addition,
        right: boxnum!(3.0),
    });
}

#[test]
fn function_statement() {
    assert_statement!("

    function foo() {
        return bar;
    }

    ", FunctionStatement {
        name: "foo".to_string(),
        params: vec![],
        body: vec![
            ReturnStatement {
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

    ", FunctionStatement {
        name: "foo".to_string(),
        params: vec![
            param!("a"),
            param!("b"),
            param!("c"),
        ],
        body: vec![
            ReturnStatement {
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

    ", IfStatement {
        test: LiteralExpression(LiteralTrue),
        consequent: Box::new(BlockStatement {
            body: vec![ExpressionStatement(
                ident!("foo")
            )]
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

    ", IfStatement {
        test: LiteralExpression(LiteralTrue),
        consequent: Box::new(BlockStatement {
            body: vec![ExpressionStatement(
                ident!("foo")
            )]
        }),
        alternate: Some(Box::new(BlockStatement {
            body: vec![ExpressionStatement(
                ident!("bar")
            )]
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

    ", IfStatement {
        test: LiteralExpression(LiteralTrue),
        consequent: Box::new(BlockStatement {
            body: vec![ExpressionStatement(
                ident!("foo")
            )]
        }),
        alternate: Some(Box::new(IfStatement {
            test: LiteralExpression(LiteralFalse),
            consequent: Box::new(BlockStatement {
                body: vec![ExpressionStatement(
                    ident!("bar")
                )]
            }),
            alternate: Some(Box::new(BlockStatement {
                body: vec![ExpressionStatement(
                    ident!("baz")
                )]
            })),
        })),
    });
}

#[test]
fn if_no_block_statement() {
    assert_statement!("if (true) foo;", IfStatement {
        test: LiteralExpression(LiteralTrue),
        consequent: Box::new(ExpressionStatement(
            ident!("foo")
        )),
        alternate: None,
    });
}

#[test]
fn if_else_no_block_statement() {
    assert_statement!("if (true) foo; else bar;", IfStatement {
        test: LiteralExpression(LiteralTrue),
        consequent: Box::new(ExpressionStatement(
            ident!("foo")
        )),
        alternate: Some(Box::new(ExpressionStatement(
            ident!("bar")
        ))),
    })
}

#[test]
fn for_statement() {
    assert_statement!("for (i = 0; i < 10; i++) {}", ForStatement {
        init: Some(Box::new(ExpressionStatement(
            BinaryExpression {
                left: Box::new(ident!("i")),
                operator: OperatorType::Assign,
                right: Box::new(num!(0.0)),
            }
        ))),
        test: Some(BinaryExpression {
            left: Box::new(ident!("i")),
            operator: OperatorType::Lesser,
            right: Box::new(num!(10.0)),
        }),
        update: Some(PostfixExpression {
            operator: OperatorType::Increment,
            operand: Box::new(ident!("i")),
        }),
        body: Box::new(BlockStatement {
            body: Vec::new(),
        }),
    });
}

#[test]
fn for_declare_statement() {
    assert_statement!("for (let i = 0; i < 10; i++) {}", ForStatement {
        init: Some(Box::new(VariableDeclarationStatement {
            kind: VariableDeclarationKind::Let,
            declarators: vec![
                VariableDeclarator {
                    name: "i".to_string(),
                    value: Some(num!(0.0)),
                }
            ],
        })),
        test: Some(BinaryExpression {
            left: Box::new(ident!("i")),
            operator: OperatorType::Lesser,
            right: Box::new(num!(10.0)),
        }),
        update: Some(PostfixExpression {
            operator: OperatorType::Increment,
            operand: Box::new(ident!("i")),
        }),
        body: Box::new(BlockStatement {
            body: Vec::new(),
        }),
    });
}

#[test]
fn for_empty_statement() {
    assert_statement!("for (;;) {}", ForStatement {
        init: None,
        test: None,
        update: None,
        body: Box::new(BlockStatement {
            body: Vec::new(),
        }),
    });
}

#[test]
fn for_in_statement() {
    assert_statement!("for (item in object) {}", ForInStatement {
        left: Box::new(ExpressionStatement(ident!("item"))),
        right: ident!("object"),
        body: Box::new(BlockStatement {
            body: Vec::new(),
        }),
    });
}


#[test]
fn for_in_declare_statement() {
    assert_statement!("for (let item in object) {}", ForInStatement {
        left: Box::new(VariableDeclarationStatement {
            kind: VariableDeclarationKind::Let,
            declarators: vec![
                VariableDeclarator {
                    name: "item".to_string(),
                    value: None,
                }
            ],
        }),
        right: ident!("object"),
        body: Box::new(BlockStatement {
            body: Vec::new(),
        }),
    });
}

#[test]
fn for_of_statement() {
    assert_statement!("for (item of array) {}", ForOfStatement {
        left: Box::new(ExpressionStatement(ident!("item"))),
        right: ident!("array"),
        body: Box::new(BlockStatement {
            body: Vec::new(),
        }),
    });
}


#[test]
fn for_of_declare_statement() {
    assert_statement!("for (let item of array) {}", ForOfStatement {
        left: Box::new(VariableDeclarationStatement {
            kind: VariableDeclarationKind::Let,
            declarators: vec![
                VariableDeclarator {
                    name: "item".to_string(),
                    value: None,
                }
            ],
        }),
        right: ident!("array"),
        body: Box::new(BlockStatement {
            body: Vec::new(),
        }),
    });
}

#[test]
fn while_statement() {
    assert_statement!("

    while (true) {
        foo;
    }

    ", WhileStatement {
        test: LiteralExpression(LiteralTrue),
        body: Box::new(BlockStatement {
            body: vec![ExpressionStatement(
                ident!("foo")
            )]
        }),
    });
}

#[test]
fn while_no_block_statement() {
    assert_statement!("while (true) foo;", WhileStatement {
        test: LiteralExpression(LiteralTrue),
        body: Box::new(ExpressionStatement(
            ident!("foo")
        )),
    });
}

#[test]
fn arrow_function() {
    assert_expression!("

    () => {
        bar;
    }

    ", ArrowFunctionExpression {
        params: vec![],
        body: Box::new(BlockStatement {
            body: vec![
                ExpressionStatement(ident!("bar"))
            ]
        })
    });
}

#[test]
fn arrow_function_shorthand() {
    assert_expression!("n => n * n", ArrowFunctionExpression {
        params: vec![
            param!("n")
        ],
        body: Box::new(ExpressionStatement(
            BinaryExpression {
                left: Box::new(ident!("n")),
                operator: Multiplication,
                right: Box::new(ident!("n")),
            }
        )),
    });
}

#[test]
fn arrow_function_with_params() {
    assert_expression!("

    (a, b, c) => {
        bar;
    }

    ", ArrowFunctionExpression {
        params: vec![
            param!("a"),
            param!("b"),
            param!("c"),
        ],
        body: Box::new(BlockStatement {
            body: vec![
                ExpressionStatement(ident!("bar"))
            ]
        })
    });
}

#[test]
fn function_expression() {
    assert_expression!("

    (function () {
        return bar;
    })

    ", FunctionExpression {
        name: None,
        params: vec![],
        body: vec![
            ReturnStatement {
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

    ", FunctionExpression {
        name: Some("foo".to_string()),
        params: vec![],
        body: vec![
            ReturnStatement {
                value: Some(ident!("bar"))
            }
        ]
    });
}

#[test]
fn expression_statement() {
    assert_statement!("foo", ExpressionStatement(ident!("foo")));
}

#[test]
fn sequence_expression_statement() {
    assert_statement!("foo, bar, baz", ExpressionStatement(
        SequenceExpression(vec![
            ident!("foo"),
            ident!("bar"),
            ident!("baz"),
        ])
    ));
}

#[test]
fn sequence_in_accessor() {
    assert_expression!("foo[1, 2, 3]", MemberExpression {
        object: Box::new(ident!("foo")),
        property: Box::new(MemberKey::Computed(
            SequenceExpression(vec![
                num!(1.0),
                num!(2.0),
                num!(3.0),
            ])
        ))
    });
}

#[test]
fn object_literal_member() {
    assert_expression!("({foo:100})", ObjectExpression(vec![
        ObjectMember::Literal {
            key: "foo".to_string(),
            value: num!(100.0),
        }
    ]));
}

#[test]
fn object_computed_member() {
    assert_expression!("({[100]:100})", ObjectExpression(vec![
        ObjectMember::Computed {
            key: num!(100.0),
            value: num!(100.0),
        }
    ]));
}

#[test]
fn object_shorthand_member() {
    assert_expression!("({foo})", ObjectExpression(vec![
        ObjectMember::Shorthand {
            key: "foo".to_string(),
        }
    ]));
}

#[test]
fn object_method_member() {
    assert_expression!("({foo() {} })", ObjectExpression(vec![
        ObjectMember::Method {
            name: "foo".to_string(),
            params: vec![],
            body: vec![],
        }
    ]));
}

#[test]
fn object_computed_method_member() {
    assert_expression!("({[100]() {} })", ObjectExpression(vec![
        ObjectMember::ComputedMethod {
            name: num!(100.0),
            params: vec![],
            body: vec![],
        }
    ]));
}
