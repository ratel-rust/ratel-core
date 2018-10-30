use ast::ExpressionNode;
use ast::expression::ObjectExpression;
use parser::expression::ExpressionHandler;
use parser::Parser;

pub struct ObjectExpressionHandler;

impl ExpressionHandler for ObjectExpressionHandler {
    fn expression<'ast>(par: &mut Parser<'ast>) -> ExpressionNode<'ast> {
        let start = par.lexer.start_then_consume();
        let body = par.property_list();
        let end = par.lexer.end_then_consume();

        par.node_at(start, end, ObjectExpression { body })
    }
}
