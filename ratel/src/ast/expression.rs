use ast::{Node, NodeList, Literal, OperatorKind, Function, Class, EmptyName, OptionalName};
use ast::{Identifier, IdentifierNode, BlockNode, ExpressionNode, Statement, ExpressionList, Pattern};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PropertyKey<'ast> {
    Computed(ExpressionNode<'ast>),
    Literal(&'ast str),
    Binary(&'ast str),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Property<'ast> {
    Shorthand(&'ast str),
    Literal {
        key: Node<'ast, PropertyKey<'ast>>,
        value: ExpressionNode<'ast>,
    },
    Method {
        key: Node<'ast, PropertyKey<'ast>>,
        value: Node<'ast, Function<'ast, EmptyName>>,
    },
    Spread {
        argument: ExpressionNode<'ast>,
    }
}

/// While not technically necessary, having a type
/// helps with implementing the visitor pattern on AST.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ThisExpression;

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
    pub object: ExpressionNode<'ast>,
    pub property: IdentifierNode<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct MetaPropertyExpression<'ast> {
    pub meta: IdentifierNode<'ast>,
    pub property: IdentifierNode<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ComputedMemberExpression<'ast> {
    pub object: ExpressionNode<'ast>,
    pub property: ExpressionNode<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct CallExpression<'ast> {
    pub callee: ExpressionNode<'ast>,
    pub arguments: ExpressionList<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct BinaryExpression<'ast> {
    pub operator: OperatorKind,
    pub left: ExpressionNode<'ast>,
    pub right: ExpressionNode<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PrefixExpression<'ast> {
    pub operator: OperatorKind,
    pub operand: ExpressionNode<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PostfixExpression<'ast> {
    pub operator: OperatorKind,
    pub operand: ExpressionNode<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ConditionalExpression<'ast> {
    pub test: ExpressionNode<'ast>,
    pub consequent: ExpressionNode<'ast>,
    pub alternate: ExpressionNode<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TemplateLiteral<'ast> {
    pub expressions: ExpressionList<'ast>,
    pub quasis: NodeList<'ast, &'ast str>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TaggedTemplateExpression<'ast> {
    pub tag: ExpressionNode<'ast>,
    pub quasi: Node<'ast, TemplateLiteral<'ast>>
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SpreadExpression<'ast> {
    pub argument: ExpressionNode<'ast>
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ArrowBody<'ast> {
    Expression(ExpressionNode<'ast>),
    Block(BlockNode<'ast, Statement<'ast>>)
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ArrowExpression<'ast> {
    pub params: NodeList<'ast, Pattern<'ast>>,
    pub body: ArrowBody<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ObjectExpression<'ast> {
    pub body: NodeList<'ast, Property<'ast>>,
}

pub type FunctionExpression<'ast> = Function<'ast, OptionalName<'ast>>;
pub type ClassExpression<'ast> = Class<'ast, OptionalName<'ast>>;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Expression<'ast> {
    Void,
    This(ThisExpression),
    Identifier(Identifier<'ast>),
    Literal(Literal<'ast>),
    Sequence(SequenceExpression<'ast>),
    Array(ArrayExpression<'ast>),
    Member(MemberExpression<'ast>),
    ComputedMember(ComputedMemberExpression<'ast>),
    MetaProperty(MetaPropertyExpression<'ast>),
    Call(CallExpression<'ast>),
    Binary(BinaryExpression<'ast>),
    Prefix(PrefixExpression<'ast>),
    Postfix(PostfixExpression<'ast>),
    Conditional(ConditionalExpression<'ast>),
    Template(TemplateLiteral<'ast>),
    TaggedTemplate(TaggedTemplateExpression<'ast>),
    Spread(SpreadExpression<'ast>),
    Arrow(ArrowExpression<'ast>),
    Object(ObjectExpression<'ast>),
    Function(FunctionExpression<'ast>),
    Class(ClassExpression<'ast>),
}

macro_rules! impl_from {
    ($( $type:ty => $variant:ident ),*) => ($(
        impl<'ast> From<$type> for Expression<'ast> {
            #[inline]
            fn from(val: $type) -> Expression<'ast> {
                Expression::$variant(val)
            }
        }
    )*)
}

impl_from! {
    ThisExpression => This,
    Identifier<'ast> => Identifier,
    Literal<'ast> => Literal,
    SequenceExpression<'ast> => Sequence,
    ArrayExpression<'ast> => Array,
    MemberExpression<'ast> => Member,
    ComputedMemberExpression<'ast> => ComputedMember,
    MetaPropertyExpression<'ast> => MetaProperty,
    CallExpression<'ast> => Call,
    BinaryExpression<'ast> => Binary,
    PrefixExpression<'ast> => Prefix,
    PostfixExpression<'ast> => Postfix,
    ConditionalExpression<'ast> => Conditional,
    TemplateLiteral<'ast> => Template,
    TaggedTemplateExpression<'ast> => TaggedTemplate,
    SpreadExpression<'ast> => Spread,
    ArrowExpression<'ast> => Arrow,
    ObjectExpression<'ast> => Object,
    FunctionExpression<'ast> => Function,
    ClassExpression<'ast> => Class
}

impl<'ast> Expression<'ast> {
    #[inline]
    pub fn binding_power(&self) -> u8 {
        use self::Expression::*;

        match *self {
            Member(_) | MetaProperty(_) | Arrow(_) => 18,

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

    #[inline]
    pub fn is_lvalue(&self) -> bool {
        use self::Expression::*;

        match *self {
            Identifier(_) |
            Member(_)     |
            Object(_)     |
            Array(_)      |
            Spread(_)     => true,
            _             => false
        }
    }
}
