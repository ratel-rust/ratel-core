use visitor::{Visitor, Visitable};
use ast::{Expression, ExpressionNode, StatementNode, Literal};
use ast::expression::*;


impl<'ast> Visitable<'ast> for ExpressionNode<'ast> {
    type Parent = StatementNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        use self::Expression::*;

        match self.item {
            Void => {},
            This(ref this) => {
                visitor.on_this_expression(self, ctx);
            },
            Identifier(ref ident) => {
                visitor.on_identifier_expression(ident, self, ctx);
            },
            Literal(ref literal) => {
                visitor.on_literal_expression(literal, self, ctx);
            },
            Sequence(ref sequence) => {
                sequence.visit(visitor, ctx);
                visitor.on_sequence_expression(sequence, self, ctx);
            },
            Array(ref array) => {
                array.visit(visitor, ctx);
                visitor.on_array_expression(array, self, ctx);
            },
            Member(ref member) => {
                member.visit(visitor, ctx);
                visitor.on_member_expression(member, self, ctx);
            },
            ComputedMember(ref computed) => {
                computed.visit(visitor, ctx);
                visitor.on_computed_member_expression(computed, self, ctx);
            },
            Call(ref call) => {
                call.visit(visitor, ctx);
                visitor.on_call_expression(call, self, ctx);
            },
            Binary(ref binary) => {
                binary.visit(visitor, ctx);
                visitor.on_binary_expression(binary, self, ctx);
            },
            Prefix(ref prefix) => {
                prefix.visit(visitor, ctx);
                visitor.on_prefix_expression(prefix, self, ctx);
            },
            Postfix(ref postfix) => {
                postfix.visit(visitor, ctx);
                visitor.on_postfix_expression(postfix, self, ctx);
            },
            Conditional(ref conditional) => {
                conditional.visit(visitor, ctx);
                visitor.on_conditional_expression(conditional, self, ctx);
            },
            Template(ref template) => {
                template.visit(visitor, ctx);
                visitor.on_template_literal(template, self, ctx);
            },
            TaggedTemplate(ref tagged) => {
                tagged.visit(visitor, ctx);
                visitor.on_tagged_template_expression(tagged, self, ctx);
            },
            Spread(ref spread) => {
                spread.visit(visitor, ctx);
                visitor.on_spread_expression(spread, self, ctx);
            },
            Arrow(ref arrow) => {
                arrow.visit(visitor, ctx);
                visitor.on_arrow_expression(arrow, self, ctx);
            },
            Object(ref object) => {
                object.visit(visitor, ctx);
                visitor.on_object_expression(object, self, ctx);
            },
            Function(ref function) => {
                function.visit(visitor, ctx);
                visitor.on_function_expression(function, self, ctx);
            },
            Class(ref class) => {
                class.visit(visitor, ctx);
                visitor.on_class_expression(class, self, ctx);
            }
        }
    }
}

impl<'ast> Visitable<'ast> for ThisExpression {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit<V: Visitor<'ast>>(&self, _: &V, _: &mut V::Context) {}
}

impl<'ast> Visitable<'ast> for Identifier<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit<V: Visitor<'ast>>(&self, visitor: &V, ctx: &mut V::Context) {
        visitor.on_identifier_use(self, ctx);
    }
}

impl<'ast> Visitable<'ast> for Literal<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit<V: Visitor<'ast>>(&self, _: &V, _: &mut V::Context) {}
}

impl<'ast> Visitable<'ast> for SequenceExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.body.visit(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for ArrayExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.body.visit(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for MemberExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.object.visit(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for ComputedMemberExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.object.visit(visitor, ctx);
        self.property.visit(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for CallExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.callee.visit(visitor, ctx);
        self.arguments.visit(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for BinaryExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.left.visit(visitor, ctx);
        self.right.visit(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for PrefixExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.operand.visit(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for PostfixExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.operand.visit(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for ConditionalExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

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

impl<'ast> Visitable<'ast> for TemplateLiteral<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.expressions.visit(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for TaggedTemplateExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.tag.visit(visitor, ctx);
        self.quasi.visit(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for SpreadExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.argument.visit(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for ArrowBody<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        match *self {
            ArrowBody::Expression(ref expression) => expression.visit(visitor, ctx),
            ArrowBody::Block(ref block)           => block.body.visit(visitor, ctx),
        }
    }
}

impl<'ast> Visitable<'ast> for ArrowExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.params.visit(visitor, ctx);
        self.body.visit(visitor, ctx);
    }
}

impl<'ast> Visitable<'ast> for ObjectExpression<'ast> {
    type Parent = ExpressionNode<'ast>;

    #[inline]
    fn visit<V>(&self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.body.visit(visitor, ctx);
    }
}
