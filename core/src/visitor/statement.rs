use visitor::{Visitor, Visitable, NoParent};
use ast::{Statement, StatementNode};
use ast::statement::*;


impl<'ast> Visitable<'ast> for StatementNode<'ast> {
    type Parent = NoParent;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
        where V: Visitor<'ast>
    {
        use self::Statement::*;

        match self.item {
            Empty => {},
            Expression(ref expression)   => {
                expression.visit(visitor, ctx);
                visitor.on_expression_statement(expression, self, ctx);
            },
            // Declaration(ref declaration) => visitor.on_declaration_statement(declaration, self, ctx),
            // Return(ref return_statement) => visitor.on_return_statement(return_statement, self, ctx),
            // Break(ref break_statement)   => visitor.on_break_statement(break_statement, self, ctx),
            // Throw(ref throw)             => visitor.on_throw_statement(throw, self, ctx),
            // If(ref if_statement)         => visitor.on_if_statement(if_statement, self, ctx),
            // While(ref while_statement)   => visitor.on_while_statement(while_statement, self, ctx),
            // Do(ref do_statement)         => visitor.on_do_statement(do_statement, self, ctx),
            // For(ref for_statement)       => visitor.on_for_statement(for_statement, self, ctx),
            // ForIn(ref for_in)            => visitor.on_for_in_statement(for_in, self, ctx),
            // ForOf(ref for_of)            => visitor.on_for_of_statement(for_of, self, ctx),
            // Try(ref try)                 => visitor.on_try_statement(try, self, ctx),
            // Labeled(ref labeled)         => visitor.on_labeled_statement(labeled, self, ctx),
            Block(ref block)             => {
                block.visit(visitor, ctx);
                visitor.on_block_statement(block, self, ctx);
            },
            // Continue(ref cont)           => visitor.on_continue_statement(cont, self, ctx),
            // Switch(ref switch)           => visitor.on_switch_statement(switch, self, ctx),
            Function(ref function)       => {
                function.visit(visitor, ctx);
                visitor.on_function_statement(function, self, ctx);
            },
            Class(ref class)             => {
                class.visit(visitor, ctx);
                visitor.on_class_statement(class, self, ctx);
            }
            _ => unimplemented!()
        }
    }
}

impl<'ast> Visitable<'ast> for BlockStatement<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit<V: Visitor<'ast>>(&self, visitor: &V, ctx: &mut V::Context) {
        self.body.visit(visitor, ctx);
    }
}
