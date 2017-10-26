use ast::{Ptr, Loc, List, IdentifierPtr, ParameterList, ExpressionPtr, StatementList, Property};

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
pub struct MandatoryName<'ast>(pub IdentifierPtr<'ast>);

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct OptionalName<'ast>(pub Option<IdentifierPtr<'ast>>);

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

impl<'ast> From<IdentifierPtr<'ast>> for MandatoryName<'ast> {
    #[inline]
    fn from(name: IdentifierPtr<'ast>) -> Self {
        MandatoryName(name)
    }
}

impl<'ast> From<IdentifierPtr<'ast>> for OptionalName<'ast> {
    #[inline]
    fn from(name: IdentifierPtr<'ast>) -> Self {
        OptionalName(Some(name))
    }
}

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
    pub body: StatementList<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ClassMember<'ast> {
    Constructor {
        params: ParameterList<'ast>,
        body: StatementList<'ast>,
    },
    Method {
        is_static: bool,
        property: Property<'ast>,
        params: ParameterList<'ast>,
        body: StatementList<'ast>,
    },
    Value {
        is_static: bool,
        property: Property<'ast>,
        value: ExpressionPtr<'ast>,
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Class<'ast, N: Name<'ast>> {
    pub name: N,
    pub extends: Option<IdentifierPtr<'ast>>,
    pub body: List<'ast, Loc<ClassMember<'ast>>>,
}
