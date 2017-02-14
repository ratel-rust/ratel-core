use error::{Error, Result};

use parser::Parser;
use lexer::Token::*;
use lexer::Token;
use ast::{Node, Index, Item, OperatorKind};

impl<'src> Parser<'src> {
    #[inline(always)]
    pub fn expression(&mut self, lbp: u8) -> Result<Node<'src>> {
        let token = next!(self);
        self.expression_from(token, lbp)
    }

    #[inline(always)]
    pub fn expression_from(&mut self, token: Token<'src>, lbp: u8) -> Result<Node<'src>> {
        let left = match token {
            This               => Item::This.at(0, 0),
            Literal(value)     => Item::ValueExpr(value).at(0, 0),
            // LitBoolean(value)  => Expression::Literal(Value::Boolean(value)),
            // LitBinary(value)   => Expression::Literal(Value::Binary(value)),
            // LitNumber(value)   => Expression::Literal(Value::Number(value)),
            // LitString(value)   => Expression::Literal(Value::String(value)),
            // LitQuasi(value)    => Expression::Literal(Value::RawQuasi(value)),
            Identifier(value)  => Item::Identifier(value.into()).at(0, 0),
            // Operator(Division) => try!(self.regular_expression()),
            // Operator(optype)   => try!(self.prefix_expression(optype)),
            ParenOpen          => try!(self.paren_expression()),
            // BracketOpen        => try!(self.array_expression()),
            // BraceOpen          => try!(self.object_expression()),
            // Function           => try!(self.function_expression()),
            // Class              => try!(self.class_expression()),
            // Template(kind)     => try!(self.template_expression(None, kind)),
            _                  => unexpected_token!(self)
        };

        self.complex_expression(left, lbp)
    }

    #[inline(always)]
    pub fn complex_expression(&mut self, mut left: Node<'src>, lbp: u8) -> Result<Node<'src>> {
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

                ParenOpen => {
                    if lbp > 18 {
                        break;
                    }

                    self.consume();

                    Item::CallExpr {
                        callee: self.store(left),
                        arguments: try!(self.expression_list()),
                    }.at(0, 0)
                },

                _ => break
            }
        }

        Ok(left)
    }


    #[inline(always)]
    pub fn infix_expression(&mut self, left: Node<'src>, bp: u8, op: OperatorKind) -> Result<Node<'src>> {
        use ast::OperatorKind::*;

        Ok(match op {
            Increment | Decrement => {
                // TODO: op.end
                Node::new(left.start, left.end, Item::PostfixExpr {
                    operator: op,
                    operand: self.store(left),
                })
            },

            Accessor => {
                let right = try!(self.expression(bp));

                Node::new(left.start, right.end, Item::MemberExpr {
                    object: self.store(left),
                    property: self.store(right),
                })
            },

            _ => {
                if !op.infix() {
                    unexpected_token!(self);
                }

                if op.assignment() {
                    // TODO: verify that left is assignable
                }

                let right = try!(self.expression(bp));

                Node::new(left.start, right.end, Item::BinaryExpr {
                    parenthesized: false,
                    operator: op,
                    left: self.store(left),
                    right: self.store(right),
                })
            }
        })
    }

    pub fn expression_list(&mut self) -> Result<Option<Index>> {
        let expression = match next!(self) {
            ParenClose => return Ok(None),
            token      => try!(self.expression_from(token, 0)),
        };

        let mut previous = self.store(expression);
        let root = Some(previous);

        loop {
            let expression = match next!(self) {
                ParenClose => break,
                Comma      => try!(self.expression(0)),
                _          => unexpected_token!(self),
            };

            let index = self.store(expression);
            self.program.items[previous].next = Some(index);

            previous = index;
        }

        Ok(root)
    }

    #[inline(always)]
    fn paren_expression(&mut self) -> Result<Node<'src>> {
        match next!(self) {
            // ParenClose => {
            //     expect!(self, Operator(FatArrow));

            //     self.arrow_function_expression(None)
            // },
            token => {
                let expression = try!(self.expression_from(token, 0));
                // let expression = try!(self.sequence_or(expression));

                expect!(self, ParenClose);

                Ok(expression)

                // Ok(expression.parenthesize())
            }
        }
    }
}
