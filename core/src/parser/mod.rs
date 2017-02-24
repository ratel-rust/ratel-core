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
                store: Store::new(),
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
        self.program.store.insert(node)
    }

    #[inline(always)]
    fn store_in_loc(&mut self, item: Item<'src>) -> Index {
        let node = self.in_loc(item);
        self.store(node)
    }

    #[inline(always)]
    fn chain(&mut self, previous: Index, node: Node<'src>) -> Index {
        let index = self.store(node);
        self.program.store[previous].next = Some(index);
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
    use parser::parse;
    use parser::Item::*;

    #[test]
    fn empty_parse() {
        let program = parse("").unwrap();

        assert_eq!(0, program.store.len());
        assert_eq!(None, program.root);
        assert_list!(program.statements().items());
    }

    #[test]
    fn empty_statements() {
        let program = parse(";;;").unwrap();

        assert_eq!(3, program.store.len());

        // Statements are linked
        assert_list!(
            program.statements().items(),

            EmptyStatement,
            EmptyStatement,
            EmptyStatement
        );
    }
}
