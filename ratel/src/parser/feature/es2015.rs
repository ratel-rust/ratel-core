use lexer::Token::*;

use super::Feature;

pub static ES2015: Feature = |set| {
    set.statements.extend(&[
        (Yield,          |par| {
          let expression = par.yield_expression();
          par.expression_statement(expression)
        }),
    ]);
};
