use parser::statement::{StatementHandler, StatementHandlerFn};
use parser::expression::{ExpressionHandler, ExpressionHandlerFn};
use parser::nested::NestedHandlerFn;
use lexer::{Token, TokenTable};

mod es5;

pub use self::es5::ES5;

pub struct ExpressionTables {
    pub default: TokenTable<ExpressionHandlerFn>,
    pub call: TokenTable<ExpressionHandlerFn>,
    pub array: TokenTable<ExpressionHandlerFn>,
    pub nested: NestedExpressionTables,
}

#[derive(Default)]
pub struct NestedExpressionTables {
    pub any: TokenTable<NestedHandlerFn>,
    pub for_loop: TokenTable<NestedHandlerFn>,
    pub bp: [TokenTable<NestedHandlerFn>; 16],
}

pub struct FeatureSet {
    pub statements: TokenTable<StatementHandlerFn>,
    pub expressions: ExpressionTables,
}

pub type Feature = fn(&mut FeatureSet);

impl Default for ExpressionTables {
    fn default() -> Self {
        use parser::expression::error;

        let empty: TokenTable<ExpressionHandlerFn> = Token::table(error, &[]);

        ExpressionTables {
            default: empty,
            call: empty,
            array: empty,
            nested: NestedExpressionTables::default(),
        }
    }
}

impl Default for FeatureSet {
    fn default() -> Self {
        use parser::statement::error;

        FeatureSet {
            statements: Token::table(error, &[]),
            expressions: ExpressionTables::default(),
        }
    }
}

impl FeatureSet {
    pub fn set_expression<H>(&mut self, token: Token, _handler: H)
    where
        H: ExpressionHandler + StatementHandler,
    {
        self.expressions.default.set(token, H::expression);
        self.statements.set(token, H::statement);
    }
}
