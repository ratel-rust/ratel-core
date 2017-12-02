use ast::{Node, NodeList, Identifier, Literal, Pattern};
use ast::{ExpressionList, StatementList, ExpressionNode, StatementNode};

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
    // scope control
    fn on_enter_block(body: StatementList<'ast>);
    fn on_variable_use(ident: &Identifier<'ast>);
    fn on_variable_declare(ident: &Identifier<'ast>);
    fn on_leave_block();

    // expressions
    fn on_this_expression(node: &ExpressionNode<'ast>);
    fn on_identifier_expression(item: &Identifier<'ast>, node: &ExpressionNode<'ast>);
    fn on_literal_expression(item: &Literal<'ast>, node: &ExpressionNode<'ast>);
    fn on_sequence_expression(item: &SequenceExpression<'ast>, node: &ExpressionNode<'ast>);
    fn on_array_expression(item: &ArrayExpression<'ast>, node: &ExpressionNode<'ast>);
    fn on_member_expression(item: &MemberExpression<'ast>, node: &ExpressionNode<'ast>);
    fn on_computed_member_expression(item: &ComputedMemberExpression<'ast>, node: &ExpressionNode<'ast>);
    fn on_call_expression(item: &CallExpression<'ast>, node: &ExpressionNode<'ast>);
    fn on_binary_expression(item: &BinaryExpression<'ast>, node: &ExpressionNode<'ast>);
    fn on_prefix_expression(item: &PrefixExpression<'ast>, node: &ExpressionNode<'ast>);
    fn on_postfix_expression(item: &PostfixExpression<'ast>, node: &ExpressionNode<'ast>);
    fn on_conditional_expression(item: &ConditionalExpression<'ast>, node: &ExpressionNode<'ast>);
    fn on_template_literal(item: &TemplateLiteral<'ast>, node: &ExpressionNode<'ast>);
    fn on_tagged_template_expression(item: &TaggedTemplateExpression<'ast>, node: &ExpressionNode<'ast>);
    fn on_spread_expression(item: &SpreadExpression<'ast>, node: &ExpressionNode<'ast>);
    fn on_arrow_expression(item: &ArrowExpression<'ast>, node: &ExpressionNode<'ast>);
    fn on_object_expression(item: &ObjectExpression<'ast>, node: &ExpressionNode<'ast>);
    fn on_function_expression(item: &FunctionExpression<'ast>, node: &ExpressionNode<'ast>);
    fn on_class_expression(item: &ClassExpression<'ast>, node: &ExpressionNode<'ast>);

    // statements
    fn on_expression_statement(item: &ExpressionNode<'ast>, node: &StatementNode<'ast>);
    fn on_declaration_statement(item: &DeclarationStatement, node: &StatementNode<'ast>);
    fn on_return_statement(item: &ReturnStatement, node: &StatementNode<'ast>);
    fn on_break_statement(item: &BreakStatement, node: &StatementNode<'ast>);
    fn on_continue_statement(item: &ContinueStatement, node: &StatementNode<'ast>);
    fn on_throw_statement(item: &ThrowStatement, node: &StatementNode<'ast>);
    fn on_if_statement(item: &IfStatement, node: &StatementNode<'ast>);
    fn on_while_statement(item: &WhileStatement, node: &StatementNode<'ast>);
    fn on_do_statement(item: &DoStatement, node: &StatementNode<'ast>);
    fn on_for_statement(item: &ForStatement, node: &StatementNode<'ast>);
    fn on_for_in_statement(item: &ForInStatement, node: &StatementNode<'ast>);
    fn on_for_of_statement(item: &ForOfStatement, node: &StatementNode<'ast>);
    fn on_try_statement(item: &TryStatement, node: &StatementNode<'ast>);
    fn on_block_statement(item: &BlockStatement<'ast>, node: &StatementNode<'ast>);
    fn on_labeled_statement(item: &LabeledStatement, node: &StatementNode<'ast>);
    fn on_switch_statement(item: &SwitchStatement, node: &StatementNode<'ast>);
    fn on_function_statement(item: &FunctionStatement<'ast>, node: &StatementNode<'ast>);
    fn on_class_statement(item: &ClassStatement<'ast>, node: &StatementNode<'ast>);
}

pub trait Visitable<'ast> {
    type Parent;

    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context) where V: Visitor<'ast>;
}

impl<'ast> Visitable<'ast> for Pattern<'ast> {
    type Parent = Node<'ast, Self>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        match *self {
            Pattern::Void => {},
            Pattern::Identifier(ref ident) => visitor.on_variable_declare(ident, ctx),
            Pattern::ObjectPattern {
                ref properties,
            } => {
                properties.visit(visitor, ctx);
            },
            Pattern::ArrayPattern {
                ref elements,
            } => {
                elements.visit(visitor, ctx);
            },
            Pattern::RestElement {
                ref argument,
            } => {
                argument.visit(visitor, ctx);
            },
            Pattern::AssignmentPattern {
                ref left,
                ref right,
            } => {
                left.visit(visitor, ctx);
                right.visit(visitor, ctx);
            }
        }
    }
}

impl<'ast> Visitable<'ast> for PropertyKey<'ast> {
    type Parent = Node<'ast, Self>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        match *self {
            PropertyKey::Computed(ref expression) => expression.visit(visitor, ctx),
            PropertyKey::Literal(_) | PropertyKey::Binary(_) => {},
        }
    }
}

impl<'ast> Visitable<'ast> for Property<'ast> {
    type Parent = Node<'ast, Self>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        match *self {
            Property::Shorthand(ref ident) => visitor.on_variable_use(ident, ctx),
            Property::Literal {
                ref key,
                ref value,
            } => {
                key.visit(visitor, ctx);
                value.visit(visitor, ctx);
            },
            Property::Method {
                ref key,
                ref value,
            } => {
                key.visit(visitor, ctx);
                value.visit(visitor, ctx);
            }
        }
    }
}

impl<'ast, T> Visitable<'ast> for Option<T> where
    T: Visitable<'ast>
{
    type Parent = T::Parent;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
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
    where
        V: Visitor<'ast>,
    {
        (**self).visit(visitor, ctx);
    }
}

impl<'ast, T> Visitable<'ast> for NodeList<'ast, T> where
    T: Visitable<'ast, Parent = Node<'ast, T>>,
{
    type Parent = NoParent;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
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
    where
        V: Visitor<'ast>,
    {
        for node in self.iter() {
            node.visit(visitor, ctx);
        }
    }
}

impl<'ast> Visitable<'ast> for StatementList<'ast> {
    type Parent = NoParent;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        visitor.on_enter_block(*self, ctx);
        for node in self.iter() {
            node.visit(visitor, ctx);
        }
        visitor.on_leave_block(ctx);
    }
}
