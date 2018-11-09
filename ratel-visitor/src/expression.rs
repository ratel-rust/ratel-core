use ratel::ast::{Identifier, Expression, ExpressionNode, StatementNode, Literal};
use ratel::ast::expression::*;

use {Visitor, Visitable, ParentNode};


impl<'ast> Visitable<'ast> for ExpressionNode<'ast> {
    type Parent = StatementNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        use self::Expression::*;

        match self.item {
            Void => {
                // Void doesn't have children, we return early to avoid calling pop_parent
                return;
            },
            This(_) => {
                visitor.on_this_expression(&self, ctx);
                return;
            },
            Identifier(ref ident) => {
                visitor.on_identifier_expression(ident, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                ident.traverse(visitor, ctx);
            },
            Literal(ref literal) => {
                visitor.on_literal_expression(literal, self, ctx);
                return;
            },
            Sequence(ref sequence) => {
                visitor.on_sequence_expression(sequence, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                sequence.traverse(visitor, ctx);
            },
            Array(ref array) => {
                visitor.on_array_expression(array, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                array.traverse(visitor, ctx);
            },
            Member(ref member) => {
                visitor.on_member_expression(member, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                member.traverse(visitor, ctx);
            },
            ComputedMember(ref computed) => {
                visitor.on_computed_member_expression(computed, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                computed.traverse(visitor, ctx);
            },
            MetaProperty(ref property) => {
                visitor.on_meta_property(property, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                property.traverse(visitor, ctx);
            },
            Call(ref call) => {
                visitor.on_call_expression(call, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                call.traverse(visitor, ctx);
            },
            Binary(ref binary) => {
                visitor.on_binary_expression(binary, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                binary.traverse(visitor, ctx);
            },
            Prefix(ref prefix) => {
                visitor.on_prefix_expression(prefix, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                prefix.traverse(visitor, ctx);
            },
            Postfix(ref postfix) => {
                visitor.on_postfix_expression(postfix, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                postfix.traverse(visitor, ctx);
            },
            Conditional(ref conditional) => {
                visitor.on_conditional_expression(conditional, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                conditional.traverse(visitor, ctx);
            },
            Template(ref template) => {
                visitor.on_template_literal(template, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                template.traverse(visitor, ctx);
            },
            TaggedTemplate(ref tagged) => {
                visitor.on_tagged_template_expression(tagged, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                tagged.traverse(visitor, ctx);
            },
            Spread(ref spread) => {
                visitor.on_spread_expression(spread, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                spread.traverse(visitor, ctx);
            },
            Arrow(ref arrow) => {
                visitor.on_arrow_expression(arrow, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                arrow.traverse(visitor, ctx);
            },
            Object(ref object) => {
                visitor.on_object_expression(object, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                object.traverse(visitor, ctx);
            },
            Function(ref function) => {
                visitor.on_function_expression(function, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                function.traverse(visitor, ctx);
            },
            Class(ref class) => {
                visitor.on_class_expression(class, self, ctx);
                visitor.push_parent(ParentNode::from(self), ctx);
                class.traverse(visitor, ctx);
            },
            Yield(ref expression) => {
                visitor.on_yield_expression(expression, self, ctx);
                expression.argument.traverse(visitor, ctx);
            }
        }
        visitor.pop_parent(ctx);
    }
}

impl<'ast> Visitable<'ast> for ThisExpression {
    type Parent = ExpressionNode<'ast>;

    fn traverse<V: Visitor<'ast>>(&self, _: &V, _: &mut V::Context) {}
}

impl<'ast> Visitable<'ast> for Identifier<'ast> {
    type Parent = ExpressionNode<'ast>;

    fn traverse<V: Visitor<'ast>>(&self, visitor: &V, ctx: &mut V::Context) {
        visitor.on_reference_use(self, ctx);
    }
}

impl<'ast> Visitable<'ast> for Literal<'ast> {
    type Parent = ExpressionNode<'ast>;

    fn traverse<V: Visitor<'ast>>(&self, _: &V, _: &mut V::Context) {}
}

impl<'ast> Visitable<'ast> for SequenceExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.body.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for ArrayExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.body.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for MemberExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.object.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for ComputedMemberExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.object.traverse(visitor, ctx);
        self.property.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for MetaPropertyExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.meta.traverse(visitor, ctx);
        self.property.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for CallExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.callee.traverse(visitor, ctx);
        self.arguments.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for BinaryExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.left.traverse(visitor, ctx);
        self.right.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for PrefixExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.operand.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for PostfixExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.operand.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for ConditionalExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
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

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.expressions.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for TaggedTemplateExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.tag.traverse(visitor, ctx);
        self.quasi.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for SpreadExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.argument.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for ArrowBody<'ast> {
    type Parent = ExpressionNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
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

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.params.traverse(visitor, ctx);
        self.body.traverse(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for ObjectExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.body.traverse(visitor, ctx);
    }
}
