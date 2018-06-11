use ast::statement::*;
use ast::{Block, DeclarationKind, Declarator, Loc, Statement};
use astgen::SerializeInLoc;
use serde::ser::{Serialize, SerializeStruct, Serializer};

// TODO: DRY with BlockStatement
impl<'ast> Serialize for Loc<Block<'ast, SwitchCase<'ast>>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.body.serialize(serializer)
    }
}

impl<'ast> SerializeInLoc for DeclarationStatement<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "VariableDeclaration", 2, |state| {
            state.serialize_field("kind", &self.kind)?;
            state.serialize_field("declarations", &self.declarators)
        })
    }
}

impl<'ast> SerializeInLoc for Declarator<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "VariableDeclarator", 2, |state| {
            state.serialize_field("id", &self.id)?;
            state.serialize_field("init", &self.init)
        })
    }
}

impl<'ast> Serialize for DeclarationKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use self::DeclarationKind::*;
        match *self {
            Const => serializer.serialize_str("const"),
            Let => serializer.serialize_str("let"),
            Var => serializer.serialize_str("var"),
        }
    }
}

impl<'ast> Serialize for ForInit<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use self::ForInit::*;

        match *self {
            Declaration(declaration) => {
                let state = declaration.serialize(serializer)?;
                state.end()
            }
            Expression(expression) => expression.serialize(serializer),
        }
    }
}

impl<'ast> SerializeInLoc for TryStatement<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "TryStatement", 3, |state| {
            state.serialize_field("block", &self.block)?;
            state.serialize_field("handler", &self.handler)?;
            state.serialize_field("finalizer", &self.finalizer)
        })
    }
}

impl<'ast> SerializeInLoc for CatchClause<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "CatchClause", 2, |state| {
            state.serialize_field("param", &self.param)?;
            state.serialize_field("body", &self.body)
        })
    }
}

impl<'ast> SerializeInLoc for BlockStatement<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "BlockStatement", 1, |state| {
            state.serialize_field("body", &self.body)
        })
    }
}

impl<'ast> SerializeInLoc for LabeledStatement<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "LabeledStatement", 2, |state| {
            state.serialize_field("label", &self.label)?;
            state.serialize_field("body", &self.body)
        })
    }
}

impl<'ast> SerializeInLoc for SwitchStatement<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "SwitchStatement", 2, |state| {
            state.serialize_field("discriminant", &self.discriminant)?;
            state.serialize_field("cases", &*self.cases)
        })
    }
}

impl<'ast> SerializeInLoc for SwitchCase<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "SwitchCase", 2, |state| {
            state.serialize_field("test", &self.test)?;
            state.serialize_field("consequent", &self.consequent)
        })
    }
}
impl<'ast> SerializeInLoc for ForInit<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        match *self {
            ForInit::Declaration(ref declaration) => declaration.serialize(serializer),
            ForInit::Expression(ref expression) => expression.item.serialize(serializer),
        }
    }
}

impl<'ast> SerializeInLoc for ForStatement<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "ForStatement", 4, |state| {
            state.serialize_field("init", &self.init)?;
            state.serialize_field("test", &self.test)?;
            state.serialize_field("update", &self.update)?;
            state.serialize_field("body", &self.body)
        })
    }
}

impl<'ast> SerializeInLoc for ForInStatement<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "ForInStatement", 3, |state| {
            state.serialize_field("left", &self.left)?;
            state.serialize_field("right", &self.right)?;
            state.serialize_field("body", &self.body)
        })
    }
}

impl<'ast> SerializeInLoc for ForOfStatement<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "ForOfStatement", 3, |state| {
            state.serialize_field("left", &self.left)?;
            state.serialize_field("right", &self.right)?;
            state.serialize_field("body", &self.body)
        })
    }
}
impl<'ast> SerializeInLoc for IfStatement<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "IfStatement", 3, |state| {
            state.serialize_field("test", &self.test)?;
            state.serialize_field("consequent", &self.consequent)?;
            state.serialize_field("alternate", &self.alternate)
        })
    }
}

impl<'ast> SerializeInLoc for Statement<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        use self::Statement::*;

        match *self {
            Empty => self.in_loc(serializer, "EmptyStatement", 0, |_| Ok(())),
            Expression(ref expression) => {
                self.in_loc(serializer, "ExpressionStatement", 1, |state| {
                    state.serialize_field("expression", expression)
                })
            }
            Declaration(declaration) => declaration.serialize(serializer),
            Return(ReturnStatement { ref value }) => {
                self.in_loc(serializer, "ReturnStatement", 1, |state| {
                    state.serialize_field("argument", value)
                })
            }
            Break(BreakStatement { ref label }) => {
                self.in_loc(serializer, "BreakStatement", 1, |state| {
                    state.serialize_field("label", label)
                })
            }
            Continue(ContinueStatement { ref label }) => {
                self.in_loc(serializer, "ContinueStatement", 1, |state| {
                    state.serialize_field("label", label)
                })
            }
            Throw(ThrowStatement { ref value }) => {
                self.in_loc(serializer, "ThrowStatement", 1, |state| {
                    state.serialize_field("argument", value)
                })
            }
            If(statement) => statement.serialize(serializer),
            While(WhileStatement { ref test, ref body }) => {
                self.in_loc(serializer, "WhileStatement", 2, |state| {
                    state.serialize_field("test", test)?;
                    state.serialize_field("body", body)
                })
            }
            Do(DoStatement { ref body, ref test }) => {
                self.in_loc(serializer, "DoWhileStatement", 2, |state| {
                    state.serialize_field("body", body)?;
                    state.serialize_field("test", test)
                })
            }
            For(statement) => statement.serialize(serializer),
            ForIn(statement) => statement.serialize(serializer),
            ForOf(statement) => statement.serialize(serializer),
            Try(statement) => statement.serialize(serializer),
            Block(statement) => statement.serialize(serializer),
            Labeled(statement) => statement.serialize(serializer),
            Function(statement) => statement.serialize(serializer),
            Class(statement) => statement.serialize(serializer),
            Switch(statement) => statement.serialize(serializer),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_statement_empty() {
        expect_parse!(";", {
            "type": "Program",
            "body": [
                {
                    "type": "EmptyStatement",
                    "start": 0,
                    "end": 1,
                }
              ],
              "start": 0,
              "end": 1,
        });
    }

    #[test]
    fn test_statement_expression_statement() {
        expect_parse!("foo;", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "Identifier",
                        "name": "foo",
                        "start": 0,
                        "end": 3
                    },
                    "start": 0,
                    "end": 3,
                }
              ],
              "start": 0,
              "end": 3,
        });
    }

    #[test]
    fn test_declaration_statement() {
        expect_parse!("var a", {
            "type": "Program",
            "body": [
                {
                    "type": "VariableDeclaration",
                    "kind": "var",
                    "declarations": [
                        {
                            "type": "VariableDeclarator",
                            "id": {
                                "type": "Identifier",
                                "name": "a",
                                "start": 4,
                                "end": 5
                            },
                            "init": null,
                            "start": 4,
                            "end": 5,
                        }
                    ],
                    "start": 0,
                    "end": 5,
                }
              ],
              "start": 0,
              "end": 5,
        });

        expect_parse!("let a", {
            "type": "Program",
            "body": [
                {
                    "type": "VariableDeclaration",
                    "kind": "let",
                    "declarations": [
                        {
                            "type": "VariableDeclarator",
                            "id": {
                                "type": "Identifier",
                                "name": "a",
                                "start": 4,
                                "end": 5
                            },
                            "init": null,
                            "start": 4,
                            "end": 5,
                        }
                    ],
                    "start": 0,
                    "end": 5,
                }
              ],
              "start": 0,
              "end": 5,
        });

        expect_parse!("const a", {
            "type": "Program",
            "body": [
                {
                    "type": "VariableDeclaration",
                    "kind": "const",
                    "declarations": [
                        {
                            "type": "VariableDeclarator",
                            "id": {
                                "type": "Identifier",
                                "name": "a",
                                "start": 6,
                                "end": 7
                            },
                            "init": null,
                            "start": 6,
                            "end": 7,
                        }
                    ],
                    "start": 0,
                    "end": 7,
                }
              ],
              "start": 0,
              "end": 7,
        });

        expect_parse!("const a = 2", {
            "type": "Program",
            "body": [
                {
                    "type": "VariableDeclaration",
                    "kind": "const",
                    "declarations": [
                        {
                            "type": "VariableDeclarator",
                            "id": {
                                "type": "Identifier",
                                "name": "a",
                                "start": 6,
                                "end": 7
                            },
                            "init": {
                                "type": "Literal",
                                "value": 2,
                                "raw": "2",
                                "start": 10,
                                "end": 11
                            },
                            "start": 6,
                            "end": 11,
                        }
                    ],
                    "start": 0,
                    "end": 11,
                }
              ],
              "start": 0,
              "end": 11,
        });

        expect_parse!("const [a] = [2]", {
            "type": "Program",
            "body": [
                {
                    "type": "VariableDeclaration",
                    "kind": "const",
                    "declarations": [
                        {
                            "type": "VariableDeclarator",
                            "id": {
                                "type": "ArrayPattern",
                                "elements": [
                                    {
                                        "type": "Identifier",
                                        "name": "a",
                                        "start": 7,
                                        "end": 8
                                    }
                                ],
                                "start": 6,
                                "end": 9
                            },
                            "init": {
                                "type": "ArrayExpression",
                                "elements": [
                                    {
                                        "type": "Literal",
                                        "value": 2,
                                        "raw": "2",
                                        "start": 13,
                                        "end": 14
                                    }
                                ],
                                "start": 12,
                                "end": 15

                            },
                            "start": 6,
                            "end": 15,
                        }
                    ],
                    "start": 0,
                    "end": 15,
                }
              ],
              "start": 0,
              "end": 15,
        });
    }

    #[test]
    fn test_statement_return_statement() {
        expect_parse!("return;", {
            "type": "Program",
            "body": [
                {
                    "type": "ReturnStatement",
                    "argument": null,
                    "start": 0,
                    "end": 6,
                }
              ],
              "start": 0,
              "end": 6,
        });

        expect_parse!("return foo;", {
            "type": "Program",
            "body": [
                {
                    "type": "ReturnStatement",
                    "argument": {
                        "type": "Identifier",
                        "name": "foo",
                        "start": 7,
                        "end": 10
                    },
                    "start": 0,
                    "end": 10,
                }
              ],
              "start": 0,
              "end": 10,
        });
    }

    #[test]
    fn test_statement_break_statement() {
        expect_parse!("break;", {
            "type": "Program",
            "body": [
                {
                    "type": "BreakStatement",
                    "label": null,
                    "start": 0,
                    "end": 5,
                }
              ],
              "start": 0,
              "end": 5,
        });

        expect_parse!("break foo;", {
            "type": "Program",
            "body": [
                {
                    "type": "BreakStatement",
                    "label": {
                        "type": "Identifier",
                        "name": "foo",
                        "start": 6,
                        "end": 9
                    },
                    "start": 0,
                    "end": 9,
                }
              ],
              "start": 0,
              "end": 9,
        });
    }

    #[test]
    fn test_statement_continue_statement() {
        expect_parse!("continue;", {
            "type": "Program",
            "body": [
                {
                    "type": "ContinueStatement",
                    "label": null,
                    "start": 0,
                    "end": 8,
                }
              ],
              "start": 0,
              "end": 8
        });

        expect_parse!("continue foo;", {
            "type": "Program",
            "body": [
                {
                    "type": "ContinueStatement",
                    "label": {
                        "type": "Identifier",
                        "name": "foo",
                        "start": 9,
                        "end": 12
                    },
                    "start": 0,
                    "end": 12,
                }
              ],
              "start": 0,
              "end": 12,
        });
    }

    #[test]
    fn test_statement_throw_statement() {
        expect_parse!("throw foo;", {
            "type": "Program",
            "body": [
                {
                    "type": "ThrowStatement",
                    "argument": {
                        "type": "Identifier",
                        "name": "foo",
                        "start": 6,
                        "end": 9
                    },
                    "start": 0,
                    "end": 9,
                }
              ],
              "start": 0,
              "end": 9,
        });
    }

    #[test]
    fn test_statement_if_statement() {
        expect_parse!("if (true) {}", {
            "type": "Program",
            "body": [
                {
                    "type": "IfStatement",
                    "test": {
                        "type": "Literal",
                        "value": true,
                        "raw": "true",
                        "start": 4,
                        "end": 8
                    },
                    "consequent": {
                        "type": "BlockStatement",
                        "body": [],
                        "start": 10,
                        "end": 12
                    },
                    "alternate": null,
                    "start": 0,
                    "end": 12,
                }
              ],
              "start": 0,
              "end": 12,
        });

        expect_parse!("if (true) {} else {}", {
            "type": "Program",
            "body": [
                {
                    "type": "IfStatement",
                    "test": {
                        "type": "Literal",
                        "value": true,
                        "raw": "true",
                        "start": 4,
                        "end": 8
                    },
                    "consequent": {
                        "type": "BlockStatement",
                        "body": [],
                        "start": 10,
                        "end": 12
                    },
                    "alternate": {
                        "type": "BlockStatement",
                        "body": [],
                        "start": 18,
                        "end": 20
                    },
                    "start": 0,
                    "end": 20,
                }
              ],
              "start": 0,
              "end": 20,
        });
    }

    #[test]
    fn test_while_statement() {
        expect_parse!("while (false) {}", {
            "type": "Program",
            "body": [
                {
                    "type": "WhileStatement",
                    "test": {
                        "type": "Literal",
                        "value": false,
                        "raw": "false",
                        "start": 7,
                        "end": 12
                    },
                    "body": {
                        "type": "BlockStatement",
                        "body": [],
                        "start": 14,
                        "end": 16
                    },
                    "start": 0,
                    "end": 16,
                }
              ],
              "start": 0,
              "end": 16,
        });
    }

    #[test]
    fn test_do_statement() {
        expect_parse!("do {} while (false)", {
            "type": "Program",
            "body": [
                {
                    "type": "DoWhileStatement",
                    "body": {
                        "type": "BlockStatement",
                        "body": [],
                        "start": 3,
                        "end": 5
                    },
                    "test": {
                        "type": "Literal",
                        "value": false,
                        "raw": "false",
                        "start": 13,
                        "end": 18
                    },
                    "start": 0,
                    "end": 19,
                }
              ],
              "start": 0,
              "end": 19,
        });
    }

    #[test]
    fn test_for_statement() {
        expect_parse!("for (;;) {}", {
            "type": "Program",
            "body": [
                {
                    "type": "ForStatement",
                    "init": null,
                    "test": null,
                    "update": null,
                    "body": {
                        "type": "BlockStatement",
                        "body": [],
                        "start": 9,
                        "end": 11
                    },
                    "start": 0,
                    "end": 11,
                }
              ],
              "start": 0,
              "end": 11,
        });

        expect_parse!("for (i; i < 10; i++) {}", {
            "type": "Program",
            "body": [
                {
                    "type": "ForStatement",
                    "init": {
                        "type": "Identifier",
                        "name": "i",
                        "start": 5,
                        "end": 6,
                    },
                    "test": {
                        "type": "BinaryExpression",
                        "operator": "<",
                        "left": {
                            "type": "Identifier",
                            "name": "i",
                            "start": 8,
                            "end": 9,
                        },
                        "right": {
                            "type": "Literal",
                            "value": 10,
                            "raw": "10",
                            "start": 12,
                            "end": 14,
                        },
                        "start": 8,
                        "end": 14,
                    },
                    "update": {
                        "type": "UpdateExpression",
                        "operator": "++",
                        "argument": {
                            "type": "Identifier",
                            "name": "i",
                            "start": 16,
                            "end": 17,
                        },
                        "prefix": false,
                        "start": 16,
                        "end": 19,
                    },
                    "body": {
                        "type": "BlockStatement",
                        "body": [],
                        "start": 21,
                        "end": 23,
                    },
                    "start": 0,
                    "end": 23,
                }
            ],
            "start": 0,
            "end": 23,
        });

        expect_parse!("for (key in {}) {}", {
            "type": "Program",
            "body": [
                {
                    "type": "ForInStatement",
                    "left": {
                        "type": "Identifier",
                        "name": "key",
                        "start": 5,
                        "end": 8,
                    },
                    "right": {
                        "type": "ObjectExpression",
                        "properties": [],
                        "start": 12,
                        "end": 14,
                    },
                    "body": {
                        "type": "BlockStatement",
                        "body": [],
                        "start": 16,
                        "end": 18,
                    },
                    "start": 0,
                    "end": 18,
                }
            ],
            "start": 0,
            "end": 18,
        });

        expect_parse!("for (key of {}) {}", {
            "type": "Program",
            "body": [
                {
                    "type": "ForOfStatement",
                    "left": {
                        "type": "Identifier",
                        "name": "key",
                        "start": 5,
                        "end": 8,
                    },
                    "right": {
                        "type": "ObjectExpression",
                        "properties": [],
                        "start": 12,
                        "end": 14,
                    },
                    "body": {
                        "type": "BlockStatement",
                        "body": [],
                        "start": 16,
                        "end": 18,
                    },
                    "start": 0,
                    "end": 18,
                }
            ],
            "start": 0,
            "end": 18,
        });
    }

    #[test]
    fn test_try_statement() {
        expect_parse!("try {} catch (e) {}", {
            "type": "Program",
            "body": [
                {
                    "type": "TryStatement",
                    "block": {
                        "type": "BlockStatement",
                        "body": [],
                        "start": 4,
                        "end": 6,
                    },
                    "handler": {
                        "type": "CatchClause",
                        "param": {
                            "type": "Identifier",
                            "name": "e",
                            "start": 14,
                            "end": 15,
                        },
                        "body": {
                            "type": "BlockStatement",
                            "body": [],
                            "start": 17,
                            "end": 19,
                        },
                        "start": 7,
                        "end": 19,
                    },
                    "finalizer": null,
                    "start": 0,
                    "end": 19,
                }
            ],
            "start": 0,
            "end": 19,
        });
    }

    #[test]
    fn test_block_statement() {
        expect_parse!("{2}", {
            "type": "Program",
            "body": [
                {
                    "type": "BlockStatement",
                    "body": [
                        {
                            "type": "ExpressionStatement",
                            "expression": {
                                "type": "Literal",
                                "value": 2,
                                "raw": "2",
                                "start": 1,
                                "end": 2
                            },
                            "start": 1,
                            "end": 2,
                        }
                    ],
                    "start": 0,
                    "end": 3,
                }
            ],
            "start": 0,
            "end": 3,
        });
    }

    #[test]
    fn test_function_statement() {
        expect_parse!("function foo () {}", {
            "type": "Program",
            "body": [
                {
                    "type": "FunctionDeclaration",
                    "generator": false,
                    "id": {
                        "type": "Identifier",
                        "name": "foo",
                        "start": 9,
                        "end": 12
                    },
                    "params": [],
                    "body": {
                        "type": "BlockStatement",
                        "body": [],
                        "start": 16,
                        "end": 18,
                    },
                    "start": 0,
                    "end": 18,
                }
            ],
            "start": 0,
            "end": 18,
        });

        expect_parse!("function* foo () {}", {
            "type": "Program",
            "body": [
                {
                    "type": "FunctionDeclaration",
                    "generator": true,
                    "id": {
                        "type": "Identifier",
                        "name": "foo",
                        "start": 10,
                        "end": 13
                    },
                    "params": [],
                    "body": {
                        "type": "BlockStatement",
                        "body": [],
                        "start": 17,
                        "end": 19,
                    },
                    "start": 0,
                    "end": 19,
                }
            ],
            "start": 0,
            "end": 19,
        });

        expect_parse!("function foo (a, value = true) {}", {
            "type": "Program",
            "body": [
                {
                    "type": "FunctionDeclaration",
                    "generator": false,
                    "id": {
                        "type": "Identifier",
                        "name": "foo",
                        "start": 9,
                        "end": 12
                    },
                    "params": [
                        {
                            "type": "Identifier",
                            "name": "a",
                            "start": 14,
                            "end": 15
                        },
                        {
                            "type": "AssignmentPattern",
                            "left": {
                                "type": "Identifier",
                                "name": "value",
                                "start": 17,
                                "end": 22
                            },
                            "right": {
                                "type": "Literal",
                                "value": true,
                                "raw": "true",
                                "start": 25,
                                "end": 29,
                            },
                            "start": 17,
                            "end": 29
                        }
                    ],
                    "body": {
                        "type": "BlockStatement",
                        "body": [],
                        "start": 31,
                        "end": 33,
                    },
                    "start": 0,
                    "end": 33,
                }
            ],
            "start": 0,
            "end": 33,
        });
    }
}
