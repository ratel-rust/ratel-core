use ast::{List, ExpressionPtr, Function, Name, OptionalName, Block};
use ast::expression::{ArrowExpression, ArrowBody};
use ast::statement::ReturnStatement;
use visitor::Visitor;
use transformer::Transformer;

pub struct TransformArrow;

impl<'ast> Visitor<'ast> for TransformArrow {
    type Context = Transformer<'ast>;

    fn on_arrow_expression(node: &ArrowExpression<'ast>, ptr: &ExpressionPtr<'ast>, t: &mut Transformer<'ast>) {
        let body = match node.body {
            ArrowBody::Block(block)     => block,
            ArrowBody::Expression(expr) => {
                let ret = t.alloc_as_loc(expr, ReturnStatement {
                    value: Some(expr)
                });

                t.alloc_as_loc(ret, Block {
                    body: List::from(t.arena, ret)
                })
            }
        };

        t.swap(ptr, Function {
            name: OptionalName::empty(),
            params: node.params,
            body,
        });
    }
}
