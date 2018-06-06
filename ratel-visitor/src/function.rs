use ratel::ast::{Function, Class, ClassMember, Name, EmptyName, OptionalName, MandatoryName};
use ratel::ast::{Node, ExpressionNode, StatementNode};

use {Visitable, Visitor, ScopeKind, NoParent};


impl<'ast> Visitable<'ast> for EmptyName {
    type Parent = NoParent;

    #[inline]
    fn traverse<V>(&'ast self, _: &V, _: &mut V::Context)
    where
        V: Visitor<'ast>,
    {}
}

impl<'ast> Visitable<'ast> for OptionalName<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn traverse<V>(&'ast self, _: &V, _: &mut V::Context)
    where
        V: Visitor<'ast>,
    {}
}

impl<'ast> Visitable<'ast> for MandatoryName<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        visitor.on_reference_declaration(&(self.0).item, ctx);
    }
}

impl<'ast, N> Visitable<'ast> for Function<'ast, N>
where
    N: Visitable<'ast> + Name<'ast>,
{
    type Parent = N::Parent;

    #[inline]
    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.name.traverse(visitor, ctx);

        // Call visit on the StatementList instead of BlockNode since we
        // need to make sure that function parameters end up inside the block
        visitor.on_enter_scope(ScopeKind::Function, ctx);
        self.params.traverse(visitor, ctx);
        self.body.body.traverse(visitor, ctx);
        visitor.on_leave_scope(ctx);
    }
}

impl<'ast> Visitable<'ast> for ClassMember<'ast> {
    type Parent = Node<'ast, Self>;

    #[inline]
    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
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
                key.traverse(visitor, ctx);
                value.traverse(visitor, ctx);
            },
            Literal {
                ref key,
                ref value,
                ..
            } => {
                key.traverse(visitor, ctx);
                value.traverse(visitor, ctx);
            },
        }
    }
}

impl<'ast, N> Visitable<'ast> for Class<'ast, N>
where
    N: Visitable<'ast> + Name<'ast>,
{
    type Parent = N::Parent;

    #[inline]
    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.name.traverse(visitor, ctx);
        self.extends.traverse(visitor, ctx);
        self.body.body.traverse(visitor, ctx);
    }
}
