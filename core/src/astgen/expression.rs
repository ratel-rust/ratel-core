use serde::ser::{Serialize, Serializer, SerializeStruct};
use ast::{Block, Expression, Loc, OperatorKind};
use ast::expression::*;
use astgen::SerializeInLoc;

#[inline]
fn expression_type<'ast>(operator: OperatorKind, prefix: bool) -> &'static str {
    use self::OperatorKind::*;

    match operator {
        Assign              |
        AddAssign           |
        ExponentAssign      |
        MultiplyAssign      |
        DivideAssign        |
        RemainderAssign     |
        BSLAssign           |
        BSRAssign           |
        UBSRAssign          |
        BitOrAssign         |
        BitXorAssign        |
        SubtractAssign      |
        BitAndAssign        => "AssignmentExpression",
        LogicalAnd          |
        LogicalOr           => "LogicalExpression",
        Increment           |
        Decrement           => "UpdateExpression",
        Typeof              |
        Void                |
        Delete              => "UnaryExpression",
        Subtraction         |
        Addition            |
        LogicalNot          |
        BitwiseNot          => if prefix { "UnaryExpression" } else { "BinaryExpression" },
        New                 => "NewExpression",
        _                   => "BinaryExpression"
    }
}

impl<'ast> SerializeInLoc for SpreadExpression<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        unimplemented!()
    }
}

impl<'ast> SerializeInLoc for MemberExpression<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        unimplemented!()
    }
}

impl<'ast> SerializeInLoc for ComputedMemberExpression<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        unimplemented!()
    }
}

impl<'ast> SerializeInLoc for CallExpression<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "CallExpression", 2, |state| {
            state.serialize_field("callee", &*self.callee)?;
            state.serialize_field("arguments", &self.arguments)
        })
    }
}

impl<'ast> SerializeInLoc for ConditionalExpression<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "ConditionalExpression", 3, |state| {
            state.serialize_field("test", &*self.test)?;
            state.serialize_field("consequent", &*self.consequent)?;
            state.serialize_field("alternate", &*self.alternate)
        })
    }
}

impl<'ast> Serialize for Loc<ArrowBody<'ast>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use self::ArrowBody::*;
        match self.item {
            Expression(expression) => serializer.serialize_some(&*expression),
            Block(expression) => serializer.serialize_some(&*expression),
        }
    }
}

impl<'ast> SerializeInLoc for ArrowExpression<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "ArrowFunctionExpression", 2, |state| {
            state.serialize_field("params", &self.params)?;
            state.serialize_field("body", &Loc::new(0, 0, self.body))
        })
    }
}

impl<'ast> SerializeInLoc for SequenceExpression<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "SequenceExpression", 1, |state| {
            state.serialize_field("expressions", &self.body)
        })
    }
}

impl<'ast> SerializeInLoc for PrefixExpression<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        let prefix = true;
        let expr_type = expression_type(self.operator, prefix);

        if let OperatorKind::New = self.operator {
            use self::Expression::*;
            match self.operand.item {
                Call(CallExpression { callee, arguments }) => {
                    self.in_loc(serializer, expr_type, 2, |state| {
                        state.serialize_field("callee", &*callee)?;
                        state.serialize_field("arguments", &arguments)
                    })
                },
                Literal(_) => {
                    self.in_loc(serializer, expr_type, 2, |state| {
                        let arguments: Vec<Loc<Expression>> = vec![];
                        state.serialize_field("callee", &*self.operand)?;
                        state.serialize_field("arguments", &arguments)
                    })
                },
                _ => unimplemented!()
            }
        } else {
            self.in_loc(serializer, expr_type, 3, |state| {
                state.serialize_field("operator", &self.operator)?;
                state.serialize_field("argument", &*self.operand)?;
                state.serialize_field("prefix", &prefix)
            })
        }
    }
}

impl<'ast> SerializeInLoc for PostfixExpression<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        let prefix = false;
        let expr_type = expression_type(self.operator, prefix);
        self.in_loc(serializer, expr_type, 3, |state| {
            state.serialize_field("operator", &self.operator)?;
            state.serialize_field("argument", &*self.operand)?;
            state.serialize_field("prefix", &prefix)
        })
    }
}

impl<'ast> SerializeInLoc for ObjectExpression<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "ObjectExpression", 1, |state| {
            state.serialize_field("properties", &self.body)
        })
    }
}

impl<'ast> Serialize for OperatorKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_some(&self.as_str())
    }
}

impl<'ast> SerializeInLoc for ArrayExpression<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "ArrayExpression", 1, |state| {
            state.serialize_field("elements", &self.body)
        })
    }
}

impl<'ast> SerializeInLoc for BinaryExpression<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "BinaryExpression", 3, |state| {
            state.serialize_field("operator", &self.operator)?;
            state.serialize_field("left", &*self.left)?;
            state.serialize_field("right", &*self.right)
        })
    }
}

impl<'ast> SerializeInLoc for Expression<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        use self::Expression::*;

        match *self {

            Error { .. } => panic!("Module contains errors"),
            Void => unimplemented!(),
            This(_) => self.in_loc(serializer, "ThisExpression", 0, |_| Ok(())),
            Identifier(ident) => {
                self.in_loc(serializer, "Identifier", 1, |state| {
                    state.serialize_field("name", &ident)
                })
            },
            Literal(value) => value.serialize(serializer),
            Array(value) => value.serialize(serializer),
            Sequence(expression) => expression.serialize(serializer),
            Binary(expression) => expression.serialize(serializer),
            Prefix(expression) => expression.serialize(serializer),
            Postfix(expression) => expression.serialize(serializer),
            Object(expression) => expression.serialize(serializer),
            Template(expression) => expression.serialize(serializer),
            Spread(expression) => expression.serialize(serializer),
            Member(expression) => expression.serialize(serializer),
            ComputedMember(expression) => expression.serialize(serializer),
            Call(expression) => expression.serialize(serializer),
            Conditional(expression) => expression.serialize(serializer),
            Arrow(expression) => expression.serialize(serializer),
            Function(expression) => expression.serialize(serializer),
            Class(expression) => expression.serialize(serializer),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use parser::{parse};
    use astgen::generate_ast;

    #[test]
    fn test_spread_expression() {}

    #[test]
    fn test_this_expression() {
        expect_parse!("this;", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "ThisExpression",
                        "start": 0,
                        "end": 4
                    },
                    "start": 0,
                    "end": 4,
                }
              ],
              "start": 0,
              "end": 0,
        });
    }

    #[test]
    fn test_identifier_expression() {
        expect_parse!("foo;", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "Identifier",
                        "name": "foo",
                        // FIXME
                        "start": 3,
                        "end": 4
                    },
                    "start": 0,
                    "end": 4,
                }
              ],
              "start": 0,
              "end": 0,
        });
    }

    #[test]
    fn test_literal_expression() {
        expect_parse!("'foo';", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "StringLiteral",
                        "value": "'foo'",
                        "start": 0,
                        "end": 5
                    },
                    "start": 0,
                    "end": 5,
                }
              ],
              "start": 0,
              "end": 0,
        });
    }

    #[test]
    fn test_array_expression() {
        expect_parse!("[true, 0, 'foo', bar];", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "ArrayExpression",
                        "elements": [
                            {
                                "type": "Literal",
                                "value": true,
                                "start": 1,
                                "end": 5
                            },
                            {
                                "type": "Literal",
                                // FIXME
                                "value": "0",
                                "start": 7,
                                "end": 8
                            },
                            {
                                // FIXME
                                "type": "StringLiteral",
                                "value": "'foo'",
                                "start": 10,
                                "end": 15
                            },
                            {
                                "type": "Identifier",
                                "name": "bar",
                                "start": 17,
                                "end": 20
                            },
                        ],
                        "start": 0,
                        "end": 21
                    },
                    "start": 0,
                    "end": 21,
                }
              ],
              "start": 0,
              "end": 0,
        });
    }

    #[test]
    fn test_sequence_expression() {
        expect_parse!("true, false;", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "SequenceExpression",
                        "expressions": [
                            {
                                "type": "Literal",
                                "value": true,
                                "start": 0,
                                "end": 4
                            },
                            {
                                "type": "Literal",
                                "value": false,
                                "start": 6,
                                "end": 11
                            },
                        ],
                        "start": 0,
                        // FIXME
                        "end": 0
                        // "end": 11
                    },
                    "start": 0,
                    // FIXME
                    "end": 0
                    // "end": 11,
                }
              ],
              "start": 0,
              "end": 0,
        });
    }

    #[test]
    fn test_binary_expression() {
        expect_parse!("a > 0;", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "BinaryExpression",
                        "operator": ">",
                        "left": {
                            "type": "Identifier",
                            "name": "a",
                            // FIXME
                            "start": 2,
                            "end": 3
                        },
                        "right": {
                            "type": "Literal",
                            "value": "0",
                            // FIXME
                            "start": 4,
                            "end": 5
                        },
                        "start": 2,
                        // FIXME
                        "end": 5
                        // "end": 11
                    },
                    "start": 0,
                    // FIXME
                    "end": 5
                    // "end": 11,
                }
              ],
              "start": 0,
              "end": 0,
        });

        expect_parse!("new Foo();", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "NewExpression",
                        "callee": {
                            "type": "Identifier",
                            "name": "Foo",
                            "start": 4,
                            "end": 7
                        },
                        "arguments": [],
                        "start": 0,
                        "end": 0
                    },
                    "start": 0,
                    "end": 0
                }
              ],
              "start": 0,
              "end": 0,
        });

        expect_parse!("new Foo(0, true);", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "NewExpression",
                        "callee": {
                            "type": "Identifier",
                            "name": "Foo",
                            "start": 4,
                            "end": 7
                        },
                        "arguments": [
                            {
                                "type": "Literal",
                                "value": "0",
                                "start": 8,
                                "end": 9
                            },
                            {
                                "type": "Literal",
                                "value": true,
                                "start": 11,
                                "end": 15
                            },
                        ],
                        "start": 0,
                        "end": 0
                    },
                    "start": 0,
                    "end": 0
                }
              ],
              "start": 0,
              "end": 0,
        });

        expect_parse!("new 'foo';", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "NewExpression",
                        "callee": {
                            "type": "StringLiteral",
                            "value": "'foo'",
                            "start": 4,
                            "end": 9
                        },
                        "arguments": [],
                        "start": 0,
                        "end": 0
                    },
                    "start": 0,
                    "end": 0
                }
              ],
              "start": 0,
              "end": 0,
        });
    }

    #[test]
    fn test_prefix_expression () {
        expect_parse!("++i", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "UpdateExpression",
                        "operator": "++",
                        "argument": {
                            "type": "Identifier",
                            "name": "i",
                            "start": 2,
                            "end": 3
                        },
                        "prefix": true,
                        "start": 0,
                        "end": 0
                    },
                    "start": 0,
                    "end": 0
                }
              ],
              "start": 0,
              "end": 0,
        });

        expect_parse!("+0", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "UnaryExpression",
                        "operator": "+",
                        "argument": {
                            "type": "Literal",
                            "value": "0",
                            "start": 1,
                            "end": 2
                        },
                        "prefix": true,
                        "start": 0,
                        "end": 0
                    },
                    "start": 0,
                    "end": 0
                }
              ],
              "start": 0,
              "end": 0,
        });
    }

    #[test]
    fn test_postfix_expression () {}

    #[test]
    fn test_condititional_expression () {
        expect_parse!("a ? true : false", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "ConditionalExpression",
                        "test": {
                            "type": "Identifier",
                            "name": "a",
                            // FIXME
                            "start": 2,
                            "end": 3
                        },
                        "consequent": {
                            "type": "Literal",
                            "value": true,
                            "start": 4,
                            "end": 8
                        },
                        "alternate": {
                            "type": "Literal",
                            "value": false,
                            "start": 11,
                            "end": 16
                        },
                        "start": 0,
                        "end": 0
                    },
                    "start": 0,
                    "end": 0
                }
              ],
              "start": 0,
              "end": 0,
        });
    }

    #[test]
    fn test_arrow_function_expression () {
        expect_parse!("(b) => b", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "ArrowFunctionExpression",
                        "params": [
                            {
                                "type": "Identifier",
                                "name": "b",
                                "start": 1,
                                "end": 2
                            }
                        ],
                        "body": {
                            "type": "Identifier",
                            "name": "b",
                            "start": 7,
                            "end": 8
                        },
                        "start": 0,
                        "end": 0
                    },
                    "start": 0,
                    "end": 0
                }
              ],
              "start": 0,
              "end": 0,
        });
        expect_parse!("(b) => {}", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "ArrowFunctionExpression",
                        "params": [
                            {
                                "type": "Identifier",
                                "name": "b",
                                "start": 1,
                                "end": 2
                            }
                        ],
                        "body": {
                            "type": "BlockStatement",
                            "body": [],
                            "start": 7,
                            "end": 9
                        },
                        "start": 0,
                        "end": 0
                    },
                    "start": 0,
                    "end": 0
                }
              ],
              "start": 0,
              "end": 0,
        });
        expect_parse!("() => {}", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "ArrowFunctionExpression",
                        "params": [],
                        "body": {
                            "type": "BlockStatement",
                            "body": [],
                            "start": 6,
                            "end": 8
                        },
                        "start": 0,
                        "end": 0
                    },
                    "start": 0,
                    "end": 0
                }
              ],
              "start": 0,
              "end": 0,
        });
    }
}
