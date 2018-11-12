use ratel::ast::{NodeList, ExpressionNode, Function, Name, OptionalName, Block};
use ratel::ast::expression::{ArrowExpression, ArrowBody};
use ratel::ast::statement::ReturnStatement;
use ratel_visitor::Visitor;

use TransformerCtxt;

pub struct TransformArrow<'ast> {
    ctx: TransformerCtxt<'ast>
}

impl<'ast> TransformArrow<'ast> {
    pub fn new(ctx: TransformerCtxt<'ast>) -> TransformArrow<'ast> {
        TransformArrow {
            ctx
        }
    }
}

impl<'ast> Visitor<'ast> for TransformArrow<'ast> {
    fn on_arrow_expression(&mut self, node: &ArrowExpression<'ast>, ptr: &'ast ExpressionNode<'ast>) {
        let body = match node.body {
            ArrowBody::Block(block)     => block,
            ArrowBody::Expression(expr) => {
                let ret = self.ctx.alloc_as_loc(expr, ReturnStatement {
                    value: Some(expr)
                });

                self.ctx.alloc_as_loc(ret, Block {
                    body: NodeList::from(self.ctx.arena, ret)
                })
            }
        };

        self.ctx.swap(ptr, Function {
            name: OptionalName::empty(),
            generator: false,
            params: node.params,
            body,
        });
    }
}
