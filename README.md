# HoneyBadger

WIP ES2015+ to ES5 transpiler + bundler + minifier in Rust.

Because Webpack+Babel+UglifyJS are both awesome and terrible at the same time.

This is nowhere near done, but this tree my MacBook Air from 2012 can generate in sub millisecond time:

```
Program {
    body: [
        VariableDeclarationStatement {
            kind: Let,
            declarations: [
                (
                    "foo",
                    Literal(
                        LiteralString(
                            "lorem"
                        )
                    )
                ),
                (
                    "bar",
                    Literal(
                        LiteralString(
                            "ipsum"
                        )
                    )
                )
            ]
        },
        VariableDeclarationStatement {
            kind: Const,
            declarations: [
                (
                    "pi",
                    Literal(
                        LiteralFloat(
                            3.14
                        )
                    )
                )
            ]
        },
        VariableDeclarationStatement {
            kind: Var,
            declarations: [
                (
                    "binary",
                    Literal(
                        LiteralInteger(
                            42
                        )
                    )
                ),
                (
                    "octal",
                    Literal(
                        LiteralInteger(
                            42
                        )
                    )
                ),
                (
                    "hexal",
                    Literal(
                        LiteralInteger(
                            42
                        )
                    )
                )
            ]
        },
        VariableDeclarationStatement {
            kind: Let,
            declarations: [
                (
                    "pojo",
                    ObjectExpression(
                        [
                            (
                                Static(
                                    "id"
                                ),
                                Literal(
                                    LiteralFloat(
                                        9001
                                    )
                                )
                            ),
                            (
                                Static(
                                    "name"
                                ),
                                Literal(
                                    LiteralString(
                                        "Maciej"
                                    )
                                )
                            ),
                            (
                                Static(
                                    "is-radical"
                                ),
                                Literal(
                                    LiteralTrue
                                )
                            ),
                            (
                                Computed(
                                    BinaryExpression {
                                        operator: Add,
                                        left: Identifier(
                                            "foo"
                                        ),
                                        right: Identifier(
                                            "bar"
                                        )
                                    }
                                ),
                                Literal(
                                    LiteralString(
                                        "totally"
                                    )
                                )
                            )
                        ]
                    )
                )
            ]
        },
        ExpressionStatement(
            CallExpression {
                callee: MemberExpression {
                    object: ArrayExpression(
                        [
                            Literal(
                                LiteralFloat(
                                    1
                                )
                            ),
                            Literal(
                                LiteralFloat(
                                    2
                                )
                            ),
                            Literal(
                                LiteralFloat(
                                    3
                                )
                            )
                        ]
                    ),
                    property: Static(
                        "forEach"
                    )
                },
                arguments: [
                    ArrowFunctionExpression {
                        params: [
                            Parameter {
                                name: "n"
                            }
                        ],
                        body: Expression(
                            BinaryExpression {
                                operator: Multiply,
                                left: Identifier(
                                    "n"
                                ),
                                right: Identifier(
                                    "n"
                                )
                            }
                        )
                    }
                ]
            }
        ),
        ExpressionStatement(
            CallExpression {
                callee: MemberExpression {
                    object: ArrayExpression(
                        [
                            Literal(
                                LiteralString(
                                    "fooz"
                                )
                            ),
                            Literal(
                                LiteralString(
                                    "baz"
                                )
                            )
                        ]
                    ),
                    property: Static(
                        "map"
                    )
                },
                arguments: [
                    ArrowFunctionExpression {
                        params: [
                            Parameter {
                                name: "item"
                            }
                        ],
                        body: Expression(
                            CallExpression {
                                callee: MemberExpression {
                                    object: Identifier(
                                        "item"
                                    ),
                                    property: Static(
                                        "toUpperCase"
                                    )
                                },
                                arguments: []
                            }
                        )
                    }
                ]
            }
        ),
        FunctionStatement {
            name: "helloKitty",
            params: [
                Parameter {
                    name: "count"
                },
                Parameter {
                    name: "name"
                }
            ],
            body: [
                WhileStatement {
                    condition: PostfixExpression {
                        operator: Decrement,
                        argument: Identifier(
                            "count"
                        )
                    },
                    body: Expression(
                        CallExpression {
                            callee: MemberExpression {
                                object: Identifier(
                                    "console"
                                ),
                                property: Static(
                                    "log"
                                )
                            },
                            arguments: [
                                Identifier(
                                    "name"
                                )
                            ]
                        }
                    )
                },
                ReturnStatement(
                    Literal(
                        LiteralFalse
                    )
                )
            ]
        },
        ExpressionStatement(
            CallExpression {
                callee: Identifier(
                    "helloKitty"
                ),
                arguments: []
            }
        ),
        VariableDeclarationStatement {
            kind: Let,
            declarations: [
                (
                    "emptyArray",
                    ArrayExpression(
                        []
                    )
                )
            ]
        },
        ClassStatement {
            name: "Foo",
            extends: None,
            body: [
                ClassProperty {
                    is_static: false,
                    name: "x",
                    value: Literal(
                        LiteralFloat(
                            0
                        )
                    )
                },
                ClassProperty {
                    is_static: false,
                    name: "y",
                    value: Literal(
                        LiteralFloat(
                            0
                        )
                    )
                },
                ClassProperty {
                    is_static: true,
                    name: "isFoo",
                    value: Literal(
                        LiteralTrue
                    )
                },
                ClassConstructor {
                    params: [],
                    body: [
                        ExpressionStatement(
                            CallExpression {
                                callee: MemberExpression {
                                    object: Identifier(
                                        "console"
                                    ),
                                    property: Static(
                                        "log"
                                    )
                                },
                                arguments: [
                                    Literal(
                                        LiteralString(
                                            "New instance of Foo"
                                        )
                                    )
                                ]
                            }
                        )
                    ]
                },
                ClassMethod {
                    is_static: false,
                    name: "bar",
                    params: [
                        Parameter {
                            name: "n"
                        }
                    ],
                    body: [
                        ExpressionStatement(
                            CallExpression {
                                callee: MemberExpression {
                                    object: Identifier(
                                        "console"
                                    ),
                                    property: Static(
                                        "log"
                                    )
                                },
                                arguments: [
                                    BinaryExpression {
                                        operator: Add,
                                        left: Literal(
                                            LiteralString(
                                                "Called bar with "
                                            )
                                        ),
                                        right: Identifier(
                                            "n"
                                        )
                                    }
                                ]
                            }
                        )
                    ]
                },
                ClassMethod {
                    is_static: true,
                    name: "baz",
                    params: [],
                    body: [
                        ExpressionStatement(
                            CallExpression {
                                callee: MemberExpression {
                                    object: Identifier(
                                        "console"
                                    ),
                                    property: Static(
                                        "log"
                                    )
                                },
                                arguments: [
                                    Literal(
                                        LiteralString(
                                            "Static method baz!"
                                        )
                                    )
                                ]
                            }
                        )
                    ]
                }
            ]
        }
    ]
}
Took 0.131372ms
```
