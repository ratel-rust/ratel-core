use ast::ExpressionNode;
use ast::expression::ArrayExpression;
use parser::expression::{ExpressionHandler, ARRAY_CONTEXT};
use parser::{Parser, B0};

pub struct ArrayExpressionHandler;

impl ExpressionHandler for ArrayExpressionHandler {
    fn expression<'ast>(par: &mut Parser<'ast>) -> ExpressionNode<'ast> {
        let start = par.lexer.start_then_consume();
        let body = par.array_elements(|par| par.expression_in_context::<B0>(&ARRAY_CONTEXT));
        let end = par.lexer.end_then_consume();

        par.node_at(start, end, ArrayExpression { body })
    }
}
