use error::{Error, Result};

use parser::Parser;
use lexer::Token::*;
use lexer::Token;
use ast::Item;

impl<'src> Parser<'src> {
    #[inline]
    pub fn statement(&mut self, token: Token) -> Result<Item> {
        match token {
            // Semicolon          => Ok(Statement::Empty),
            // BraceOpen          => self.block_statement(),
            // Declaration(kind)  => self.variable_declaration_statement(kind),
            // Return             => self.return_statement(),
            // Break              => self.break_statement(),
            // Function           => self.function_statement(),
            // Class              => self.class_statement(),
            // If                 => self.if_statement(),
            // While              => self.while_statement(),
            // Do                 => self.do_statement(),
            // For                => self.for_statement(),
            // Identifier(label)  => self.labeled_or_expression_statement(label),
            // Throw              => self.throw_statement(),
            // Try                => self.try_statement(),
            _                  => self.expression_statement(token),
        }
    }

    #[inline]
    pub fn expression_statement(&mut self, token: Token) -> Result<Item> {
        let expression = try!(self.expression_from(token, 0));

        let index = self.program.items.insert(0, 0, expression);

        expect_semicolon!(self);

        Ok(Item::ExpressionStatement(index))
    }
}
