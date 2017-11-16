use serde::ser::{Serialize, Serializer, SerializeStruct};
use serde_json;
use ast::{StatementList, Statement, Loc, Expression, ExpressionPtr};
use ast::{Declarator, DeclaratorId};

use astgen::function::ClassBody;

#[derive(Debug)]
struct CatchClause<'ast> {
    param: ExpressionPtr<'ast>,
    body: StatementList<'ast>,
}

#[derive(Debug)]
pub struct BlockStatement<'ast> {
    pub body: StatementList<'ast>
}

impl<'ast> Serialize for Loc<BlockStatement<'ast>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut state = serializer.serialize_struct("BlockStatement", 4)?;
        state.serialize_field("type", &"BlockStatement")?;
        state.serialize_field("body", &self.body)?;
        state.serialize_field("start", &self.start)?;
        state.serialize_field("end", &self.end)?;
        state.end()
    }
}

impl<'ast> Serialize for Loc<CatchClause<'ast>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut state = serializer.serialize_struct("CatchClause", 5)?;
        state.serialize_field("type", &"CatchClause")?;
        state.serialize_field("param", &*self.param)?;
        let body = Loc::new(self.start, self.end, BlockStatement { body: self.body });
        state.serialize_field("body", &body)?;
        state.serialize_field("start", &self.start)?;
        state.serialize_field("end", &self.end)?;
        state.end()
    }
}

impl<'ast> Serialize for Loc<Declarator<'ast>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {

        let mut state = serializer.serialize_struct("VariableDeclarator", 4)?;
        state.serialize_field("type", &"VariableDeclarator")?;

        state.serialize_field("id", &Loc::new(self.start, self.end, self.name))?;
        if let Some(value) = self.value {
           state.serialize_field("init", &*value)?;
        } else {
           state.serialize_field("init", &serde_json::Value::Null)?;
        }

        state.serialize_field("start", &self.start)?;
        state.serialize_field("end", &self.end)?;
        state.end()
    }
}

impl<'ast> Serialize for Loc<DeclaratorId<'ast>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        match self.item {
            DeclaratorId::Identifier(ident) => {
                let value = &Loc::new(self.start, self.end, Expression::Identifier(ident));
                serializer.serialize_some(value)
            },
            DeclaratorId::Pattern(expr) => {
                match expr.item {
                    Expression::Array { body } => {
                        let mut state = serializer.serialize_struct("ArrayPattern", 4)?;
                        state.serialize_field("type", &"ArrayPattern")?;
                        state.serialize_field("elements", &body)?;
                        state.serialize_field("self", &self.start)?;
                        state.serialize_field("end", &self.end)?;
                        return state.end();
                    },
                    _ => {
                        panic!("Unimplemented: ParameterKey::Pattern(expr)");
                    }
                }
            }
        }
    }
}

impl<'ast> Serialize for Loc<Statement<'ast>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
      use self::Statement::*;

      let mut state = match self.item {
        Error { .. } => panic!("Module contains errors"),
        Empty => {
            let mut state = serializer.serialize_struct("EmptyStatement", 3)?;
            state.serialize_field("type", &"EmptyStatement")?;
            state
        },
        Expression { expression } => {
            let mut state = serializer.serialize_struct("ExpressionStatement", 4)?;
            state.serialize_field("type", &"ExpressionStatement")?;
            state.serialize_field("expression", &*expression)?;
            state
        },
        Declaration { kind, declarators } => {
            let mut state = serializer.serialize_struct("VariableDeclaration", 5)?;
            state.serialize_field("type", &"VariableDeclaration")?;
            state.serialize_field("kind", &kind)?;
            state.serialize_field("declarations", &declarators)?;
            state
        },
        Return { value } => {
            let mut state = serializer.serialize_struct("ReturnStatement", 4)?;
            state.serialize_field("type", &"ReturnStatement")?;
            if let Some(expr) = value {
                state.serialize_field("argument", &*expr)?;
            } else {
                state.serialize_field("argument", &serde_json::Value::Null)?;
            }
            state
        },
        Break { label } => {
            let mut state = serializer.serialize_struct("BreakStatement", 4)?;
            state.serialize_field("type", &"BreakStatement")?;

            if let Some(expr) = label {
                state.serialize_field("label", &*expr)?;
            } else {
                state.serialize_field("label", &serde_json::Value::Null)?;
            }
            state
        },
        Throw { value } => {
            let mut state = serializer.serialize_struct("ThrowStatement", 4)?;
            state.serialize_field("type", &"ThrowStatement")?;
            state.serialize_field("argument", &*value)?;
            state
        },
        If { test, consequent, alternate } => {
            let mut state = serializer.serialize_struct("IfStatement", 6)?;
            state.serialize_field("type", &"IfStatement")?;
            state.serialize_field("test", &*test)?;
            state.serialize_field("consequent", &*consequent)?;
            if let Some(alternate) = alternate {
                state.serialize_field("alternate", &*alternate)?;
            } else {
                state.serialize_field("alternate", &serde_json::Value::Null)?;
            }
            state
        },
        While { test, body } => {
            let mut state = serializer.serialize_struct("WhileStatement", 5)?;
            state.serialize_field("type", &"WhileStatement")?;
            state.serialize_field("test", &*test)?;
            state.serialize_field("body", &*body)?;
            state
        },
        Do { body, test } => {
            let mut state = serializer.serialize_struct("DoWhileStatement", 5)?;
            state.serialize_field("type", &"DoWhileStatement")?;
            state.serialize_field("body", &*body)?;
            state.serialize_field("test", &*test)?;
            state
        },
        For { init, test, update, body } => {
            let mut state = serializer.serialize_struct("ForStatement", 7)?;
            state.serialize_field("type", &"ForStatement")?;

            if let Some(init) = init {
                state.serialize_field("init", &*init)?;
            } else {
                state.serialize_field("init", &serde_json::Value::Null)?;
            }

            if let Some(test) = test {
                state.serialize_field("test", &*test)?;
            } else {
                state.serialize_field("test", &serde_json::Value::Null)?;
            }

            if let Some(update) = update {
                state.serialize_field("update", &*update)?;
            } else {
                state.serialize_field("update", &serde_json::Value::Null)?;
            }

            state.serialize_field("body", &*body)?;
            state
        },
        ForIn { left, right, body } => {
            let mut state = serializer.serialize_struct("ForInStatement", 6)?;
            state.serialize_field("type", &"ForInStatement")?;
            state.serialize_field("left", &*left)?;
            state.serialize_field("right", &*right)?;
            state.serialize_field("body", &*body)?;
            state
        },
        ForOf { left, right, body } => {
            let mut state = serializer.serialize_struct("ForOfStatement", 6)?;
            state.serialize_field("type", &"ForOfStatement")?;
            state.serialize_field("left", &*left)?;
            state.serialize_field("right", &*right)?;
            state.serialize_field("body", &*body)?;
            state
        },
        Try { body, error, handler } => {
            let mut state = serializer.serialize_struct("TryStatement", 5)?;
            state.serialize_field("type", &"TryStatement")?;
            state.serialize_field("block", &Loc::new(self.start, self.end, BlockStatement { body: body }))?;
            let handler = Loc::new(self.start, self.end, CatchClause {
                param: error,
                body: handler
            });

            state.serialize_field("handler", &handler)?;
            state
        },
        Block { body } => {
            let mut state = serializer.serialize_struct("BlockStatement", 4)?;
            state.serialize_field("type", &"BlockStatement")?;
            state.serialize_field("body", &body)?;
            state
        },
        Labeled { label, body } => {
            let mut state = serializer.serialize_struct("LabeledStatement", 5)?;
            state.serialize_field("type", &"LabeledStatement")?;
            state.serialize_field("label", &label)?;
            state.serialize_field("body", &*body)?;
            state
        },
        Function { function } => {
            let mut state = serializer.serialize_struct("FunctionDeclaration", 6)?;
            state.serialize_field("type", &"FunctionDeclaration")?;
            state.serialize_field("name", &Loc::new(self.start, self.end, function.name))?;
            state.serialize_field("params", &function.params)?;

            match function.body.only_element() {
                Some(&Loc { item: Block { .. } , .. }) => {
                    state.serialize_field("body", &function.body)?;
                },
                _ => {
                let body = BlockStatement { body: function.body };
                    state.serialize_field("body", &Loc::new(self.start, self.end, body))?;
                }
            };
            state
        },
        Class { class } => {
            let mut state = serializer.serialize_struct("ClassDeclaration", 6)?;
            state.serialize_field("type", &"ClassDeclaration")?;
            state.serialize_field("id", &Loc::new(self.start, self.end, class.name))?;
            if let Some(extends) = class.extends {
                state.serialize_field("superClass", &*extends)?;
            } else {
                state.serialize_field("superClass", &serde_json::Value::Null)?;
            }
            state.serialize_field("body", &Loc::new(self.start, self.end, ClassBody { body: class.body }))?;
            state
        },
        Continue { label } => {
            let mut state = serializer.serialize_struct("ContinueStatement", 4)?;
            state.serialize_field("type", &"ContinueStatement")?;
            if let Some(label) = label {
                state.serialize_field("label", &*label)?;
            } else  {
                state.serialize_field("label", &serde_json::Value::Null)?;
            }
            state
        },
        Switch { discriminant, cases } => {
            let mut state = serializer.serialize_struct("SwitchStatement", 5)?;
            state.serialize_field("type", &"SwitchStatement")?;
            state.serialize_field("discriminant", &*discriminant)?;
            state.serialize_field("cases", &cases)?;
            state
        },
        SwitchCase { test, consequent } => {
            let mut state = serializer.serialize_struct("SwitchCase", 5)?;
            state.serialize_field("type", &"SwitchCase")?;

            if let Some(test) = test {
                state.serialize_field("test", &*test)?;
            } else {
                state.serialize_field("test", &serde_json::Value::Null)?;
            }

            state.serialize_field("consequent", &consequent)?;
            state
        }
      };

      state.serialize_field("start", &self.start)?;
      state.serialize_field("end", &self.end)?;
      state.end()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use parser::{parse};
    use astgen::generate_ast;

    #[test]
    fn test_empty_statement() {
        expect_parse!(";", {
            "type": "Program",
            "body": [
                {
                    "type": "EmptyStatement",
                    "start": 0,
                    "end": 1,
                }
            ],
            "end": 0,
            "start": 0
        });
    }

    #[test]
    fn test_expression_statement() {
        expect_parse!("true;", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "Literal",
                        "start": 0,
                        "end": 4,
                        "value": true
                    },
                    "start": 0,
                    "end": 4,
                }
            ],
            "end": 0,
            "start": 0
        });
    }

    #[test]
    fn test_declaration_statement() {
        expect_parse!("var a", {
            "type": "Program",
            "body": [
                {
                    "declarations": [
                        {
                            "end": 0,
                            "id": {
                                "end": 0,
                                "name": "a",
                                "start": 0,
                                "type": "Identifier"
                            },
                            "init": null,
                            "start": 0,
                            "type": "VariableDeclarator"
                        }
                    ],
                    "end": 0,
                    "kind": "var",
                    "start": 0,
                    "type": "VariableDeclaration"
                }
            ],
            "end": 0,
            "start": 0,
        });
    }

    #[test]
    fn test_return_statement() {
        expect_parse!("return null", {
            "type": "Program",
            "body": [
                {
                    "type": "ReturnStatement",
                    "argument": {
                        "type": "Literal",
                        "value": "null",
                        "start": 7,
                        "end": 11,
                    },
                    "end": 0,
                    "start": 0,
                }
            ],
            "end": 0,
            "start": 0
        });
        expect_parse!("return true", {
            "type": "Program",
            "body": [
                {
                        "type": "ReturnStatement",
                        "argument": {
                        "type": "Literal",
                        "start": 7,
                        "end": 11,
                        "value": true,
                    },
                    "end": 0,
                    "start": 0,
                }
            ],
            "end": 0,
            "start": 0
        });
    }

    #[test]
    fn test_break_statement() {
        expect_parse!("break", {
            "type": "Program",
            "body": [
                {
                    "end": 0,
                    "label": null,
                    "start": 0,
                    "type": "BreakStatement"
                }
            ],
            "end": 0,
            "start": 0,
        }
        );
    }

    #[test]
    fn test_throw_statement() {
        expect_parse!("throw a", {
            "type": "Program",
            "body": [
                {
                    "argument": {
                        "end": 7,
                        "name": "a",
                        "start": 6,
                        "type": "Identifier"
                    },
                    "end": 0,
                    "start": 0,
                    "type": "ThrowStatement"
                }
            ],
            "end": 0,
            "start": 0,
        });
    }

    #[test]
    fn test_if_statement() {
        expect_parse!("if (true) {}", {
            "type": "Program",
            "body": [
                {
                    "type": "IfStatement",
                    "test": {
                        "type": "Literal",
                        "value": true,
                        "start": 4,
                        "end": 8,
                    },
                    "consequent": {
                        "type": "BlockStatement",
                        "body": [],
                        "start": 0,
                        "end": 0,
                    },
                    "alternate": null,
                    "start": 0,
                    "end": 0,
                }
            ],
            "end": 0,
            "start": 0
        });
    }

    #[test]
    fn test_if_else_statement() {
        expect_parse!("if (true) {} else { false }", {
            "type": "Program",
            "body": [
                {
                    "type": "IfStatement",
                    "test": {
                        "type": "Literal",
                        "value": true,
                        "start": 4,
                        "end": 8,
                    },
                    "consequent": {
                        "type": "BlockStatement",
                        "body": [
                        ],
                        "start": 0,
                        "end": 0,
                    },
                    "alternate": {
                        "type": "BlockStatement",
                        "body": [
                            {
                                "type": "ExpressionStatement",
                                "expression": {
                                    "type": "Literal",
                                    "start": 20,
                                    "end": 25,
                                    "value": false
                                },
                                "start": 20,
                                "end": 25,
                            }
                        ],
                        "start": 0,
                        "end": 0,
                    },
                    "start": 0,
                    "end": 0,
                }
            ],
            "end": 0,
            "start": 0
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
                        "start": 7,
                        "end": 12,
                    },
                    "body": {
                        "type": "BlockStatement",
                        "body": [
                        ],
                        "start": 0,
                        "end": 0,
                    },
                    "start": 0,
                    "end": 0,
                }
            ],
            "end": 0,
            "start": 0
        });
    }

    #[test]
    fn test_do_statement() {
        expect_parse!("do { true; } while (false)", {
            "type": "Program",
            "body": [
                {
                    "type": "DoWhileStatement",
                    "test": {
                        "type": "Literal",
                        "value": false,
                        "start": 20,
                        "end": 25,
                    },
                    "body": {
                        "type": "BlockStatement",
                        "body": [
                            {
                                "type": "ExpressionStatement",
                                "expression": {
                                    "type": "Literal",
                                    "value": true,
                                    "start": 5,
                                    "end": 9,
                                },
                                "start":5,
                                "end": 9,
                            },
                        ],
                        "start": 0,
                        "end": 0,
                    },
                    "start": 0,
                    "end": 0,
                }
            ],
            "end": 0,
            "start": 0
        });
    }

    #[test]
    fn test_for_statement() {
        expect_parse!("for (i;;) {}", {
            "type": "Program",
            "body": [
                {
                    "type": "ForStatement",
                    "init": {
                        "type": "ExpressionStatement",
                        "expression": {
                            "type": "Identifier",
                            "name": "i",
                            "start": 5,
                            "end": 6,
                        },
                        "start": 0,
                        "end": 0,
                    },
                    "test": null,
                    "update": null,
                    "body": {
                        "type": "BlockStatement",
                        "body": [],
                        "start": 0,
                        "end": 0,
                    },
                    "start": 0,
                    "end": 0,
                }
            ],
            "end": 0,
            "start": 0
        });

        expect_parse!("for (i; i < 10; i++) {}", {
            "type": "Program",
            "body": [
                {
                    "type": "ForStatement",
                    "init": {
                        "type": "ExpressionStatement",
                        "expression": {
                            "type": "Identifier",
                            "name": "i",
                            "start": 5,
                            "end": 6,
                        },
                        "start": 0,
                        "end": 0,
                    },
                    "test": {
                        "type": "BinaryExpression",
                        "end": 14,
                        "left": {
                            "end": 9,
                            "name": "i",
                            "start": 8,
                            "type": "Identifier"
                        },
                        "operator": "<",
                        "right": {
                            "end": 14,
                            "start": 12,
                            "type": "Literal",
                            "value": "10"
                        },
                        "start": 8,
                    },
                    "update": {
                        "type": "UpdateExpression",
                        "operator": "++",
                        "prefix": false,
                        "argument": {
                            "end": 17,
                            "name": "i",
                            "start": 16,
                            "type": "Identifier"
                        },
                        "start": 16,
                        "end": 17,
                    },
                    "body": {
                        "type": "BlockStatement",
                        "body": [],
                        "start": 0,
                        "end": 0,
                    },
                    "start": 0,
                    "end": 0,
                }
            ],
            "end": 0,
            "start": 0
        });

        expect_parse!("for (key in {}) {} ", {
            "type": "Program",
            "body": [
                {
                    "type": "ForInStatement",
                    "left": {
                        "type": "ExpressionStatement",
                        "expression": {
                            "type": "Identifier",
                            "name": "key",
                            "start": 5,
                            "end": 8,
                        },
                        "start": 0,
                        "end": 0,
                    },
                    "right": {
                        "type": "ObjectExpression",
                        "properties": [],
                        "start": 14,
                        "end": 15,
                    },
                    "body": {
                        "type": "BlockStatement",
                        "body": [],
                        "start": 0,
                        "end": 0,
                    },
                    "start": 0,
                    "end": 0,
                }
            ],
            "end": 0,
            "start": 0
        });

        expect_parse!("for (key of {}) {} ", {
            "type": "Program",
            "body": [
                {
                    "type": "ForOfStatement",
                    "left": {
                        "type": "ExpressionStatement",
                        "expression": {
                            "type": "Identifier",
                            "name": "key",
                            "start": 5,
                            "end": 8,
                        },
                        "start": 0,
                        "end": 0,
                    },
                    "right": {
                        "type": "ObjectExpression",
                        "properties": [],
                        "start": 14,
                        "end": 15,
                    },
                    "body": {
                        "type": "BlockStatement",
                        "body": [],
                        "start": 0,
                        "end": 0,
                    },
                    "start": 0,
                    "end": 0,
                }
            ],
            "end": 0,
            "start": 0
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
                        "start": 0,
                        "end": 0,
                    },
                    "handler": {
                        "type": "CatchClause",
                        "param": {
                            "type": "Identifier",
                            "name": "e",
                            "start": 15,
                            "end": 16,
                        },
                        "body": {
                            "type": "BlockStatement",
                            "body": [],
                            "start": 0,
                            "end": 0,
                        },
                        "start": 0,
                        "end": 0,
                    },
                    "start": 0,
                    "end": 0,
                }
            ],
            "end": 0,
            "start": 0
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
                                "value": "2",
                                "start": 1,
                                "end": 2
                            },
                            "start": 1,
                            "end": 2,
                        }
                    ],
                    "start": 0,
                    "end": 0,
                }
            ],
            "end": 0,
            "start": 0
        });
    }

    #[test]
    fn test_labeled_statement() {}

    #[test]
    fn test_function_statement() {}

    #[test]
    fn test_class_statement() {}

    #[test]
    fn test_continue_statement() {}

    #[test]
    fn test_switch_statement() {}

}
