#[macro_use]
mod macros;

use std::mem;
use error::{Error, ParseResult, Result};

use ast::{Program, Store, Index, Item, OperatorKind};
use lexer::{Lexer, Token};
use lexer::Token::*;

pub struct Parser<'src> {
    /// Lexer will produce tokens from the source
    lexer: Lexer<'src>,

    /// Current token, to be used by peek! and next! macros
    token: Option<Token>,

    /// AST under construction
    program: Program<'src>,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str) -> Self {
        Parser {
            lexer: Lexer::new(source),
            token: None,
            program: Program {
                source: source,
                root: None,
                items: Store::new(),
            }
        }
    }

    #[inline]
    fn consume(&mut self) {
        self.token = None;
    }

    #[inline]
    fn statement(&mut self, token: Token) -> Result<Item> {
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
    fn expression(&mut self, lbp: u8) -> Result<Item> {
        let token = next!(self);
        self.expression_from(token, lbp)
    }

    #[inline]
    fn expression_from(&mut self, token: Token, lbp: u8) -> Result<Item> {
        let left = match token {
            Identifier(value)  => Item::Identifier(value.into()),
            _                  => unexpected_token!(self)
        };

        self.complex_expression(left, lbp)
    }

    #[inline]
    fn complex_expression(&mut self, mut left: Item, lbp: u8) -> Result<Item> {
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
    fn infix_expression(&mut self, left: Item, bp: u8, op: OperatorKind) -> Result<Item> {
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

    #[inline]
    fn expression_statement(&mut self, token: Token) -> Result<Item> {
        let expression = try!(self.expression_from(token, 0));

        let index = self.program.items.insert(0, 0, expression);

        expect_semicolon!(self);

        Ok(Item::ExpressionStatement(index))
    }

    #[inline]
    fn parse(&mut self) -> Result<()> {
        let mut statement = match next!(self) {
            EndOfProgram => return Ok(()),
            token        => try!(self.statement(token))
        };

        let mut previous = self.program.items.insert(0, 0, statement);

        self.program.root = Some(previous);

        loop {
            statement = match next!(self) {
                EndOfProgram => break,
                token        => try!(self.statement(token))
            };

            let index = self.program.items.insert(0, 0, statement);

            self.program.items[previous].next = Some(index);

            previous = index;
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

    macro_rules! assert_item {
        ($item:expr, $m:pat => $eval:expr) => {
            match $item {
                $m => assert!($eval),
                _ => panic!("Failed assert_item")
            }
        }
    }

    #[test]
    fn empty_parse() {
        let program = parse("").unwrap();

        assert_eq!(program.items.len(), 0);
        assert_eq!(program.root, None);
        assert_eq!(program.statements().next(), None);
    }

    #[test]
    fn parse_ident_expr() {
        let src = "foo; bar; baz;";

        let program = parse(src).unwrap();

        let items = &program.items;

        // 3 times statement and expression
        assert_eq!(items.len(), 6);

        // First statement is after first expression
        assert_eq!(program.root, Some(1));

        // Statements are linked
        let mut stmts = program.statements();
        assert_eq!(stmts.next().unwrap(), &Item::ExpressionStatement(0));
        assert_eq!(stmts.next().unwrap(), &Item::ExpressionStatement(2));
        assert_eq!(stmts.next().unwrap(), &Item::ExpressionStatement(4));
        assert_eq!(stmts.next(), None);

        // Match identifiers
        assert_item!(items[0].item, Item::Identifier(ref i) => i.as_str(src) == "foo");
        assert_item!(items[2].item, Item::Identifier(ref i) => i.as_str(src) == "bar");
        assert_item!(items[4].item, Item::Identifier(ref i) => i.as_str(src) == "baz");
    }

    #[test]
    fn parse_binary_and_postfix_expr() {
        let src = "foo + bar; baz++;";

        let program = parse(src).unwrap();

        let items = &program.items;

        // 2 statements, 3 simple expressions, one binary expression, one postfix expression
        assert_eq!(items.len(), 7);

        // First statement is after binary expression and two of it's side expressions
        assert_eq!(program.root, Some(3));

        // Statements are linked
        let mut stmts = program.statements();
        assert_eq!(stmts.next().unwrap(), &Item::ExpressionStatement(2));
        assert_eq!(stmts.next().unwrap(), &Item::ExpressionStatement(5));
        assert_eq!(stmts.next(), None);

        // Binary expression
        assert_eq!(items[2].item, Item::BinaryExpr {
            parenthesized: false,
            operator: OperatorKind::Addition,
            left: 0,
            right: 1,
        });
        assert_item!(items[0].item, Item::Identifier(ref i) => i.as_str(src) == "foo");
        assert_item!(items[1].item, Item::Identifier(ref i) => i.as_str(src) == "bar");

        // Postfix expression
        assert_eq!(items[5].item, Item::PostfixExpr {
            operator: OperatorKind::Increment,
            operand: 4
        });
        assert_item!(items[4].item, Item::Identifier(ref i) => i.as_str(src) == "baz");
    }

}
