use ratel::ast::{Function, Class, ClassMember, Name, EmptyName, OptionalName, MandatoryName};
use ratel::ast::{Node, ExpressionNode, StatementNode};

use {Visitable, Visitor, ScopeKind, NoParent};


impl<'ast> Visitable<'ast> for EmptyName {
    type Parent = NoParent;

    #[inline]
    fn visit_with<V>(&'ast self, _: &mut V)
    where
        V: Visitor<'ast>,
    {}
}

impl<'ast> Visitable<'ast> for OptionalName<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, _: &mut V)
    where
        V: Visitor<'ast>,
    {}
}

impl<'ast> Visitable<'ast> for MandatoryName<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        visitor.on_reference_declaration(&(self.0).item);
    }
}

impl<'ast, N> Visitable<'ast> for Function<'ast, N>
where
    N: Visitable<'ast> + Name<'ast>,
{
    type Parent = N::Parent;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.name.visit_with(visitor);

        // Call visit on the StatementList instead of BlockNode since we
        // need to make sure that function parameters end up inside the block
        visitor.on_enter_scope(ScopeKind::Function);
        self.params.visit_with(visitor);
        self.body.body.visit_with(visitor);
        visitor.on_leave_scope();
    }
}

impl<'ast> Visitable<'ast> for ClassMember<'ast> {
    type Parent = Node<'ast, Self>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
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
                key.visit_with(visitor);
                value.visit_with(visitor);
            },
            Literal {
                ref key,
                ref value,
                ..
            } => {
                key.visit_with(visitor);
                value.visit_with(visitor);
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
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.name.visit_with(visitor);
        self.extends.visit_with(visitor);
        self.body.body.visit_with(visitor);
    }
}
