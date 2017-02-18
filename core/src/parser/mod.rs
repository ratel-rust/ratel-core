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
    token: Option<Token<'src>>,

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

    #[inline(always)]
    fn consume(&mut self) {
        self.token = None;
    }

    #[inline(always)]
    fn in_loc(&self, item: Item<'src>) -> Node<'src> {
        let (start, end) = self.lexer.loc();

        Node::new(start, end, item)
    }

    #[inline(always)]
    fn store(&mut self, node: Node<'src>) -> Index {
        self.program.items.insert(node)
    }

    #[inline(always)]
    fn store_in_loc(&mut self, item: Item<'src>) -> Index {
        let node = self.in_loc(item);
        self.store(node)
    }

    #[inline(always)]
    fn chain(&mut self, previous: Index, node: Node<'src>) -> Index {
        let index = self.store(node);
        self.program.items[previous].next = Some(index);
        index
    }

    #[inline(always)]
    fn chain_in_loc(&mut self, previous: Index, item: Item<'src>) -> Index {
        let node = self.in_loc(item);
        self.chain(previous, node)
    }

    #[inline(always)]
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

            previous = self.chain(previous, statement);
        }

        Ok(())
    }

    #[inline(always)]
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

            previous = self.chain(previous, statement);
        }

        Ok(root)
    }

    #[inline(always)]
    fn block_body(&mut self) -> Result<Option<Index>> {
        expect!(self, BraceOpen);
        self.block_body_tail()
    }

    #[inline(always)]
    fn parameter_list(&mut self) -> Result<Option<Index>> {
        let name = match next!(self) {
            ParenClose       => return Ok(None),
            Identifier(name) => name,
            _                => unexpected_token!(self),
        };

        let mut previous = self.store_in_loc(Item::Identifier(name.into()));
        let root = Some(previous);

        loop {
            let name = match next!(self) {
                ParenClose => break,
                Comma      => expect_identifier!(self),
                _          => unexpected_token!(self),
            };

            previous = self.chain_in_loc(previous, Item::Identifier(name.into()));
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
    use ast::OperatorKind;
    use super::parse;
    use super::Item::*;

    macro_rules! assert_ident {
        ($expect:expr, $item:expr) => {
            assert_eq!(Identifier($expect.into()), $item);
        }
    }

    macro_rules! assert_list {
        ($iter:expr $( ,$item:expr)*) => ({
            let mut iter = $iter;
            $(
                assert_eq!($item, *iter.next().unwrap());
            )*
            assert_eq!(None, iter.next());
        })
    }

    #[test]
    fn empty_parse() {
        let program = parse("").unwrap();

        assert_eq!(0, program.items.len());
        assert_eq!(None, program.root);
        assert_list!(program.statements());
    }

    #[test]
    fn empty_statements() {
        let program = parse(";;;").unwrap();

        assert_eq!(3, program.items.len());

        // Statements are linked
        assert_list!(
            program.statements(),

            EmptyStatement,
            EmptyStatement,
            EmptyStatement
        );
    }

    #[test]
    fn parse_ident_expr() {
        let src = "foo; bar; baz;";

        let program = parse(src).unwrap();

        // 3 times statement and expression
        assert_eq!(6, program.items.len());

        // Statements are linked
        assert_list!(
            program.statements(),

            ExpressionStatement(0),
            ExpressionStatement(2),
            ExpressionStatement(4)
        );

        // Match identifiers
        assert_ident!("foo", program[0]);
        assert_ident!("bar", program[2]);
        assert_ident!("baz", program[4]);
    }

    #[test]
    fn parse_binary_and_postfix_expr() {
        let src = "foo + bar; baz++;";

        let program = parse(src).unwrap();

        // 2 statements, 3 simple expressions, one binary expression, one postfix expression
        assert_eq!(7, program.items.len());

        // Statements are linked
        assert_list!(
            program.statements(),

            ExpressionStatement(2),
            ExpressionStatement(5)
        );

        // Binary expression
        assert_eq!(
            program[2],

            BinaryExpr {
                parenthesized: false,
                operator: OperatorKind::Addition,
                left: 0,
                right: 1,
            }
        );

        assert_ident!("foo", program[0]);
        assert_ident!("bar", program[1]);

        // Postfix expression
        assert_eq!(
            program[5],

            PostfixExpr {
                operator: OperatorKind::Increment,
                operand: 4
            }
        );

        assert_ident!("baz", program[4]);
    }

    #[test]
    fn function_statement_empty() {
        let src = "function foo() {}";

        let program = parse(src).unwrap();

        assert_list!(
            program.statements(),

            FunctionStatement {
                name: "foo".into(),
                params: None,
                body: None,
            }
        );
    }

    #[test]
    fn function_statement_params() {
        let src = "function foo(bar, baz) {}";

        let program = parse(src).unwrap();

        assert_list!(
            program.statements(),

            FunctionStatement {
                name: "foo".into(),
                params: Some(0),
                body: None,
            }
        );

        assert_list!(
            program.items.list(0),

            Identifier("bar".into()),
            Identifier("baz".into())
        );
    }

    #[test]
    fn function_statement_body() {
        let src = "function foo() { bar; baz; }";

        let program = parse(src).unwrap();

        assert_list!(
            program.statements(),

            FunctionStatement {
                name: "foo".into(),
                params: None,
                body: Some(1),
            }
        );

        assert_list!(
            program.items.list(1),

            ExpressionStatement(0),
            ExpressionStatement(2)
        );

        assert_ident!("bar", program[0]);
        assert_ident!("baz", program[2]);
    }

    #[test]
    fn call_expression() {
        let src = "foo();";

        let program = parse(src).unwrap();

        assert_list!(
            program.statements(),

            ExpressionStatement(1)
        );

        assert_eq!(
            program[1],

            CallExpr {
                callee: 0,
                arguments: None,
            }
        );

        assert_ident!("foo", program[0]);
    }

    #[test]
    fn member_expression() {
        let src = "foo.bar";

        let program = parse(src).unwrap();

        assert_list!(
            program.statements(),

            ExpressionStatement(2)
        );

        assert_eq!(
            program[2],

            MemberExpr {
                object: 0,
                property: 1,
            }
        );

        assert_ident!("foo", program[0]);
        assert_ident!("bar", program[1]);
    }
}
