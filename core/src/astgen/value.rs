use serde::ser::{Serialize, Serializer, SerializeStruct};
use ast;
use ast::{Expression, Loc, Property, DeclarationKind, ObjectMember};
use ast::Function;

#[derive(Debug, Serialize)]
pub struct RegExLiteral<'ast> {
    pub pattern: &'ast str,
    pub flags: &'ast str
}

#[derive(Debug)]
pub struct TemplateElement<'ast> {
    pub tail: bool,
    pub value: &'ast str
}

#[derive(Debug, Serialize)]
pub struct TemplateElementValue<'ast> {
    pub raw: &'ast str,
    pub cooked: &'ast str
}

#[derive(Debug)]
pub struct TemplateLiteral<'ast> {
    pub quasis: Vec<Loc<TemplateElement<'ast>>>,
    pub expressions: Vec<Loc<Expression<'ast>>>,
}

fn parse_regex (value: &str) -> (&str, &str) {
    let mut end = value.len() - 1;
    for index in (0..value.len()).rev() {
            if "/" == &value[index..(index+1)] {
                    end = index;
                    break;
            }
    };

    (&value[1..end], &value[(end+1)..value.len()])
}

impl<'ast> Serialize for Loc<TemplateElement<'ast>> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer
        {

            let mut state = serializer.serialize_struct("TemplateElement", 5)?;
            state.serialize_field("type", &"TemplateElement")?;
            state.serialize_field("tail", &self.tail)?;
            let value = TemplateElementValue {
                raw: self.value,
                cooked: self.value
            };
            state.serialize_field("value", &value)?;
            state.serialize_field("start", &self.start)?;
            state.serialize_field("end", &self.end)?;
            state.end()
        }
}

impl<'ast> Serialize for Loc<TemplateLiteral<'ast>> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer
        {
            let mut state = serializer.serialize_struct("TemplateLiteral", 5)?;
            state.serialize_field("type", &"TemplateLiteral")?;
            state.serialize_field("quasis", &self.quasis)?;
            state.serialize_field("expressions", &self.expressions)?;
            state.serialize_field("start", &self.start)?;
            state.serialize_field("end", &self.end)?;
            state.end()
        }
}

impl<'ast> Serialize for ast::Value<'ast> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer
        {
            use self::ast::Value::*;
            match *self {
                Undefined => serializer.serialize_str("undefined"),
                Null      => serializer.serialize_str("null"),
                True      => serializer.serialize_bool(true),
                False     => serializer.serialize_bool(false),
                // FIXME
                Number(number)    => {
                    serializer.serialize_str(number)
                },
                Binary(number)    => {
                    serializer.serialize_str(number)
                },
                String(value)     => serializer.serialize_str(value),
                Template(value)   => {
                    let element = Loc::new(0, 0, TemplateElement {
                        tail: true,
                        value
                    });

                    let expr = Loc::new(0, 0, TemplateLiteral {
                        quasis: vec![element],
                        expressions: vec![]
                    });
                    serializer.serialize_some(&expr)
                },

                RegEx(value)      => {
                    let (pattern, flags) = parse_regex(value);
                    let regex = RegExLiteral { pattern, flags };
                    serializer.serialize_some(&regex)
                }
            }
        }
}

impl<'ast> Serialize for Loc<Property<'ast>> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer
        {
            use self::Property::*;

            return match self.item {
                Computed(expr) => {
                    serializer.serialize_some(&*expr)
                },
                Literal(value) => {
                    serializer.serialize_some(&Loc::new(self.start, self.end, Expression::Identifier(value)))
                },
                Binary(value) => {
                    serializer.serialize_some(&Loc::new(self.start, self.end, Expression::Identifier(value)))
                },
            }
        }
}

impl<'ast> Serialize for DeclarationKind {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer
        {
            use self::DeclarationKind::*;
            match *self {
                Const => serializer.serialize_str("const"),
                Let => serializer.serialize_str("let"),
                Var => serializer.serialize_str("var"),
            }
        }
}

impl<'ast> Serialize for Loc<ObjectMember<'ast>> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer
        {
            use self::ObjectMember::*;
            let mut state = serializer.serialize_struct("Property", 8)?;
            state.serialize_field("type", "Property")?;
            match self.item {
                Shorthand(value) => {
                        state.serialize_field("key", &value)?;
                        state.serialize_field("method", &false)?;
                        state.serialize_field("shorthand", &true)?;
                        state.serialize_field("computed", &false)?;
                        state.serialize_field("value", &value)?;
                        // FIXME
                        state.serialize_field("kind", &"init")?;
                }

                Value { property, value } => {
                        let computed = if let Property::Computed(_) = property.item { true } else { false };

                        state.serialize_field("key", &*property)?;
                        state.serialize_field("method", &false)?;
                        state.serialize_field("shorthand", &false)?;
                        state.serialize_field("computed", &computed)?;
                        state.serialize_field("value", &*value)?;
                        // FIXME
                        state.serialize_field("kind", &"init")?;
                }

                Method { property, params, body } => {
                        let function: Expression = Expression::Function {
                            function: Function {
                                    name: ast::Name::empty(),
                                    params,
                                    body
                            }
                        };
                        state.serialize_field("key", &*property)?;
                        state.serialize_field("method", &true)?;
                        state.serialize_field("shorthand", &false)?;
                        state.serialize_field("computed", &false)?;
                        state.serialize_field("value", &Loc::new(self.start, self.end, function))?;
                        // FIXME
                        state.serialize_field("kind", &"init")?;
                }
            };
            state.end()
        }
}
