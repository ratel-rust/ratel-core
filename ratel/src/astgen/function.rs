use serde::ser::{Serialize, Serializer, SerializeStruct};
use astgen::SerializeInLoc;
use ast::{Function, Class, Name, MandatoryName, OptionalName, EmptyName, ClassMember, Block};
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
                    state.serialize_field("key", &*key)?;
                    state.serialize_field("value", &value)
                })
            },
            Literal { .. } => {
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

pub trait NameType<'ast>: Name<'ast> {
    const IN_CLASS: &'static str = "ClassExpression";
    const IN_FUNCTION: &'static str = "FunctionExpression";
}

impl<'ast> NameType<'ast> for EmptyName {}

impl<'ast> Serialize for EmptyName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_none()
    }
}

impl<'ast> NameType<'ast> for OptionalName<'ast> {}

impl<'ast> Serialize for OptionalName<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (self.0).serialize(serializer)
    }
}

impl<'ast> NameType<'ast> for MandatoryName<'ast> {
    const IN_CLASS: &'static str = "ClassDeclaration";
    const IN_FUNCTION: &'static str = "FunctionDeclaration";
}

impl<'ast> Serialize for MandatoryName<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (self.0).serialize(serializer)
    }
}

impl<'ast, N> SerializeInLoc for Class<'ast, N>
where
    N: Serialize + NameType<'ast>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, N::IN_CLASS, 3, |state| {
            state.serialize_field("id", &self.name)?;
            state.serialize_field("superClass", &self.extends)?;
            state.serialize_field("body", &self.body)
        })
    }
}

impl<'ast, N> SerializeInLoc for Function<'ast, N>
where
    N: Serialize + NameType<'ast>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
    where
        S: Serializer,
    {
        self.in_loc(serializer, N::IN_FUNCTION, 3, |state| {
            state.serialize_field("generator", &self.generator)?;
            state.serialize_field("id", &self.name)?;
            state.serialize_field("params", &self.params)?;
            state.serialize_field("body", &self.body)
        })
    }
}