use ast::{Node, List, ExpressionList, StatementList, ExpressionNode};
use ast::{Literal, Pattern};

use ast::expression::*;
use ast::statement::*;

#[macro_use]
mod build;
mod function;
mod expression;
mod statement;


// Like Batman!
pub type NoParent = ();


build! {
    // meta
    enter_scope                   => StatementList<'ast>;
    leave_scope                   => StatementList<'ast>;

    // expressions
    on_this                       => ThisExpression;
    on_identifier                 => Identifier<'ast>;
    on_literal                    => Literal<'ast>;
    on_array_expression           => ArrayExpression<'ast>;
    on_arrow_expression           => ArrowExpression<'ast>;
    on_binary_expression          => BinaryExpression<'ast>;
    on_call_expression            => CallExpression<'ast>;
    on_class_expression           => ClassExpression<'ast>;
    on_computed_member_expression => ComputedMemberExpression<'ast>;
    on_conditional_expression     => ConditionalExpression<'ast>;
    on_function_expression        => FunctionExpression<'ast>;
    on_member_expression          => MemberExpression<'ast>;
    on_object_expression          => ObjectExpression<'ast>;
    on_postfix_expression         => PostfixExpression<'ast>;
    on_prefix_expression          => PrefixExpression<'ast>;
    on_sequence_expression        => SequenceExpression<'ast>;
    on_spread_expression          => SpreadExpression<'ast>;
    on_template_expression        => TemplateExpression<'ast>;

    // statements
    on_expression_statement  => ExpressionNode<'ast>;
    on_block_statement       => BlockStatement<'ast>;
    // on_break_statement       => BreakStatement;
    // on_continue_statement    => ContinueStatement;
    on_class_statement       => ClassStatement<'ast>;
    // on_declaration_statement => DeclarationStatement;
    // on_do_statement          => DoStatement;
    // on_for_in_statement      => ForInStatement;
    // on_for_init              => ForInit;
    // on_for_of_statement      => ForOfStatement;
    // on_for_statement         => ForStatement;
    on_function_statement    => FunctionStatement<'ast>;
    // on_if_statement          => IfStatement;
    // on_labeled_statement     => LabeledStatement;
    // on_return_statement      => ReturnStatement;
    // on_switch_case           => SwitchCase;
    // on_switch_statement      => SwitchStatement;
    // on_throw_statement       => ThrowStatement;
    // on_try_statement         => TryStatement;
    // on_while_statement       => WhileStatement;
}

pub trait Visitable<'ast> {
    type Parent;

    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context) where V: Visitor<'ast>;
}

impl<'ast> Visitable<'ast> for Pattern<'ast> {
    type Parent = Node<'ast, Self>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
        where V: Visitor<'ast>
    {
        unimplemented!();
    }
}

impl<'ast> Visitable<'ast> for Property<'ast> {
    type Parent = Node<'ast, Self>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
        where V: Visitor<'ast>
    {
        unimplemented!();
    }
}

impl<'ast, T> Visitable<'ast> for Option<T> where
    T: Visitable<'ast>
{
    type Parent = T::Parent;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
        where V: Visitor<'ast>
    {
        if let Some(ref visitable) = *self {
            visitable.visit(visitor, ctx);
        }
    }
}

// Requiring that `Parent = Node<'ast, T>` means that we avoid having
// a default implementation for (Expression|Statement)(Node|List)
impl<'ast, T> Visitable<'ast> for Node<'ast, T> where
    T: Visitable<'ast, Parent = Node<'ast, T>>,
{
    type Parent = NoParent;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
        where V: Visitor<'ast>
    {
        (**self).visit(visitor, ctx);
    }
}

impl<'ast, T> Visitable<'ast> for List<'ast, T> where
    T: Visitable<'ast, Parent = Node<'ast, T>>,
{
    type Parent = NoParent;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
        where V: Visitor<'ast>
    {
        for item in self {
            item.visit(visitor, ctx);
        }
    }
}

impl<'ast> Visitable<'ast> for ExpressionList<'ast> {
    type Parent = NoParent;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
        where V: Visitor<'ast>
    {
        for ptr in self.ptr_iter() {
            ptr.visit(visitor, ctx);
        }
    }
}

impl<'ast> Visitable<'ast> for StatementList<'ast> {
    type Parent = NoParent;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
        where V: Visitor<'ast>
    {
        visitor.enter_scope(self, &(), ctx);
        for ptr in self.ptr_iter() {
            ptr.visit(visitor, ctx);
        }
        visitor.leave_scope(self, &(), ctx);
    }
}
