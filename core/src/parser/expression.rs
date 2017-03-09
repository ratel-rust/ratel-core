use parser::Parser;
use lexer::Token::*;
use lexer::Token;
use ast::{null, idx, OptIndex, Item, Node};
use ast::OperatorKind::*;

impl<'src> Parser<'src> {
    #[inline(always)]
    pub fn expression(&mut self, lbp: u8) -> Node<'src> {
        let token = next!(self);
        self.expression_from(token, lbp)
    }

    #[inline(always)]
    pub fn expression_from(&mut self, token: Token<'src>, lbp: u8) -> Node<'src> {
        let left = match token {
            This               => self.in_loc(Item::This),
            Literal(value)     => self.in_loc(Item::ValueExpr(value)),
            Identifier(value)  => self.in_loc(Item::identifier(value)),
            Operator(Division) => self.regular_expression(),
            // Operator(optype)   => self.prefix_expression(optype),
            ParenOpen          => self.paren_expression(),
            BracketOpen        => self.array_expression(),
            BraceOpen          => self.object_expression(),
            // Function           => self.function_expression(),
            // Class              => self.class_expression(),
            // Template(kind)     => self.template_expression(None, kind),
            _                  => unexpected_token!(self)
        };

        self.complex_expression(left, lbp)
    }

    #[inline(always)]
    pub fn complex_expression(&mut self, mut left: Node<'src>, lbp: u8) -> Node<'src> {
        loop {
            left = match peek!(self) {
                Operator(op @ Increment) |
                Operator(op @ Decrement) => {
                    self.consume();

                    // TODO: op.end
                    Node::new(left.start, left.end, Item::PostfixExpr {
                        operator: op,
                        operand: self.store(left),
                    })
                }

                Operator(op) => {
                    self.consume();

                    let rbp = op.binding_power();

                    if lbp > rbp {
                        break;
                    }

                    if !op.infix() {
                        unexpected_token!(self);
                    }

                    if op.assignment() {
                        // TODO: verify that left is assignable
                    }

                    let right = self.expression(rbp);

                    Node::new(left.start, right.end, Item::BinaryExpr {
                        parenthesized: false,
                        operator: op,
                        left: self.store(left),
                        right: self.store(right),
                    })
                },

                Accessor(member) => {
                    self.consume();

                    let right = self.in_loc(Item::identifier(member));

                    Node::new(left.start, right.end, Item::MemberExpr {
                        object: self.store(left),
                        property: self.store(right),
                    })
                },

                ParenOpen => {
                    if lbp > 18 {
                        break;
                    }

                    self.consume();

                    Item::CallExpr {
                        callee: self.store(left),
                        arguments: self.expression_list(),
                    }.at(0, 0)
                },

                _ => break
            }
        }

        left
    }

    pub fn expression_list(&mut self) -> OptIndex {
        let expression = match next!(self) {
            ParenClose => return null(),
            token      => self.expression_from(token, 0),
        };

        let mut previous = self.store(expression);
        let root = idx(previous);

        loop {
            let expression = match next!(self) {
                ParenClose => break,
                Comma      => self.expression(0),
                _          => unexpected_token!(self),
            };

            previous = self.chain(previous, expression);
        }

        root
    }

    #[inline(always)]
    fn paren_expression(&mut self) -> Node<'src> {
        match next!(self) {
            // ParenClose => {
            //     expect!(self, Operator(FatArrow));

            //     self.arrow_function_expression(None)
            // },
            token => {
                let expression = self.expression_from(token, 0);
                // let expression = self.sequence_or(expression);

                expect!(self, ParenClose);

                expression

                // Ok(expression.parenthesize())
            }
        }
    }

    #[inline(always)]
    pub fn object_expression(&mut self) -> Node<'src> {
        let member = match next!(self) {
            BraceClose => return self.in_loc(Item::ObjectExpr { body: null() }),

            Identifier(ident) => {
                let ident = ident.into();
                let (start, end) = self.lexer.loc();

                match next!(self) {
                    Comma => Item::ShorthandMember(ident).at(start, end),
                    BraceClose => {
                        let member = Item::ShorthandMember(ident).at(start, end);

                        return Item::ObjectExpr { body: self.store(member).into() }.at(start, end)
                    },
                    _ => unexpected_token!(self)
                }
            },

            _ => unexpected_token!(self)
        };

        let mut previous = self.store(member);
        let root = idx(previous);

        loop {
            match next!(self) {
                Identifier(ident) => {
                    let ident = ident.into();
                    let (start, end) = self.lexer.loc();

                    match next!(self) {
                        Comma => {
                            previous = self.chain(previous, Item::ShorthandMember(ident).at(start, end));

                            continue;
                        },
                        BraceClose => {
                            self.chain(previous, Item::ShorthandMember(ident).at(start, end));

                            break;
                        },
                        _ => unexpected_token!(self),
                    }
                },

                BraceClose => break,

                _ => unexpected_token!(self),
            }

            // match next!(self) {
            //     Comma => {},
            //     BraceClose => break,
            //     _ => unexpected_token!(self)
            // }
        }

        Item::ObjectExpr { body: root }.at(0, 0)
    }

    #[inline(always)]
    pub fn array_expression(&mut self) -> Node<'src> {
        let expression = match next!(self) {
            BracketClose => {
                return Item::ArrayExpr(null()).at(0,0)
            },
            token => self.expression_from(token, 0)
        };

        let mut previous = self.store(expression);
        let root = previous;

        loop {
            let expression = match next!(self) {
                BracketClose => break,
                Comma      => self.expression(0),
                _          => unexpected_token!(self),
            };

            previous = self.chain(previous, expression);
        }

        Item::ArrayExpr(root.into()).at(0,0)
    }

    #[inline(always)]
    pub fn regular_expression(&mut self) -> Node<'src> {
        let value = match self.lexer.read_regular_expression() {
            Literal(value) => value,
            _              => unexpected_token!(self),
        };

        Item::ValueExpr(value).at(0, 0)
    }

}

#[cfg(test)]
mod test {
    use ast::{null, idx, OperatorKind, Value};
    use parser::parse;
    use parser::Item::*;

    #[test]
    fn parse_ident_expr() {
        let src = "foo; bar; baz;";

        let program = parse(src).unwrap();

        // 3 times statement and expression
        assert_eq!(6, program.store.len());

        // Statements are linked
        assert_list!(
            program.statements().items(),

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
        assert_eq!(7, program.store.len());

        // Statements are linked
        assert_list!(
            program.statements().items(),

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
    fn call_expression() {
        let src = "foo();";

        let program = parse(src).unwrap();

        assert_list!(
            program.statements().items(),

            ExpressionStatement(1)
        );

        assert_eq!(
            program[1],

            CallExpr {
                callee: 0,
                arguments: null(),
            }
        );

        assert_ident!("foo", program[0]);
    }

    #[test]
    fn member_expression() {
        let src = "foo.bar";

        let program = parse(src).unwrap();

        assert_list!(
            program.statements().items(),

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
    fn keyword_member_expression() {
        let src = "foo.function";

        let program = parse(src).unwrap();

        assert_list!(
            program.statements().items(),

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
        assert_ident!("function", program[1]);
    }

    #[test]
    fn regular_expression() {
        let src = r#"/^[A-Z]+\/[\d]+/g"#;
        let program = parse(src).unwrap();
        assert_eq!(ValueExpr(Value::RegEx { pattern: "^[A-Z]+\\/[\\d]+", flags: "g" }), program[0]);
    }

    #[test]
    fn array_expression() {
        let src = "[0, 1, 2]";

        let program = parse(src).unwrap();

        assert_eq!(5, program.store.len());
        assert_eq!(program[3], ArrayExpr(idx(0)));
        assert_list!(
            program.statements().items(),
            ExpressionStatement(3)
        );

        assert_eq!(program[3], ArrayExpr(idx(0)));
        assert_eq!(program[0], ValueExpr(Value::Number("0")));
        assert_eq!(program[1], ValueExpr(Value::Number("1")));
        assert_eq!(program[2], ValueExpr(Value::Number("2")));
    }
}
