#[macro_use]
mod macros;
mod expression;
mod statement;

use error::Result;

use ast::{Program, Store, Node, Index, Item};
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
    fn store(&mut self, node: Node) -> Index {
        self.program.items.insert(node)
    }

    #[inline]
    fn parse(&mut self) -> Result<()> {
        let statement = match next!(self) {
            EndOfProgram => return Ok(()),
            token        => try!(self.statement(token))
        };

        let mut previous = self.store(statement);

        self.program.root = Some(previous);

        loop {
            let statement = match next!(self) {
                EndOfProgram => break,
                token        => try!(self.statement(token))
            };

            let index = self.store(statement);

            self.program.items[previous].next = Some(index);

            previous = index;
        }

        Ok(())
    }

    #[inline]
    fn block_body_tail(&mut self) -> Result<Option<Index>> {
        let statement = match next!(self) {
            BraceClose => return Ok(None),
            token      => try!(self.statement(token)),
        };

        let mut previous = self.store(statement);
        let root = Some(previous);

        loop {
            let statement = match next!(self) {
                BraceClose => break,
                token      => try!(self.statement(token)),
            };

            let index = self.store(statement);
            self.program.items[previous].next = Some(index);

            previous = index;
        }

        Ok(root)
    }

    #[inline]
    fn block_body(&mut self) -> Result<Option<Index>> {
        expect!(self, BraceOpen);
        self.block_body_tail()
    }

    fn parameter_list(&mut self) -> Result<Option<Index>> {
        let name = match next!(self) {
            ParenClose       => return Ok(None),
            Identifier(name) => name,
            _                => unexpected_token!(self),
        };

        let mut previous = self.store(Item::Identifier(name.into()).at(0, 0));
        let root = Some(previous);

        loop {
            let name = match next!(self) {
                ParenClose => break,
                Comma      => expect_identifier!(self),
                _          => unexpected_token!(self),
            };

            let index = self.store(Item::Identifier(name.into()).at(0, 0));
            self.program.items[previous].next = Some(index);

            previous = index;
        }

        Ok(root)

        // let mut default_params = false;

        // loop {
        //     let name = match next!(self) {
        //         ParenClose       => break,
        //         Identifier(name) => name,
        //         _ => unexpected_token!(self)
        //     };

        //     list.push(match peek!(self) {
        //         Operator(Assign) => {
        //             self.consume();
        //             let expression = try!(self.expression(0));
        //             default_params = true;
        //             Parameter {
        //                 name: name.into(),
        //                 default: Some(Box::new(expression))
        //             }
        //         }
        //         _ => {
        //             if default_params {
        //                 unexpected_token!(self);
        //             }
        //             Parameter {
        //                 name: name.into(),
        //                 default: None
        //             }
        //         }
        //     });

        //     match next!(self) {
        //         ParenClose => break,
        //         Comma      => {},
        //         _          => unexpected_token!(self)
        //     }
        // }

        // Ok(list)
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
    use ast::OperatorKind;

    macro_rules! assert_item {
        ($item:expr, $m:pat => $eval:expr) => {
            match $item {
                $m => assert!($eval),
                _ => panic!("Failed assert_item")
            }
        }
    }

    macro_rules! assert_ident {
        ($item:expr, $src:ident, $expect:expr) => {
            assert_item!($item, Item::Identifier(ref i) => i.as_str($src) == $expect);
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
    fn empty_statements() {
        let program = parse(";;;").unwrap();

        assert_eq!(program.items.len(), 3);

        // Statements are linked
        let mut stmts = program.statements();
        assert_eq!(stmts.next().unwrap(), &Item::EmptyStatement);
        assert_eq!(stmts.next().unwrap(), &Item::EmptyStatement);
        assert_eq!(stmts.next().unwrap(), &Item::EmptyStatement);
        assert_eq!(stmts.next(), None);
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
        assert_ident!(items[0].item, src, "foo");
        assert_ident!(items[2].item, src, "bar");
        assert_ident!(items[4].item, src, "baz");
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
        assert_ident!(items[0].item, src, "foo");
        assert_ident!(items[1].item, src, "bar");

        // Postfix expression
        assert_eq!(items[5].item, Item::PostfixExpr {
            operator: OperatorKind::Increment,
            operand: 4
        });
        assert_ident!(items[4].item, src, "baz");
    }

    #[test]
    fn function_statement_empty() {
        let src = "function foo() {}";

        let program = parse(src).unwrap();

        let mut stmts = program.statements();

        match *stmts.next().unwrap() {
            Item::FunctionStatement {
                ref name,
                params: None,
                body: None,
            } => assert_eq!(name.as_str(src), "foo"),
            _ => panic!()
        }

        assert_eq!(stmts.next(), None);
    }

    #[test]
    fn function_statement_params() {
        let src = "function foo(bar, baz) {}";

        let program = parse(src).unwrap();

        let items = &program.items;
        let mut stmts = program.statements();

        match *stmts.next().unwrap() {
            Item::FunctionStatement {
                ref name,
                params: Some(0),
                body: None,
            } => assert_eq!(name.as_str(src), "foo"),
            _ => panic!()
        }

        // Params are linked
        let mut params = program.items.list(0);
        assert_ident!(*params.next().unwrap(), src, "bar");
        assert_ident!(*params.next().unwrap(), src, "baz");
        assert_eq!(params.next(), None);
    }

    #[test]
    fn function_statement_body() {
        let src = "function foo() { bar; baz; }";

        let program = parse(src).unwrap();
    }

    #[test]
    fn call_expression() {
        let src = "foo();";

        let program = parse(src).unwrap();
    }

    #[test]
    fn member_expression() {
        let src = "foo.bar";

        let program = parse(src).unwrap();
    }
}
