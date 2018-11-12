use ast::expression::{PropertyKey, TaggedTemplateExpression, TemplateLiteral};
use ast::{Expression, Literal, Loc, NodeList, Pattern, Property};
use astgen::SerializeInLoc;
use serde::ser::{Serialize, SerializeSeq, SerializeStruct, Serializer};

#[derive(Debug, Serialize, PartialEq)]
pub struct RegExLiteral<'ast> {
    pub pattern: &'ast str,
    pub flags: &'ast str,
}

#[inline]
pub fn parse_regex(value: &str) -> RegExLiteral {
    let mut end = value.len() - 1;
    for index in (0..value.len()).rev() {
        if "/" == &value[index..=index] {
            end = index;
            break;
        }
    }

    RegExLiteral {
        pattern: &value[1..end],
        flags: &value[(end + 1)..value.len()],
    }
}

#[inline]
pub fn is_float(value: &str) -> bool {
    value.bytes().any(|c| c == b'.')
}

#[derive(Debug)]
pub struct TemplateElement<'ast> {
    pub tail: bool,
    pub value: &'ast str,
}

#[derive(Debug, Serialize)]
pub struct TemplateElementValue<'ast> {
    pub raw: &'ast str,
    pub cooked: &'ast str,
}

#[derive(Debug)]
pub struct TemplateQuasis<'ast>(NodeList<'ast, &'ast str>);

impl<'ast> Serialize for TemplateQuasis<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None)?;
        let mut quasis = (self.0).iter().peekable();

        while let Some(q) = quasis.next() {
            seq.serialize_element(&Loc::new(
                q.start + 1,
                q.end - 1,
                TemplateElement {
                    tail: quasis.peek().is_none(),
                    value: q.item,
                },
            ))?;
        }

        seq.end()
    }
}

impl<'ast> SerializeInLoc for &'ast str {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "Identifier", 1, |state| {
            state.serialize_field("name", *self)
        })
    }
}

impl<'ast> SerializeInLoc for TemplateLiteral<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "TemplateLiteral", 2, |state| {
            state.serialize_field("quasis", &TemplateQuasis(self.quasis))?;
            state.serialize_field("expressions", &self.expressions)
        })
    }
}

impl<'ast> SerializeInLoc for TaggedTemplateExpression<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "TaggedTemplateExpression", 2, |state| {
            state.serialize_field("tag", &self.tag)?;
            state.serialize_field("quasi", &self.quasi)
        })
    }
}

impl<'ast> SerializeInLoc for Property<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        use self::Property::*;
        match *self {
            Shorthand(value) => {
                Expression::Identifier(value).serialize(serializer)
            }
            Literal { key, value } => {
                let computed = if let PropertyKey::Computed(_) = key.item {
                    true
                } else {
                    false
                };

                self.in_loc(serializer, "Property", 6, |state| {
                    state.serialize_field("key", &*key)?;
                    state.serialize_field("method", &false)?;
                    state.serialize_field("shorthand", &false)?;
                    state.serialize_field("computed", &computed)?;
                    state.serialize_field("value", &value)?;
                    state.serialize_field("kind", &"init")
                })
            }
            Method { key, value } => self.in_loc(serializer, "Property", 6, |state| {
                state.serialize_field("key", &*key)?;
                state.serialize_field("method", &true)?;
                state.serialize_field("shorthand", &false)?;
                state.serialize_field("computed", &false)?;
                state.serialize_field("value", &value)?;
                state.serialize_field("kind", &"init")
            }),
            Spread { argument } => self.in_loc(serializer, "SpreadProperty", 1, |state| {
                state.serialize_field("argument", &argument)
            })
        }
    }
}

impl<'ast> Serialize for Loc<PropertyKey<'ast>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use self::PropertyKey::*;

        match self.item {
            Computed(expr) => serializer.serialize_some(&*expr),
            Literal(value) => serializer.serialize_some(&Loc::new(
                self.start,
                self.end,
                Expression::Identifier(value),
            )),
            Binary(value) => serializer.serialize_some(&Loc::new(
                self.start,
                self.end,
                Expression::Identifier(value),
            )),
        }
    }
}

impl<'ast> SerializeInLoc for Literal<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        use self::Literal::*;

        // FIXME: StringLiteral vs. Literal
        let literal_type = match *self {
            String(_) => "Literal",
            _ => "Literal",
        };

        self.in_loc(serializer, literal_type, 1, |state| {
            match *self {
                Undefined => {
                    state.serialize_field("raw", &"undefined")
                },
                Null => {
                    state.serialize_field("value", &())?;
                    state.serialize_field("raw", &"null")
                },
                True => {
                    state.serialize_field("value", &true)?;
                    state.serialize_field("raw", &"true")
                },
                False => {
                    state.serialize_field("value", &false)?;
                    state.serialize_field("raw", &"false")
                },
                Number(number) => {
                    if number.len() > 2 {
                        let prefix = &number[0..2];
                        let is_hexdecimal = prefix == "0x" || prefix == "0X";
                        let is_octal = prefix == "0o" || prefix == "0O";
                        if is_hexdecimal || is_octal {
                            let value = unsafe { number.get_unchecked(2..number.len()) };
                            let radix = if is_hexdecimal { 16 } else { 8 };
                            state.serialize_field(
                                "value",
                                &i32::from_str_radix(value, radix).expect("Invalid number"),
                            )?;
                            return state.serialize_field("raw", &number)
                        }
                    }

                    if is_float(number) {
                        let value: f64 = number.parse().expect("Invalid number");
                        state.serialize_field("value", &value)?;
                        return state.serialize_field("raw", &number)
                    }

                    let value: i64 = number.parse().expect("Invalid number");
                    state.serialize_field("value", &value)?;
                    state.serialize_field("raw", &number)
                }
                Binary(number) => {
                    let value = unsafe { number.get_unchecked(2..number.len()) };
                    state.serialize_field(
                        "value",
                        &i32::from_str_radix(value, 2).expect("Invalid number"),
                    )?;
                    state.serialize_field("raw", &number)
                }
                String(value) => {
                    let parsed_value = unsafe { value.get_unchecked(1..value.len() - 1) };
                    state.serialize_field("value", &parsed_value)?;
                    state.serialize_field("raw", &value)
                },
                RegEx(value) => {
                    state.serialize_field("regex", &parse_regex(value))?;
                    state.serialize_field("raw", &value)
                },
            }
        })
    }
}

impl<'ast> SerializeInLoc for Pattern<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        use self::Pattern::*;

        match *self {
            Void => unreachable!(),
            Identifier(a) => Expression::Identifier(a).serialize(serializer),
            ObjectPattern { properties } => self.in_loc(serializer, "ObjectPattern", 1, |state| {
                state.serialize_field("keys", &properties)
            }),
            ArrayPattern { elements } => self.in_loc(serializer, "ArrayPattern", 1, |state| {
                state.serialize_field("elements", &elements)
            }),
            AssignmentPattern { left, right } => {
                self.in_loc(serializer, "AssignmentPattern", 2, |state| {
                    state.serialize_field("left", &*left)?;
                    state.serialize_field("right", &*right)
                })
            }
            RestElement { argument } => self.in_loc(serializer, "RestElement", 1, |state| {
                state.serialize_field("argument", &argument)
            }),
        }
    }
}

impl<'ast> SerializeInLoc for TemplateElement<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "TemplateElement", 2, |state| {
            state.serialize_field("tail", &self.tail)?;
            let value = TemplateElementValue {
                raw: self.value,
                cooked: self.value,
            };
            state.serialize_field("value", &value)
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json;

    #[test]
    fn test_parse_regex() {
        assert_eq!(
            parse_regex("/foo/"),
            RegExLiteral {
                pattern: "foo",
                flags: ""
            }
        );
        assert_eq!(
            parse_regex("/bar/mg"),
            RegExLiteral {
                pattern: "bar",
                flags: "mg"
            }
        );
    }

    #[test]
    fn test_is_float() {
        assert_eq!(is_float("3142"), false);
        assert_eq!(is_float("3.142"), true);
        assert_eq!(is_float("1."), true);
        assert_eq!(is_float(".1"), true);
    }

    #[test]
    fn test_value_undefined() {
        expect_parse!("undefined", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "Literal",
                        "raw": "undefined",
                        "start": 0,
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
    fn test_value_null() {
        expect_parse!("null", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "Literal",
                        "value": serde_json::Value::Null,
                        "raw": "null",
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
    fn test_value_true() {
        expect_parse!("true", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "Literal",
                        "value": true,
                        "raw": "true",
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
    fn test_value_false() {
        expect_parse!("false", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "Literal",
                        "value": false,
                        "raw": "false",
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
    fn test_value_number() {
        expect_parse!("0", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "Literal",
                        "value": 0,
                        "raw": "0",
                        "start": 0,
                        "end": 1
                    },
                    "start": 0,
                    "end": 1,
                }
            ],
            "start": 0,
            "end": 1,
        });

        expect_parse!("255", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "Literal",
                        "value": 255,
                        "raw": "255",
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

        expect_parse!("0.234", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "Literal",
                        "value": 0.234,
                        "raw": "0.234",
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

        expect_parse!("0xff", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "Literal",
                        "value": 255,
                        "raw": "0xff",
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

        expect_parse!("0b1111", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "Literal",
                        "value": 15,
                        "raw": "0b1111",
                        "start": 0,
                        "end": 6
                    },
                    "start": 0,
                    "end": 6,
                }
            ],
            "start": 0,
            "end": 6,
        });

        expect_parse!("0o1111", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "Literal",
                        "value": 585,
                        "raw": "0o1111",
                        "start": 0,
                        "end": 6
                    },
                    "start": 0,
                    "end": 6,
                }
            ],
            "start": 0,
            "end": 6,
        });
    }

    #[test]
    fn test_value_string() {
        expect_parse!("'foo'", {
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
    fn test_regex() {
        expect_parse!(r#"/^\b\w+/m"#, {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "Literal",
                        "regex": {
                            "pattern": r#"^\b\w+"#,
                            "flags": "m"
                        },
                        "raw": "/^\\b\\w+/m",
                        "start": 0,
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
    fn test_template() {
        expect_parse!("``", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "TemplateLiteral",
                        "quasis": [
                            {
                                "type": "TemplateElement",
                                "tail": true,
                                "value": {
                                    "raw": "",
                                    "cooked": "",
                                },
                                "start": 1,
                                "end": 1
                            }
                        ],
                        "expressions": [],
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

        expect_parse!("foo``", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "TaggedTemplateExpression",
                        "tag": {
                            "type": "Identifier",
                            "name": "foo",
                            "start": 0,
                            "end": 3
                        },
                        "quasi": {
                            "type": "TemplateLiteral",
                            "quasis": [
                                {
                                    "type": "TemplateElement",
                                    "tail": true,
                                    "value": {
                                        "raw": "",
                                        "cooked": "",
                                    },
                                    "start": 4,
                                    "end": 4
                                }
                            ],
                            "expressions": [],
                            "start": 3,
                            "end": 5
                        },
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

        expect_parse!("``", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "TemplateLiteral",
                        "quasis": [
                            {
                                "type": "TemplateElement",
                                "tail": true,
                                "value": {
                                    "raw": "",
                                    "cooked": "",
                                },
                                "start": 1,
                                "end": 1
                            }
                        ],
                        "expressions": [],
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

        expect_parse!("`foo${bar}baz`", {
            "type": "Program",
            "body": [
                {
                    "type": "ExpressionStatement",
                    "expression": {
                        "type": "TemplateLiteral",
                        "quasis": [
                            {
                                "type": "TemplateElement",
                                "tail": false,
                                "value": {
                                    "raw": "foo",
                                    "cooked": "foo",
                                },
                                "start": 1,
                                "end": 5
                            },
                            {
                                "type": "TemplateElement",
                                "tail": true,
                                "value": {
                                    "raw": "baz",
                                    "cooked": "baz",
                                },
                                "start": 10,
                                "end": 13
                            }
                        ],
                        "expressions": [
                            {
                                "type": "Identifier",
                                "name": "bar",
                                "start": 6,
                                "end": 9
                            }
                        ],
                        // FIXME
                        "start": 0,
                        "end": 14
                    },
                    "start": 0,
                    "end": 14,
                }
            ],
            "start": 0,
            "end": 14,
        });
    }

    #[test]
    fn test_pattern() {
        expect_parse!("let {a, b} = { a: 1, b: 2 };", {
            "type":"Program",
            "body":[
                {
                    "type":"VariableDeclaration",
                    "kind":"let",
                    "declarations":[
                        {
                            "type":"VariableDeclarator",
                            "id":{
                                "type":"ObjectPattern",
                                "keys":[
                                    {
                                        "type":"Identifier",
                                        "name":"a",
                                        "start":5,
                                        "end":6
                                    },
                                    {
                                        "type":"Identifier",
                                        "name":"b",
                                        "start":8,
                                        "end":9
                                    }
                                ],
                                "start":4,
                                "end":10
                            },
                            "init":{
                                "type":"ObjectExpression",
                                "properties":[
                                    {
                                        "type":"Property",
                                        "key":{
                                            "type":"Identifier",
                                            "name":"a",
                                            "start":15,
                                            "end":16
                                        },
                                        "method":false,
                                        "shorthand":false,
                                        "computed":false,
                                        "value":{
                                            "type":"Literal",
                                            "value":1,
                                            "raw":"1",
                                            "start":18,
                                            "end":19
                                        },
                                        "kind":"init",
                                        "start":15,
                                        "end":19
                                    },
                                    {
                                        "type":"Property",
                                        "key":{
                                            "type":"Identifier",
                                            "name":"b",
                                            "start":21,
                                            "end":22
                                        },
                                        "method":false,
                                        "shorthand":false,
                                        "computed":false,
                                        "value":{
                                            "type":"Literal",
                                            "value":2,
                                            "raw":"2",
                                            "start":24,
                                            "end":25
                                        },
                                        "kind":"init",
                                        "start":21,
                                        "end":25
                                    }
                                ],
                                "start":13,
                                "end":27
                            },
                            "start":4,
                            "end":27
                        }
                    ],
                    "start":0,
                    "end":28
                }
            ],
            "start":0,
            "end":28
        });
        expect_parse!("let [a, b] = [1, 2];", {
            "type":"Program",
            "body":[
                {
                    "type":"VariableDeclaration",
                    "kind":"let",
                    "declarations":[
                        {
                            "type":"VariableDeclarator",
                            "id":{
                                "type":"ArrayPattern",
                                "elements":[
                                    {
                                        "type":"Identifier",
                                        "name":"a",
                                        "start":5,
                                        "end":6
                                    },
                                    {
                                        "type":"Identifier",
                                        "name":"b",
                                        "start":8,
                                        "end":9
                                    }
                                ],
                                "start":4,
                                "end":10
                            },
                            "init":{
                                "type":"ArrayExpression",
                                "elements":[
                                    {
                                        "type":"Literal",
                                        "value":1,
                                        "raw":"1",
                                        "start":14,
                                        "end":15
                                    },
                                    {
                                        "type":"Literal",
                                        "value":2,
                                        "raw":"2",
                                        "start":17,
                                        "end":18
                                    }
                                ],
                                "start":13,
                                "end":19
                            },
                            "start":4,
                            "end":19
                        }
                    ],
                    "start":0,
                    "end":20
                }
            ],
            "start":0,
            "end":20
        });
    }
}
