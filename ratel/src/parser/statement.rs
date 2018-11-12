use toolshed::list::{ListBuilder, GrowableList};
use parser::{Parser, Parse, ANY, B0};
use lexer::Token::*;
use lexer::Asi;
use ast::{Node, NodeList, Declarator, DeclarationKind};
use ast::{Statement, StatementNode, Expression, ExpressionNode, Class, Function, Pattern};
use ast::expression::BinaryExpression;
use ast::statement::{ThrowStatement, ContinueStatement, BreakStatement, ReturnStatement};
use ast::statement::{TryStatement, CatchClause, IfStatement, WhileStatement, DoStatement};
use ast::statement::{DeclarationStatement, ForStatement, ForInStatement, ForOfStatement};
use ast::statement::{SwitchStatement, SwitchCase, LabeledStatement, ForInit};
use ast::OperatorKind::*;


type StatementHandler = for<'ast> fn(&mut Parser<'ast>) -> StatementNode<'ast>;

static STMT_HANDLERS: [StatementHandler; 108] = [
    ____, EMPT, ____, ____, PRN,  ____, ARR,  ____, BLCK, ____, ____, NEW,
//  EOF   ;     :     ,     (     )     [     ]     {     }     =>    NEW

    OP,   OP,   OP,   OP,   OP,   OP,   OP,   ____, REG,  ____, ____, OP,
//  ++    --    !     ~     TYPOF VOID  DELET *     /     %     **    +

    OP,   ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//  -     <<    >>    >>>   <     <=    >     >=    INSOF IN    ===   !==

    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//  ==    !=    &     ^     |     &&    ||    ?     =     +=    -=    **=

    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, VAR,  LET,
//  *=    /=    %=    <<=   >>=   >>>=  &=    ^=    |=    ...   VAR   LET

    CONS, BRK,  DO,   ____, ____, ____, ____, CLAS, ____, RET,  WHL,  ____,
//  CONST BREAK DO    CASE  ELSE  CATCH EXPRT CLASS EXTND RET   WHILE FINLY

    ____, ____, CONT, FOR,  SWCH, ____, ____, FUNC, THIS, ____, IF,   THRW,
//  SUPER WITH  CONT  FOR   SWTCH YIELD DBGGR FUNCT THIS  DEFLT IF    THROW

    ____, TRY,  ____, TRUE, FALS, NULL, UNDE, STR,  NUM,  BIN,  ____, ____,
//  IMPRT TRY   STATI TRUE  FALSE NULL  UNDEF STR   NUM   BIN   REGEX ENUM

    ____, ____, ____, ____, ____, ____, LABL, ____, TPLE, TPLS, ____, ____,
//  IMPL  PCKG  PROT  IFACE PRIV  PUBLI IDENT ACCSS TPL_O TPL_C ERR_T ERR_E
];


macro_rules! create_handlers {
    ($( const $name:ident = |$par:ident| $code:expr; )*) => {
        $(
            #[allow(non_snake_case)]
            fn $name<'ast>($par: &mut Parser<'ast>) -> StatementNode<'ast> {
                $code
            }
        )*
    };
}

/// Shared expression handlers that produce StatementNode<'ast>
use parser::expression::handlers::{
    PRN, ARR, OP, NEW, REG, THIS, TRUE, FALS, NULL, UNDE, STR, NUM, BIN, TPLS, TPLE
};

create_handlers! {
    const ____ = |par| {
        let loc = par.lexer.start();
        par.error::<()>();
        par.alloc_at_loc(loc, loc, Statement::Empty)
    };
    const EMPT = |par| {
        let stmt = par.alloc_in_loc(Statement::Empty);
        par.lexer.consume();

        stmt
    };
    const BLCK = |par| par.block_statement();
    const VAR  = |par| par.variable_declaration_statement(DeclarationKind::Var);
    const LET  = |par| par.variable_declaration_statement(DeclarationKind::Let);
    const CONS = |par| par.variable_declaration_statement(DeclarationKind::Const);
    const RET  = |par| par.return_statement();
    const BRK  = |par| par.break_statement();
    const THRW = |par| par.throw_statement();
    const CONT = |par| par.continue_statement();
    const FUNC = |par| par.function_statement();
    const CLAS = |par| par.class_statement();
    const IF   = |par| par.if_statement();
    const WHL  = |par| par.while_statement();
    const DO   = |par| par.do_statement();
    const FOR  = |par| par.for_statement();
    const TRY  = |par| par.try_statement();
    const SWCH = |par| par.switch_statement();
    const LABL = |par| par.labeled_or_expression_statement();
}

impl<'ast> Parse<'ast> for Statement<'ast> {
    type Output = Node<'ast, Self>;

    #[inline]
    fn parse(par: &mut Parser<'ast>) -> Self::Output {
        par.statement()
    }
}

impl<'ast> Parse<'ast> for SwitchCase<'ast> {
    type Output = Node<'ast, Self>;

    #[inline]
    fn parse(par: &mut Parser<'ast>) -> Self::Output {
        let start = par.lexer.start();
        let test = match par.lexer.token {
            Case => {
                par.lexer.consume();

                Some(par.expression::<ANY>())
            },
            Default => {
                par.lexer.consume();

                None
            },
            _ => {
                par.error::<()>();

                None
            }
        };

        let mut end = par.lexer.end();
        expect!(par, Colon);

        let builder = GrowableList::new();

        loop {
            match par.lexer.token {
                Case | Default | BraceClose => break,
                _ => {
                    let statement = par.statement();
                    end = statement.end;
                    builder.push(par.arena, statement);
                }
            }
        }

        par.alloc_at_loc(start, end, SwitchCase {
            test,
            consequent: builder.as_list()
        })
    }
}

impl<'ast> Parser<'ast> {
    #[inline]
    pub fn statement(&mut self) -> StatementNode<'ast> {
        unsafe { (*(&STMT_HANDLERS as *const StatementHandler).offset(self.lexer.token as isize))(self) }
    }

    /// Expect a semicolon to terminate a statement. Will assume a semicolon
    /// following the ASI rules.
    #[inline]
    fn expect_semicolon(&mut self) {
        match self.asi() {
            Asi::ExplicitSemicolon => self.lexer.consume(),
            Asi::ImplicitSemicolon => {},
            Asi::NoSemicolon       => self.error(),
        }
    }

    #[inline]
    pub fn block_statement(&mut self) -> StatementNode<'ast> {
        let start = self.lexer.start_then_consume();
        let block = self.raw_block();
        let end   = self.lexer.end_then_consume();

        self.alloc_at_loc(start, end, block)
    }

    #[inline]
    pub fn expression_statement(&mut self, expression: ExpressionNode<'ast>) -> StatementNode<'ast> {
        let expression = self.nested_expression::<ANY>(expression);

        self.wrap_expression(expression)
    }

    #[inline]
    pub fn wrap_expression(&mut self, expression: ExpressionNode<'ast>) -> StatementNode<'ast> {
        self.expect_semicolon();
        self.alloc_at_loc(expression.start, expression.end, expression)
    }

    #[inline]
    pub fn labeled_or_expression_statement(&mut self) -> StatementNode<'ast> {
        let label = self.lexer.token_as_str();
        let (start, end) = self.lexer.loc();

        self.lexer.consume();

        if self.lexer.token == Colon {
            self.lexer.consume();

            let body = self.statement();

            return self.alloc_at_loc(start, body.end, LabeledStatement {
                label,
                body,
            });
        }

        let expression = self.alloc_at_loc(start, end, label);
        let expression = self.nested_expression::<ANY>(expression);

        self.expect_semicolon();

        self.alloc_at_loc(start, expression.end, expression)
    }

    #[inline]
    pub fn function_statement(&mut self) -> StatementNode<'ast> {
        let start = self.lexer.start_then_consume();
        let function = Function::parse(self);

        self.alloc_at_loc(start, function.body.end, function)
    }

    #[inline]
    fn class_statement(&mut self) -> StatementNode<'ast> {
        let start = self.lexer.start_then_consume();
        let class = Class::parse(self);

        self.alloc_at_loc(start, class.body.end, class)
    }

    #[inline]
    pub fn variable_declaration_statement(&mut self, kind: DeclarationKind) -> StatementNode<'ast> {
        let start = self.lexer.start_then_consume();
        let declarators = self.variable_declarators();
        let end = self.lexer.end();
        let declaration = self.alloc_at_loc(start, end, DeclarationStatement {
            kind,
            declarators
        });

        self.expect_semicolon();

        declaration
    }

    #[inline]
    pub fn variable_declarator(&mut self) -> Node<'ast, Declarator<'ast>> {
        let id = Pattern::parse(self);

        let (init, end) = match self.lexer.token {
            OperatorAssign => {
                self.lexer.consume();
                let init = self.expression::<B0>();

                (Some(init), init.end)
            },
            _ => (None, id.end)
        };

        self.alloc_at_loc(id.start, end, Declarator {
            id,
            init,
        })
    }

    #[inline]
    pub fn variable_declarators(&mut self) -> NodeList<'ast, Declarator<'ast>> {
        let builder = ListBuilder::new(self.arena, self.variable_declarator());

        match self.lexer.token {
            Comma => self.lexer.consume(),
            _     => return builder.as_list(),
        }

        loop {
            builder.push(self.arena, self.variable_declarator());

            match self.lexer.token {
                Comma => self.lexer.consume(),
                _     => return builder.as_list(),
            }
        }
    }

    #[inline]
    pub fn return_statement(&mut self) -> StatementNode<'ast> {
        let (start, mut end) = self.lexer.loc();
        self.lexer.consume();

        let value = match self.asi() {
            Asi::NoSemicolon => {
                let expression = self.expression::<ANY>();
                end = expression.end;

                self.expect_semicolon();

                Some(expression)
            }

            Asi::ImplicitSemicolon => None,
            Asi::ExplicitSemicolon => {
                self.lexer.consume();

                None
            }
        };

        self.alloc_at_loc(start, end, ReturnStatement { value })
    }

    #[inline]
    pub fn break_statement(&mut self) -> StatementNode<'ast> {
        let (start, mut end) = self.lexer.loc();
        self.lexer.consume();

        let label = match self.asi() {
            Asi::ExplicitSemicolon => {
                self.lexer.consume();
                None
            },
            Asi::ImplicitSemicolon => None,
            Asi::NoSemicolon => {
                let label = self.identifier();
                end = label.end;

                self.expect_semicolon();

                Some(label)
            }
        };

        self.alloc_at_loc(start, end, BreakStatement { label })
    }

    #[inline]
    pub fn continue_statement(&mut self) -> StatementNode<'ast> {
        let (start, mut end) = self.lexer.loc();
        self.lexer.consume();

        let label = match self.asi() {
            Asi::ExplicitSemicolon => {
                self.lexer.consume();
                None
            },
            Asi::ImplicitSemicolon => None,
            Asi::NoSemicolon => {
                let label = self.identifier();
                end = label.end;

                self.expect_semicolon();

                Some(label)
            }
        };

        self.alloc_at_loc(start, end, ContinueStatement { label })
    }

    #[inline]
    pub fn throw_statement(&mut self) -> StatementNode<'ast> {
        let start = self.lexer.start_then_consume();
        let value = self.expression::<ANY>();

        self.expect_semicolon();

        self.alloc_at_loc(start, value.end, ThrowStatement { value })
    }

    #[inline]
    pub fn try_statement(&mut self) -> StatementNode<'ast> {
        let start = self.lexer.start_then_consume();
        let block = self.block();

        let (handler, finalizer, end) = match self.lexer.token {
            Catch => {
                let start = self.lexer.start_then_consume();
                expect!(self, ParenOpen);
                let param = Pattern::parse(self);
                expect!(self, ParenClose);
                let body = self.block();

                let handler = self.alloc_at_loc(start, body.end, CatchClause {
                    param,
                    body,
                });

                match self.lexer.token {
                    Finally => {
                        self.lexer.consume();
                        let block = self.block();

                        (Some(handler), Some(block), block.end)
                    },
                    _ => (Some(handler), None, handler.end)
                }
            },
            Finally => {
                self.lexer.consume();
                let block = self.block();

                (None, Some(block), block.end)
            },
            _ => {
                self.error::<()>();

                (None, None, block.end)
            }
        };

        self.alloc_at_loc(start, end, TryStatement {
            block,
            handler,
            finalizer,
        })
    }

    #[inline]
    pub fn if_statement(&mut self) -> StatementNode<'ast> {
        let start = self.lexer.start_then_consume();
        expect!(self, ParenOpen);
        let test = self.expression::<ANY>();
        expect!(self, ParenClose);

        let consequent = self.statement();

        let (alternate, end) = match self.lexer.token {
            Else => {
                self.lexer.consume();
                let alternate = self.statement();
                (Some(alternate), alternate.end)
            },
            _ => (None, consequent.end)
        };

        self.alloc_at_loc(start, end, IfStatement {
            test,
            consequent,
            alternate,
        })
    }

    #[inline]
    pub fn while_statement(&mut self) -> StatementNode<'ast> {
        let start = self.lexer.start_then_consume();
        expect!(self, ParenOpen);
        let test = self.expression::<ANY>();
        expect!(self, ParenClose);

        let body = self.statement();

        self.alloc_at_loc(start, body.end, WhileStatement {
            test,
            body,
        })
    }

    #[inline]
    pub fn do_statement(&mut self) -> StatementNode<'ast> {
        let start = self.lexer.start_then_consume();
        let body = self.statement();
        expect!(self, While);
        expect!(self, ParenOpen);
        let test = self.expression::<ANY>();
        let end = self.lexer.end();
        expect!(self, ParenClose);

        self.alloc_at_loc(start, end, DoStatement {
            body,
            test,
        })
    }

    #[inline]
    fn for_init(&mut self, kind: DeclarationKind) -> Node<'ast, ForInit<'ast>> {
        let start = self.lexer.start_then_consume();
        let declarators = self.variable_declarators();
        let end = self.lexer.end();
        
        self.alloc_at_loc(start, end, DeclarationStatement {
            kind,
            declarators
        })
    }

    #[inline]
    fn for_statement(&mut self) -> StatementNode<'ast> {
        let start = self.lexer.start_then_consume();
        expect!(self, ParenOpen);

        let init = match self.lexer.token {
            Semicolon => {
                self.lexer.consume();
                None
            },
            DeclarationVar   => Some(self.for_init(DeclarationKind::Var)),
            DeclarationLet   => Some(self.for_init(DeclarationKind::Let)),
            DeclarationConst => Some(self.for_init(DeclarationKind::Const)),
            _ => {
                let init = self.expression::<ANY>();

                if let Expression::Binary(BinaryExpression {
                    operator: In,
                    left,
                    right,
                    ..
                }) = init.item {
                    let left = self.alloc_at_loc(left.start, left.end, left);

                    return self.for_in_statement_from_parts(start, left, right);
                }

                Some(self.alloc_at_loc(init.start, init.end, init))
            },
        };

        if let Some(ref init) = init {
            match self.lexer.token {
                OperatorIn => {
                    self.lexer.consume();
                    return self.for_in_statement(start, *init);
                },
                Identifier if self.lexer.token_as_str() == "of" => {
                    self.lexer.consume();
                    return self.for_of_statement(start, *init);
                },
                _ => expect!(self, Semicolon)
            }
        }

        let test = match self.lexer.token {
            Semicolon => {
                self.lexer.consume();
                None
            },
            _ => {
                let test = self.expression::<ANY>();
                expect!(self, Semicolon);

                Some(test)
            }
        };

        let update = match self.lexer.token {
            ParenClose => {
                self.lexer.consume();
                None
            },
            _         => {
                let update = self.expression::<ANY>();
                expect!(self, ParenClose);

                Some(update)
            }
        };

        let body = self.statement();

        self.alloc_at_loc(start, body.end, ForStatement {
            init,
            test,
            update,
            body,
        })
    }

    fn for_in_statement_from_parts(&mut self, start: u32, left: Node<'ast, ForInit<'ast>>, right: ExpressionNode<'ast>) -> StatementNode<'ast> {
        expect!(self, ParenClose);

        let body = self.statement();

        self.alloc_at_loc(start, body.end, ForInStatement {
            left,
            right,
            body,
        })
    }

    fn for_in_statement(&mut self, start: u32, left: Node<'ast, ForInit<'ast>>) -> StatementNode<'ast> {
        let right = self.expression::<ANY>();

        expect!(self, ParenClose);

        let body = self.statement();

        self.alloc_at_loc(start, body.end, ForInStatement {
            left,
            right,
            body,
        })
    }

    fn for_of_statement(&mut self, start: u32, left: Node<'ast, ForInit<'ast>>) -> StatementNode<'ast> {
        let right = self.expression::<ANY>();

        expect!(self, ParenClose);

        let body = self.statement();

        self.alloc_at_loc(start, body.end, ForOfStatement {
            left,
            right,
            body,
        })
    }

    fn switch_statement(&mut self) -> StatementNode<'ast> {
        let start = self.lexer.start_then_consume();
        expect!(self, ParenOpen);

        let discriminant = self.expression::<ANY>();

        expect!(self, ParenClose);

        let cases = self.block();

        self.alloc_at_loc(start, cases.end, SwitchStatement {
            discriminant,
            cases
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use parser::parse;
    use parser::mock::Mock;
    use ast::{NodeList, Literal, Function, Class, OperatorKind, BlockStatement};
    use ast::expression::*;

    #[test]
    fn block_statement() {
        let src = "{ true }";
        let mock = Mock::new();

        let expected = mock.list([
            BlockStatement {
                body: mock.list([
                    mock.ptr(Literal::True)
                ])
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn labeled_block_statement() {
        let src = "foobar: { true }";
        let mock = Mock::new();

        let expected = mock.list([
            LabeledStatement {
                label: "foobar",
                body: mock.ptr(BlockStatement {
                    body: mock.list([
                        mock.ptr(Literal::True)
                    ])
                })
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn if_statement() {
        let src = "if (true) foo;";
        let mock = Mock::new();

        let expected = mock.list([
            IfStatement {
                test: mock.ptr(Literal::True),
                consequent: mock.ptr(mock.ptr("foo")),
                alternate: None
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn if_else_statement() {
        let src = "if (true) foo; else { bar; }";
        let mock = Mock::new();

        let expected = mock.list([
            IfStatement {
                test: mock.ptr(Literal::True),
                consequent: mock.ptr(mock.ptr("foo")),
                alternate: Some(mock.ptr(BlockStatement {
                    body: mock.list([
                        mock.ptr("bar")
                    ])
                }))
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn while_statement() {
        let src = "while (true) foo;";
        let mock = Mock::new();

        let expected = mock.list([
            WhileStatement {
                test: mock.ptr(Literal::True),
                body: mock.ptr(mock.ptr("foo"))
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn while_statement_block() {
        let src = "while (true) { foo; }";
        let mock = Mock::new();

        let expected = mock.list([
            WhileStatement {
                test: mock.ptr(Literal::True),
                body: mock.ptr(BlockStatement {
                    body: mock.list([
                        mock.ptr("foo")
                    ])
                })
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn do_statement() {
        let src = "do foo; while (true)";
        let mock = Mock::new();

        let expected = mock.list([
            DoStatement {
                body: mock.ptr(mock.ptr("foo")),
                test: mock.ptr(Literal::True)
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn break_statement() {
        let src = "break;";
        let mock = Mock::new();

        let expected = mock.list([
            BreakStatement {
                label: None,
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn break_statement_label() {
        let src = "break foo;";
        let mock = Mock::new();

        let expected = mock.list([
            BreakStatement {
                label: Some(mock.ptr("foo")),
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn throw_statement() {
        let src = "throw '3'";
        let mock = Mock::new();

        let expected = mock.list([
            ThrowStatement {
                value: mock.ptr(Literal::String("'3'")),
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn try_statement() {
        let src = "try {} catch (err) {}";
        let mock = Mock::new();

        let expected = mock.list([
            TryStatement {
                block: mock.empty_block(),
                handler: Some(mock.ptr(CatchClause {
                    param: mock.ptr(Pattern::Identifier("err")),
                    body: mock.empty_block()
                })),
                finalizer: None
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn try_statement_finally() {
        let src = "try { foo; } finally { bar; }";
        let mock = Mock::new();

        let expected = mock.list([
            TryStatement {
                block: mock.block([
                    mock.ptr("foo")
                ]),
                handler: None,
                finalizer: Some(mock.block([
                    mock.ptr("bar")
                ])),
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn try_statement_full() {
        let src = "try { foo; } catch (err) { bar; } finally { qux; }";
        let mock = Mock::new();

        let expected = mock.list([
            TryStatement {
                block: mock.block([
                    mock.ptr("foo")
                ]),
                handler: Some(mock.ptr(CatchClause {
                    param: mock.ptr(Pattern::Identifier("err")),
                    body: mock.block([
                        mock.ptr("bar")
                    ])
                })),
                finalizer: Some(mock.block([
                    mock.ptr("qux")
                ])),
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn try_statement_no_tail() {
        assert!(parse("try {}").is_err())
    }

    #[test]
    fn variable_declaration_statement() {
        let src = "var x, y, z = 42;";
        let mock = Mock::new();

        let expected = mock.list([
            DeclarationStatement {
                kind: DeclarationKind::Var,
                declarators: mock.list([
                    Declarator {
                        id: mock.ptr(Pattern::Identifier("x")),
                        init: None,
                    },
                    Declarator {
                        id: mock.ptr(Pattern::Identifier("y")),
                        init: None,
                    },
                    Declarator {
                        id: mock.ptr(Pattern::Identifier("z")),
                        init: Some(mock.number("42"))
                    }
                ])
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn variable_declaration_statement_spread() {
        let src = "const a = {...foo}";
        let mock = Mock::new();

        let expected = mock.list([
            DeclarationStatement {
                kind: DeclarationKind::Const,
                declarators: mock.list([
                    Declarator {
                        id: mock.ptr(Pattern::Identifier("a")),
                        init: Some(mock.ptr(ObjectExpression {
                            body: mock.list([
                                Property::Spread {
                                    argument: mock.ptr("foo")
                                },
                            ])
                        }))
                    }
                ])
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn variable_declaration_statement_destructuring_array() {
        let src = "let [x, y] = [1, 2];";
        let mock = Mock::new();

        let expected = mock.list([
            DeclarationStatement {
                kind: DeclarationKind::Let,
                declarators: mock.list([
                    Declarator {
                        id: mock.ptr(Pattern::ArrayPattern {
                            elements: mock.list([
                                Pattern::Identifier("x"),
                                Pattern::Identifier("y")
                            ])
                        }),
                        init: Some(mock.ptr(ArrayExpression {
                            body: mock.list([
                                Expression::Literal(Literal::Number("1")),
                                Expression::Literal(Literal::Number("2")),
                            ])
                        })),
                    },
                ])
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn variable_declaration_statement_destructuring_array_sparse() {
        let src = "let [, foo] = bar;";
        let mock = Mock::new();

        let expected = mock.list([
            DeclarationStatement {
                kind: DeclarationKind::Let,
                declarators: mock.list([
                    Declarator {
                        id: mock.ptr(Pattern::ArrayPattern {
                            elements: mock.list([
                                Pattern::Void,
                                Pattern::Identifier("foo")
                            ])
                        }),
                        init: Some(mock.ptr("bar")),
                    },
                ])
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn variable_declaration_statement_destructuring_object() {
        let src = "const { x, y } = { a, b };";
        let mock = Mock::new();

        let expected = mock.list([
            DeclarationStatement {
                kind: DeclarationKind::Const,
                declarators: mock.list([
                    Declarator {
                        id: mock.ptr(Pattern::ObjectPattern {
                            properties: mock.list([
                                Property::Shorthand("x"),
                                Property::Shorthand("y"),
                            ])
                        }),
                        init: Some(mock.ptr(ObjectExpression {
                            body: mock.list([
                                Property::Shorthand("a"),
                                Property::Shorthand("b"),
                            ])
                        })),
                    },
                ])
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn for_statement() {
        let src = "for (let i = 0; i < 10; i++) {}";
        let mock = Mock::new();

        let expected = mock.list([
            ForStatement {
                init: Some(mock.ptr(DeclarationStatement {
                    kind: DeclarationKind::Let,
                    declarators: mock.list([
                        Declarator {
                            id: mock.ptr(Pattern::Identifier("i")),
                            init: Some(mock.number("0")),
                        }
                    ]),
                })),
                test: Some(mock.ptr(BinaryExpression {
                    operator: OperatorKind::Lesser,
                    left: mock.ptr("i"),
                    right: mock.number("10"),
                })),
                update: Some(mock.ptr(PostfixExpression {
                    operator: OperatorKind::Increment,
                    operand: mock.ptr("i")
                })),
                body: mock.ptr(BlockStatement {
                    body: NodeList::empty()
                })
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn empty_for_statement() {
        let src = "for (;;) {}";
        let mock = Mock::new();

        let expected = mock.list([
            ForStatement {
                init: None,
                test: None,
                update: None,
                body: mock.ptr(BlockStatement {
                    body: NodeList::empty()
                })
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }


    #[test]
    fn for_statement_continue() {
        let src = "for (let i = 0, j = 10; i < 10; i++, j--) { continue; }";
        let mock = Mock::new();

        let expected = mock.list([
            ForStatement {
                init: Some(mock.ptr(DeclarationStatement {
                    kind: DeclarationKind::Let,
                    declarators: mock.list([
                        Declarator {
                            id: mock.ptr(Pattern::Identifier("i")),
                            init: Some(mock.number("0")),
                        },
                        Declarator {
                            id: mock.ptr(Pattern::Identifier("j")),
                            init: Some(mock.number("10")),
                        }
                    ]),
                })),
                test: Some(mock.ptr(BinaryExpression {
                    operator: OperatorKind::Lesser,
                    left: mock.ptr("i"),
                    right: mock.number("10"),
                })),
                update: Some(mock.ptr(SequenceExpression {
                    body: mock.list([
                        Expression::Postfix(PostfixExpression {
                            operator: OperatorKind::Increment,
                            operand: mock.ptr("i")
                        }),
                        Expression::Postfix(PostfixExpression {
                            operator: OperatorKind::Decrement,
                            operand: mock.ptr("j")
                        })
                    ])
                })),
                body: mock.ptr(BlockStatement {
                    body: mock.list([
                        ContinueStatement {
                            label: None
                        }
                    ]),
                })
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }


    #[test]
    fn for_statement_sequences() {
        let src = "for (let i = 0, j = 10; i < 10; i++, j--) {}";
        let mock = Mock::new();

        let expected = mock.list([
            ForStatement {
                init: Some(mock.ptr(DeclarationStatement {
                    kind: DeclarationKind::Let,
                    declarators: mock.list([
                        Declarator {
                            id: mock.ptr(Pattern::Identifier("i")),
                            init: Some(mock.number("0")),
                        },
                        Declarator {
                            id: mock.ptr(Pattern::Identifier("j")),
                            init: Some(mock.number("10")),
                        }
                    ]),
                })),
                test: Some(mock.ptr(BinaryExpression {
                    operator: OperatorKind::Lesser,
                    left: mock.ptr("i"),
                    right: mock.number("10"),
                })),
                update: Some(mock.ptr(SequenceExpression {
                    body: mock.list([
                        Expression::Postfix(PostfixExpression {
                            operator: OperatorKind::Increment,
                            operand: mock.ptr("i")
                        }),
                        Expression::Postfix(PostfixExpression {
                            operator: OperatorKind::Decrement,
                            operand: mock.ptr("j")
                        })
                    ])
                })),
                body: mock.ptr(BlockStatement {
                    body: NodeList::empty()
                })
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn function_statement() {
        let src = "function foo() {}";
        let mock = Mock::new();

        let expected = mock.list([
            Function {
                name: mock.name("foo"),
                generator: false,
                params: NodeList::empty(),
                body: mock.empty_block(),
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn function_statement_must_have_name() {
        assert!(parse("function() {}").is_err());
    }

    #[test]
    fn class_statement() {
        let src = "class Foo {}";
        let mock = Mock::new();

        let expected = mock.list([
            Class {
                name: mock.name("Foo"),
                extends: None,
                body: mock.empty_block(),
            }
        ]);

        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    #[should_panic]
    fn class_statement_must_have_name() {
        parse("class {}").unwrap();
    }

    #[test]
    fn switch_statement() {
        let src = r#"
        switch (times) {
            case 3:
                break;
            case 2:
                return b;
            case "1":
            default:
                return false;
        }
        "#;
        let mock = Mock::new();

        let expected = mock.list([
            SwitchStatement {
                discriminant: mock.ptr("times"),
                cases: mock.block([
                    SwitchCase {
                        test: Some(mock.number("3")),
                        consequent: mock.list([
                            BreakStatement { label: None }
                        ])
                    },
                    SwitchCase {
                        test: Some(mock.number("2")),
                        consequent: mock.list([
                            ReturnStatement { value: Some(mock.ptr("b")) }
                        ])
                    },
                    SwitchCase {
                        test: Some(mock.ptr(Expression::Literal(Literal::String("\"1\"")))),
                        consequent: NodeList::empty()
                    },
                    SwitchCase {
                        test: None,
                        consequent: mock.list([
                            ReturnStatement { value: Some(mock.ptr(Expression::Literal(Literal::False))) }
                        ])
                    },
                ])
            }
        ]);
        assert_eq!(parse(src).unwrap().body(), expected);
    }
}
