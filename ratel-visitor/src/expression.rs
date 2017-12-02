use ratel::ast::{Identifier, Expression, ExpressionNode, StatementNode, Literal};
use ratel::ast::expression::*;

use {Visitor, Visitable};


impl<'ast> Visitable<'ast> for ExpressionNode<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        use self::Expression::*;

        match self.item {
            Void => {},
            This(_) => {
                visitor.on_this_expression(self, ctx);
            },
            Identifier(ref ident) => {
                ident.traverse(visitor, ctx);
                visitor.on_identifier_expression(ident, self, ctx);
            },
            Literal(ref literal) => {
                visitor.on_literal_expression(literal, self, ctx);
            },
            Sequence(ref sequence) => {
                sequence.traverse(visitor, ctx);
                visitor.on_sequence_expression(sequence, self, ctx);
            },
            Array(ref array) => {
                array.traverse(visitor, ctx);
                visitor.on_array_expression(array, self, ctx);
            },
            Member(ref member) => {
                member.traverse(visitor, ctx);
                visitor.on_member_expression(member, self, ctx);
            },
            ComputedMember(ref computed) => {
                computed.traverse(visitor, ctx);
                visitor.on_computed_member_expression(computed, self, ctx);
            },
            Call(ref call) => {
                call.traverse(visitor, ctx);
                visitor.on_call_expression(call, self, ctx);
            },
            Binary(ref binary) => {
                binary.traverse(visitor, ctx);
                visitor.on_binary_expression(binary, self, ctx);
            },
            Prefix(ref prefix) => {
                prefix.traverse(visitor, ctx);
                visitor.on_prefix_expression(prefix, self, ctx);
            },
            Postfix(ref postfix) => {
                postfix.traverse(visitor, ctx);
                visitor.on_postfix_expression(postfix, self, ctx);
            },
            Conditional(ref conditional) => {
                conditional.traverse(visitor, ctx);
                visitor.on_conditional_expression(conditional, self, ctx);
            },
            Template(ref template) => {
                template.traverse(visitor, ctx);
                visitor.on_template_literal(template, self, ctx);
            },
            TaggedTemplate(ref tagged) => {
                tagged.traverse(visitor, ctx);
                visitor.on_tagged_template_expression(tagged, self, ctx);
            },
            Spread(ref spread) => {
                spread.traverse(visitor, ctx);
                visitor.on_spread_expression(spread, self, ctx);
            },
            Arrow(ref arrow) => {
                arrow.traverse(visitor, ctx);
                visitor.on_arrow_expression(arrow, self, ctx);
            },
            Object(ref object) => {
                object.traverse(visitor, ctx);
                visitor.on_object_expression(object, self, ctx);
            },
            Function(ref function) => {
                function.traverse(visitor, ctx);
                visitor.on_function_expression(function, self, ctx);
            },
            Class(ref class) => {
                class.traverse(visitor, ctx);
                visitor.on_class_expression(class, self, ctx);
            }
        }
    }
}

impl<'ast> Visitable<'ast> for ThisExpression {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn traverse<V: Visitor<'ast>>(&self, _: &V, _: &mut V::Context) {}
}

impl<'ast> Visitable<'ast> for Identifier<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn traverse<V: Visitor<'ast>>(&self, visitor: &V, ctx: &mut V::Context) {
        visitor.on_variable_use(self, ctx);
    }
}

impl<'ast> Visitable<'ast> for Literal<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn traverse<V: Visitor<'ast>>(&self, _: &V, _: &mut V::Context) {}
}

impl<'ast> Visitable<'ast> for SequenceExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.body.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for ArrayExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.body.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for MemberExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.object.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for ComputedMemberExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.object.traverse(visitor, ctx);
        self.property.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for CallExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.callee.traverse(visitor, ctx);
        self.arguments.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for BinaryExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.left.traverse(visitor, ctx);
        self.right.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for PrefixExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.operand.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for PostfixExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.operand.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for ConditionalExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

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

impl<'ast> Visitable<'ast> for TemplateLiteral<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.expressions.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for TaggedTemplateExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.tag.traverse(visitor, ctx);
        self.quasi.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for SpreadExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.argument.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for ArrowBody<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        match *self {
            ArrowBody::Expression(ref expression) => expression.traverse(visitor, ctx),
            ArrowBody::Block(ref block)           => block.body.traverse(visitor, ctx),
        }
    }
}

impl<'ast> Visitable<'ast> for ArrowExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.params.traverse(visitor, ctx);
        self.body.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for ObjectExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn traverse<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.body.traverse(visitor, ctx);
    }
}
