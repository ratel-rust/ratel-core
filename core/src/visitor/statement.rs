use visitor::{Visitor, Visitable, NoParent};
use ast::{Node, Statement, StatementNode};
use ast::statement::*;


impl<'ast> Visitable<'ast> for StatementNode<'ast> {
    type Parent = NoParent;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        use self::Statement::*;

        match self.item {
            Empty => {},
            Expression(ref expression)   => {
                expression.visit(visitor, ctx);
                visitor.on_expression_statement(expression, self, ctx);
            },
            Declaration(ref declaration) => {
                declaration.visit(visitor, ctx);
                visitor.on_declaration_statement(declaration, self, ctx);
            },
            Return(ref return_statement) => {
                return_statement.visit(visitor, ctx);
                visitor.on_return_statement(return_statement, self, ctx);
            },
            Break(ref break_statement) => {
                break_statement.visit(visitor, ctx);
                visitor.on_break_statement(break_statement, self, ctx);
            },
            Continue(ref continue_statement) => {
                continue_statement.visit(visitor, ctx);
                visitor.on_continue_statement(continue_statement, self, ctx);
            },
            Throw(ref throw) => {
                throw.visit(visitor, ctx);
                visitor.on_throw_statement(throw, self, ctx);
            },
            If(ref if_statement) => {
                if_statement.visit(visitor, ctx);
                visitor.on_if_statement(if_statement, self, ctx);
            },
            While(ref while_statement) => {
                while_statement.visit(visitor, ctx);
                visitor.on_while_statement(while_statement, self, ctx);
            },
            Do(ref do_statement) => {
                do_statement.visit(visitor, ctx);
                visitor.on_do_statement(do_statement, self, ctx);
            },
            For(ref for_statement) => {
                for_statement.visit(visitor, ctx);
                visitor.on_for_statement(for_statement, self, ctx);
            },
            ForIn(ref for_in) => {
                for_in.visit(visitor, ctx);
                visitor.on_for_in_statement(for_in, self, ctx);
            },
            ForOf(ref for_of) => {
                for_of.visit(visitor, ctx);
                visitor.on_for_of_statement(for_of, self, ctx);
            },
            Try(ref try) => {
                try.visit(visitor, ctx);
                visitor.on_try_statement(try, self, ctx);
            },
            Labeled(ref labeled) => {
                labeled.visit(visitor, ctx);
                visitor.on_labeled_statement(labeled, self, ctx);
            },
            Block(ref block) => {
                block.visit(visitor, ctx);
                visitor.on_block_statement(block, self, ctx);
            },
            Switch(ref switch) => {
                switch.visit(visitor, ctx);
                visitor.on_switch_statement(switch, self, ctx);
            },
            Function(ref function) => {
                function.visit(visitor, ctx);
                visitor.on_function_statement(function, self, ctx);
            },
            Class(ref class) => {
                class.visit(visitor, ctx);
                visitor.on_class_statement(class, self, ctx);
            }
        }
    }
}

impl<'ast> Visitable<'ast> for BlockStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        visitor.on_enter_block(ctx);
        self.body.visit(visitor, ctx);
        visitor.on_leave_block(ctx);
    }
}

impl<'ast> Visitable<'ast> for Node<'ast, BlockStatement<'ast>> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.item.visit(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for Declarator<'ast> {
    type Parent = Node<'ast, Self>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.id.visit(visitor, ctx);
        self.init.visit(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for DeclarationStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.declarators.visit(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for ReturnStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.value.visit(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for BreakStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit<V: Visitor<'ast>>(&self, _visitor: &V, _ctx: &mut V::Context) {
        // FIXME:
        // INTENTIONALLY KEPT EMPTY FOR NOW!
        // The identifier here is a label reference, _not_ a variable!
    }
}

impl<'ast> Visitable<'ast> for ContinueStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit<V: Visitor<'ast>>(&self, _visitor: &V, _ctx: &mut V::Context) {
        // FIXME:
        // INTENTIONALLY KEPT EMPTY FOR NOW!
        // The identifier here is a label reference, _not_ a variable!
    }
}

impl<'ast> Visitable<'ast> for ThrowStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.value.visit(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for IfStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.test.visit(visitor, ctx);
        self.consequent.visit(visitor, ctx);
        self.alternate.visit(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for WhileStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.test.visit(visitor, ctx);
        self.body.visit(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for DoStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.body.visit(visitor, ctx);
        self.test.visit(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for ForInit<'ast> {
    type Parent = Node<'ast, Self>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        match *self {
            ForInit::Declaration(ref declaration) => declaration.visit(visitor, ctx),
            ForInit::Expression(ref expression) => expression.visit(visitor, ctx),
        }
    }
}

impl<'ast> Visitable<'ast> for ForStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.init.visit(visitor, ctx);
        self.test.visit(visitor, ctx);
        self.update.visit(visitor, ctx);
        self.body.visit(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for ForInStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.left.visit(visitor, ctx);
        self.right.visit(visitor, ctx);
        self.body.visit(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for ForOfStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.left.visit(visitor, ctx);
        self.right.visit(visitor, ctx);
        self.body.visit(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for CatchClause<'ast> {
    type Parent = Node<'ast, Self>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.param.visit(visitor, ctx);
        self.body.visit(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for TryStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.block.visit(visitor, ctx);
        self.handler.visit(visitor, ctx);
        self.finalizer.visit(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for LabeledStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        // FIXME: newtype for label
        self.body.visit(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for SwitchCase<'ast> {
    type Parent = Node<'ast, Self>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.test.visit(visitor, ctx);
        self.consequent.visit(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for SwitchStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.discriminant.visit(visitor, ctx);
        visitor.on_enter_block(ctx);
        self.cases.body.visit(visitor, ctx);
        visitor.on_leave_block(ctx);
    }
}
