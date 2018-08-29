use ratel::ast::{Node, Loc, Expression, ExpressionNode, OperatorKind};
use ratel::ast::expression::{BinaryExpression, MemberExpression, CallExpression};
use ratel_visitor::Visitor;

use TransformerCtxt;

pub struct PresetES2016<'ast> {
    ctx: TransformerCtxt<'ast>
}

static MATH: &Loc<Expression> = &Loc {
    start: 0,
    end: 0,
    item: Expression::Identifier("Math")
};
static POW: &Loc<&str> = &Loc {
    start: 0,
    end: 0,
    item: "pow"
};

impl<'ast> Visitor<'ast> for PresetES2016<'ast> {
    fn on_binary_expression(&mut self, node: &BinaryExpression<'ast>, ptr: &ExpressionNode<'ast>) {
        match node.operator {
            OperatorKind::Exponent => {
                let callee = self.ctx.alloc(MemberExpression {
                    object: Node::new(MATH),
                    property: Node::new(POW),
                });
                let arguments = self.ctx.list([node.left, node.right]);

                self.ctx.swap(ptr, CallExpression {
                    callee,
                    arguments
                });
            },

            OperatorKind::ExponentAssign => {
                let callee = self.ctx.alloc(MemberExpression {
                    object: Node::new(MATH),
                    property: Node::new(POW),
                });
                let arguments = self.ctx.list([node.left, node.right]);
                let right = self.ctx.alloc(CallExpression {
                    callee,
                    arguments
                });

                self.ctx.swap(ptr, BinaryExpression {
                    operator: OperatorKind::Assign,
                    left: node.left,
                    right,
                });
            },

            _ => {}
        }
    }
}
