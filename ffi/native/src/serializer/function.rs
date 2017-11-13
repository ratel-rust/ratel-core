use ast;
use ast::{Loc, Function, MandatoryName, OptionalName, ClassMember, ParameterPtr, ParameterKey};
use serde_json;
use serializer::Serializable;

impl<'ast> Serializable<'ast> for ast::Loc<Function<'ast, MandatoryName<'ast>>> {
    #[inline]
    fn serialize<'a>(&self) -> Option<serde_json::Value> {
        // FIXME
        let body = match self.body.only_element() {
            Some(&Loc { item: ast::Statement::Block { .. } , .. }) => {
                self.body.serialize()
            },
            _ => {
                Some(json!({
                    "type": "BlockStatement",
                    "body": self.body.serialize(),
                    "start": 0,
                    "end": 0,
                }))

            }
        };

        Some(json!({
            "type": "FunctionExpression",
            "id": self.name.serialize(),
            "params": self.params.serialize(),
            "body": body,
            "start": self.start,
            "end": self.end,
        }))
    }
}

impl<'ast> Serializable<'ast> for Loc<Function<'ast, OptionalName<'ast>>> {
    #[inline]
    fn serialize<'a>(&self) -> Option<serde_json::Value> {
        let item = self.item;
        // FIXME
        let body = match item.body.only_element() {
            Some(&Loc { item: ast::Statement::Block { .. } , .. }) => {
                item.body.serialize()
            },
            _ => {
                Some(json!({
                    "type": "BlockStatement",
                    "body": item.body.serialize(),
                    "start": 0,
                    "end": 0,
                }))

            }
        };

        Some(json!({
            "type": "FunctionExpression",
            "id": item.name.serialize(),
            "params": item.params.serialize(),
            "body": body,
            "start": self.start,
            "end": self.end,
        }))
    }
}

impl<'ast> Serializable<'ast> for Loc<ast::ClassMember<'ast>> {
    #[inline]
    fn serialize<'a>(&self) -> Option<serde_json::Value> {
        use self::ClassMember::*;
        let result = match self.item {
            Error { .. } => panic!("Module contains errors"),
            Constructor { params, body} => {
                // FIXME
                let value = ast::Ptr::new(&Loc::new(self.start, self.end, ast::Expression::Function {
                    function: Function {
                        name: ast::Name::empty(),
                        params,
                        body
                    }
                })).serialize();

                json!({
                    "type": "MethodDefinition",
                    "kind": "constructor",
                    "static": false,
                    "computed": false,
                    "key": {
                        "type": "Identifier",
                        "name": "constructor",
                        //FIXME
                        "start": 0,
                        "end": 0,
                    },
                    "value": value,
                    "start": self.start,
                    "end": self.end,
                })
            },
            Method { is_static, property, params, body } => {
                //FIXME
                let ident_name = ast::Ptr::new(&Loc::new(self.start, self.end, property)).serialize();
                let value = ast::Ptr::new(&Loc::new(self.start, self.end, ast::Expression::Function {
                    function: Function {
                        name: ast::Name::empty(),
                        params,
                        body
                    }
                })).serialize();

                json!({
                    "type": "MethodDefinition",
                    "kind": "method",
                    "static": is_static,
                    "computed": false,
                    "key": ident_name,
                    "value": value,
                    "start": self.start,
                    "end": self.end,
                })
            },
            Value { is_static, property, value } => {
                //FIXME
                let loc = &Loc::new(self.start, self.end, property);
                let key = ast::Ptr::new(loc);
                json!({
                    "type": "ClassProperty",
                    "static": is_static,
                    "computed": false,
                    "key": key.serialize(),
                    "value": value.serialize(),
                    "start": self.start,
                    "end": self.end,
                })
            }
        };
        Some(result)
    }
}

impl<'ast> Serializable<'ast> for ParameterPtr<'ast> {
    #[inline]
    fn serialize(&self) -> Option<serde_json::Value> {

        let key = match self.item.key {
            ParameterKey::Identifier(value) => value,
            ParameterKey::Pattern(expr) => {
                panic!("Unimplemented: ParameterKey::Pattern(expr)");
            }
        };
        let result = match self.item.value {
            None => {
                json!({
                    "type": "Identifier",
                    "name": key,
                    "start": self.start,
                    "end": self.end
                })
            },
            Some(value) => {
                json!({
                    "type": "AssignmentPattern",
                    // FIXME
                    "left": {
                        "type": "Identifier",
                        "name": key,
                        "start": 0,
                        "end": 0,
                    },
                    "right": value.serialize(),
                    "start": self.start,
                    "end": self.end
                })
            }
        };
        Some(result)
    }
}

impl<'ast> Serializable<'ast> for MandatoryName<'ast> {
    #[inline]
    fn serialize<'a>(&self) -> Option<serde_json::Value> {
        let MandatoryName(ptr) = *self;
        ptr.serialize()
    }
}

impl<'ast> Serializable<'ast> for OptionalName<'ast> {
    #[inline]
    fn serialize<'a>(&self) -> Option<serde_json::Value> {
        if let ast::OptionalName(Some(name)) = *self {
            return Some(json!({
                "type": "Identifier",
                "start": name.start,
                "end": name.end,
                "name": name.item
            }))
        }
        None
    }
}
