use ast::{Ptr, Loc, IdentifierPtr, IdentifierList, StatementList};
use arena::Arena;

pub trait Name<'ast> {
    fn empty(&'ast Arena) -> Self;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MandatoryName<'ast>(pub IdentifierPtr<'ast>);

#[derive(Debug, Clone, PartialEq)]
pub struct OptionalName<'ast>(pub Option<IdentifierPtr<'ast>>);

impl<'ast> Name<'ast> for MandatoryName<'ast> {
    fn empty(arena: &'ast Arena) -> Self {
        MandatoryName(Ptr::new(arena.alloc(Loc::new(0, 0, ""))))
    }
}

impl<'ast> Name<'ast> for OptionalName<'ast> {
    fn empty(_: &Arena) -> Self {
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

#[derive(Debug, Clone, PartialEq)]
pub struct Function<'ast, N: Name<'ast>> {
    pub name: N,
    pub params: IdentifierList<'ast>,
    pub body: StatementList<'ast>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Class<'ast, N: Name<'ast>> {
    pub name: N,
    pub extends: Option<IdentifierPtr<'ast>>,
    pub body: StatementList<'ast>,
}
