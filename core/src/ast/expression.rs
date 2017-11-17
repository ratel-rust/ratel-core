use ast::{Loc, List, Value, OperatorKind, Function, Class, OptionalName};
use ast::{PropertyPtr, IdentifierPtr};
use ast::{ExpressionPtr, ExpressionList, StatementPtr, StatementList, ParameterList};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Property<'ast> {
    Computed(ExpressionPtr<'ast>),
    Literal(&'ast str),
    Binary(&'ast str),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ObjectMember<'ast> {
    Shorthand(&'ast str),
    Value {
        property: PropertyPtr<'ast>,
        value: ExpressionPtr<'ast>,
    },
    Method {
        property: PropertyPtr<'ast>,
        params: ParameterList<'ast>,
        body: StatementList<'ast>,
    },
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SequenceExpr<'ast> {
    pub body: ExpressionList<'ast>
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ArrayExpr<'ast> {
    pub body: ExpressionList<'ast>
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct MemberExpr<'ast> {
    pub object: ExpressionPtr<'ast>,
    pub property: IdentifierPtr<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ComputedMemberExpr<'ast> {
    pub object: ExpressionPtr<'ast>,
    pub property: ExpressionPtr<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct CallExpr<'ast> {
    pub callee: ExpressionPtr<'ast>,
    pub arguments: ExpressionList<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct BinaryExpr<'ast> {
    pub operator: OperatorKind,
    pub left: ExpressionPtr<'ast>,
    pub right: ExpressionPtr<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PrefixExpr<'ast> {
    pub operator: OperatorKind,
    pub operand: ExpressionPtr<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PostfixExpr<'ast> {
    pub operator: OperatorKind,
    pub operand: ExpressionPtr<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ConditionalExpr<'ast> {
    pub test: ExpressionPtr<'ast>,
    pub consequent: ExpressionPtr<'ast>,
    pub alternate: ExpressionPtr<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TemplateExpr<'ast> {
    pub tag: Option<ExpressionPtr<'ast>>,
    pub expressions: ExpressionList<'ast>,
    pub quasis: List<'ast, Loc<&'ast str>>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ArrowExpr<'ast> {
    pub params: ParameterList<'ast>,
    pub body: StatementPtr<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ObjectExpr<'ast> {
    pub body: List<'ast, Loc<ObjectMember<'ast>>>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Expression<'ast> {
    Error,
    Void,
    This,
    Identifier(&'ast str),
    Value(Value<'ast>),
    Sequence(SequenceExpr<'ast>),
    Array(ArrayExpr<'ast>),
    Member(MemberExpr<'ast>),
    ComputedMember(ComputedMemberExpr<'ast>),
    Call(CallExpr<'ast>),
    Binary(BinaryExpr<'ast>),
    Prefix(PrefixExpr<'ast>),
    Postfix(PostfixExpr<'ast>),
    Conditional(ConditionalExpr<'ast>),
    Template(TemplateExpr<'ast>),
    Arrow(ArrowExpr<'ast>),
    Object(ObjectExpr<'ast>),
    Function(Function<'ast, OptionalName<'ast>>),
    Class(Class<'ast, OptionalName<'ast>>),
}

macro_rules! impl_from {
    ($( $type:ident => $variant:ident ),*) => ($(
        impl<'ast> From<$type<'ast>> for Expression<'ast> {
            #[inline]
            fn from(val: $type<'ast>) -> Expression<'ast> {
                Expression::$variant(val)
            }
        }
    )*)
}

impl_from! {
    Value => Value,
    SequenceExpr => Sequence,
    ArrayExpr => Array,
    MemberExpr => Member,
    ComputedMemberExpr => ComputedMember,
    CallExpr => Call,
    BinaryExpr => Binary,
    PrefixExpr => Prefix,
    PostfixExpr => Postfix,
    ConditionalExpr => Conditional,
    TemplateExpr => Template,
    ArrowExpr => Arrow,
    ObjectExpr => Object
}

impl<'ast> Expression<'ast> {
    #[inline]
    pub fn at(self, start: u32, end: u32) -> Loc<Expression<'ast>> {
        Loc::new(start, end, self)
    }

    #[inline]
    pub fn binding_power(&self) -> u8 {
        use self::Expression::*;

        match *self {
            Member(_) | Arrow(_) => 18,

            Call(_) => 17,

            Prefix(_) => 15,

            Binary(BinaryExpr { ref operator, .. })   |
            Postfix(PostfixExpr { ref operator, .. }) => operator.binding_power(),

            Conditional(_) => 4,

            Sequence(_) => 0,

            _  => 100,
        }
    }

    #[inline]
    pub fn is_allowed_as_bare_statement(&self) -> bool {
        use self::Expression::*;

        match *self {
            Object(_)   |
            Function(_) |
            Class(_)    => false,
            _           => true,
        }
    }
}

impl<'ast> Property<'ast> {
    #[inline]
    pub fn is_constructor(&self) -> bool {
        match *self {
            Property::Literal("constructor") => true,
            _                                => false,
        }
    }
}
