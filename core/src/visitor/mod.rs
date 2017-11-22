use ast::{Ptr, List, ExpressionList, StatementList, ExpressionPtr, Statement, StatementPtr, Block};
use ast::{OptionalName, MandatoryName, Function, Class, Literal};

use ast::expression::*;
use ast::statement::*;

type FunctionExpression<'ast> = Function<'ast, OptionalName<'ast>>;
type FunctionStatement<'ast> = Function<'ast, MandatoryName<'ast>>;
type ClassExpression<'ast> = Class<'ast, OptionalName<'ast>>;
type ClassStatement<'ast> = Class<'ast, MandatoryName<'ast>>;
type BlockStatement<'ast> = Block<'ast, Statement<'ast>>;
type Identifier<'ast> = &'ast str;

#[macro_use]
mod build;
mod expression;
mod statement;

pub struct Yes;
pub struct No;

build! {
    #[expressions]
    on_identifier                 => Identifier;
    on_literal                    => Literal;

    on_array_expression           => ArrayExpression;
    on_arrow_expression           => ArrowExpression;
    on_binary_expression          => BinaryExpression;
    on_call_expression            => CallExpression;
    on_class_expression           => ClassExpression;
    on_computed_member_expression => ComputedMemberExpression;
    on_conditional_expression     => ConditionalExpression;
    on_function_expression        => FunctionExpression;
    on_member_expression          => MemberExpression;
    on_object_expression          => ObjectExpression;
    on_postfix_expression         => PostfixExpression;
    on_prefix_expression          => PrefixExpression;
    on_sequence_expression        => SequenceExpression;
    on_spread_expression          => SpreadExpression;
    on_template_expression        => TemplateExpression;

    #[statements]
    on_expression_statement  => ExpressionPtr;
    on_block_statement       => BlockStatement;
    on_break_statement       => BreakStatement;
    on_continue_statement    => ContinueStatement;
    on_class_statement       => ClassStatement;
    on_declaration_statement => DeclarationStatement;
    on_do_statement          => DoStatement;
    on_for_in_statement      => ForInStatement;
    on_for_init              => ForInit;
    on_for_of_statement      => ForOfStatement;
    on_for_statement         => ForStatement;
    on_function_statement    => FunctionStatement;
    on_if_statement          => IfStatement;
    on_labeled_statement     => LabeledStatement;
    on_return_statement      => ReturnStatement;
    on_switch_case           => SwitchCase;
    on_switch_statement      => SwitchStatement;
    on_throw_statement       => ThrowStatement;
    on_try_statement         => TryStatement;
    on_while_statement       => WhileStatement;
}

pub trait Visitable<'ast> {
    type Iterable;

    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context) where V: Visitor<'ast>;
}

impl<'ast, T> Visitable<'ast> for Option<T> where
    T: Visitable<'ast>
{
    type Iterable = T::Iterable;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
        where V: Visitor<'ast>
    {
        if let Some(ref visitable) = *self {
            visitable.visit(visitor, ctx);
        }
    }
}

impl<'ast, T> Visitable<'ast> for Ptr<'ast, T> where
    // P: Visitable<'ast> + 'ast,
    T: Visitable<'ast, Iterable = Yes>,
{
    type Iterable = No;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
        where V: Visitor<'ast>
    {
        (**self).visit(visitor, ctx);
    }
}

impl<'ast, T> Visitable<'ast> for List<'ast, T> where
    // P: Visitable<'ast> + 'ast,
    T: Visitable<'ast, Iterable = Yes>,
{
    type Iterable = No;

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
    type Iterable = No;

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
    type Iterable = No;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
        where V: Visitor<'ast>
    {
        for ptr in self.ptr_iter() {
            ptr.visit(visitor, ctx);
        }
    }
}

// impl<'ast, T> Visitable<'ast> for BlockPtr<'ast, T> where
//     T: Visitable<'ast>,
// {
//     #[inline]
//     fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
//         where V: Visitor<'ast>
//     {
//         self.body.visit(visitor, ctx);
//     }
// }

// impl<'ast> Visitable<'ast> for ExpressionList<'ast> {
//     #[inline]
//     fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
//         where V: Visitor<'ast>
//     {
//         for item in self.ptr_iter() {
//             item.visit(visitor, ctx);
//         }
//     }
// }

// impl<'ast> Visitable<'ast> for StatementList<'ast> {
//     #[inline]
//     fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
//         where V: Visitor<'ast>
//     {
//         for item in self.ptr_iter() {
//             item.visit(visitor, ctx);
//         }
//     }
// }
