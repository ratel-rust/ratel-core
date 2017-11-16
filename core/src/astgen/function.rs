use serde::ser::{Serialize, Serializer, SerializeStruct};
use ast;
use ast::{ParameterKey, Expression, Loc};
use ast::{OptionalName, MandatoryName, Function, ClassMember, Parameter, List, Statement};
use astgen::statement::BlockStatement;

#[derive(Debug)]
pub struct ClassBody<'ast> {
  pub body: List<'ast, Loc<ClassMember<'ast>>>
}

impl<'ast> Serialize for Loc<ClassBody<'ast>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut state = serializer.serialize_struct("ClassBody", 4)?;
        state.serialize_field("type", &"ClassBody")?;
        state.serialize_field("body", &self.body)?;
        state.serialize_field("start", &self.start)?;
        state.serialize_field("end", &self.end)?;
        state.end()
    }
}

impl<'ast> Serialize for ast::Loc<Function<'ast, MandatoryName<'ast>>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut state = serializer.serialize_struct("FunctionExpression", 6)?;
        state.serialize_field("type", &"FunctionExpression")?;
        state.serialize_field("id", &Loc::new(self.start, self.end, self.name))?;
        state.serialize_field("params", &self.params)?;
        match self.body.only_element() {
            Some(&Loc { item: Statement::Block { .. } , .. }) => {
                state.serialize_field("body", &self.body)?;
            },
            _ => {
              let body = BlockStatement { body: self.body };
              state.serialize_field("body", &Loc::new(self.start, self.end, body))?;
            }
        };

        state.serialize_field("start", &self.start)?;
        state.serialize_field("end", &self.end)?;
        state.end()
    }
}

impl<'ast> Serialize for ast::Loc<Function<'ast, OptionalName<'ast>>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut state = serializer.serialize_struct("FunctionExpression", 5)?;
        state.serialize_field("type", &"FunctionExpression")?;
        state.serialize_field("id", &Loc::new(self.start, self.end, self.name))?;

        match self.body.only_element() {
            Some(&Loc { item: Statement::Block { .. } , .. }) => {
                state.serialize_field("body", &self.body)?;
            },
            _ => {
              let body = BlockStatement { body: self.body };
              state.serialize_field("body", &Loc::new(self.start, self.end, body))?;
            }
        };

        state.serialize_field("start", &self.start)?;
        state.serialize_field("end", &self.end)?;
        state.end()
    }
}

impl<'ast> Serialize for Loc<Parameter<'ast>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {

        let key = match self.item.key {
            ParameterKey::Identifier(value) => value,
            ParameterKey::Pattern(_) => {
                panic!("Unimplemented: ParameterKey::Pattern(expr)");
            }
        };
        let mut state = match self.item.value {
            None => {
                let mut state = serializer.serialize_struct("Identifier", 4)?;
                state.serialize_field("type", &"Identifier")?;
                state.serialize_field("name", &key)?;
                state
            },
            Some(value) => {
                let mut state = serializer.serialize_struct("AssignmentPattern", 5)?;
                state.serialize_field("type", &"AssignmentPattern")?;
                state.serialize_field("left", &Loc::new(self.start, self.end, Expression::Identifier(key)))?;
                state.serialize_field("right", &*value)?;
                state
            }
        };

        state.serialize_field("start", &self.start)?;
        state.serialize_field("end", &self.end)?;
        state.end()
    }
}

impl<'ast> Serialize for Loc<OptionalName<'ast>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        if let ast::OptionalName(Some(name)) = self.item {
          let expr = Loc::new(name.start, name.end, Expression::Identifier(name.item));
          serializer.serialize_some(&expr)
        } else {
          serializer.serialize_none()
        }
    }
}

impl<'ast> Serialize for Loc<MandatoryName<'ast>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let MandatoryName(name) = self.item;
        let expr = Loc::new(name.start, name.end, Expression::Identifier(name.item));
        serializer.serialize_some(&expr)
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
                state.serialize_field("value", &*value)?;
                unimplemented!()
            }
        };

        state.serialize_field("start", &self.start)?;
        state.serialize_field("end", &self.end)?;
        state.end()
    }
}
