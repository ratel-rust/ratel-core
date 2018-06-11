use ast::{Node, Loc, IdentifierNode, ExpressionNode};
use ast::{BlockNode, Statement, PatternList, PropertyKey};

pub trait Name<'ast>: Copy {
    fn empty() -> Self;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct EmptyName;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct MandatoryName<'ast>(pub IdentifierNode<'ast>);

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct OptionalName<'ast>(pub Option<IdentifierNode<'ast>>);

pub type Method<'ast> = Function<'ast, EmptyName>;

impl<'ast> Name<'ast> for EmptyName {
    fn empty() -> Self {
        EmptyName
    }
}

impl<'ast> Name<'ast> for MandatoryName<'ast> {
    fn empty() -> Self {
        MandatoryName(Node::new(&Loc {
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
impl<'ast> From<IdentifierNode<'ast>> for MandatoryName<'ast> {
    #[inline]
    fn from(name: IdentifierNode<'ast>) -> Self {
        MandatoryName(name)
    }
}

#[cfg(test)]
impl<'ast> From<IdentifierNode<'ast>> for OptionalName<'ast> {
    #[inline]
    fn from(name: IdentifierNode<'ast>) -> Self {
        OptionalName(Some(name))
    }
}

#[cfg(test)]
impl<'ast> From<Option<IdentifierNode<'ast>>> for OptionalName<'ast> {
    #[inline]
    fn from(name: Option<IdentifierNode<'ast>>) -> Self {
        OptionalName(name)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Function<'ast, N: Name<'ast>> {
    pub name: N,
    pub generator: bool,
    pub params: PatternList<'ast>,
    pub body: BlockNode<'ast, Statement<'ast>>,
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
        key: Node<'ast, PropertyKey<'ast>>,
        kind: MethodKind,
        value: Node<'ast, Function<'ast, EmptyName>>,
    },
    Literal {
        is_static: bool,
        key: Node<'ast, PropertyKey<'ast>>,
        value: ExpressionNode<'ast>,
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Class<'ast, N: Name<'ast>> {
    pub name: N,
    pub extends: Option<ExpressionNode<'ast>>,
    pub body: BlockNode<'ast, ClassMember<'ast>>,
}
