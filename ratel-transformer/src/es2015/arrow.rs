use ratel::ast::{NodeList, ExpressionNode, Function, Name, OptionalName, Block};
use ratel::ast::expression::{ArrowExpression, ArrowBody};
use ratel::ast::statement::ReturnStatement;
use ratel_visitor::{StaticVisitor, DynamicVisitor};

use Transformer;

pub struct TransformArrow;

impl<'ast> StaticVisitor<'ast> for TransformArrow {
    type Context = Transformer<'ast>;

    fn on_arrow_expression(node: &ArrowExpression<'ast>, ptr: &'ast ExpressionNode<'ast>, t: &mut Transformer<'ast>) {
        let body = match node.body {
            ArrowBody::Block(block)     => block,
            ArrowBody::Expression(expr) => {
                let ret = t.alloc_as_loc(expr, ReturnStatement {
                    value: Some(expr)
                });

                t.alloc_as_loc(ret, Block {
                    body: NodeList::from(t.arena, ret)
                })
            }
        };

        t.swap(ptr, Function {
            name: OptionalName::empty(),
            generator: false,
            params: node.params,
            body,
        });
    }

    fn register(dv: &mut DynamicVisitor<'ast, Transformer<'ast>>) {
        dv.on_arrow_expression.push(TransformArrow::on_arrow_expression);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ratel_visitor::Visitor;
    use std::mem::size_of;

    #[test]
    fn transform_arrow_impls_visitor() {
        let _: &Visitor<Context = Transformer> = &TransformArrow;
    }

    #[test]
    fn transform_arrow_can_be_composed() {
        let _: &Visitor<Context = Transformer> = &(TransformArrow, TransformArrow);

        assert_eq!(size_of::<(TransformArrow, TransformArrow)>(), 0);
    }

    #[test]
    fn can_register_on_dv() {
        let mut dv = DynamicVisitor::new();

        TransformArrow.register(&mut *dv);

        assert_eq!(dv.on_arrow_expression.len(), 1);
        assert_eq!(dv.on_expression_statement.len(), 0);
    }
}
