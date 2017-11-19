use ast::{Ptr, Loc, IdentifierPtr, ParameterList, ExpressionPtr};
use ast::{BlockPtr, Statement, Property};

pub trait Name<'ast> {
    fn empty() -> Self;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Parameter<'ast> {
    pub key: ParameterKey<'ast>,
    pub value: Option<ExpressionPtr<'ast>>
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ParameterKey<'ast> {
    Identifier(&'ast str),
    Pattern(ExpressionPtr<'ast>)
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct EmptyName;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct MandatoryName<'ast>(pub IdentifierPtr<'ast>);

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct OptionalName<'ast>(pub Option<IdentifierPtr<'ast>>);

impl<'ast> Name<'ast> for EmptyName {
    fn empty() -> Self {
        EmptyName
    }
}

impl<'ast> Name<'ast> for MandatoryName<'ast> {
    fn empty() -> Self {
        MandatoryName(Ptr::new(&Loc {
            start: 0,
            end: 0,
            item: ""
        }))
    }
}

impl<'ast> Name<'ast> for OptionalName<'ast> {
    fn empty() -> Self {
        OptionalName(None)
    }
}

#[cfg(test)]
impl<'ast> From<IdentifierPtr<'ast>> for MandatoryName<'ast> {
    #[inline]
    fn from(name: IdentifierPtr<'ast>) -> Self {
        MandatoryName(name)
    }
}

#[cfg(test)]
impl<'ast> From<IdentifierPtr<'ast>> for OptionalName<'ast> {
    #[inline]
    fn from(name: IdentifierPtr<'ast>) -> Self {
        OptionalName(Some(name))
    }
}

#[cfg(test)]
impl<'ast> From<Option<IdentifierPtr<'ast>>> for OptionalName<'ast> {
    #[inline]
    fn from(name: Option<IdentifierPtr<'ast>>) -> Self {
        OptionalName(name)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Function<'ast, N: Name<'ast>> {
    pub name: N,
    pub params: ParameterList<'ast>,
    pub body: BlockPtr<'ast, Statement<'ast>>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MethodKind {
    Constructor,
    Method,
    Get,
    Set,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ClassMember<'ast> {
    Error,
    Method {
        is_static: bool,
        key: Property<'ast>,
        kind: MethodKind,
        value: Ptr<'ast, Function<'ast, EmptyName>>,
    },
    Literal {
        is_static: bool,
        key: Property<'ast>,
        value: ExpressionPtr<'ast>,
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Class<'ast, N: Name<'ast>> {
    pub name: N,
    pub extends: Option<ExpressionPtr<'ast>>,
    pub body: BlockPtr<'ast, ClassMember<'ast>>,
}
