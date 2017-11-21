use ast::{Ptr, List, Literal, OperatorKind, Function, Class, EmptyName, OptionalName};
use ast::{IdentifierPtr, BlockPtr, ExpressionPtr, Statement, ExpressionList, Pattern};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PropertyKey<'ast> {
    Computed(ExpressionPtr<'ast>),
    Literal(&'ast str),
    Binary(&'ast str),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Property<'ast> {
    Shorthand(&'ast str),
    Literal {
        key: Ptr<'ast, PropertyKey<'ast>>,
        value: ExpressionPtr<'ast>,
    },
    Method {
        key: Ptr<'ast, PropertyKey<'ast>>,
        value: Ptr<'ast, Function<'ast, EmptyName>>,
    },
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SequenceExpression<'ast> {
    pub body: ExpressionList<'ast>
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ArrayExpression<'ast> {
    pub body: ExpressionList<'ast>
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct MemberExpression<'ast> {
    pub object: ExpressionPtr<'ast>,
    pub property: IdentifierPtr<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ComputedMemberExpression<'ast> {
    pub object: ExpressionPtr<'ast>,
    pub property: ExpressionPtr<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct CallExpression<'ast> {
    pub callee: ExpressionPtr<'ast>,
    pub arguments: ExpressionList<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct BinaryExpression<'ast> {
    pub operator: OperatorKind,
    pub left: ExpressionPtr<'ast>,
    pub right: ExpressionPtr<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PrefixExpression<'ast> {
    pub operator: OperatorKind,
    pub operand: ExpressionPtr<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PostfixExpression<'ast> {
    pub operator: OperatorKind,
    pub operand: ExpressionPtr<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ConditionalExpression<'ast> {
    pub test: ExpressionPtr<'ast>,
    pub consequent: ExpressionPtr<'ast>,
    pub alternate: ExpressionPtr<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TemplateExpression<'ast> {
    pub tag: Option<ExpressionPtr<'ast>>,
    pub expressions: ExpressionList<'ast>,
    pub quasis: List<'ast, &'ast str>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SpreadExpression<'ast> {
    pub argument: ExpressionPtr<'ast>
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ArrowBody<'ast> {
    Expression(ExpressionPtr<'ast>),
    Block(BlockPtr<'ast, Statement<'ast>>)
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ArrowExpression<'ast> {
    pub params: List<'ast, Pattern<'ast>>,
    pub body: ArrowBody<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ObjectExpression<'ast> {
    pub body: List<'ast, Property<'ast>>,
}

pub type FunctionExpression<'ast> = Function<'ast, OptionalName<'ast>>;
pub type ClassExpression<'ast> = Class<'ast, OptionalName<'ast>>;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Expression<'ast> {
    Error,
    Void,
    This,
    Identifier(&'ast str),
    Literal(Literal<'ast>),
    Sequence(SequenceExpression<'ast>),
    Array(ArrayExpression<'ast>),
    Member(MemberExpression<'ast>),
    ComputedMember(ComputedMemberExpression<'ast>),
    Call(CallExpression<'ast>),
    Binary(BinaryExpression<'ast>),
    Prefix(PrefixExpression<'ast>),
    Postfix(PostfixExpression<'ast>),
    Conditional(ConditionalExpression<'ast>),
    Template(TemplateExpression<'ast>),
    Spread(SpreadExpression<'ast>),
    Arrow(ArrowExpression<'ast>),
    Object(ObjectExpression<'ast>),
    Function(FunctionExpression<'ast>),
    Class(ClassExpression<'ast>),
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

impl<'ast> From<&'ast str> for Expression<'ast> {
    #[inline]
    fn from(val: &'ast str) -> Expression<'ast> {
        Expression::Identifier(val)
    }
}

impl_from! {
    Literal => Literal,
    SequenceExpression => Sequence,
    ArrayExpression => Array,
    MemberExpression => Member,
    ComputedMemberExpression => ComputedMember,
    CallExpression => Call,
    BinaryExpression => Binary,
    PrefixExpression => Prefix,
    PostfixExpression => Postfix,
    ConditionalExpression => Conditional,
    TemplateExpression => Template,
    SpreadExpression => Spread,
    ArrowExpression => Arrow,
    ObjectExpression => Object,
    FunctionExpression => Function,
    ClassExpression => Class
}

impl<'ast> Expression<'ast> {
    #[inline]
    pub fn binding_power(&self) -> u8 {
        use self::Expression::*;

        match *self {
            Member(_) | Arrow(_) => 18,

            Call(_) => 17,

            Prefix(_) => 15,

            Binary(BinaryExpression { ref operator, .. })   |
            Postfix(PostfixExpression { ref operator, .. }) => operator.binding_power(),

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
