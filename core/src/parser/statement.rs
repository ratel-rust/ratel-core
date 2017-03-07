use error::Result;

use parser::Parser;
use lexer::Token::*;
use lexer::Token;
use ast::{Node, Index, Item, VariableDeclarationKind};
use ast::OperatorKind::*;

impl<'src> Parser<'src> {
    #[inline]
    pub fn statement(&mut self, token: Token<'src>) -> Result<Node<'src>> {
        match token {
            Semicolon          => Ok(self.in_loc(Item::EmptyStatement)),
            BraceOpen          => self.block_statement(),
            Declaration(kind)  => self.variable_declaration_statement(kind),
            Return             => self.return_statement(),
            Break              => self.break_statement(),
            Function           => self.function_statement(),
            // Class              => self.class_statement(),
            If                 => self.if_statement(),
            While              => self.while_statement(),
            Do                 => self.do_statement(),
            // For                => self.for_statement(),
            // Identifier(label)  => self.labeled_or_expression_statement(label),
            Throw              => self.throw_statement(),
            Try                => self.try_statement(),
            _                  => self.expression_statement(token),
        }
    }

    #[inline(always)]
    fn expect_statement(&mut self) -> Result<Node<'src>> {
        let token = next!(self);
        self.statement(token)
    }

    #[inline(always)]
    pub fn block_statement(&mut self) -> Result<Node<'src>> {
        self.block_body_tail().map(|body| Item::BlockStatement {
            body: body
        }.at(0, 0))
    }

    #[inline(always)]
    pub fn expression_statement(&mut self, token: Token<'src>) -> Result<Node<'src>> {
        let expression = self.expression_from(token, 0)?;

        let start = expression.start;
        let end = expression.end;
        let index = self.store(expression);

        expect_semicolon!(self);

        Ok(Item::ExpressionStatement(index).at(start, end))
    }

    #[inline(always)]
    pub fn function_statement(&mut self) -> Result<Node<'src>> {
        let name = expect_identifier!(self);

        expect!(self, ParenOpen);

        Ok(Item::FunctionStatement {
            name: name.into(),
            params: self.parameter_list()?,
            body: self.block_body()?,
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
                    let expression = self.expression(0)?;

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
        let declaration = Item::DeclarationStatement {
            kind: kind,
            declarators: self.variable_declarators()?
        }.at(0, 0);

        expect_semicolon!(self);

        Ok(declaration)
    }

    #[inline(always)]
    pub fn variable_declarator(&mut self) -> Result<Node<'src>> {
        let name = match next!(self) {
            BraceOpen        => self.object_expression()?,
            BracketOpen      => self.array_expression()?,
            Identifier(name) => Item::Identifier(name.into()).at(0, 0),
            _                => unexpected_token!(self),
        };
        let name = self.store(name);

        let value = match peek!(self) {
            Operator(Assign) => {
                self.consume();
                let value = self.expression(0)?;
                Some(self.store(value))
            },
            _ => None
        };

        Ok(Item::VariableDeclarator {
            name: name,
            value: value,
        }.at(0, 0))
    }

    #[inline(always)]
    pub fn variable_declarators(&mut self) -> Result<Index> {
        let node = self.variable_declarator()?;

        let mut previous = self.store(node);
        let root = previous;

        match peek!(self) {
            Comma => self.consume(),
            _     => return Ok(root),
        }

        loop {
            let node = self.variable_declarator()?;

            previous = self.chain(previous, node);

            match peek!(self) {
                Comma => self.consume(),
                _     => return Ok(root),
            }
        }
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
        let value = self.expression(0)?;
        expect_semicolon!(self);

        Ok(Item::ThrowStatement {
            value: self.store(value)
        }.at(0, 0))
    }

    #[inline(always)]
    pub fn try_statement(&mut self) -> Result<Node<'src>> {
        let body = self.block_body()?;
        expect!(self, Catch);
        expect!(self, ParenOpen);

        let error = expect_identifier!(self);
        expect!(self, ParenClose);

        let handler = self.block_body()?;
        expect_semicolon!(self);

        Ok(Item::TryStatement {
            body: body,
            error: error.into(),
            handler: handler
        }.at(0, 0))
    }

    #[inline(always)]
    pub fn if_statement(&mut self) -> Result<Node<'src>> {
        expect!(self, ParenOpen);

        let test = self.expression(0)?;
        let test = self.store(test);
        expect!(self, ParenClose);

        let consequent = self.expect_statement()?;
        let consequent = self.store(consequent);

        let alternate = match peek!(self) {
            Else => {
                self.consume();
                let statement = self.expect_statement()?;
                Some(self.store(statement))
            },
            _ => None
        };

        Ok(Item::IfStatement {
            test: test,
            consequent: consequent,
            alternate: alternate
        }.at(0, 0))
    }

    #[inline(always)]
    pub fn while_statement(&mut self) -> Result<Node<'src>> {
        expect!(self, ParenOpen);

        let test = self.expression(0)?;
        let test = self.store(test);
        expect!(self, ParenClose);

        let body = self.expect_statement()?;

        Ok(Item::WhileStatement {
            test: test,
            body: self.store(body)
        }.at(0, 0))
    }

    #[inline(always)]
    pub fn do_statement(&mut self) -> Result<Node<'src>> {
        let body = self.expect_statement()?;
        expect!(self, While);

        let test = self.expression(0)?;

        Ok(Item::DoStatement {
            body: self.store(body),
            test: self.store(test),
        }.at(0, 0))
    }
}

#[cfg(test)]
mod test {
    use ast::Item::*;
    use ast::Ident::*;
    use ast::{Value, VariableDeclarationKind};
    use parser::parse;

    #[test]
    fn function_statement_empty() {
        let src = "function foo() {}";

        let program = parse(src).unwrap();

        assert_list!(
            program.statements().items(),

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
            program.statements().items(),

            FunctionStatement {
                name: "foo".into(),
                params: Some(0),
                body: None,
            }
        );

        assert_list!(
            program.store.nodes(0).items(),

            Identifier("bar".into()),
            Identifier("baz".into())
        );
    }

    #[test]
    fn function_statement_body() {
        let src = "function foo() { bar; baz; }";

        let program = parse(src).unwrap();

        assert_list!(
            program.statements().items(),

            FunctionStatement {
                name: "foo".into(),
                params: None,
                body: Some(1),
            }
        );

        assert_list!(
            program.store.nodes(1).items(),

            ExpressionStatement(0),
            ExpressionStatement(2)
        );

        assert_ident!("bar", program[0]);
        assert_ident!("baz", program[2]);
    }

    #[test]
    fn block_statement() {
        let src = "{ true }";
        let program = parse(src).unwrap();
        assert_list!(
            program.statements().items(),
            BlockStatement { body: Some(1) }
        );
        assert_eq!(program[1], ExpressionStatement(0));
        assert_eq!(program[0], ValueExpr(Value::True));
    }

    #[test]
    fn if_statement() {
        let src = "if (true) foo;";
        let program = parse(src).unwrap();
        assert_list!(
            program.statements().items(),

            IfStatement {
                test: 0,
                consequent: 2,
                alternate: None,
            }
        );
        assert_eq!(program[0], ValueExpr(Value::True));
        assert_eq!(program[2], ExpressionStatement(1));
        assert_ident!("foo", program[1]);
    }

    #[test]
    fn if_else_statement() {
        let src = "if (true) foo; else { bar; }";
        let program = parse(src).unwrap();

        assert_list!(
            program.statements().items(),

            IfStatement {
                test: 0,
                consequent: 2,
                alternate: Some(5),
            }
        );
        assert_eq!(program[0], ValueExpr(Value::True));
        assert_eq!(program[2], ExpressionStatement(1));
        assert_eq!(program[5], BlockStatement {
            body: Some(4)
        });
        assert_ident!("foo", program[1]);
        assert_eq!(program[4], ExpressionStatement(3));
        assert_ident!("bar", program[3]);
    }

    #[test]
    fn while_statement() {
        let src = "while (true) foo;";
        let program = parse(src).unwrap();
        assert_list!(
            program.statements().items(),

            WhileStatement {
                test: 0,
                body: 2
            }
        );
        assert_eq!(program[0], ValueExpr(Value::True));
        assert_eq!(program[2], ExpressionStatement(1));
        assert_ident!("foo", program[1]);
    }

    #[test]
    fn while_statement_block() {
        let src = "while (true) { foo; }";
        let program = parse(src).unwrap();
        assert_list!(
            program.statements().items(),

            WhileStatement {
                test: 0,
                body: 3
            }
        );
        assert_eq!(program[0], ValueExpr(Value::True));
        assert_eq!(program[3], BlockStatement {
            body: Some(2)
        });
        assert_eq!(program[2], ExpressionStatement(1));
        assert_ident!("foo", program[1]);
    }

    #[test]
    fn do_statement() {
        let src = "do foo; while (true)";
        let program = parse(src).unwrap();
        assert_list!(
            program.statements().items(),

            DoStatement {
                body: 1,
                test: 2
            }
        );
        assert_eq!(program[1], ExpressionStatement(0));
        assert_ident!("foo", program[0]);
        assert_eq!(program[2], ValueExpr(Value::True));
    }

    #[test]
    fn break_statement() {
        let src = "break;";
        let program = parse(src).unwrap();
        assert_list!(
            program.statements().items(),
            BreakStatement { label: None }
        );
    }

    #[test]
    fn break_statement_label() {
        let src = "break foo;";
        let program = parse(src).unwrap();
        assert_list!(
            program.statements().items(),
            BreakStatement { label: Some(0) }
        );
        assert_ident!("foo", program[0]);
    }

    #[test]
    fn throw_statement() {
        let src = "throw '3'";
        let program = parse(src).unwrap();
        assert_list!(
            program.statements().items(),
            ThrowStatement { value: 0 }
        );
        assert_eq!(program[0], ValueExpr(Value::String("'3'")));
    }

    #[test]
    fn try_statement_empty() {
        let src = "try {} catch (err) {}";
        let program = parse(src).unwrap();
        assert_list!(
            program.statements().items(),
            TryStatement { body: None, error: "err".into(), handler: None }
        );
    }

    #[test]
    fn try_statement() {
        let src = "try { foo; } catch (err) { bar; }";
        let program = parse(src).unwrap();
        assert_list!(
            program.statements().items(),
            TryStatement { body: Some(1), error: "err".into(), handler: Some(3) }
        );
        assert_eq!(program[1], ExpressionStatement(0));
        assert_eq!(program[3], ExpressionStatement(2));
        assert_ident!("foo", program[0]);
        assert_ident!("bar", program[2]);
    }

    #[test]
    fn variable_declaration_statement() {
        let src = "var x, y, z = 42;";
        let program = parse(src).unwrap();

        assert_list!(
            program.statements().items(),
            DeclarationStatement { kind: VariableDeclarationKind::Var, declarators: 1 }
        );

        assert_list!(
            program.store.nodes(1).items(),
            VariableDeclarator { name: 0, value: None },
            VariableDeclarator { name: 2, value: None },
            VariableDeclarator { name: 4, value: Some(5) }
        );

        assert_ident!("x", program[0]);
        assert_ident!("y", program[2]);
        assert_ident!("z", program[4]);
        assert_eq!(ValueExpr(Value::Number("42")), program[5]);
    }

    #[test]
    fn variable_declaration_statement_destructuring_array() {
        let src = "let [x, y] = [1, 2];";
        let program = parse(src).unwrap();

        assert_list!(
            program.statements().items(),
            DeclarationStatement { kind: VariableDeclarationKind::Let, declarators: 6 }
        );

        assert_list!(
            program.store.nodes(6).items(),
            VariableDeclarator { name: 2, value: Some(5) }
        );

        assert_eq!(program[2], ArrayExpr(Some(0)));
        assert_list!(
            program.store.nodes(0).items(),
            Identifier(Insitu("x")),
            Identifier(Insitu("y"))
        );

        assert_eq!(program[5], ArrayExpr(Some(3)));
        assert_list!(
            program.store.nodes(3).items(),
            ValueExpr(Value::Number("1")),
            ValueExpr(Value::Number("2"))
        );
    }

    #[test]
    fn variable_declaration_statement_destructuring_object() {
        let src = "const { x, y } = { a, b };";
        let program = parse(src).unwrap();

        assert_list!(
            program.statements().items(),
            DeclarationStatement { kind: VariableDeclarationKind::Const, declarators: 6 }
        );

        assert_eq!(program[6], VariableDeclarator { name: 2, value: Some(5) });

        assert_eq!(program[2], ObjectExpr { body: Some(0) });
        assert_list!(
            program.store.nodes(0).items(),
            ShorthandMember(Insitu("x")),
            ShorthandMember(Insitu("y"))
        );

        assert_eq!(program[5], ObjectExpr { body: Some(3) });
        assert_list!(
            program.store.nodes(3).items(),
            ShorthandMember(Insitu("a")),
            ShorthandMember(Insitu("b"))
        );
    }
}
