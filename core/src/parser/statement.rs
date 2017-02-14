use error::{Error, Result};

use parser::Parser;
use lexer::Token::*;
use lexer::Token;
use ast::{Node, Index, Item, OperatorKind, VariableDeclarationKind};
use ast::Item::*;

impl<'src> Parser<'src> {
    #[inline(always)]
    pub fn statement(&mut self, token: Token<'src>) -> Result<Node<'src>> {
        match token {
            Semicolon          => Ok(EmptyStatement.at(0, 0)),
            // BraceOpen          => self.block_statement(),
            Declaration(kind)  => self.variable_declaration_statement(kind),
            Return             => self.return_statement(),
            // Break              => self.break_statement(),
            Function           => self.function_statement(),
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

    #[inline(always)]
    pub fn expression_statement(&mut self, token: Token<'src>) -> Result<Node<'src>> {
        let expression = try!(self.expression_from(token, 0));

        let start = expression.start;
        let end = expression.end;
        let index = self.store(expression);

        expect_semicolon!(self);

        Ok(ExpressionStatement(index).at(start, end))
    }

    #[inline(always)]
    pub fn function_statement(&mut self) -> Result<Node<'src>> {
        let name = expect_identifier!(self);

        expect!(self, ParenOpen);

        Ok(Item::FunctionStatement {
            name: name.into(),
            params: try!(self.parameter_list()),
            body: try!(self.block_body()),
        }.at(0, 0))
    }

    #[inline(always)]
    fn return_statement(&mut self) -> Result<Node<'src>> {
        let value = match peek!(self) {
            EndOfProgram => None,
            Semicolon    => None,
            _            => {
                if self.lexer.asi() {
                    None
                } else {
                    let expression = try!(self.expression(0));

                    Some(self.store(expression))
                }
            }
        };

        expect_semicolon!(self);

        Ok(Item::ReturnStatement {
            value: value,
        }.at(0, 0))
    }

    #[inline(always)]
    pub fn variable_declaration_statement(&mut self, kind: VariableDeclarationKind) -> Result<Node<'src>> {
        let declaration = Item::DeclarationStatemenet {
            kind: kind,
            declarators: try!(self.variable_declarators())
        }.at(0, 0);

        expect_semicolon!(self);

        Ok(declaration)
    }

    #[inline(always)]
    pub fn variable_declarators(&mut self) -> Result<Index> {
        let name = expect_identifier!(self);

        expect!(self, Operator(OperatorKind::Assign));

        let value = try!(self.expression(0));

        let name = self.store(Item::Identifier(name.into()).at(0, 0));
        let value = self.store(value);

        let id = self.store(Item::VariableDeclarator {
            name: name,
            value: Some(value),
        }.at(0, 0));

        Ok(id)
    }
}
