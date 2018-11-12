use serde::ser::{Serialize, Serializer, SerializeStruct};
use ast::{Expression, Loc, OperatorKind};
use ast::expression::*;
use astgen::SerializeInLoc;

#[inline]
fn expression_type(operator: OperatorKind, prefix: bool) -> &'static str {
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
        self.in_loc(serializer, "SpreadElement", 1, |state| {
            state.serialize_field("argument", &self.argument)
        })
    }
}

impl<'ast> SerializeInLoc for MemberExpression<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "MemberExpression", 3, |state| {
                state.serialize_field("object", &self.object)?;
                state.serialize_field("property", &self.property)?;
                state.serialize_field("computed", &false)
        })
    }
}

impl<'ast> SerializeInLoc for ComputedMemberExpression<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "MemberExpression", 3, |state| {
                state.serialize_field("object", &self.object)?;
                state.serialize_field("property", &self.property)?;
                state.serialize_field("computed", &true)
        })
    }
}

impl<'ast> SerializeInLoc for MetaPropertyExpression<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "MetaProperty", 2, |state| {
                state.serialize_field("meta", &self.meta)?;
                state.serialize_field("property", &self.property)
        })
    }
}

impl<'ast> SerializeInLoc for CallExpression<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "CallExpression", 2, |state| {
            state.serialize_field("callee", &self.callee)?;
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
            state.serialize_field("test", &self.test)?;
            state.serialize_field("consequent", &self.consequent)?;
            state.serialize_field("alternate", &self.alternate)
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
            Expression(ref expression) => serializer.serialize_some(expression),
            Block(ref expression) => serializer.serialize_some(expression),
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
                Call(CallExpression { ref callee, ref arguments }) => {
                    self.in_loc(serializer, expr_type, 2, |state| {
                        state.serialize_field("callee", callee)?;
                        state.serialize_field("arguments", arguments)
                    })
                },
                Literal(_) => {
                    self.in_loc(serializer, expr_type, 2, |state| {
                        // 0 byte array, will be optimized away
                        let arguments: [(); 0] = [];
                        state.serialize_field("callee", &self.operand)?;
                        state.serialize_field("arguments", &arguments)
                    })
                },
                _ => unimplemented!()
            }
        } else {
            self.in_loc(serializer, expr_type, 3, |state| {
                state.serialize_field("operator", &self.operator)?;
                state.serialize_field("argument", &self.operand)?;
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
            state.serialize_field("argument", &self.operand)?;
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

            let mut elems = self.body.iter()
                                     .map(|loc| if loc.item != Expression::Void { Some(loc) } else { None } )
                                     .collect::<Vec<_>>();
            loop {
                let should_pop = match elems.last() {
                    Some(&None) => true,
                    Some(&Some(_)) => break,
                    None => break,
                };

                if should_pop {
                    elems.pop();
                }
            }

            state.serialize_field("elements", &elems)
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
            state.serialize_field("left", &self.left)?;
            state.serialize_field("right", &self.right)
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
            Void => unreachable!(),
            This(_) => self.in_loc(serializer, "ThisExpression", 0, |_| Ok(())),
            Identifier(ref ident) => {
                self.in_loc(serializer, "Identifier", 1, |state| {
                    state.serialize_field("name", ident)
                })
            },
            Literal(ref value)             => value.serialize(serializer),
            Array(ref value)               => value.serialize(serializer),
            Sequence(ref expression)       => expression.serialize(serializer),
            Binary(ref expression)         => expression.serialize(serializer),
            Prefix(ref expression)         => expression.serialize(serializer),
            Postfix(ref expression)        => expression.serialize(serializer),
            Object(ref expression)         => expression.serialize(serializer),
            Template(ref expression)       => expression.serialize(serializer),
            TaggedTemplate(ref expression) => expression.serialize(serializer),
            Spread(ref expression)         => expression.serialize(serializer),
            Member(ref expression)         => expression.serialize(serializer),
            ComputedMember(ref expression) => expression.serialize(serializer),
            MetaProperty(ref expression)   => expression.serialize(serializer),
            Call(ref expression)           => expression.serialize(serializer),
            Conditional(ref expression)    => expression.serialize(serializer),
            Arrow(ref expression)          => expression.serialize(serializer),
            Function(ref expression)       => expression.serialize(serializer),
            Class(ref expression)          => expression.serialize(serializer),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_void_expression() {
        expect_parse!("[1,]", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "ArrayExpression",
                        "elements": [
                            {
                                "type": "Literal",
                                "value": 1,
                                "raw": "1",
                                "start": 1,
                                "end": 2
                            },
                        ],
                        "start": 0,
                        "end": 4
                    },
                    "start": 0,
                    "end": 4
                }
            ],
            "start": 0,
            "end": 4
        });

        expect_parse!("[1,,]", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "ArrayExpression",
                        "elements": [
                            {
                                "type": "Literal",
                                "value": 1,
                                "raw": "1",
                                "start": 1,
                                "end": 2
                            },
                        ],
                        "start": 0,
                        "end": 5
                    },
                    "start": 0,
                    "end": 5
                }
            ],
            "start": 0,
            "end": 5
        });

        expect_parse!("[,1,]", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "ArrayExpression",
                        "elements": [
                            null,
                            {
                                "type": "Literal",
                                "value": 1,
                                "raw": "1",
                                "start": 2,
                                "end": 3
                            },
                        ],
                        "start": 0,
                        "end": 5
                    },
                    "start": 0,
                    "end": 5
                }
            ],
            "start": 0,
            "end": 5
        });
    }

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
              "end": 4,
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
    fn test_literal_expression() {
        expect_parse!("'foo';", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "Literal",
                        "value": "foo",
                        "raw": "'foo'",
                        "start": 0,
                        "end": 5
                    },
                    "start": 0,
                    "end": 5,
                }
              ],
              "start": 0,
              "end": 5,
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
                                "raw": "true",
                                "start": 1,
                                "end": 5
                            },
                            {
                                "type": "Literal",
                                "value": 0,
                                "raw": "0",
                                "start": 7,
                                "end": 8
                            },
                            {
                                "type": "Literal",
                                "value": "foo",
                                "raw": "'foo'",
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
              "end": 21,
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
                                "raw": "true",
                                "start": 0,
                                "end": 4
                            },
                            {
                                "type": "Literal",
                                "value": false,
                                "raw": "false",
                                "start": 6,
                                "end": 11
                            },
                        ],
                        "start": 0,
                        "end": 12
                    },
                    "start": 0,
                    "end": 12
                }
              ],
              "start": 0,
              "end": 12,
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
                            "start": 0,
                            "end": 1
                        },
                        "right": {
                            "type": "Literal",
                            "value": 0,
                            "raw": "0",
                            "start": 4,
                            "end": 5
                        },
                        "start": 0,
                        "end": 5
                    },
                    "start": 0,
                    "end": 5
                }
              ],
              "start": 0,
              "end": 5,
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
                        "end": 10
                    },
                    "start": 0,
                    "end": 10
                }
              ],
              "start": 0,
              "end": 10,
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
                                "value": 0,
                                "raw": "0",
                                "start": 8,
                                "end": 9
                            },
                            {
                                "type": "Literal",
                                "value": true,
                                "raw": "true",
                                "start": 11,
                                "end": 15
                            },
                        ],
                        "start": 0,
                        "end": 17
                    },
                    "start": 0,
                    "end": 17
                }
              ],
              "start": 0,
              "end": 17,
        });

        expect_parse!("new 'foo';", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "NewExpression",
                        "callee": {
                            "type": "Literal",
                            "value": "foo",
                            "raw": "\'foo\'",
                            "start": 4,
                            "end": 9
                        },
                        "arguments": [],
                        "start": 0,
                        "end": 10
                    },
                    "start": 0,
                    "end": 10
                }
              ],
              "start": 0,
              "end": 10,
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
                        "end": 3
                    },
                    "start": 0,
                    "end": 3
                }
              ],
              "start": 0,
              "end": 3,
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
                            "value": 0,
                            "raw": "0",
                            "start": 1,
                            "end": 2
                        },
                        "prefix": true,
                        "start": 0,
                        "end": 2
                    },
                    "start": 0,
                    "end": 2,
                }
              ],
              "start": 0,
              "end": 2,
        });
    }
    // FIXME
    #[test]
    fn test_postfix_expression () {
        expect_parse!("i++;", {
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
                            "start": 0,
                            "end": 1
                        },
                        "prefix": false,
                        "start": 0,
                        "end": 3
                    },
                    "start": 0,
                    "end": 3
                }
              ],
              "start": 0,
              "end": 3
        });
    }

    #[test]
    fn test_object_expression () {
        expect_parse!("const a = {}", {
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
                                "type": "ObjectExpression",
                                "properties": [],
                                "start": 10,
                                "end": 12
                            },
                            "start": 6,
                            "end": 12
                        }
                    ],
                    "start": 0,
                    "end": 12,
                }
            ],
            "start": 0,
            "end": 12,
        });

        expect_parse!("const a = { foo: 'bar' }", {
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
                                "end": 7,
                            },
                            "init": {
                                "type": "ObjectExpression",
                                "properties": [
                                    {
                                        "type": "Property",
                                        "key": {
                                            "type": "Identifier",
                                            "name": "foo",
                                            "start": 12,
                                            "end": 15
                                        },
                                        "method": false,
                                        "shorthand": false,
                                        "computed": false,
                                        "value": {
                                            "type": "Literal",
                                            "value": "bar",
                                            "raw": "'bar'",
                                            "start": 17,
                                            "end": 22,
                                        },
                                        "kind": "init",
                                        "start": 12,
                                        "end": 22,
                                    }
                                ],
                                "start": 10,
                                "end": 24,
                            },
                            "start": 6,
                            "end": 24,
                        }
                    ],
                    "start": 0,
                    "end": 24,
                }
            ],
            "start": 0,
            "end": 24
        });

        expect_parse!("const a = { [foo]: 'bar' }", {
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
                                "end": 7,
                            },
                            "init": {
                                "type": "ObjectExpression",
                                "properties": [
                                    {
                                        "type": "Property",
                                        "key": {
                                            "type": "Identifier",
                                            "name": "foo",
                                            "start": 13,
                                            "end": 16
                                        },
                                        "method": false,
                                        "shorthand": false,
                                        "computed": true,
                                        "value": {
                                            "type": "Literal",
                                            "value": "bar",
                                            "raw": "'bar'",
                                            "start": 19,
                                            "end": 24
                                        },
                                        "kind": "init",
                                        "start": 12,
                                        "end": 24,
                                    }
                                ],
                                "start": 10,
                                "end": 26,
                            },
                            "start": 6,
                            "end": 26,
                        }
                    ],
                    "start": 0,
                    "end": 26,
                }
            ],
            "start": 0,
            "end": 26
        });

        expect_parse!("const a = { get (a) {} }", {
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
                                "end": 7,
                            },
                            "init": {
                                "type": "ObjectExpression",
                                "properties": [
                                    {
                                        "type": "Property",
                                        "key": {
                                            "type": "Identifier",
                                            "name": "get",
                                            "start": 12,
                                            "end": 15
                                        },
                                        "method": true,
                                        "shorthand": false,
                                        "computed": false,
                                        "value": {
                                            "type": "FunctionExpression",
                                            "generator": false,
                                            "id": null,
                                            "params": [
                                                {
                                                    "type": "Identifier",
                                                    "name": "a",
                                                    "start": 17,
                                                    "end": 18
                                                }
                                            ],
                                            "body": {
                                                "type": "BlockStatement",
                                                "body": [],
                                                "start": 20,
                                                "end": 22
                                            },
                                            "start": 16,
                                            "end": 22
                                        },
                                        "kind": "init",
                                        "start": 12,
                                        "end": 22,
                                    }
                                ],
                                "start": 10,
                                "end": 24,
                            },
                            "start": 6,
                            "end": 24,
                        }
                    ],
                    "start": 0,
                    "end": 24,
                }
            ],
            "start": 0,
            "end": 24
        });
    }

    #[test]
    fn test_meta_property_expression() {
        expect_parse!("function Handler () { new.target; }", {
          "type": "Program",
          "body": [
            {
              "type": "FunctionDeclaration",
              "generator": false,
              "id": {
                "type": "Identifier",
                "name": "Handler",
                "start": 9,
                "end": 16,
              },
              "params": [],
              "body": {
                "type": "BlockStatement",
                "body": [
                  {
                    "type": "ExpressionStatement",
                    "expression": {
                      "type": "MetaProperty",
                      "meta": {
                        "type": "Identifier",
                        "name": "new",
                        "start": 22,
                        "end": 25,
                      },
                      "property": {
                        "type": "Identifier",
                        "name": "target",
                        "start": 25,
                        "end": 32,
                      },
                      "start": 22,
                      "end": 32,
                    },
                    "start": 22,
                    "end": 32,
                  }
                ],
                "start": 20,
                "end": 35,
              },
              "start": 0,
              "end": 35,
            }
          ],
          "start": 0,
          "end": 35,
        });
    }

    #[test]
    fn test_member_expression () {
        expect_parse!("foo.bar", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "MemberExpression",
                        "object": {
                            "type": "Identifier",
                            "name": "foo",
                            "start": 0,
                            "end": 3
                        },
                        "property": {
                            "type": "Identifier",
                            "name": "bar",
                            "start": 7,
                            "end": 7
                        },
                        "computed": false,
                        "start": 0,
                        "end": 7
                    },
                    "start": 0,
                    "end": 7
                }
            ],
            "start": 0,
            "end": 7
        });
    }

    #[test]
    fn test_computed_member_expression () {
        expect_parse!("foo[bar]", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "MemberExpression",
                        "object": {
                            "type": "Identifier",
                            "name": "foo",
                            "start": 0,
                            "end": 3
                        },
                        "property": {
                            "type": "Identifier",
                            "name": "bar",
                            "start": 4,
                            "end": 7
                        },
                        "computed": true,
                        "start": 0,
                        "end": 8
                    },
                    "start": 0,
                    "end": 8
                }
            ],
            "start": 0,
            "end": 8
        });
    }

    #[test]
    fn test_spread_expression () {
        expect_parse!("function foo (a, ...opts) {}", {
            "type": "Program",
            "body": [
                {
                    "type": "FunctionDeclaration",
                    "generator": false,
                    "id": {
                        "type": "Identifier",
                        "name": "foo",
                        "start": 9,
                        "end": 12,
                    },
                    "params": [
                        {
                            "type": "Identifier",
                            "name": "a",
                            "start": 14,
                            "end": 15,
                        },
                        {
                            "type": "RestElement",
                            "argument": {
                                "type": "Identifier",
                                "name": "opts",
                                "start": 20,
                                "end": 24,
                            },
                            "start": 17,
                            "end": 24,
                        }
                    ],
                    "body": {
                        "type": "BlockStatement",
                        "body": [],
                        "start": 26,
                        "end": 28,
                    },
                    "start": 0,
                    "end": 28,
                }
            ],
            "start": 0,
            "end": 28
        });

        expect_parse!("[head, ...iter, tail]", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "ArrayExpression",
                        "elements": [
                            {
                                "type": "Identifier",
                                "name": "head",
                                "start": 1,
                                "end": 5,
                            },
                            {
                                "type": "SpreadElement",
                                "argument": {
                                    "type": "Identifier",
                                    "name": "iter",
                                    "start": 10,
                                    "end": 14
                                },
                                "start": 7,
                                "end": 14,
                            },
                            {
                                "type": "Identifier",
                                "name": "tail",
                                "start": 16,
                                "end": 20
                            }
                        ],
                        "start": 0,
                        "end": 21,
                    },
                    "start": 0,
                    "end": 21,
                }
            ],
            "start": 0,
            "end": 21
        });
    }

    #[test]
    fn test_call_expression () {
        expect_parse!("foo()", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "CallExpression",
                        "callee": {
                            "type": "Identifier",
                            "name": "foo",
                            "start": 0,
                            "end": 3
                        },
                        "arguments": [],
                        "start": 3,
                        "end": 5
                    },
                    "start": 0,
                    "end": 5
                }
              ],
              "start": 0,
              "end": 5,
        });
    }

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
                            "start": 0,
                            "end": 1
                        },
                        "consequent": {
                            "type": "Literal",
                            "value": true,
                            "raw": "true",
                            "start": 4,
                            "end": 8
                        },
                        "alternate": {
                            "type": "Literal",
                            "value": false,
                            "raw": "false",
                            "start": 11,
                            "end": 16
                        },
                        "start": 0,
                        "end": 16
                    },
                    "start": 0,
                    "end": 16
                }
              ],
              "start": 0,
              "end": 16,
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
                        "start": 1,
                        "end": 8
                    },
                    "start": 1,
                    "end": 8
                }
              ],
              "start": 1,
              "end": 8,
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
                        "start": 1,
                        "end": 9
                    },
                    "start": 1,
                    "end": 9
                }
              ],
              "start": 1,
              "end": 9,
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
                        "end": 8
                    },
                    "start": 0,
                    "end": 8
                }
              ],
              "start": 0,
              "end": 8,
        });
        expect_parse!("n => n", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "ArrowFunctionExpression",
                        "params": [
                            {
                                "type": "Identifier",
                                "name": "n",
                                "start": 0,
                                "end": 1
                            }
                        ],
                        "body": {
                            "type": "Identifier",
                            "name": "n",
                            "start": 5,
                            "end": 6
                        },
                        "start": 0,
                        "end": 6
                    },
                    "start": 0,
                    "end": 6
                }
              ],
              "start": 0,
              "end": 6,
        });
    }

    #[test]
    fn test_function_expression () {
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
                        "end": 18
                    },
                    "start": 0,
                    "end": 18
                }
            ],
            "start": 0,
            "end": 18
        });

        expect_parse!("function foo (a, b = 2) {}", {
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
                                "name": "b",
                                "start": 17,
                                "end": 18
                            },
                            "right": {
                                "type": "Literal",
                                "value": 2,
                                "raw": "2",
                                "start": 21,
                                "end": 22
                            },
                            "start": 17,
                            "end": 22
                        }
                    ],
                    "body": {
                        "type": "BlockStatement",
                        "body": [],
                        "start": 24,
                        "end": 26
                    },
                    "start": 0,
                    "end": 26
                }
            ],
            "start": 0,
            "end": 26
        });
    }

    #[test]
    fn test_class_expression() {
        expect_parse!("class Foo {}", {
            "type": "Program",
            "body": [
                {
                    "type": "ClassDeclaration",
                    "id": {
                        "type": "Identifier",
                        "name": "Foo",
                        "start": 6,
                        "end": 9,
                    },
                    "superClass": null,
                    "body": {
                        "type": "ClassBody",
                        "body": [],
                        "start": 10,
                        "end": 12
                    },
                    "start": 0,
                    "end": 12,
                }
            ],
            "start": 0,
            "end": 12
        });

        expect_parse!("class Foo extends Bar {}", {
            "type": "Program",
            "body": [
                {
                    "type": "ClassDeclaration",
                    "id": {
                        "type": "Identifier",
                        "name": "Foo",
                        "start": 6,
                        "end": 9,
                    },
                    "superClass": {
                        "type": "Identifier",
                        "name": "Bar",
                        "start": 18,
                        "end": 21,
                    },
                    "body": {
                        "type": "ClassBody",
                        "body": [],
                        "start": 22,
                        "end": 24
                    },
                    "start": 0,
                    "end": 24,
                }
            ],
            "start": 0,
            "end": 24
        });

        expect_parse!(r"class Foo { bar() {} }", {
            "type": "Program",
            "body": [
                {
                    "type": "ClassDeclaration",
                    "id": {
                        "type": "Identifier",
                        "name": "Foo",
                        "start": 6,
                        "end": 9,
                    },
                    "superClass": null,
                    "body": {
                        "type": "ClassBody",
                        "body": [
                        {
                            "type": "MethodDefinition",
                            "kind": "method",
                            "static": false,
                            "computed": false,
                            "key": {
                                "type": "Identifier",
                                "name": "bar",
                                "start": 12,
                                "end": 15,
                            },
                            "value": {
                                "type": "FunctionExpression",
                                "generator": false,
                                "id": null,
                                "params": [],
                                "body": {
                                    "type": "BlockStatement",
                                    "body": [],
                                    "start": 18,
                                    "end": 20
                                },
                                "start": 15,
                                "end": 20,
                            },
                            "start": 12,
                            "end": 20,
                        }
                        ],
                        "start": 10,
                        "end": 22
                    },
                    "start": 0,
                    "end": 22,
                }
            ],
            "start": 0,
            "end": 22
        });

        expect_parse!("class Foo { static bar() {} }", {
            "type": "Program",
            "body": [
                {
                    "type": "ClassDeclaration",
                    "id": {
                        "type": "Identifier",
                        "name": "Foo",
                        "start": 6,
                        "end": 9,
                    },
                    "superClass": null,
                    "body": {
                        "type": "ClassBody",
                        "body": [
                        {
                            "type": "MethodDefinition",
                            "kind": "method",
                            "static": true,
                            "computed": false,
                            "key": {
                                "type": "Identifier",
                                "name": "bar",
                                "start": 19,
                                "end": 22,
                            },
                            "value": {
                                "type": "FunctionExpression",
                                "generator": false,
                                "id": null,
                                "params": [],
                                "body": {
                                    "type": "BlockStatement",
                                    "body": [],
                                    "start": 25,
                                    "end": 27
                                },
                                "start": 22,
                                "end": 27,
                            },
                            "start": 12,
                            "end": 27,
                        }
                        ],
                        "start": 10,
                        "end": 29
                    },
                    "start": 0,
                    "end": 29,
                }
            ],
            "start": 0,
            "end": 29
        });
    }
}
