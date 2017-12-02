use ast::{Node, Loc, Expression, ExpressionNode, OperatorKind};
use ast::expression::{BinaryExpression, MemberExpression, CallExpression};
use transformer::Transformer;
use visitor::{StaticVisitor, DynamicVisitor};

pub struct PresetES2016;

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

impl<'ast> StaticVisitor<'ast> for PresetES2016 {
    type Context = Transformer<'ast>;

    fn on_binary_expression(node: &BinaryExpression<'ast>, ptr: &ExpressionNode<'ast>, t: &mut Transformer<'ast>) {
        match node.operator {
            OperatorKind::Exponent => {
                let callee = t.alloc(MemberExpression {
                    object: Node::from_static(MATH),
                    property: Node::from_static(POW),
                });
                let arguments = t.list([node.left, node.right]);

                t.swap(ptr, CallExpression {
                    callee,
                    arguments
                });
            },

            OperatorKind::ExponentAssign => {
                let callee = t.alloc(MemberExpression {
                    object: Node::from_static(MATH),
                    property: Node::from_static(POW),
                });
                let arguments = t.list([node.left, node.right]);
                let right = t.alloc(CallExpression {
                    callee,
                    arguments
                });

                t.swap(ptr, BinaryExpression {
                    operator: OperatorKind::Assign,
                    left: node.left,
                    right,
                });
            },

            _ => {}
        }
    }

    #[inline]
    fn register(dv: &mut DynamicVisitor<'ast, Transformer<'ast>>) {
        dv.on_binary_expression.push(PresetES2016::on_binary_expression);
    }
}
