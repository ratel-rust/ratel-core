use ratel::ast::{Identifier, Expression, ExpressionNode, StatementNode, Literal};
use ratel::ast::expression::*;

use {Visitor, Visitable, ParentNode};


impl<'ast> Visitable<'ast> for ExpressionNode<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
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
                visitor.on_this_expression(&self);
                return;
            },
            Identifier(ref ident) => {
                visitor.on_identifier_expression(ident, self);
                visitor.push_parent(ParentNode::from(self));
                ident.visit_with(visitor);
            },
            Literal(ref literal) => {
                visitor.on_literal_expression(literal, self);
                return;
            },
            Sequence(ref sequence) => {
                visitor.on_sequence_expression(sequence, self);
                visitor.push_parent(ParentNode::from(self));
                sequence.visit_with(visitor);
            },
            Array(ref array) => {
                visitor.on_array_expression(array, self);
                visitor.push_parent(ParentNode::from(self));
                array.visit_with(visitor);
            },
            Member(ref member) => {
                visitor.on_member_expression(member, self);
                visitor.push_parent(ParentNode::from(self));
                member.visit_with(visitor);
            },
            ComputedMember(ref computed) => {
                visitor.on_computed_member_expression(computed, self);
                visitor.push_parent(ParentNode::from(self));
                computed.visit_with(visitor);
            },
            MetaProperty(ref property) => {
                visitor.on_meta_property(property, self);
                visitor.push_parent(ParentNode::from(self));
                property.visit_with(visitor);
            },
            Call(ref call) => {
                visitor.on_call_expression(call, self);
                visitor.push_parent(ParentNode::from(self));
                call.visit_with(visitor);
            },
            Binary(ref binary) => {
                visitor.on_binary_expression(binary, self);
                visitor.push_parent(ParentNode::from(self));
                binary.visit_with(visitor);
            },
            Prefix(ref prefix) => {
                visitor.on_prefix_expression(prefix, self);
                visitor.push_parent(ParentNode::from(self));
                prefix.visit_with(visitor);
            },
            Postfix(ref postfix) => {
                visitor.on_postfix_expression(postfix, self);
                visitor.push_parent(ParentNode::from(self));
                postfix.visit_with(visitor);
            },
            Conditional(ref conditional) => {
                visitor.on_conditional_expression(conditional, self);
                visitor.push_parent(ParentNode::from(self));
                conditional.visit_with(visitor);
            },
            Template(ref template) => {
                visitor.on_template_literal(template, self);
                visitor.push_parent(ParentNode::from(self));
                template.visit_with(visitor);
            },
            TaggedTemplate(ref tagged) => {
                visitor.on_tagged_template_expression(tagged, self);
                visitor.push_parent(ParentNode::from(self));
                tagged.visit_with(visitor);
            },
            Spread(ref spread) => {
                visitor.on_spread_expression(spread, self);
                visitor.push_parent(ParentNode::from(self));
                spread.visit_with(visitor);
            },
            Arrow(ref arrow) => {
                visitor.on_arrow_expression(arrow, self);
                visitor.push_parent(ParentNode::from(self));
                arrow.visit_with(visitor);
            },
            Object(ref object) => {
                visitor.on_object_expression(object, self);
                visitor.push_parent(ParentNode::from(self));
                object.visit_with(visitor);
            },
            Function(ref function) => {
                visitor.on_function_expression(function, self);
                visitor.push_parent(ParentNode::from(self));
                function.visit_with(visitor);
            },
            Class(ref class) => {
                visitor.on_class_expression(class, self);
                visitor.push_parent(ParentNode::from(self));
                class.visit_with(visitor);
            }
        }
        visitor.pop_parent();
    }
}

impl<'ast> Visitable<'ast> for ThisExpression {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit_with<V: Visitor<'ast>>(&self, _: &mut V) {}
}

impl<'ast> Visitable<'ast> for Identifier<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit_with<V: Visitor<'ast>>(&self, visitor: &mut V) {
        visitor.on_reference_use(self);
    }
}

impl<'ast> Visitable<'ast> for Literal<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit_with<V: Visitor<'ast>>(&self, _: &mut V) {}
}

impl<'ast> Visitable<'ast> for SequenceExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.body.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for ArrayExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.body.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for MemberExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.object.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for ComputedMemberExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.object.visit_with(visitor);
        self.property.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for MetaPropertyExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.meta.visit_with(visitor);
        self.property.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for CallExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.callee.visit_with(visitor);
        self.arguments.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for BinaryExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.left.visit_with(visitor);
        self.right.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for PrefixExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.operand.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for PostfixExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.operand.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for ConditionalExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

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

impl<'ast> Visitable<'ast> for TemplateLiteral<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.expressions.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for TaggedTemplateExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.tag.visit_with(visitor);
        self.quasi.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for SpreadExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.argument.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for ArrowBody<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        match *self {
            ArrowBody::Expression(ref expression) => expression.visit_with(visitor),
            ArrowBody::Block(ref block)           => block.body.visit_with(visitor),
        }
    }
}

impl<'ast> Visitable<'ast> for ArrowExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.params.visit_with(visitor);
        self.body.visit_with(visitor);
    }
}

impl<'ast> Visitable<'ast> for ObjectExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.body.visit_with(visitor);
    }
}
