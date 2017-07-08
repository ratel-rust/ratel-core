use parser::Parser;
use lexer::Token::*;
use lexer::Token;
use ast::{Loc, List, ListBuilder, Expression, ObjectMember, OperatorKind};
use ast::OperatorKind::*;
use std::cell::Cell;

impl<'ast> Parser<'ast> {
    #[inline]
    pub fn expression(&mut self, lbp: u8) -> Loc<Expression<'ast>> {
        let token = self.next();
        self.expression_from(token, lbp)
    }

    #[inline]
    pub fn expression_from(&mut self, token: Token<'ast>, lbp: u8) -> Loc<Expression<'ast>> {
        let mut left = match token {
            This               => self.in_loc(Expression::This),
            Literal(value)     => self.in_loc(Expression::Value(value)),
            Identifier(value)  => self.in_loc(Expression::Identifier(value)),
            Operator(Division) => self.regular_expression(),
            Operator(optype)   => self.prefix_expression(optype),
            ParenOpen          => self.paren_expression(),
            BracketOpen        => self.array_expression(),
            BraceOpen          => self.object_expression(),
            Function           => self.function_expression(),
            // Class              => self.class_expression(),
            // Template(kind)     => self.template_expression(None, kind),
            _                  => unexpected_token!(self)
        };

        loop {
            left = match self.peek() {
                Operator(op @ Increment) |
                Operator(op @ Decrement) => {
                    self.consume();

                    // TODO: op.end
                    Loc::new(left.start, left.end, Expression::Postfix {
                        operator: op,
                        operand: self.alloc(left),
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

                    Loc::new(left.start, right.end, Expression::Binary {
                        parenthesized: Cell::new(false),
                        operator: op,
                        left: self.alloc(left),
                        right: self.alloc(right),
                    })
                },

                Accessor(member) => {
                    self.consume();

                    let right = self.in_loc(member);

                    Loc::new(left.start, right.end, Expression::Member {
                        object: self.alloc(left),
                        property: self.alloc(right),
                    })
                },

                ParenOpen => {
                    if lbp > 18 {
                        break;
                    }

                    self.consume();

                    Expression::Call {
                        callee: self.alloc(left),
                        arguments: self.expression_list(),
                    }.at(0, 0)
                },

                _ => break
            }
        }

        left
    }

    #[inline]
    pub fn expression_list(&mut self) -> List<'ast, Loc<Expression<'ast>>> {
        let expression = match self.next() {
            ParenClose => return List::empty(),
            token      => self.expression_from(token, 0),
        };

        let mut builder = ListBuilder::new(self.arena, expression);

        loop {
            let expression = match self.next() {
                ParenClose => break,
                Comma      => self.expression(0),
                _          => unexpected_token!(self),
            };

            builder.push(expression);
        }

        builder.into_list()
    }

    #[inline]
    pub fn paren_expression(&mut self) -> Loc<Expression<'ast>> {
        match self.next() {
            // ParenClose => {
            //     expect!(self, Operator(FatArrow));

            //     self.arrow_function_expression(None)
            // },
            token => {
                let expression = self.expression_from(token, 0);
                // let expression = self.sequence_or(expression);

                expect!(self, ParenClose);

                expression.item.parenthesize();
                expression

                // Ok(expression.parenthesize())
            }
        }
    }

    #[inline]
    fn prefix_expression(&mut self, operator: OperatorKind) -> Loc<Expression<'ast>> {
        if !operator.prefix() {
            unexpected_token!(self);
        }

        let operand = self.expression(15);

        Expression::Prefix {
            operator: operator,
            operand: self.alloc(operand),
        }.at(0, 0)
    }

    #[inline]
    pub fn object_expression(&mut self) -> Loc<Expression<'ast>> {
        let member = match self.next() {
            Identifier(ident) => {
                let (start, end) = self.loc();

                match self.next() {
                    Comma => Loc::new(start, end, ObjectMember::Shorthand(ident)),
                    BraceClose => {
                        let member = Loc::new(start, end, ObjectMember::Shorthand(ident));
                        let builder = ListBuilder::new(self.arena, member);

                        return Expression::Object { body: builder.into_list() }.at(start, end)
                    },
                    _ => unexpected_token!(self)
                }
            },

            BraceClose => return self.in_loc(Expression::Object { body: List::empty() }),

            _ => unexpected_token!(self)
        };

        let mut builder = ListBuilder::new(self.arena, member);

        loop {
            match self.next() {
                Identifier(ident) => {
                    let ident = ident.into();
                    let (start, end) = self.loc();

                    match self.next() {
                        Comma => {
                            builder.push(Loc::new(start, end, ObjectMember::Shorthand(ident)));

                            continue;
                        },
                        BraceClose => {
                            builder.push(Loc::new(start, end, ObjectMember::Shorthand(ident)));

                            break;
                        },
                        _ => unexpected_token!(self),
                    }
                },

                BraceClose => break,

                _ => unexpected_token!(self),
            }

            // match self.next() {
            //     Comma => {},
            //     BraceClose => break,
            //     _ => unexpected_token!(self)
            // }
        }

        Expression::Object {
            body: builder.into_list()
        }.at(0, 0)
    }

    #[inline]
    pub fn array_expression(&mut self) -> Loc<Expression<'ast>> {
        let expression = match self.next() {
            BracketClose => return Expression::Array { body: List::empty() }.at(0,0),
            token        => self.expression_from(token, 0)
        };

        let mut builder = ListBuilder::new(self.arena, expression);

        loop {
            let expression = match self.next() {
                BracketClose => break,
                Comma        => self.expression(0),
                _            => unexpected_token!(self),
            };

            builder.push(expression);
        }

        Expression::Array {
            body: builder.into_list()
        }.at(0,0)
    }

    #[inline]
    pub fn regular_expression(&mut self) -> Loc<Expression<'ast>> {
        let value = match self.lexer.read_regular_expression() {
            Literal(value) => value,
            _              => unexpected_token!(self),
        };

        Expression::Value(value).at(0, 0)
    }

    #[inline]
    pub fn function_expression(&mut self) -> Loc<Expression<'ast>> {
        let name = match self.peek() {
            Identifier(name) => {
                self.consume();
                Some(self.alloc_in_loc(name))
            },
            _ => None
        };

        Expression::Function {
            function: self.function(name)
        }.at(0, 0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ast::{OperatorKind, Value, Statement, Function};
    use parser::parse;
    use parser::mock::Mock;

    #[test]
    fn parse_ident_expr() {
        let module = parse("foobar;").unwrap();

        let expected = Expression::Identifier("foobar");

        assert_expr!(module, expected);
    }

    #[test]
    fn parse_binary_expr() {
        let src = "foo + bar;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Binary {
            parenthesized: Cell::new(false),
            operator: OperatorKind::Addition,
            left: mock.ident("foo"),
            right: mock.ident("bar"),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn parse_parenthesized_binary_expr() {
        let src = "(2 + 2);";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Binary {
            parenthesized: Cell::new(true),
            operator: OperatorKind::Addition,
            left: mock.number("2"),
            right: mock.number("2"),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn parse_postfix_expr() {
        let src = "baz++;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Postfix {
            operator: OperatorKind::Increment,
            operand: mock.ident("baz"),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn call_expression() {
        let src = "foo();";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Call {
            callee: mock.ident("foo"),
            arguments: List::empty(),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn member_expression() {
        let src = "foo.bar";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Member {
            object: mock.ident("foo"),
            property: mock.ptr("bar"),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn keyword_member_expression() {
        let src = "foo.function";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Member {
            object: mock.ident("foo"),
            property: mock.ptr("function"),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn regular_expression() {
        let src = r#"/^[A-Z]+\/[\d]+/g"#;
        let module = parse(src).unwrap();

        let expected = Expression::Value(Value::RegEx("/^[A-Z]+\\/[\\d]+/g"));

        assert_expr!(module, expected);
    }

    #[test]
    fn array_expression() {
        let src = "[0, 1, 2]";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Array {
            body: mock.list([
                Expression::Value(Value::Number("0")),
                Expression::Value(Value::Number("1")),
                Expression::Value(Value::Number("2")),
            ])
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn function_expression() {
        let src = "(function () {})";
        let module = parse(src).unwrap();

        let expected = Expression::Function {
            function: Function {
                name: None.into(),
                params: List::empty(),
                body: List::empty()
            }
        };

        assert_expr!(module, expected);
    }
}
