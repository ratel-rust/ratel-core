use serde::ser::{Serialize, Serializer, SerializeStruct};
use astgen::SerializeInLoc;
use ast::{Loc, Function, MandatoryName, OptionalName, EmptyName, ClassMember, Block};
use ast::expression::ClassExpression;
use ast::MethodKind;

impl<'ast> Serialize for MethodKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use self::MethodKind::*;
        match *self {
            Constructor => serializer.serialize_str("constructor"),
            Method => serializer.serialize_str("method"),
            Get => serializer.serialize_str("get"),
            Set => serializer.serialize_str("set"),
        }
    }
}

impl<'ast> SerializeInLoc for ClassMember<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        use self::ClassMember::*;

        match *self {
            Error { .. } => panic!("Module contains errors"),
            Method { is_static, key, kind, value } => {
                self.in_loc(serializer, "MethodDefinition", 5, |state| {
                    state.serialize_field("kind", &kind)?;
                    state.serialize_field("static", &is_static)?;
                    state.serialize_field("computed", &false)?;
                    state.serialize_field("key", &Loc::new(0, 0, key))?;
                    state.serialize_field("value", &*value)
                })
            },
            Literal { is_static, key, value } => {
                unimplemented!()
            }
        }
    }
}

// TODO: DRY with BlockStatement
impl<'ast> SerializeInLoc for Block<'ast, ClassMember<'ast>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "ClassBody", 1, |state| {
            state.serialize_field("body", &self.body)
        })
    }
}

impl<'ast> Serialize for OptionalName<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (self.0).serialize(serializer)
    }
}

impl<'ast> Serialize for MandatoryName<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (*self.0).serialize(serializer)
    }
}

impl<'ast> SerializeInLoc for ClassExpression<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "ClassExpression", 3, |state| {
            state.serialize_field("id", &self.name)?;
            if let Some(value) = self.extends {
                state.serialize_field("superClass", &*value)?;
            } else {
                state.serialize_field("superClass", &())?
            }
            state.serialize_field("body", &*self.body)
        })
    }
}

impl<'ast> SerializeInLoc for Function<'ast, MandatoryName<'ast>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "FunctionDeclaration", 3, |state| {
            state.serialize_field("id", &self.name)?;
            state.serialize_field("params", &self.params)?;
            state.serialize_field("body", &*self.body)
        })
    }
}

impl<'ast> SerializeInLoc for Function<'ast, OptionalName<'ast>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "FunctionExpression", 3, |state| {
            state.serialize_field("id", &self.name)?;
            state.serialize_field("params", &self.params)?;
            state.serialize_field("body", &*self.body)
        })
    }
}

impl<'ast> SerializeInLoc for Function<'ast, EmptyName> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, "FunctionExpression", 3, |state| {
            state.serialize_field("id", &())?;
            state.serialize_field("params", &self.params)?;
            state.serialize_field("body", &*self.body)
        })
    }
}
