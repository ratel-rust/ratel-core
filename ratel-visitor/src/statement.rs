use ratel::ast::{Node, Statement, StatementNode};
use ratel::ast::statement::*;

use {Visitor, Visitable, NoParent};


impl<'ast> Visitable<'ast> for StatementNode<'ast> {
    type Parent = NoParent;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        use self::Statement::*;

        match self.item {
            Empty => {},
            Expression(ref expression)   => {
                expression.traverse(visitor, ctx);
                visitor.on_expression_statement(expression, self, ctx);
            },
            Declaration(ref declaration) => {
                declaration.traverse(visitor, ctx);
                visitor.on_declaration_statement(declaration, self, ctx);
            },
            Return(ref return_statement) => {
                return_statement.traverse(visitor, ctx);
                visitor.on_return_statement(return_statement, self, ctx);
            },
            Break(ref break_statement) => {
                break_statement.traverse(visitor, ctx);
                visitor.on_break_statement(break_statement, self, ctx);
            },
            Continue(ref continue_statement) => {
                continue_statement.traverse(visitor, ctx);
                visitor.on_continue_statement(continue_statement, self, ctx);
            },
            Throw(ref throw) => {
                throw.traverse(visitor, ctx);
                visitor.on_throw_statement(throw, self, ctx);
            },
            If(ref if_statement) => {
                if_statement.traverse(visitor, ctx);
                visitor.on_if_statement(if_statement, self, ctx);
            },
            While(ref while_statement) => {
                while_statement.traverse(visitor, ctx);
                visitor.on_while_statement(while_statement, self, ctx);
            },
            Do(ref do_statement) => {
                do_statement.traverse(visitor, ctx);
                visitor.on_do_statement(do_statement, self, ctx);
            },
            For(ref for_statement) => {
                for_statement.traverse(visitor, ctx);
                visitor.on_for_statement(for_statement, self, ctx);
            },
            ForIn(ref for_in) => {
                for_in.traverse(visitor, ctx);
                visitor.on_for_in_statement(for_in, self, ctx);
            },
            ForOf(ref for_of) => {
                for_of.traverse(visitor, ctx);
                visitor.on_for_of_statement(for_of, self, ctx);
            },
            Try(ref try) => {
                try.traverse(visitor, ctx);
                visitor.on_try_statement(try, self, ctx);
            },
            Labeled(ref labeled) => {
                labeled.traverse(visitor, ctx);
                visitor.on_labeled_statement(labeled, self, ctx);
            },
            Block(ref block) => {
                block.traverse(visitor, ctx);
                visitor.on_block_statement(block, self, ctx);
            },
            Switch(ref switch) => {
                switch.traverse(visitor, ctx);
                visitor.on_switch_statement(switch, self, ctx);
            },
            Function(ref function) => {
                function.traverse(visitor, ctx);
                visitor.on_function_statement(function, self, ctx);
            },
            Class(ref class) => {
                class.traverse(visitor, ctx);
                visitor.on_class_statement(class, self, ctx);
            }
        }
    }
}

impl<'ast> Visitable<'ast> for BlockStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        visitor.on_enter_block(ctx);
        self.body.traverse(visitor, ctx);
        visitor.on_leave_block(ctx);
    }
}

impl<'ast> Visitable<'ast> for Node<'ast, BlockStatement<'ast>> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.item.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for Declarator<'ast> {
    type Parent = Node<'ast, Self>;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.id.traverse(visitor, ctx);
        self.init.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for DeclarationStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.declarators.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for ReturnStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.value.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for BreakStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn traverse<V: Visitor<'ast>>(&self, _visitor: &V, _ctx: &mut V::Context) {
        // FIXME:
        // INTENTIONALLY KEPT EMPTY FOR NOW!
        // The identifier here is a label reference, _not_ a variable!
    }
}

impl<'ast> Visitable<'ast> for ContinueStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn traverse<V: Visitor<'ast>>(&self, _visitor: &V, _ctx: &mut V::Context) {
        // FIXME:
        // INTENTIONALLY KEPT EMPTY FOR NOW!
        // The identifier here is a label reference, _not_ a variable!
    }
}

impl<'ast> Visitable<'ast> for ThrowStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.value.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for IfStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
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

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.test.traverse(visitor, ctx);
        self.body.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for DoStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.body.traverse(visitor, ctx);
        self.test.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for ForInit<'ast> {
    type Parent = Node<'ast, Self>;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
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

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
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

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
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

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
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

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.param.traverse(visitor, ctx);
        self.body.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for TryStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
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

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        // FIXME: newtype for label
        self.body.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for SwitchCase<'ast> {
    type Parent = Node<'ast, Self>;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.test.traverse(visitor, ctx);
        self.consequent.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for SwitchStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.discriminant.traverse(visitor, ctx);
        visitor.on_enter_block(ctx);
        self.cases.body.traverse(visitor, ctx);
        visitor.on_leave_block(ctx);
    }
}
