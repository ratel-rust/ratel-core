use error::{Error, Result};

use parser::Parser;
use lexer::Token::*;
use lexer::Token;
use ast::{Item, OperatorKind};

impl<'src> Parser<'src> {
    #[inline]
    pub fn expression(&mut self, lbp: u8) -> Result<Item> {
        let token = next!(self);
        self.expression_from(token, lbp)
    }

    #[inline]
    pub fn expression_from(&mut self, token: Token, lbp: u8) -> Result<Item> {
        let left = match token {
            Identifier(value)  => Item::Identifier(value.into()),
            _                  => unexpected_token!(self)
        };

        self.complex_expression(left, lbp)
    }

    #[inline]
    pub fn complex_expression(&mut self, mut left: Item, lbp: u8) -> Result<Item> {
        loop {
            left = match peek!(self) {
                Operator(op) => {
                    let rbp = op.binding_power();

                    if lbp > rbp {
                        break;
                    }

                    self.consume();

                    try!(self.infix_expression(left, rbp, op))
                },

                _ => break
            }
        }

        Ok(left)
    }


    #[inline]
    pub fn infix_expression(&mut self, left: Item, bp: u8, op: OperatorKind) -> Result<Item> {
        use ast::OperatorKind::*;

        Ok(match op {
            Increment | Decrement => Item::PostfixExpr {
                operator: op,
                operand: self.program.items.insert(0, 0, left),
            },

            _ => {
                if !op.infix() {
                    unexpected_token!(self);
                }

                if op.assignment() {
                    // TODO: verify that left is assignable
                }

                let right = self.expression(bp)?;

                Item::BinaryExpr {
                    parenthesized: false,
                    operator: op,
                    left: self.program.items.insert(0, 0, left),
                    right: self.program.items.insert(0, 0, right),
                }
            }
        })
    }
}
