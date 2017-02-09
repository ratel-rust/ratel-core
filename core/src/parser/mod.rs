#[macro_use]
mod macros;

use std::mem;

use ast::{Program, Store, Statement, Expression, OperatorKind};
use error::{Error, ParseResult, Result};
use tokenizer::Tokenizer;
use lexicon::Token;
use lexicon::Token::*;

pub struct Parser<'src> {
    /// Tokenizer will produce tokens from the source
    tokenizer: Tokenizer<'src>,

    /// Current token, to be used by peek! and next! macros
    token: Option<Token>,

    /// AST under construction
    program: Program<'src>,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str) -> Self {
        Parser {
            tokenizer: Tokenizer::new(source),
            token: None,
            program: Program {
                source: source,
                expressions: Store::new(),
                statements: Store::new(),
            }
        }
    }

    #[inline]
    fn consume(&mut self) {
        self.token = None;
    }

    #[inline]
    fn statement(&mut self, token: Token) -> Result<Statement> {
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
    fn expression(&mut self, lbp: u8) -> Result<Expression> {
        let token = next!(self);
        self.expression_from(token, lbp)
    }

    #[inline]
    fn expression_from(&mut self, token: Token, lbp: u8) -> Result<Expression> {
        let left = match token {
            Identifier(value)  => Expression::Identifier(value.into()),
            _                  => unexpected_token!(self)
        };

        self.complex_expression(left, lbp)
    }

    #[inline]
    fn complex_expression(&mut self, mut left: Expression, lbp: u8) -> Result<Expression> {
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
    fn infix_expression(&mut self, left: Expression, bp: u8, op: OperatorKind) -> Result<Expression> {
        use ast::OperatorKind::*;

        Ok(match op {
            Increment | Decrement => Expression::Postfix {
                operator: op,
                operand: self.program.expressions.insert(0, 0, left),
            },

            _ => {
                if !op.infix() {
                    unexpected_token!(self);
                }

                if op.assignment() {
                    // TODO: verify that left is assignable
                }

                let right = self.expression(bp)?;

                Expression::Binary {
                    parenthesized: false,
                    operator: op,
                    left: self.program.expressions.insert(0, 0, left),
                    right: self.program.expressions.insert(0, 0, right),
                }
            }
        })
    }

    #[inline]
    fn expression_statement(&mut self, token: Token) -> Result<Statement> {
        let expression = try!(self.expression_from(token, 0));

        expect_semicolon!(self);

        Ok(Statement::Expression(expression))
    }

    #[inline]
    fn parse(&mut self) -> Result<()> {
        let mut statement = match next!(self) {
            EndOfProgram => return Ok(()),
            token        => try!(self.statement(token))
        };

        let mut previous = self.program.statements.insert(0, 0, statement);

        loop {
            statement = match next!(self) {
                EndOfProgram => break,
                token        => try!(self.statement(token))
            };

            let id = self.program.statements.insert(0, 0, statement);

            self.program.statements[previous].next = Some(id);

            previous = id;
        }

        Ok(())
    }
}

pub fn parse<'src>(source: &'src str) -> Result<Program<'src>> {
    let mut parser = Parser::new(source);

    parser.parse()?;

    Ok(parser.program)
}

#[cfg(test)]
mod test {
    use super::*;
    use ast::{Ident, Slice, OperatorKind};

    #[test]
    fn parse_ident_expr() {
        let src = "foo; bar; baz;";

        let program = parse(src).unwrap();

        let exprs = &program.expressions;
        let stmts = &program.statements;

        // Statements are there
        assert_eq!(stmts.len(), 3);

        // Statements are linked
        assert_eq!(stmts[0].next, Some(1));
        assert_eq!(stmts[1].next, Some(2));
        assert_eq!(stmts[2].next, None);

        // No nested expressions
        assert_eq!(exprs.len(), 0);

        // Match identifiers
        match stmts[0].value {
            Statement::Expression(Expression::Identifier(ref ident)) => assert_eq!(ident.as_str(src), "foo"),
            _ => panic!()
        }
        match stmts[1].value {
            Statement::Expression(Expression::Identifier(ref ident)) => assert_eq!(ident.as_str(src), "bar"),
            _ => panic!()
        }
        match stmts[2].value {
            Statement::Expression(Expression::Identifier(ref ident)) => assert_eq!(ident.as_str(src), "baz"),
            _ => panic!()
        }
    }

    #[test]
    fn parse_binary_and_postfix_expr() {
        let src = "foo + bar; baz++;";

        let program = parse(src).unwrap();

        let exprs = &program.expressions;
        let stmts = &program.statements;

        // Statements are there and refer to correct expressions
        assert_eq!(stmts.len(), 2);
        assert_eq!(stmts[0].next, Some(1));
        assert_eq!(stmts[0].value, Statement::Expression(
            Expression::Binary {
                parenthesized: false,
                operator: OperatorKind::Addition,
                left: 0,
                right: 1,
            }
        ));
        assert_eq!(stmts[1].next, None);
        assert_eq!(stmts[1].value, Statement::Expression(
            Expression::Postfix {
                operator: OperatorKind::Increment,
                operand: 2
            }
        ));

        // Nested expressions
        assert_eq!(exprs.len(), 3);

        match exprs[0].value {
            Expression::Identifier(ref ident) => assert_eq!(ident.as_str(src), "foo"),
            _ => panic!()
        }
        match exprs[1].value {
            Expression::Identifier(ref ident) => assert_eq!(ident.as_str(src), "bar"),
            _ => panic!()
        }
        match exprs[2].value {
            Expression::Identifier(ref ident) => assert_eq!(ident.as_str(src), "baz"),
            _ => panic!()
        }
    }

}
