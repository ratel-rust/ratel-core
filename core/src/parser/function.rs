use parser::Parser;
use lexer::Token::*;
use ast::{Function, Name};

impl<'ast> Parser<'ast> {
    #[inline]
    pub fn function<N, I>(&mut self, name: I) -> Function<'ast, N> where
        N: Name<'ast>,
        I: Into<N>,
    {
        expect!(self, ParenOpen);

        Function {
            name: name.into(),
            params: self.parameter_list(),
            body: self.block_body(),
        }
    }
}
