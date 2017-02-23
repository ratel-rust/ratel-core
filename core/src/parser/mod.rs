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

    use parser::parse;
    use parser::Item::*;

    use ast::{OperatorKind, Value, VariableDeclarationKind};

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

    #[test]
    fn array_expression() {
        let src = "[0, 1, 2]";

        let program = parse(src).unwrap();

        assert_eq!(5, program.items.len());
        assert_eq!(program[3], ArrayExpr(Some(0)));
        assert_list!(
            program.statements(),
            ExpressionStatement(3)
        );

        assert_eq!(program[3], ArrayExpr(Some(0)));
        assert_eq!(program[0], ValueExpr(Value::Number("0")));
        assert_eq!(program[1], ValueExpr(Value::Number("1")));
        assert_eq!(program[2], ValueExpr(Value::Number("2")));
    }

    #[test]
    fn variable_declaration_statement() {
        let src = "var x, y, z = 42;";
        let program = parse(src).unwrap();

        assert_list!(
            program.statements(),
            DeclarationStatement { kind: VariableDeclarationKind::Var, declarators: 6 }
        );

        assert_eq!(program[6], VariableDeclarator { name: 5, value: Some(4) });
        assert_ident!("z", program[5]);
        assert_eq!(ValueExpr(Value::Number("42")), program[4]);
    }

    #[test]
    fn variable_declaration_statement_destructuring_array() {
        let src = "let [x, y] = [1, 2];";
        let program = parse(src).unwrap();

        assert_list!(
            program.statements(),
            DeclarationStatement { kind: VariableDeclarationKind::Let, declarators: 6 }
        );

        assert_eq!(program[6], VariableDeclarator { name: 2, value: Some(5) });

        assert_eq!(program[2], ArrayExpr(Some(0)));
        assert_ident!("x", program[0]);
        assert_ident!("y", program[1]);

        assert_eq!(program[5], ArrayExpr(Some(3)));
        assert_eq!(ValueExpr(Value::Number("1")), program[3]);
        assert_eq!(ValueExpr(Value::Number("2")), program[4]);
    }

    #[test]
    fn variable_declaration_statement_destructuring_object() {
        let src = "const { x, y } = { a, b };";
        let program = parse(src).unwrap();

        assert_list!(
            program.statements(),
            DeclarationStatement { kind: VariableDeclarationKind::Const, declarators: 6 }
        );

        assert_eq!(program[6], VariableDeclarator { name: 2, value: Some(5) });

        assert_eq!(program[2], ObjectExpr { body: Some(0) });
        assert_eq!(ShorthandMember("x".into()), program[0]);
        assert_eq!(ShorthandMember("y".into()), program[1]);

        assert_eq!(program[5], ObjectExpr { body: Some(3) });
        assert_eq!(ShorthandMember("a".into()), program[3]);
        assert_eq!(ShorthandMember("b".into()), program[4]);
    }

    #[test]
    fn regular_expression() {
        let src = r#"/^[A-Z]+\/[\d]+/g"#;
        let program = parse(src).unwrap();
        assert_eq!(ValueExpr(Value::RegEx { pattern: "^[A-Z]+\\/[\\d]+", flags: "g" }), program[0]);
    }

    #[test]
    fn break_statement() {
        let src = "break;";
        let program = parse(src).unwrap();
        assert_list!(
            program.statements(),
            BreakStatement { label: None }
        );
    }

    #[test]
    fn break_statement_label() {
        let src = "break foo;";
        let program = parse(src).unwrap();
        assert_list!(
            program.statements(),
            BreakStatement { label: Some(0) }
        );
        assert_ident!("foo", program[0]);
    }

    #[test]
    fn throw_statement() {
        let src = "throw '3'";
        let program = parse(src).unwrap();
        assert_list!(
            program.statements(),
            ThrowStatement { value: 0 }
        );
        assert_eq!(program[0], ValueExpr(Value::String("'3'")));
    }

    #[test]
    fn try_statement_empty() {
        let src = "try {} catch (err) {}";
        let program = parse(src).unwrap();
        assert_list!(
            program.statements(),
            TryStatement { body: None, error: "err".into(), handler: None }
        );
    }

    #[test]
    fn try_statement() {
        let src = "try { foo; } catch (err) { bar; }";
        let program = parse(src).unwrap();
        assert_list!(
            program.statements(),
            TryStatement { body: Some(1), error: "err".into(), handler: Some(3) }
        );
        assert_eq!(program[1], ExpressionStatement(0));
        assert_eq!(program[3], ExpressionStatement(2));
        assert_ident!("foo", program[0]);
        assert_ident!("bar", program[2]);

    }
}
