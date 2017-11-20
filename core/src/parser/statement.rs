use parser::{Parser, Parse, B0, B1};
use lexer::Token::*;
use lexer::Asi;
use ast::{Ptr, Loc, List, ListBuilder, EmptyListBuilder, Declarator, DeclaratorId, DeclarationKind};
use ast::{Statement, StatementPtr, Expression, ExpressionPtr, Class, Function};
use ast::expression::BinaryExpression;
use ast::statement::{ThrowStatement, ContinueStatement, BreakStatement, ReturnStatement};
use ast::statement::{TryStatement, CatchClause, IfStatement, WhileStatement, DoStatement};
use ast::statement::{DeclarationStatement, ForStatement, ForInStatement, ForOfStatement};
use ast::statement::{SwitchStatement, SwitchCase, LabeledStatement};
use ast::OperatorKind::*;


type StatementHandler = for<'ast> fn(&mut Parser<'ast>) -> StatementPtr<'ast>;

static STMT_HANDLERS: [StatementHandler; 108] = [
    ____, EMPT, ____, ____, PRN,  ____, ARR,  ____, BLCK, ____, ____, OP,
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
            fn $name<'ast>($par: &mut Parser<'ast>) -> StatementPtr<'ast> {
                $code
            }
        )*
    };
}

/// Shared expression handlers that produce StatementPtr<'ast>
use parser::expression::handlers::{
    PRN, ARR, OP, REG, THIS, TRUE, FALS, NULL, UNDE, STR, NUM, BIN, TPLS, TPLE
};

create_handlers! {
    const ____ = |par| return par.error();
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
    type Output = Ptr<'ast, Self>;

    #[inline]
    fn parse(par: &mut Parser<'ast>) -> Self::Output {
        par.statement()
    }
}

impl<'ast> Parse<'ast> for SwitchCase<'ast> {
    type Output = Ptr<'ast, Self>;

    #[inline]
    fn parse(par: &mut Parser<'ast>) -> Self::Output {
        let start = par.lexer.start();
        let test = match par.lexer.token {
            Case => {
                par.lexer.consume();

                Some(par.expression(B0))
            },
            Default => {
                par.lexer.consume();

                None
            },
            _ => return par.error()
        };

        let mut end = par.lexer.end();
        expect!(par, Colon);

        let mut builder = EmptyListBuilder::new(par.arena);

        loop {
            match par.lexer.token {
                Case | Default | BraceClose => break,
                _ => {
                    let statement = par.statement();
                    end = statement.end;
                    builder.push(statement);
                }
            }
        }

        par.alloc_at_loc(start, end, SwitchCase {
            test,
            consequent: builder.into_list()
        })
    }
}

impl<'ast> Parser<'ast> {
    #[inline]
    pub fn statement(&mut self) -> StatementPtr<'ast> {
        unsafe { (*(&STMT_HANDLERS as *const StatementHandler).offset(self.lexer.token as isize))(self) }
    }

    #[inline]
    pub fn block_statement(&mut self) -> StatementPtr<'ast> {
        let start = self.lexer.start_then_consume();
        let block = self.raw_block();
        let end   = self.lexer.end_then_consume();

        self.alloc_at_loc(start, end, block)
    }

    #[inline]
    pub fn expression_statement(&mut self, expression: ExpressionPtr<'ast>) -> StatementPtr<'ast> {
        let expression = self.nested_expression(expression, B0);

        self.wrap_expression(expression)
    }

    #[inline]
    pub fn wrap_expression(&mut self, expression: ExpressionPtr<'ast>) -> StatementPtr<'ast> {
        expect_semicolon!(self);

        self.alloc_at_loc(expression.start, expression.end, expression)
    }

    #[inline]
    pub fn labeled_or_expression_statement(&mut self) -> StatementPtr<'ast> {
        let label = self.lexer.token_as_str();
        let start = self.lexer.start_then_consume();

        if self.lexer.token == Colon {
            self.lexer.consume();

            let body = self.statement();

            return self.alloc_at_loc(start, body.end, LabeledStatement {
                label,
                body,
            });
        }

        let expression = self.alloc_in_loc(label);
        let expression = self.nested_expression(expression, B0);

        expect_semicolon!(self);

        self.alloc_at_loc(start, expression.end, expression)
    }

    #[inline]
    pub fn function_statement(&mut self) -> StatementPtr<'ast> {
        let start = self.lexer.start_then_consume();
        let function = Function::parse(self);

        self.alloc_at_loc(start, function.body.end, function)
    }

    #[inline]
    fn class_statement(&mut self) -> StatementPtr<'ast> {
        let start = self.lexer.start_then_consume();
        let class = Class::parse(self);

        self.alloc_at_loc(start, class.body.end, class)
    }

    #[inline]
    pub fn variable_declaration_statement(&mut self, kind: DeclarationKind) -> StatementPtr<'ast> {
        let start = self.lexer.start_then_consume();
        let declarators = self.variable_declarators();
        let end = self.lexer.end();
        let declaration = self.alloc_at_loc(start, end, DeclarationStatement {
            kind: kind,
            declarators
        });

        expect_semicolon!(self);

        declaration
    }

    #[inline]
    pub fn variable_declarator(&mut self) -> Ptr<'ast, Declarator<'ast>> {
        let name = match self.lexer.token {
            BraceOpen   => DeclaratorId::Pattern(self.object_expression()),
            BracketOpen => DeclaratorId::Pattern(self.array_expression()),
            Identifier  => {
                let name = self.lexer.token_as_str();
                self.lexer.consume();
                DeclaratorId::Identifier(name)
            },
            _ => return self.error(),
        };

        let value = match self.lexer.token {
            OperatorAssign => {
                self.lexer.consume();
                Some(self.expression(B1))
            },
            _ => None
        };

        self.alloc_at_loc(0, 0, Declarator {
            name,
            value,
        })
    }

    #[inline]
    pub fn variable_declarators(&mut self) -> List<'ast, Declarator<'ast>> {
        let mut builder = ListBuilder::new(self.arena, self.variable_declarator());

        match self.lexer.token {
            Comma => self.lexer.consume(),
            _     => return builder.into_list(),
        }

        loop {
            builder.push(self.variable_declarator());

            match self.lexer.token {
                Comma => self.lexer.consume(),
                _     => return builder.into_list(),
            }
        }
    }

    #[inline]
    pub fn return_statement(&mut self) -> StatementPtr<'ast> {
        let (start, mut end) = self.lexer.loc();
        self.lexer.consume();

        let value = match self.asi() {
            Asi::NoSemicolon => {
                let expression = self.expression(B0);
                end = expression.end;

                expect_semicolon!(self);

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
    pub fn break_statement(&mut self) -> StatementPtr<'ast> {
        let (start, mut end) = self.lexer.loc();
        self.lexer.consume();

        let label = match self.asi() {
            Asi::ExplicitSemicolon => {
                self.lexer.consume();
                None
            },
            Asi::ImplicitSemicolon => None,
            Asi::NoSemicolon => {
                let label = expect_identifier!(self);
                end = label.end;

                expect_semicolon!(self);

                Some(label)
            }
        };

        self.alloc_at_loc(start, end, BreakStatement { label })
    }

    #[inline]
    pub fn continue_statement(&mut self) -> StatementPtr<'ast> {
        let (start, mut end) = self.lexer.loc();
        self.lexer.consume();

        let label = match self.asi() {
            Asi::ExplicitSemicolon => {
                self.lexer.consume();
                None
            },
            Asi::ImplicitSemicolon => None,
            Asi::NoSemicolon => {
                let label = expect_identifier!(self);
                end = label.end;

                expect_semicolon!(self);

                Some(label)
            }
        };

        self.alloc_at_loc(start, end, ContinueStatement { label })
    }

    #[inline]
    pub fn throw_statement(&mut self) -> StatementPtr<'ast> {
        let start = self.lexer.start_then_consume();
        let value = self.expression(B0);

        expect_semicolon!(self);

        self.alloc_at_loc(start, value.end, ThrowStatement { value })
    }

    #[inline]
    pub fn try_statement(&mut self) -> StatementPtr<'ast> {
        let start = self.lexer.start_then_consume();
        let block = self.block::<Statement<'ast>>();

        let (handler, finalizer, end) = match self.lexer.token {
            Catch => {
                let start = self.lexer.start_then_consume();
                expect!(self, ParenOpen);
                let param = expect_identifier!(self);
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
            _ => return self.error()
        };

        self.alloc_at_loc(start, end, TryStatement {
            block,
            handler,
            finalizer,
        })
    }

    #[inline]
    pub fn if_statement(&mut self) -> StatementPtr<'ast> {
        let start = self.lexer.start_then_consume();
        expect!(self, ParenOpen);
        let test = self.expression(B0);
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
    pub fn while_statement(&mut self) -> StatementPtr<'ast> {
        let start = self.lexer.start_then_consume();
        expect!(self, ParenOpen);
        let test = self.expression(B0);
        expect!(self, ParenClose);

        let body = self.statement();

        self.alloc_at_loc(start, body.end, WhileStatement {
            test,
            body,
        })
    }

    #[inline]
    pub fn do_statement(&mut self) -> StatementPtr<'ast> {
        let start = self.lexer.start_then_consume();
        let body = self.statement();
        expect!(self, While);
        expect!(self, ParenOpen);
        let test = self.expression(B0);
        let end = self.lexer.end();
        expect!(self, ParenClose);

        self.alloc_at_loc(start, end, DoStatement {
            body,
            test,
        })
    }

    #[inline]
    fn for_statement(&mut self) -> StatementPtr<'ast> {
        let start = self.lexer.start_then_consume();
        expect!(self, ParenOpen);

        let init = match self.lexer.token {
            Semicolon => {
                self.lexer.consume();
                None
            },
            DeclarationVar | DeclarationLet | DeclarationConst => {
                // TODO: DRY
                let kind = match self.lexer.token {
                    DeclarationVar => DeclarationKind::Var,
                    DeclarationLet => DeclarationKind::Let,
                    DeclarationConst => DeclarationKind::Const,
                    _ => unreachable!()
                };
                self.lexer.consume();
                let declarators = self.variable_declarators();

                if let Some(&Loc {
                    item: Declarator {
                        value: Some(ref value),
                        ..
                    },
                    start: d_start,
                    ..
                }) = declarators.only_element() {
                    if let Expression::Binary(BinaryExpression { left, operator: In, right, .. }) = value.item {
                        let left = self.alloc_at_loc(d_start, left.end, DeclarationStatement {
                            kind,
                            declarators,
                        });

                        return self.for_in_statement_from_parts(start, left, right);
                    }
                }

                Some(self.alloc_at_loc(0, 0, DeclarationStatement {
                    kind,
                    declarators,
                }))
            }

            _ => {
                let expression = self.expression(B0);

                if let Expression::Binary(BinaryExpression {
                    operator: In,
                    left,
                    right,
                    ..
                }) = expression.item {
                    let left = self.wrap_expression(left);

                    return self.for_in_statement_from_parts(start, left, right);
                }

                Some(self.wrap_expression(expression))
            },
        };

        if let Some(init) = init {
            match self.lexer.token {
                OperatorIn => {
                    self.lexer.consume();
                    return self.for_in_statement(start, init);
                },
                Identifier => {
                    if self.lexer.token_as_str() != "of" {
                        return self.error();
                    }
                    self.lexer.consume();
                    return self.for_of_statement(start, init);
                },
                Semicolon => self.lexer.consume(),
                _         => return self.error(),
            }
        }

        let test = match self.lexer.token {
            Semicolon => {
                self.lexer.consume();
                None
            },
            _         => {
                let test = self.expression(B0);
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
                let update = self.expression(B0);
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

    fn for_in_statement_from_parts(&mut self, start: u32, left: StatementPtr<'ast>, right: ExpressionPtr<'ast>) -> StatementPtr<'ast> {
        expect!(self, ParenClose);

        let body = self.statement();

        self.alloc_at_loc(start, body.end, ForInStatement {
            left,
            right,
            body,
        })
    }

    fn for_in_statement(&mut self, start: u32, left: StatementPtr<'ast>) -> StatementPtr<'ast> {
        let right = self.expression(B0);

        expect!(self, ParenClose);

        let body = self.statement();

        self.alloc_at_loc(start, body.end, ForInStatement {
            left,
            right,
            body,
        })
    }

    fn for_of_statement(&mut self, start: u32, left: StatementPtr<'ast>) -> StatementPtr<'ast> {
        let right = self.expression(B0);

        expect!(self, ParenClose);

        let body = self.statement();

        self.alloc_at_loc(start, body.end, ForOfStatement {
            left,
            right,
            body,
        })
    }

    fn switch_statement(&mut self) -> StatementPtr<'ast> {
        let start = self.lexer.start_then_consume();
        expect!(self, ParenOpen);

        let discriminant = self.expression(B0);

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
    use ast::{List, Literal, ObjectMember, Function, Class, OperatorKind, BlockStatement};
    use ast::expression::*;

    #[test]
    fn block_statement() {
        let src = "{ true }";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            BlockStatement {
                body: mock.list([
                    mock.ptr(Literal::True)
                ])
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn labeled_block_statement() {
        let src = "foobar: { true }";
        let module = parse(src).unwrap();
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

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn if_statement() {
        let src = "if (true) foo;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            IfStatement {
                test: mock.ptr(Literal::True),
                consequent: mock.ptr(mock.ptr("foo")),
                alternate: None
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn if_else_statement() {
        let src = "if (true) foo; else { bar; }";
        let module = parse(src).unwrap();
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

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn while_statement() {
        let src = "while (true) foo;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            WhileStatement {
                test: mock.ptr(Literal::True),
                body: mock.ptr(mock.ptr("foo"))
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn while_statement_block() {
        let src = "while (true) { foo; }";
        let module = parse(src).unwrap();
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

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn do_statement() {
        let src = "do foo; while (true)";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            DoStatement {
                body: mock.ptr(mock.ptr("foo")),
                test: mock.ptr(Literal::True)
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn break_statement() {
        let src = "break;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            BreakStatement {
                label: None,
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn break_statement_label() {
        let src = "break foo;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            BreakStatement {
                label: Some(mock.ptr("foo")),
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn throw_statement() {
        let src = "throw '3'";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            ThrowStatement {
                value: mock.ptr(Literal::String("'3'")),
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn try_statement() {
        let src = "try {} catch (err) {}";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            TryStatement {
                block: mock.empty_block(),
                handler: Some(mock.ptr(CatchClause {
                    param: mock.ptr("err"),
                    body: mock.empty_block()
                })),
                finalizer: None
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn try_statement_finally() {
        let src = "try { foo; } finally { bar; }";
        let module = parse(src).unwrap();
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

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn try_statement_full() {
        let src = "try { foo; } catch (err) { bar; } finally { qux; }";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            TryStatement {
                block: mock.block([
                    mock.ptr("foo")
                ]),
                handler: Some(mock.ptr(CatchClause {
                    param: mock.ptr("err"),
                    body: mock.block([
                        mock.ptr("bar")
                    ])
                })),
                finalizer: Some(mock.block([
                    mock.ptr("qux")
                ])),
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn try_statement_no_tail() {
        assert!(parse("try {}").is_err())
    }

    #[test]
    fn variable_declaration_statement() {
        let src = "var x, y, z = 42;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            DeclarationStatement {
                kind: DeclarationKind::Var,
                declarators: mock.list([
                    Declarator {
                        name: DeclaratorId::Identifier("x"),
                        value: None,
                    },
                    Declarator {
                        name: DeclaratorId::Identifier("y"),
                        value: None,
                    },
                    Declarator {
                        name: DeclaratorId::Identifier("z"),
                        value: Some(mock.number("42"))
                    }
                ])
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn variable_declaration_statement_destructuring_array() {
        let src = "let [x, y] = [1, 2];";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            DeclarationStatement {
                kind: DeclarationKind::Let,
                declarators: mock.list([
                    Declarator {
                        name: DeclaratorId::Pattern(mock.ptr(ArrayExpression {
                            body: mock.list([
                                Expression::Identifier("x"),
                                Expression::Identifier("y"),
                            ])
                        })),
                        value: Some(mock.ptr(ArrayExpression {
                            body: mock.list([
                                Expression::Literal(Literal::Number("1")),
                                Expression::Literal(Literal::Number("2")),
                            ])
                        })),
                    },
                ])
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn variable_declaration_statement_destructuring_object() {
        let src = "const { x, y } = { a, b };";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            DeclarationStatement {
                kind: DeclarationKind::Const,
                declarators: mock.list([
                    Declarator {
                        name: DeclaratorId::Pattern(mock.ptr(ObjectExpression {
                            body: mock.list([
                                ObjectMember::Shorthand("x"),
                                ObjectMember::Shorthand("y"),
                            ])
                        })),
                        value: Some(mock.ptr(ObjectExpression {
                            body: mock.list([
                                ObjectMember::Shorthand("a"),
                                ObjectMember::Shorthand("b"),
                            ])
                        })),
                    },
                ])
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn for_statement() {
        let src = "for (let i = 0; i < 10; i++) {}";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            ForStatement {
                init: Some(mock.ptr(DeclarationStatement {
                    kind: DeclarationKind::Let,
                    declarators: mock.list([
                        Declarator {
                            name: DeclaratorId::Identifier("i"),
                            value: Some(mock.number("0")),
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
                    body: List::empty()
                })
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn empty_for_statement() {
        let src = "for (;;) {}";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            ForStatement {
                init: None,
                test: None,
                update: None,
                body: mock.ptr(BlockStatement {
                    body: List::empty()
                })
            }
        ]);

        assert_eq!(module.body(), expected);
    }


    #[test]
    fn for_statement_continue() {
        let src = "for (let i = 0, j = 10; i < 10; i++, j--) { continue; }";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            ForStatement {
                init: Some(mock.ptr(DeclarationStatement {
                    kind: DeclarationKind::Let,
                    declarators: mock.list([
                        Declarator {
                            name: DeclaratorId::Identifier("i"),
                            value: Some(mock.number("0")),
                        },
                        Declarator {
                            name: DeclaratorId::Identifier("j"),
                            value: Some(mock.number("10")),
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

        assert_eq!(module.body(), expected);
    }


    #[test]
    fn for_statement_sequences() {
        let src = "for (let i = 0, j = 10; i < 10; i++, j--) {}";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            ForStatement {
                init: Some(mock.ptr(DeclarationStatement {
                    kind: DeclarationKind::Let,
                    declarators: mock.list([
                        Declarator {
                            name: DeclaratorId::Identifier("i"),
                            value: Some(mock.number("0")),
                        },
                        Declarator {
                            name: DeclaratorId::Identifier("j"),
                            value: Some(mock.number("10")),
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
                    body: List::empty()
                })
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn function_statement() {
        let src = "function foo() {}";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Function {
                name: mock.name("foo"),
                params: List::empty(),
                body: mock.empty_block(),
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn function_statement_must_have_name() {
        assert!(parse("function() {}").is_err());
    }

    #[test]
    fn class_statement() {
        let src = "class Foo {}";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Class {
                name: mock.name("Foo"),
                extends: None,
                body: mock.empty_block(),
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    #[should_panic]
    fn class_statement_must_have_name() {
        parse("class {}").unwrap();
    }

    #[test]
    fn switch_statement() {
        let mock = Mock::new();
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
                        consequent: List::empty()
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

        let module = parse(src).unwrap();
        assert_eq!(module.body(), expected);
    }
}
