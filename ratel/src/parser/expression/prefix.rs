use ast::{ExpressionNode, OperatorKind};
use ast::expression::PrefixExpression;
use parser::expression::ExpressionHandler;
use parser::{Parser, B15};

pub struct IncrementPrefixHandler;
pub struct DecrementPrefixHandler;
pub struct LogicalNotPrefixHandler;
pub struct BitwiseNotPrefixHandler;
pub struct TypeofPrefixHandler;
pub struct VoidPrefixHandler;
pub struct DeletePrefixHandler;
pub struct AdditionPrefixHandler;
pub struct SubtractionPrefixHandler;

pub trait OperatorHandler {
    const OPERATOR: OperatorKind;
}

impl<O> ExpressionHandler for O
where
    O: OperatorHandler,
{
    fn expression<'ast>(par: &mut Parser<'ast>) -> ExpressionNode<'ast> {
        let start = par.lexer.start_then_consume();
        let operand = par.expression::<B15>();
        let end = operand.end;

        par.node_at(start, end, PrefixExpression {
            operator: Self::OPERATOR,
            operand,
        })
    }
}

use self::OperatorKind::*;

impl OperatorHandler for IncrementPrefixHandler {
    const OPERATOR: OperatorKind = Increment;
}

impl OperatorHandler for DecrementPrefixHandler {
    const OPERATOR: OperatorKind = Decrement;
}

impl OperatorHandler for LogicalNotPrefixHandler {
    const OPERATOR: OperatorKind = LogicalNot;
}

impl OperatorHandler for BitwiseNotPrefixHandler {
    const OPERATOR: OperatorKind = BitwiseNot;
}

impl OperatorHandler for TypeofPrefixHandler {
    const OPERATOR: OperatorKind = Typeof;
}

impl OperatorHandler for VoidPrefixHandler {
    const OPERATOR: OperatorKind = Void;
}

impl OperatorHandler for DeletePrefixHandler {
    const OPERATOR: OperatorKind = Delete;
}

impl OperatorHandler for AdditionPrefixHandler {
    const OPERATOR: OperatorKind = Addition;
}

impl OperatorHandler for SubtractionPrefixHandler {
    const OPERATOR: OperatorKind = Subtraction;
}
