use ast::{Function, Class, ClassMember, Name, EmptyName, OptionalName, MandatoryName};
use ast::{Node, ExpressionNode, StatementNode};
use visitor::{Visitable, Visitor, NoParent};

pub trait ChildMarker<'ast>: Name<'ast> {
    type Parent: 'ast;
}

impl<'ast> ChildMarker<'ast> for EmptyName {
    type Parent = NoParent;
}

impl<'ast> ChildMarker<'ast> for OptionalName<'ast> {
    type Parent = ExpressionNode<'ast>;
}

impl<'ast> ChildMarker<'ast> for MandatoryName<'ast> {
    type Parent = StatementNode<'ast>;
}

impl<'ast, N> Visitable<'ast> for Function<'ast, N>
where
    N: ChildMarker<'ast>
{
    type Parent = N::Parent;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        visitor.on_enter_block(self.body.body, ctx);
        self.params.visit(visitor, ctx);
        self.body.body.visit(visitor, ctx);
        visitor.on_leave_block(ctx);
    }
}

impl<'ast> Visitable<'ast> for ClassMember<'ast> {
    type Parent = Node<'ast, Self>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        use self::ClassMember::*;

        match *self {
            Error => panic!("Invalid AST"),
            Method {
                ref key,
                ref value,
                ..
            } => {
                key.visit(visitor, ctx);
                value.visit(visitor, ctx);
            },
            Literal {
                ref key,
                ref value,
                ..
            } => {
                key.visit(visitor, ctx);
                value.visit(visitor, ctx);
            },
        }
    }
}

impl<'ast, N> Visitable<'ast> for Class<'ast, N>
where
    N: ChildMarker<'ast>
{
    type Parent = N::Parent;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.extends.visit(visitor, ctx);
        self.body.body.visit(visitor, ctx);
    }
}
