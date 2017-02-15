use error::Result;

use parser::Parser;
use lexer::Token::*;
use lexer::Token;
use ast::{Node, Index, Item, OperatorKind, VariableDeclarationKind};
use ast::Item::*;

impl<'src> Parser<'src> {
    #[inline(always)]
    pub fn statement(&mut self, token: Token<'src>) -> Result<Node<'src>> {
        match token {
            Semicolon          => Ok(self.in_loc(EmptyStatement)),
            // BraceOpen          => self.block_statement(),
            Declaration(kind)  => self.variable_declaration_statement(kind),
            Return             => self.return_statement(),
            Break              => self.break_statement(),
            Function           => self.function_statement(),
            // Class              => self.class_statement(),
            // If                 => self.if_statement(),
            // While              => self.while_statement(),
            // Do                 => self.do_statement(),
            // For                => self.for_statement(),
            // Identifier(label)  => self.labeled_or_expression_statement(label),
            Throw              => self.throw_statement(),
            Try                => self.try_statement(),
            _                  => self.expression_statement(token),
        }
    }

    #[inline(always)]
    pub fn expression_statement(&mut self, token: Token<'src>) -> Result<Node<'src>> {
        let expression = try!(self.expression_from(token, 0));

        let start = expression.start;
        let end = expression.end;
        let index = self.store(expression);

        expect_semicolon!(self);

        Ok(ExpressionStatement(index).at(start, end))
    }

    #[inline(always)]
    pub fn function_statement(&mut self) -> Result<Node<'src>> {
        let name = expect_identifier!(self);

        expect!(self, ParenOpen);

        Ok(Item::FunctionStatement {
            name: name.into(),
            params: try!(self.parameter_list()),
            body: try!(self.block_body()),
        }.at(0, 0))
    }

    #[inline(always)]
    pub fn return_statement(&mut self) -> Result<Node<'src>> {
        let value = match peek!(self) {
            EndOfProgram => None,
            Semicolon    => None,
            _            => {
                if self.lexer.asi() {
                    None
                } else {
                    let expression = try!(self.expression(0));

                    Some(self.store(expression))
                }
            }
        };

        expect_semicolon!(self);

        Ok(Item::ReturnStatement {
            value: value,
        }.at(0, 0))
    }

    #[inline(always)]
    pub fn variable_declaration_statement(&mut self, kind: VariableDeclarationKind) -> Result<Node<'src>> {
        let declaration = Item::DeclarationStatemenet {
            kind: kind,
            declarators: try!(self.variable_declarators())
        }.at(0, 0);

        expect_semicolon!(self);

        Ok(declaration)
    }

    #[inline(always)]
    pub fn variable_declarators(&mut self) -> Result<Index> {
        let mut previous = None;
        loop {
            let index = match peek!(self) {
                BraceOpen => {
                    self.consume();
                    unimplemented!()
                },
                BracketOpen => {
                    self.consume();

                    let id = try!(self.array_expression());
                    let name = self.store(id);

                    let value = match peek!(self) {
                        Operator(Assign) => {
                            self.consume();
                            let value = try!(self.expression(0));
                            Some(self.store(value))
                        },
                        _ => None
                    };
                    self.store(Item::VariableDeclarator {
                        name: name,
                        value: value,
                    }.at(0, 0))
                },
                _        => {
                    let name = expect_identifier!(self);
                    let value = match peek!(self) {
                        Operator(Assign) => {
                            self.consume();
                            let value = try!(self.expression(0));
                            Some(self.store(value))
                        },
                        _ => None
                    };

                    let name = self.store(Item::Identifier(name.into()).at(0, 0));
                    self.store(Item::VariableDeclarator {
                        name: name,
                        value: value,
                    }.at(0, 0))
                }
            };

            allow!(self, Comma => continue);

            match previous {
                Some(previous) => {
                    self.program.items[previous].next = Some(index);
                },
                _ => {}
            };

            previous = Some(index);
            break;
        }

        if let Some(index) = previous {
            return Ok(index)
        }

        unexpected_token!(self)
    }

    #[inline(always)]
    pub fn break_statement(&mut self) -> Result<Node<'src>> {
        let statement = Item::BreakStatement {
            label: match peek!(self) {
                Semicolon => {
                    self.consume();
                    None
                },
                EndOfProgram => None,
                _ => {
                    if self.lexer.asi() {
                        None
                    } else {
                        let label = expect_identifier!(self);
                        let id = self.store(Item::Identifier(label.into()).at(0, 0));
                        expect_semicolon!(self);
                        Some(id)
                    }
                }
            }
        };

        Ok(statement.at(0, 0))
    }

    #[inline(always)]
    pub fn throw_statement(&mut self) -> Result<Node<'src>> {
        let value = try!(self.expression(0));
        expect_semicolon!(self);

        Ok(Item::ThrowStatement {
            value: self.store(value)
        }.at(0, 0))
    }

    #[inline(always)]
    pub fn try_statement(&mut self) -> Result<Node<'src>> {
        let body = try!(self.block_body());
        expect!(self, Catch);
        expect!(self, ParenOpen);

        let error = expect_identifier!(self);
        expect!(self, ParenClose);

        let handler = try!(self.block_body());
        expect_semicolon!(self);

        Ok(Item::TryStatement {
            body: body,
            error: error.into(),
            handler: handler
        }.at(0, 0))
    }
}

#[cfg(test)]
mod test {
    use ast::Value;
    use ast::Item::*;
    use parser::parse;

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
