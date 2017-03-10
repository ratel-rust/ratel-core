use parser::Parser;
use lexer::Token::*;
use lexer::{Asi, Token};
use ast::{null, idx, Index, Item, Node, VariableDeclarationKind};
use ast::OperatorKind::*;

impl<'src> Parser<'src> {
    #[inline]
    pub fn statement(&mut self, token: Token<'src>) -> Node<'src> {
        match token {
            Semicolon          => self.in_loc(Item::EmptyStatement),
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
    fn expect_statement(&mut self) -> Node<'src> {
        let token = self.next();
        self.statement(token)
    }

    #[inline(always)]
    pub fn block_statement(&mut self) -> Node<'src> {
        Item::BlockStatement {
            body: self.block_body_tail()
        }.at(0, 0)
    }

    #[inline(always)]
    pub fn expression_statement(&mut self, token: Token<'src>) -> Node<'src> {
        let expression = self.expression_from(token, 0);

        let start = expression.start;
        let end = expression.end;
        let index = self.store(expression);

        expect_semicolon!(self);

        Item::ExpressionStatement(index).at(start, end)
    }

    #[inline(always)]
    pub fn function_statement(&mut self) -> Node<'src> {
        let name = expect_identifier!(self);

        let name = self.store_in_loc(Item::identifier(name));

        expect!(self, ParenOpen);

        Item::FunctionStatement {
            name: name,
            params: self.parameter_list(),
            body: self.block_body(),
        }.at(0, 0)
    }

    #[inline(always)]
    pub fn return_statement(&mut self) -> Node<'src> {
        let value = match self.asi() {
            Asi::NoSemicolon => {
                let expression = self.expression(0);

                expect_semicolon!(self);

                self.store(expression).into()
            }

            Asi::ImplicitSemicolon => null(),
            Asi::ExplicitSemicolon => {
                self.consume();

                null()
            }
        };

        Item::ReturnStatement {
            value: value,
        }.at(0, 0)
    }

    #[inline(always)]
    pub fn variable_declaration_statement(&mut self, kind: VariableDeclarationKind) -> Node<'src> {
        let declaration = Item::DeclarationStatement {
            kind: kind,
            declarators: self.variable_declarators()
        }.at(0, 0);

        expect_semicolon!(self);

        declaration
    }

    #[inline(always)]
    pub fn variable_declarator(&mut self) -> Node<'src> {
        let name = match self.next() {
            BraceOpen        => self.object_expression(),
            BracketOpen      => self.array_expression(),
            Identifier(name) => self.in_loc(Item::identifier(name)),
            _                => unexpected_token!(self),
        };
        let name = self.store(name);

        let value = match self.peek() {
            Operator(Assign) => {
                self.consume();
                let value = self.expression(0);

                self.store(value).into()
            },
            _ => null()
        };

        Item::VariableDeclarator {
            name: name,
            value: value,
        }.at(0, 0)
    }

    #[inline(always)]
    pub fn variable_declarators(&mut self) -> Index {
        let node = self.variable_declarator();

        let mut previous = self.store(node);
        let root = previous;

        match self.peek() {
            Comma => self.consume(),
            _     => return root,
        }

        loop {
            let node = self.variable_declarator();

            previous = self.chain(previous, node);

            match self.peek() {
                Comma => self.consume(),
                _     => return root,
            }
        }
    }

    #[inline(always)]
    pub fn break_statement(&mut self) -> Node<'src> {
        let label = match self.asi() {
            Asi::ExplicitSemicolon => {
                self.consume();
                null()
            },
            Asi::ImplicitSemicolon => null(),
            Asi::NoSemicolon => {
                let label = expect_identifier!(self);

                expect_semicolon!(self);

                idx(self.store_in_loc(Item::identifier(label)))
            }
        };

        Item::BreakStatement {
            label: label
        }.at(0, 0)
    }

    #[inline(always)]
    pub fn throw_statement(&mut self) -> Node<'src> {
        let value = self.expression(0);
        expect_semicolon!(self);

        Item::ThrowStatement {
            value: self.store(value)
        }.at(0, 0)
    }

    #[inline(always)]
    pub fn try_statement(&mut self) -> Node<'src> {
        let body = self.block_body();
        expect!(self, Catch);
        expect!(self, ParenOpen);

        let error = expect_identifier!(self);
        let error = self.store_in_loc(Item::identifier(error));
        expect!(self, ParenClose);

        let handler = self.block_body();
        expect_semicolon!(self);

        Item::TryStatement {
            body: body,
            error: error,
            handler: handler
        }.at(0, 0)
    }

    #[inline(always)]
    pub fn if_statement(&mut self) -> Node<'src> {
        expect!(self, ParenOpen);

        let test = self.expression(0);
        let test = self.store(test);
        expect!(self, ParenClose);

        let consequent = self.expect_statement();
        let consequent = self.store(consequent);

        let alternate = match self.peek() {
            Else => {
                self.consume();
                let statement = self.expect_statement();
                self.store(statement).into()
            },
            _ => null()
        };

        Item::IfStatement {
            test: test,
            consequent: consequent,
            alternate: alternate
        }.at(0, 0)
    }

    #[inline(always)]
    pub fn while_statement(&mut self) -> Node<'src> {
        expect!(self, ParenOpen);

        let test = self.expression(0);
        let test = self.store(test);
        expect!(self, ParenClose);

        let body = self.expect_statement();

        Item::WhileStatement {
            test: test,
            body: self.store(body)
        }.at(0, 0)
    }

    #[inline(always)]
    pub fn do_statement(&mut self) -> Node<'src> {
        let body = self.expect_statement();
        expect!(self, While);

        let test = self.expression(0);

        Item::DoStatement {
            body: self.store(body),
            test: self.store(test),
        }.at(0, 0)
    }
}

#[cfg(test)]
mod test {
    use ast::Item::*;
    use ast::Ident::*;
    use ast::{null, idx, Value, VariableDeclarationKind};
    use parser::parse;

    #[test]
    fn function_statement_empty() {
        let src = "function foo() {}";

        let program = parse(src).unwrap();

        assert_list!(
            program.statements().items(),

            FunctionStatement {
                name: 0,
                params: null(),
                body: null(),
            }
        );

        assert_ident!("foo", program[0]);
    }

    #[test]
    fn function_statement_params() {
        let src = "function foo(bar, baz) {}";

        let program = parse(src).unwrap();

        assert_list!(
            program.statements().items(),

            FunctionStatement {
                name: 0,
                params: idx(1),
                body: null(),
            }
        );

        assert_ident!("foo", program[0]);

        assert_list!(
            program.store.nodes(1).items(),

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
                name: 0,
                params: null(),
                body: idx(2),
            }
        );

        assert_ident!("foo", program[0]);

        assert_list!(
            program.store.nodes(2).items(),

            ExpressionStatement(1),
            ExpressionStatement(3)
        );

        assert_ident!("bar", program[1]);
        assert_ident!("baz", program[3]);
    }

    #[test]
    fn block_statement() {
        let src = "{ true }";
        let program = parse(src).unwrap();
        assert_list!(
            program.statements().items(),
            BlockStatement { body: idx(1) }
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
                alternate: null(),
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
                alternate: idx(5),
            }
        );
        assert_eq!(program[0], ValueExpr(Value::True));
        assert_eq!(program[2], ExpressionStatement(1));
        assert_eq!(program[5], BlockStatement {
            body: idx(4)
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
            body: idx(2)
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
            BreakStatement { label: null() }
        );
    }

    #[test]
    fn break_statement_label() {
        let src = "break foo;";
        let program = parse(src).unwrap();
        assert_list!(
            program.statements().items(),
            BreakStatement { label: idx(0) }
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
            TryStatement {
                body: null(),
                error: 0,
                handler: null(),
            }
        );

        assert_ident!("err", program[0]);
    }

    #[test]
    fn try_statement() {
        let src = "try { foo; } catch (err) { bar; }";
        let program = parse(src).unwrap();
        assert_list!(
            program.statements().items(),
            TryStatement {
                body: idx(1),
                error: 2,
                handler: idx(4)
            }
        );
        assert_ident!("err", program[2]);

        assert_list!(
            program.store.nodes(1).items(),
            ExpressionStatement(0)
        );

        assert_list!(
            program.store.nodes(4).items(),
            ExpressionStatement(3)
        );

        assert_ident!("foo", program[0]);
        assert_ident!("bar", program[3]);
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
            VariableDeclarator { name: 0, value: null() },
            VariableDeclarator { name: 2, value: null() },
            VariableDeclarator { name: 4, value: idx(5) }
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
            VariableDeclarator { name: 2, value: idx(5) }
        );

        assert_eq!(program[2], ArrayExpr(idx(0)));
        assert_list!(
            program.store.nodes(0).items(),
            Identifier(Insitu("x")),
            Identifier(Insitu("y"))
        );

        assert_eq!(program[5], ArrayExpr(idx(3)));
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

        assert_eq!(program[6], VariableDeclarator { name: 2, value: idx(5) });

        assert_eq!(program[2], ObjectExpr { body: idx(0) });
        assert_list!(
            program.store.nodes(0).items(),
            ShorthandMember(Insitu("x")),
            ShorthandMember(Insitu("y"))
        );

        assert_eq!(program[5], ObjectExpr { body: idx(3) });
        assert_list!(
            program.store.nodes(3).items(),
            ShorthandMember(Insitu("a")),
            ShorthandMember(Insitu("b"))
        );
    }
}
