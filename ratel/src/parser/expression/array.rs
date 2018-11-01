use ast::ExpressionNode;
use ast::expression::ArrayExpression;
use parser::expression::ExpressionHandler;
use parser::{Parser, B0};

pub struct ArrayExpressionHandler;

impl ExpressionHandler for ArrayExpressionHandler {
    fn expression<'ast>(par: &mut Parser<'ast>) -> ExpressionNode<'ast> {
        let start = par.lexer.start_then_consume();
        let body = par.array_elements(|par| par.expression_in_array_context::<B0>());
        let end = par.lexer.end_then_consume();

        par.node_at(start, end, ArrayExpression { body })
    }
}
