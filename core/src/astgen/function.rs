use serde::ser::{Serialize, Serializer, SerializeStruct};
use ast;
use ast::{ParameterKey, Expression, Loc};
use ast::{OptionalName, MandatoryName, Function, ClassMember, Parameter, List, Statement};
use astgen::statement::BlockStatement;
use astgen::SerializeInLoc;

#[derive(Debug)]
pub struct ClassBody<'ast> {
  pub body: List<'ast, Loc<ClassMember<'ast>>>
}

impl<'ast> SerializeInLoc for ClassBody<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
        where S: Serializer
    {
        self.in_loc(serializer, "Program", 1, |state| {
            state.serialize_field("body", &self.body)
        })
    }
}


impl<'ast> SerializeInLoc for Function<'ast, MandatoryName<'ast>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
        where S: Serializer
    {
        self.in_loc(serializer, "FunctionDeclaration", 3, |state| {
            state.serialize_field("id", &self.name)?;
            state.serialize_field("params", &self.params)?;
            state.serialize_field("body", &self.body)?;

            // TODO: Add "Block" type
            let body = BlockStatement { body: self.body };

            state.serialize_field("body", &Loc::new(0, 0, body))
        })
    }
}

impl<'ast> SerializeInLoc for Function<'ast, OptionalName<'ast>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
        where S: Serializer
    {
        self.in_loc(serializer, "FunctionExpression", 3, |state| {
            state.serialize_field("id", &self.name)?;
            state.serialize_field("params", &self.params)?;
            state.serialize_field("body", &self.body)?;

            // TODO: Add "Block" type
            let body = BlockStatement { body: self.body };

            state.serialize_field("body", &Loc::new(0, 0, body))
        })
    }
}

impl<'ast> SerializeInLoc for &'ast str {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
        where S: Serializer
    {
        self.in_loc(serializer, "Identifier", 1, move |state| state.serialize_field("name", self))
    }
}

impl<'ast> SerializeInLoc for Parameter<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
        where S: Serializer
    {

        let key = match self.key {
            ParameterKey::Identifier(value) => value,
            ParameterKey::Pattern(_) => {
                panic!("Unimplemented: ParameterKey::Pattern(expr)");
            }
        };
        return match self.value {
            None => {
                self.in_loc(serializer, "Identifier", 1, |state| {
                    state.serialize_field("name", &key)
                })
            },
            Some(value) => {
                self.in_loc(serializer, "AssignmentPattern", 2, |state| {
                    state.serialize_field("left", &Loc::new(0, 0, Expression::Identifier(key)))?;
                    state.serialize_field("right", &value)
                })
            }
        }
    }
}

impl<'ast> Serialize for OptionalName<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        self.0.serialize(serializer)
    }
}

impl<'ast> Serialize for MandatoryName<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        self.0.serialize(serializer)
    }
}

impl<'ast> Serialize for Loc<ClassMember<'ast>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        use self::ClassMember::*;

        let mut state = match self.item {
            Error { .. } => panic!("Module contains errors"),
            Constructor { params, body} => {
                let mut state = serializer.serialize_struct("MethodDefinition", 8)?;

                let function = ast::Expression::Function {
                  function: Function {
                      name: ast::Name::empty(),
                      params,
                      body
                  }
                };

                state.serialize_field("type", &"MethodDefinition")?;
                state.serialize_field("kind", &"constructor")?;
                state.serialize_field("static", &false)?;
                state.serialize_field("computed", &false)?;
                state.serialize_field("key", &Loc::new(self.start, self.end, Expression::Identifier("constructor")))?;
                state.serialize_field("value", &Loc::new(self.start, self.end, function))?;
                state
            }
            Method { is_static, property, params, body } => {
                let function: Expression = Expression::Function {
                    function: Function {
                        name: ast::Name::empty(),
                        params,
                        body
                    }
                };

                let mut state = serializer.serialize_struct("MethodDefinition", 8)?;
                state.serialize_field("type", &"MethodDefinition")?;
                state.serialize_field("kind", &"method")?;
                state.serialize_field("static", &is_static)?;
                state.serialize_field("computed", &false)?;
                state.serialize_field("key", &Loc::new(self.start, self.end, property))?;
                state.serialize_field("value", &Loc::new(self.start, self.end, function))?;
                state
            },
            Value { is_static, property, value } => {
                let mut state = serializer.serialize_struct("ClassProperty", 7)?;
                state.serialize_field("type", &"ClassProperty")?;
                state.serialize_field("static", &is_static)?;
                state.serialize_field("computed", &false)?;
                state.serialize_field("key", &Loc::new(self.start, self.end, property))?;
                state.serialize_field("value", &value)?;
                unimplemented!()
            }
        };

        state.serialize_field("start", &self.start)?;
        state.serialize_field("end", &self.end)?;
        state.end()
    }
}
