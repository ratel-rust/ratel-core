use parser::statement::StatementHandlerFn;
use parser::expression::ExpressionHandlerFn;
use lexer::{Token, TokenTable};

pub struct FeatureSet {
    statements: TokenTable<StatementHandlerFn>,
    expressions_default: TokenTable<ExpressionHandlerFn>,
    expressions_call: TokenTable<ExpressionHandlerFn>,
    expressions_array: TokenTable<ExpressionHandlerFn>,
}

pub trait Feature {
    fn register(set: &mut FeatureSet) {
        if let Some(statement_handler) = Self::statement_handler() {
            for &token in Self::tokens() {
                set.statements.set(token, statement_handler);
            }
        }

        if let Some(expression_handler) = Self::expression_handler() {
            for &token in Self::tokens() {
                set.expressions_default.set(token, expression_handler);
                set.expressions_call.set(token, expression_handler);
                set.expressions_array.set(token, expression_handler);
            }
        }
    }

    fn statement_handler() -> Option<StatementHandlerFn> {
        None
    }

    fn expression_handler() -> Option<ExpressionHandlerFn> {
        None
    }

    fn tokens() -> &'static [Token] {
        &[]
    }
}
