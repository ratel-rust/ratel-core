use ast;
use ast::{Expression, ExpressionPtr, OperatorKind};
use serde_json;
use serializer::Serializable;

#[inline]
fn expression_type<'ast>(operator: OperatorKind, prefix: bool) -> &'ast str {
    use self::OperatorKind::*;

    match operator {
        Assign              |
        AddAssign           |
        SubstractAssign     |
        ExponentAssign      |
        MultiplyAssign      |
        DivideAssign        |
        RemainderAssign     |
        BSLAssign           |
        BSRAssign           |
        UBSRAssign          |
        BitOrAssign         |
        BitXorAssign        |
        BitAndAssign        => "AssignmentExpression",

        LogicalAnd          |
        LogicalOr           => "LogicalExpression",

        Increment           |
        Decrement           => "UpdateExpression",

        Typeof              |
        Delete              => "UnaryExpression",

        OperatorKind::Void  => "UnaryExpression",

        Substraction        |
        Addition            |
        LogicalNot          |
        BitwiseNot          => if prefix { "UnaryExpression" } else { "BinaryExpression" },

        _                   => "BinaryExpression"
    }
}

impl<'ast> Serializable<'ast> for ExpressionPtr<'ast> {
    #[inline]
    fn serialize(&self) -> Option<serde_json::Value> {
        use self::Expression::*;

        let result = match self.item {
            Error { .. } => panic!("Module contains errors"),
            Void => return None,
            This => {
                json!({
                    "type": "ThisExpression",
                    "start": self.start,
                    "end": self.end,
                })
            },
            Identifier(ident) => {
                json!({
                    "type": "Identifier",
                    "name": ident,
                    "start": self.start,
                    "end": self.end,
                })
            },
            Value(value) => {
                use self::ast::Value::*;

                let result = match value {
                    Undefined => {
                        json!({
                            "type": "Literal",
                            "value": "undefined",
                            "start": self.start,
                            "end": self.end,
                        })
                    },
                    Null => {
                        json!({
                            "type": "Literal",
                            "value": "null",
                            "start": self.start,
                            "end": self.end,
                        })
                    },
                    True => {
                        json!({
                            "type": "Literal",
                            "value": "true",
                            "start": self.start,
                            "end": self.end,
                        })
                    },
                    False => {
                        json!({
                            "type": "Literal",
                            "value": "false",
                            "start": self.start,
                            "end": self.end,
                        })
                    },
                    Number(number) => {
                        json!({
                            "type": "Literal",
                            "value": number,
                            "start": self.start,
                            "end": self.end,
                        })
                    },
                    Binary(number) => {
                        json!({
                            "type": "Literal",
                            "value": number,
                            "start": self.start,
                            "end": self.end,
                        })
                    },
                    String(value) => {
                        json!({
                            "type": "Literal",
                            "value": value,
                            "start": self.start,
                            "end": self.end,
                        })
                    },
                    RawQuasi(number) => {
                        panic!("Unimplemented: Value::RawQuasi");
                    },
                    RegEx(value) => {
                        let mut end = value.len() - 1;

                        for index in (0..value.len()).rev() {
                            if "/" == &value[index..(index+1)] {
                                end = index;
                                break;
                            }
                        }

                        json!({
                            "type": "Literal",
                            "regex": {
                                "pattern": &value[1..end],
                                "flags": &value[(end+1)..value.len()]
                            },
                            "start": self.start,
                            "end": self.end,
                        })
                    }
                };

                result
            },
            Sequence { body } => {
                json!({
                    "type": "SequenceExpression",
                    "expressions": body.serialize(),
                    "start": self.start,
                    "end": self.end,
                })
            },
            Array { body } => {
                json!({
                    "type": "ArrayExpression",
                    "elements": body.serialize(),
                    "start": self.start,
                    "end": self.end,
                })
            },
            Member { object, property } => {
                json!({
                    "type": "MemberExpression",
                    "object": object.serialize(),
                    "property": property.serialize(),
                    "computed": false,
                    "start": self.start,
                    "end": self.end,
                })
            },
            ComputedMember { object, property } => {
                json!({
                    "type": "MemberExpression",
                    "object": object.serialize(),
                    "property": property.serialize(),
                    "computed": true,
                    "start": self.start,
                    "end": self.end,
                })
            },
            Call { callee, arguments } => {
                json!({
                    "type": "CallExpression",
                    "callee": callee.serialize(),
                    "arguments": arguments.serialize(),
                    "start": self.start,
                    "end": self.end,
                })
            },
            Binary { operator, left, right } => {
                json!({
                    "type": expression_type(operator, false),
                    "operator": operator.as_str(),
                    "left": left.serialize(),
                    "right": right.serialize(),
                    "start": self.start,
                    "end": self.end,
                })
            },
            Prefix { operator, operand } => {
                if let OperatorKind::New = operator {
                    match operand.item {
                        Expression::Call { callee, arguments } => {
                            json!({
                                "type": "NewExpression",
                                "callee": callee.serialize(),
                                "argument": arguments.serialize(),
                                "start": self.start,
                                "end": self.end,
                            })
                        },
                        Value(value) => {
                            json!({
                                "type": "NewExpression",
                                "callee": operand.serialize(),
                                "argument": [],
                                "start": self.start,
                                "end": self.end,
                            })
                        },
                        _ => {
                            panic!("Unexpected token");
                        }
                    }
                } else {
                    json!({
                        "type": expression_type(operator, true),
                        "prefix": true,
                        "operator": operator.as_str(),
                        "argument": operand.serialize(),
                        "start": self.start,
                        "end": self.end,
                    })
                }
            },
            Postfix { operator, operand }=> {
                json!({
                    "type": expression_type(operator, false),
                    "prefix": false,
                    "operator": operator.as_str(),
                    "argument": operand.serialize(),
                    "start": self.start,
                    "end": self.end,
                })
            },
            Conditional { test, consequent, alternate } => {
                json!({
                    "type": "ConditionalExpression",
                    "test": test.serialize(),
                    "alternate": alternate.serialize(),
                    "consequent": consequent.serialize(),
                })
            },
            Template { tag, expressions, quasis } => {
                let iter = quasis.ptr_iter();
                let mut mapped = iter.map(|q| {
                    json!({
                        "type": "TemplateElement",
                        "start": q.start,
                        "end": q.end,
                        "value": { // FIXME
                            "raw": q.item,
                            "cooked": q.item
                        },
                        "tail": false
                    })
                }).collect::<Vec<_>>();

                // FIXME: Sets `tail` to `true` on the last TemplateElement.
                let mut last = mapped.pop().unwrap();
                *last.get_mut("tail").unwrap() = json!(true);
                mapped.push(last);

                let result = json!({
                    "type": "TemplateLiteral",
                    "expressions": expressions.serialize(),
                    "quasis": mapped,
                    "start": self.start,
                    "end": self.end,
                });

                match tag {
                    Some(tag) => {
                        json!({
                            "type": "TaggedTemplateExpression",
                            "tag": tag.serialize(),
                            "expression": result
                        })
                    },
                    _ => result
                }
            },
            Arrow { params, body } => {
                json!({
                    "type": "ArrowFunctionExpression",
                    "id": null,
                    "params": params.serialize(),
                    "body": {
                        // FIXME
                        "type": "BlockStatement",
                        "body": body.serialize(),
                        "start": 0,
                        "end": 0,
                    },
                    "start": self.start,
                    "end": self.end,
                })
            },
            Object { body } => {
                json!({
                    "type": "ObjectExpression",
                    "properties": body.serialize(),
                    "start": self.start,
                    "end": self.end,
                })
            },
            Function { function } => {
                json!({
                    "type": "FunctionExpression",
                    "id": function.name.serialize(),
                    "params": function.params.serialize(),

                    "body": {
                        // FIXME
                        "type": "BlockStatement",
                        "body": function.body.serialize(),
                        "start": 0,
                        "end": 0,
                    },
                    "start": self.start,
                    "end": self.end,
                })
            },
            Class { class } => {
                json!({
                    "type": "ClassExpression",
                    "id": class.name.serialize(),
                    "superClass": class.extends.serialize(),
                    "body": {
                        "type": "ClassBody",
                        "body": class.body.serialize(),
                        // FIXME
                        "start": 0,
                        "end": 0,
                    },
                    "start": self.start,
                    "end": self.end,
                })
            },
        };
        Some(result)
    }
}
