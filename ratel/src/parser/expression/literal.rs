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
        let expr = par.alloc_in_loc(Self::VALUE);
        par.lexer.consume();

        expr
    }
}

impl LiteralHandler for TrueLiteralHandler {
    const VALUE: Literal<'static> = Literal::True;
}

impl LiteralHandler for FalseLiteralHandler {
    const VALUE: Literal<'static> = Literal::False;
}

impl LiteralHandler for NullLiteralHandler {
    const VALUE: Literal<'static> = Literal::Null;
}

impl LiteralHandler for UndefinedLiteralHandler {
    const VALUE: Literal<'static> = Literal::Undefined;
}

impl ExpressionHandler for StringLiteralHandler {
    fn expression<'ast>(par: &mut Parser<'ast>) -> ExpressionNode<'ast> {
        let value = par.lexer.token_as_str();
        let expr = par.alloc_in_loc(Literal::String(value));

        par.lexer.consume();
        expr
    }
}

impl ExpressionHandler for NumberLiteralHandler {
    fn expression<'ast>(par: &mut Parser<'ast>) -> ExpressionNode<'ast> {
        let value = par.lexer.token_as_str();
        let expr = par.alloc_in_loc(Literal::Number(value));

        par.lexer.consume();
        expr
    }
}

impl ExpressionHandler for BinaryLiteralHandler {
    fn expression<'ast>(par: &mut Parser<'ast>) -> ExpressionNode<'ast> {
        let value = par.lexer.token_as_str();
        let expr = par.alloc_in_loc(Literal::Binary(value));

        par.lexer.consume();
        expr
    }
}
