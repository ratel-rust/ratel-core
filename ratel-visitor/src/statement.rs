use ratel::ast::{Node, Statement, StatementNode};
use ratel::ast::statement::*;

use {Visitor, Visitable, ParentNode, ScopeKind, NoParent};


impl<'ast> Visitable<'ast> for StatementNode<'ast> {
    type Parent = NoParent;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
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
                visitor.on_expression_statement(expression, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                expression.traverse(visitor, ctx);
            },
            Declaration(ref declaration) => {
                visitor.on_declaration_statement(declaration, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                declaration.traverse(visitor, ctx);
            },
            Return(ref return_statement) => {
                visitor.on_return_statement(return_statement, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                return_statement.traverse(visitor, ctx);
            },
            Break(ref break_statement) => {
                visitor.on_break_statement(break_statement, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                break_statement.traverse(visitor, ctx);
            },
            Continue(ref continue_statement) => {
                visitor.on_continue_statement(continue_statement, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                continue_statement.traverse(visitor, ctx);
            },
            Throw(ref throw) => {
                visitor.on_throw_statement(throw, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                throw.traverse(visitor, ctx);
            },
            If(ref if_statement) => {
                visitor.on_if_statement(if_statement, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                if_statement.traverse(visitor, ctx);
            },
            While(ref while_statement) => {
                visitor.on_while_statement(while_statement, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                while_statement.traverse(visitor, ctx);
            },
            Do(ref do_statement) => {
                visitor.on_do_statement(do_statement, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                do_statement.traverse(visitor, ctx);
            },
            For(ref for_statement) => {
                visitor.on_for_statement(for_statement, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                for_statement.traverse(visitor, ctx);
            },
            ForIn(ref for_in) => {
                visitor.on_for_in_statement(for_in, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                for_in.traverse(visitor, ctx);
            },
            ForOf(ref for_of) => {
                visitor.on_for_of_statement(for_of, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                for_of.traverse(visitor, ctx);
            },
            Try(ref try) => {
                visitor.on_try_statement(try, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                try.traverse(visitor, ctx);
            },
            Labeled(ref labeled) => {
                visitor.on_labeled_statement(labeled, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                labeled.traverse(visitor, ctx);
            },
            Block(ref block) => {
                visitor.on_block_statement(block, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                block.traverse(visitor, ctx);
            },
            Switch(ref switch) => {
                visitor.on_switch_statement(switch, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                switch.traverse(visitor, ctx);
            },
            Function(ref function) => {
                visitor.on_function_statement(function, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                function.traverse(visitor, ctx);
            },
            Class(ref class) => {
                visitor.on_class_statement(class, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                class.traverse(visitor, ctx);
            }
        }
        visitor.pop_parent(ctx);
    }
}

impl<'ast> Visitable<'ast> for BlockStatement<'ast> {
    type Parent = StatementNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        visitor.on_enter_scope(ScopeKind::Block, ctx);
        self.body.traverse(visitor, ctx);
        visitor.on_leave_scope(ctx);
    }
}

impl<'ast> Visitable<'ast> for Node<'ast, BlockStatement<'ast>> {
    type Parent = StatementNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.item.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for Declarator<'ast> {
    type Parent = Node<'ast, Self>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.id.traverse(visitor, ctx);
        self.init.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for DeclarationStatement<'ast> {
    type Parent = StatementNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.declarators.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for ReturnStatement<'ast> {
    type Parent = StatementNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.value.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for BreakStatement<'ast> {
    type Parent = StatementNode<'ast>;

    fn traverse<V: Visitor<'ast>>(&self, _visitor: &V, _ctx: &mut V::Context) {
        // FIXME:
        // INTENTIONALLY KEPT EMPTY FOR NOW!
        // The identifier here is a label reference, _not_ a variable!
    }
}

impl<'ast> Visitable<'ast> for ContinueStatement<'ast> {
    type Parent = StatementNode<'ast>;

    fn traverse<V: Visitor<'ast>>(&self, _visitor: &V, _ctx: &mut V::Context) {
        // FIXME:
        // INTENTIONALLY KEPT EMPTY FOR NOW!
        // The identifier here is a label reference, _not_ a variable!
    }
}

impl<'ast> Visitable<'ast> for ThrowStatement<'ast> {
    type Parent = StatementNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.value.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for IfStatement<'ast> {
    type Parent = StatementNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.test.traverse(visitor, ctx);
        self.consequent.traverse(visitor, ctx);
        self.alternate.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for WhileStatement<'ast> {
    type Parent = StatementNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.test.traverse(visitor, ctx);
        self.body.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for DoStatement<'ast> {
    type Parent = StatementNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.body.traverse(visitor, ctx);
        self.test.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for ForInit<'ast> {
    type Parent = Node<'ast, Self>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        match *self {
            ForInit::Declaration(ref declaration) => declaration.traverse(visitor, ctx),
            ForInit::Expression(ref expression) => expression.traverse(visitor, ctx),
        }
    }
}

impl<'ast> Visitable<'ast> for ForStatement<'ast> {
    type Parent = StatementNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.init.traverse(visitor, ctx);
        self.test.traverse(visitor, ctx);
        self.update.traverse(visitor, ctx);
        self.body.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for ForInStatement<'ast> {
    type Parent = StatementNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.left.traverse(visitor, ctx);
        self.right.traverse(visitor, ctx);
        self.body.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for ForOfStatement<'ast> {
    type Parent = StatementNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.left.traverse(visitor, ctx);
        self.right.traverse(visitor, ctx);
        self.body.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for CatchClause<'ast> {
    type Parent = Node<'ast, Self>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.param.traverse(visitor, ctx);
        self.body.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for TryStatement<'ast> {
    type Parent = StatementNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.block.traverse(visitor, ctx);
        self.handler.traverse(visitor, ctx);
        self.finalizer.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for LabeledStatement<'ast> {
    type Parent = StatementNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        // FIXME: newtype for label
        self.body.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for SwitchCase<'ast> {
    type Parent = Node<'ast, Self>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.test.traverse(visitor, ctx);
        self.consequent.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for SwitchStatement<'ast> {
    type Parent = StatementNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.discriminant.traverse(visitor, ctx);
        visitor.on_enter_scope(ScopeKind::Block, ctx);
        self.cases.body.traverse(visitor, ctx);
        visitor.on_leave_scope(ctx);
    }
}
