use ast::{ExpressionNode, Literal};
use parser::expression::ExpressionHandler;
use parser::Parser;

pub struct TrueLiteralHandler;
pub struct FalseLiteralHandler;
pub struct NullLiteralHandler;
pub struct UndefinedLiteralHandler;
pub struct StringLiteralHandler;
pub struct NumberLiteralHandler;
pub struct BinaryLiteralHandler;

pub trait LiteralHandler {
    const VALUE: Literal<'static>;
}

impl<L> ExpressionHandler for L
where
    L: LiteralHandler
{
    fn expression<'ast>(par: &mut Parser<'ast>) -> ExpressionNode<'ast> {
        par.node_consume(Self::VALUE)
    }
}

impl ExpressionHandler for TrueLiteralHandler {
    fn expression<'ast>(par: &mut Parser<'ast>) -> ExpressionNode<'ast> {
        par.node_consume(Literal::True)
    }
}

impl ExpressionHandler for FalseLiteralHandler {
    fn expression<'ast>(par: &mut Parser<'ast>) -> ExpressionNode<'ast> {
        par.node_consume(Literal::False)
    }
}

impl ExpressionHandler for NullLiteralHandler {
    fn expression<'ast>(par: &mut Parser<'ast>) -> ExpressionNode<'ast> {
        par.node_consume(Literal::Null)
    }
}

impl ExpressionHandler for UndefinedLiteralHandler {
    fn expression<'ast>(par: &mut Parser<'ast>) -> ExpressionNode<'ast> {
        par.node_consume(Literal::Undefined)
    }
}

impl ExpressionHandler for StringLiteralHandler {
    fn expression<'ast>(par: &mut Parser<'ast>) -> ExpressionNode<'ast> {
        par.node_consume_str(|value| Literal::String(value))
    }
}

impl ExpressionHandler for NumberLiteralHandler {
    fn expression<'ast>(par: &mut Parser<'ast>) -> ExpressionNode<'ast> {
        par.node_consume_str(|value| Literal::Number(value))
    }
}

impl ExpressionHandler for BinaryLiteralHandler {
    fn expression<'ast>(par: &mut Parser<'ast>) -> ExpressionNode<'ast> {
        par.node_consume_str(|value| Literal::Binary(value))
    }
}
