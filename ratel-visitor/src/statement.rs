use ratel::ast::{Node, Statement, StatementNode};
use ratel::ast::statement::*;

use {Visitor, Visitable, ParentNode, ScopeKind, NoParent};


impl<'ast> Visitable<'ast> for StatementNode<'ast> {
    type Parent = NoParent;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        use self::Statement::*;

        match self.item {
            Empty => { 
                // EmptyStatement doesn't have children, we return early to avoid calling pop_parent
                return;
            },
            Expression(ref expression) => {
                visitor.on_expression_statement(expression, self);
                visitor.push_parent(ParentNode::from(self));
                expression.visit_with(visitor);
            },
            Declaration(ref declaration) => {
                visitor.on_declaration_statement(declaration, self);
                visitor.push_parent(ParentNode::from(self));
                declaration.visit_with(visitor);
            },
            Return(ref return_statement) => {
                visitor.on_return_statement(return_statement, self);
                visitor.push_parent(ParentNode::from(self));
                return_statement.visit_with(visitor);
            },
            Break(ref break_statement) => {
                visitor.on_break_statement(break_statement, self);
                visitor.push_parent(ParentNode::from(self));
                break_statement.visit_with(visitor);
            },
            Continue(ref continue_statement) => {
                visitor.on_continue_statement(continue_statement, self);
                visitor.push_parent(ParentNode::from(self));
                continue_statement.visit_with(visitor);
            },
            Throw(ref throw) => {
                visitor.on_throw_statement(throw, self);
                visitor.push_parent(ParentNode::from(self));
                throw.visit_with(visitor);
            },
            If(ref if_statement) => {
                visitor.on_if_statement(if_statement, self);
                visitor.push_parent(ParentNode::from(self));
                if_statement.visit_with(visitor);
            },
            While(ref while_statement) => {
                visitor.on_while_statement(while_statement, self);
                visitor.push_parent(ParentNode::from(self));
                while_statement.visit_with(visitor);
            },
            Do(ref do_statement) => {
                visitor.on_do_statement(do_statement, self);
                visitor.push_parent(ParentNode::from(self));
                do_statement.visit_with(visitor);
            },
            For(ref for_statement) => {
                visitor.on_for_statement(for_statement, self);
                visitor.push_parent(ParentNode::from(self));
                for_statement.visit_with(visitor);
            },
            ForIn(ref for_in) => {
                visitor.on_for_in_statement(for_in, self);
                visitor.push_parent(ParentNode::from(self));
                for_in.visit_with(visitor);
            },
            ForOf(ref for_of) => {
                visitor.on_for_of_statement(for_of, self);
                visitor.push_parent(ParentNode::from(self));
                for_of.visit_with(visitor);
            },
            Try(ref try) => {
                visitor.on_try_statement(try, self);
                visitor.push_parent(ParentNode::from(self));
                try.visit_with(visitor);
            },
            Labeled(ref labeled) => {
                visitor.on_labeled_statement(labeled, self);
                visitor.push_parent(ParentNode::from(self));
                labeled.visit_with(visitor);
            },
            Block(ref block) => {
                visitor.on_block_statement(block, self);
                visitor.push_parent(ParentNode::from(self));
                block.visit_with(visitor);
            },
            Switch(ref switch) => {
                visitor.on_switch_statement(switch, self);
                visitor.push_parent(ParentNode::from(self));
                switch.visit_with(visitor);
            },
            Function(ref function) => {
                visitor.on_function_statement(function, self);
                visitor.push_parent(ParentNode::from(self));
                function.visit_with(visitor);
            },
            Class(ref class) => {
                visitor.on_class_statement(class, self);
                visitor.push_parent(ParentNode::from(self));
                class.visit_with(visitor);
            }
        }
        visitor.pop_parent();
    }
}

impl<'ast> Visitable<'ast> for BlockStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        visitor.on_enter_scope(ScopeKind::Block);
        self.body.visit_with(visitor);
        visitor.on_leave_scope();
    }
}

impl<'ast> Visitable<'ast> for Node<'ast, BlockStatement<'ast>> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.item.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for Declarator<'ast> {
    type Parent = Node<'ast, Self>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.id.visit_with(visitor);
        self.init.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for DeclarationStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.declarators.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for ReturnStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.value.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for BreakStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit_with<V: Visitor<'ast>>(&self, _visitor: &mut V) {
        // FIXME:
        // INTENTIONALLY KEPT EMPTY FOR NOW!
        // The identifier here is a label reference, _not_ a variable!
    }
}

impl<'ast> Visitable<'ast> for ContinueStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit_with<V: Visitor<'ast>>(&self, _visitor: &mut V) {
        // FIXME:
        // INTENTIONALLY KEPT EMPTY FOR NOW!
        // The identifier here is a label reference, _not_ a variable!
    }
}

impl<'ast> Visitable<'ast> for ThrowStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.value.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for IfStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.test.visit_with(visitor);
        self.consequent.visit_with(visitor);
        self.alternate.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for WhileStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.test.visit_with(visitor);
        self.body.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for DoStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.body.visit_with(visitor);
        self.test.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for ForInit<'ast> {
    type Parent = Node<'ast, Self>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        match *self {
            ForInit::Declaration(ref declaration) => declaration.visit_with(visitor),
            ForInit::Expression(ref expression) => expression.visit_with(visitor),
        }
    }
}

impl<'ast> Visitable<'ast> for ForStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.init.visit_with(visitor);
        self.test.visit_with(visitor);
        self.update.visit_with(visitor);
        self.body.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for ForInStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.left.visit_with(visitor);
        self.right.visit_with(visitor);
        self.body.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for ForOfStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.left.visit_with(visitor);
        self.right.visit_with(visitor);
        self.body.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for CatchClause<'ast> {
    type Parent = Node<'ast, Self>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.param.visit_with(visitor);
        self.body.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for TryStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.block.visit_with(visitor);
        self.handler.visit_with(visitor);
        self.finalizer.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for LabeledStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        // FIXME: newtype for label
        self.body.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for SwitchCase<'ast> {
    type Parent = Node<'ast, Self>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.test.visit_with(visitor);
        self.consequent.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for SwitchStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.discriminant.visit_with(visitor);
        visitor.on_enter_scope(ScopeKind::Block);
        self.cases.body.visit_with(visitor);
        visitor.on_leave_scope();
    }
}
