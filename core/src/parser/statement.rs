use parser::{Parser, Parse, B0, B1};
use lexer::Token::*;
use lexer::Asi;
use ast::{Ptr, Loc, List, ListBuilder, Declarator, DeclaratorId, DeclarationKind};
use ast::{Statement, StatementPtr, Expression, ExpressionPtr, Literal};
use ast::expression::BinaryExpression;
use ast::statement::{BlockStatement, ReturnStatement, IfStatement, WhileStatement, DoStatement};
use ast::statement::{TryStatement, ThrowStatement, ContinueStatement, BreakStatement};
use ast::statement::{DeclarationStatement, ForStatement, ForInStatement, ForOfStatement};
use ast::statement::{SwitchStatement, SwitchCaseStatement, LabeledStatement};
use ast::OperatorKind;
use ast::OperatorKind::*;
use ast::{EmptyListBuilder};


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

const ____: StatementHandler = |par| return par.error();

/// Shared expression handlers that produce StatementPtr<'ast>
use parser::expression::handlers::{
    PRN, ARR, OP, REG, THIS, TRUE, FALS, NULL, UNDE, STR, NUM, BIN, TPLS, TPLE
};

const EMPT : StatementHandler = |par| {
    let stmt = par.alloc_in_loc(Statement::Empty);
    par.lexer.consume();

    stmt
};

const BLCK : StatementHandler = |par| par.block_statement();

const VAR: StatementHandler = |par| {
    par.lexer.consume();
    par.variable_declaration_statement(DeclarationKind::Var)
};

const LET: StatementHandler = |par| {
    par.lexer.consume();
    par.variable_declaration_statement(DeclarationKind::Let)
};

const CONS: StatementHandler = |par| {
    par.lexer.consume();
    par.variable_declaration_statement(DeclarationKind::Const)
};

const RET: StatementHandler = |par| {
    par.lexer.consume();
    par.return_statement()
};

const BRK: StatementHandler = |par| {
    par.lexer.consume();
    par.break_statement()
};

const CONT: StatementHandler = |par| {
    par.lexer.consume();
    par.continue_statement()
};

const FUNC: StatementHandler = |par| par.function_statement();

const CLAS: StatementHandler = |par| par.class_statement();

const IF: StatementHandler = |par| {
    par.lexer.consume();
    par.if_statement()
};

const WHL: StatementHandler = |par| {
    par.lexer.consume();
    par.while_statement()
};

const DO: StatementHandler = |par| {
    par.lexer.consume();
    par.do_statement()
};

const FOR: StatementHandler = |par| {
    par.lexer.consume();
    par.for_statement()
};

const THRW: StatementHandler = |par| {
    par.lexer.consume();
    par.throw_statement()
};

const TRY: StatementHandler = |par| {
    par.lexer.consume();
    par.try_statement()
};

const SWCH: StatementHandler = |par| {
    par.lexer.consume();
    par.switch_statement()
};

const LABL: StatementHandler = |par| {
    let label = par.lexer.token_as_str();
    par.lexer.consume();
    par.labeled_or_expression_statement(label)
};

impl<'ast> Parse<'ast> for Statement<'ast> {
    type Output = StatementPtr<'ast>;

    #[inline]
    fn parse(par: &mut Parser<'ast>) -> Self::Output {
        par.statement()
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
    pub fn labeled_or_expression_statement(&mut self, label: &'ast str) -> StatementPtr<'ast> {
        if let Colon = self.lexer.token {
            self.lexer.consume();

            let body = self.statement();

            return self.alloc_at_loc(0, 0, LabeledStatement {
                label,
                body,
            });
        }

        let expression = self.alloc_in_loc(label);
        let expression = self.nested_expression(expression, B0);

        expect_semicolon!(self);

        self.alloc_at_loc(0, 0, expression)
    }

    #[inline]
    pub fn function_statement(&mut self) -> StatementPtr<'ast> {
        let start = self.lexer.start();
        self.lexer.consume();

        let function = self.function();

        self.alloc_at_loc(start, function.body.end, function)
    }

    #[inline]
    fn class_statement(&mut self) -> StatementPtr<'ast> {
        let start = self.lexer.start();
        self.lexer.consume();

        let class = self.class();

        self.alloc_at_loc(start, 0, class)
    }

    #[inline]
    pub fn return_statement(&mut self) -> StatementPtr<'ast> {
        let value = match self.asi() {
            Asi::NoSemicolon => {
                let expression = self.expression(B0);

                expect_semicolon!(self);

                Some(expression)
            }

            Asi::ImplicitSemicolon => None,
            Asi::ExplicitSemicolon => {
                self.lexer.consume();

                None
            }
        };

        self.alloc_at_loc(0, 0, ReturnStatement { value })
    }

    #[inline]
    pub fn variable_declaration_statement(&mut self, kind: DeclarationKind) -> StatementPtr<'ast> {
        let declarators = self.variable_declarators();

        let declaration = self.alloc_at_loc(0, 0, DeclarationStatement {
            kind: kind,
            declarators
        });

        expect_semicolon!(self);

        declaration
    }

    #[inline]
    pub fn variable_declarator(&mut self) -> Ptr<'ast, Loc<Declarator<'ast>>> {
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
    pub fn variable_declarators(&mut self) -> List<'ast, Loc<Declarator<'ast>>> {
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
    pub fn break_statement(&mut self) -> StatementPtr<'ast> {
        let label = match self.asi() {
            Asi::ExplicitSemicolon => {
                self.lexer.consume();
                None
            },
            Asi::ImplicitSemicolon => None,
            Asi::NoSemicolon => {
                let label = expect_identifier!(self);

                expect_semicolon!(self);

                Some(self.alloc_in_loc(label))
            }
        };

        self.alloc_at_loc(0, 0, BreakStatement { label })
    }

    #[inline]
    pub fn continue_statement(&mut self) -> StatementPtr<'ast> {
        let label = match self.asi() {
            Asi::ExplicitSemicolon => {
                self.lexer.consume();
                None
            },
            Asi::ImplicitSemicolon => None,
            Asi::NoSemicolon => {
                let label = expect_identifier!(self);

                expect_semicolon!(self);

                Some(self.alloc_in_loc(label))
            }
        };

        self.alloc_at_loc(0, 0, ContinueStatement { label })
    }

    #[inline]
    pub fn throw_statement(&mut self) -> StatementPtr<'ast> {
        let value = self.expression(B0);

        expect_semicolon!(self);

        self.alloc_at_loc(0, 0, ThrowStatement { value })
    }

    #[inline]
    pub fn try_statement(&mut self) -> StatementPtr<'ast> {
        let body = self.block();
        expect!(self, Catch);
        expect!(self, ParenOpen);

        let error = expect_identifier!(self);
        let error = self.alloc_in_loc(error);
        expect!(self, ParenClose);

        let handler = self.block();
        expect_semicolon!(self);

        self.alloc_at_loc(0, 0, TryStatement {
            body,
            error,
            handler,
        })
    }

    #[inline]
    pub fn if_statement(&mut self) -> StatementPtr<'ast> {
        expect!(self, ParenOpen);
        let test = self.expression(B0);
        expect!(self, ParenClose);

        let consequent = self.statement();

        let alternate = match self.lexer.token {
            Else => {
                self.lexer.consume();
                Some(self.statement())
            },
            _ => None
        };

        self.alloc_at_loc(0, 0, IfStatement {
            test,
            consequent,
            alternate,
        })
    }

    #[inline]
    pub fn while_statement(&mut self) -> StatementPtr<'ast> {
        expect!(self, ParenOpen);
        let test = self.expression(B0);
        expect!(self, ParenClose);

        let body = self.statement();

        self.alloc_at_loc(0, 0, WhileStatement {
            test,
            body,
        })
    }

    #[inline]
    pub fn do_statement(&mut self) -> StatementPtr<'ast> {
        let body = self.statement();
        expect!(self, While);

        let test = self.expression(B0);

        self.alloc_at_loc(0, 0, DoStatement {
            body,
            test,
        })
    }

    #[inline]
    fn for_statement(&mut self) -> StatementPtr<'ast> {
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
                    ..
                }) = declarators.only_element() {
                    if let Expression::Binary(BinaryExpression { operator: In, right, .. }) = value.item {
                        let left = self.alloc_at_loc(0, 0, DeclarationStatement {
                            kind,
                            declarators,
                        });

                        return self.for_in_statement_from_parts(left, right);
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
                    let left = self.alloc_at_loc(0, 0, left);

                    return self.for_in_statement_from_parts(left, right);
                }

                Some(self.alloc_at_loc(0, 0, expression))
            },
        };

        if let Some(init) = init {
            match self.lexer.token {
                OperatorIn => {
                    self.lexer.consume();
                    return self.for_in_statement(init);
                },
                Identifier => {
                    if self.lexer.token_as_str() != "of" {
                        return self.error();
                    }
                    self.lexer.consume();
                    return self.for_of_statement(init);
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

        self.alloc_at_loc(0, 0, ForStatement {
            init,
            test,
            update,
            body,
        })
    }

    fn for_in_statement_from_parts(&mut self, left: StatementPtr<'ast>, right: ExpressionPtr<'ast>) -> StatementPtr<'ast> {
        expect!(self, ParenClose);

        let body = self.statement();

        self.alloc_at_loc(0, 0, ForInStatement {
            left,
            right,
            body,
        })
    }

    fn for_in_statement(&mut self, left: StatementPtr<'ast>) -> StatementPtr<'ast> {
        let right = self.expression(B0);

        expect!(self, ParenClose);

        let body = self.statement();

        self.alloc_at_loc(0, 0, ForInStatement {
            left,
            right,
            body,
        })
    }

    fn for_of_statement(&mut self, left: StatementPtr<'ast>) -> StatementPtr<'ast> {
        let right = self.expression(B0);

        expect!(self, ParenClose);

        let body = self.statement();

        self.alloc_at_loc(0, 0, ForOfStatement {
            left,
            right,
            body,
        })
    }

    fn case_statement(&mut self, expr: Option<ExpressionPtr<'ast>>) -> StatementPtr<'ast> {
        expect!(self, Colon);

        let mut consequent = EmptyListBuilder::new(self.arena);

        loop {
            match self.lexer.token {
                Case | Default | BraceClose => {
                    break;
                },
                _ => {
                    let statement = self.statement();
                    consequent.push(statement);
                    match statement.item {
                        Statement::Break { .. } | Statement::Return { .. } => {
                            break;
                        },
                        _ => {}
                    }
                }
            }
        }

        self.alloc_at_loc(0, 0, SwitchCaseStatement {
            test: expr,
            consequent: consequent.into_list()
        })
    }

    fn switch_statement(&mut self) -> StatementPtr<'ast> {
        expect!(self, ParenOpen);

        let discriminant = self.expression(B0);

        expect!(self, ParenClose);
        expect!(self, BraceOpen);

        let mut cases = EmptyListBuilder::new(self.arena);

        loop {
            match self.lexer.token {
                BraceClose => {
                    self.lexer.consume();
                    break;
                }
                Case => {
                    self.lexer.consume();
                    let expr = self.expression(B0);
                    cases.push(self.case_statement(Some(expr)));
                },
                Default => {
                    self.lexer.consume();
                    cases.push(self.case_statement(None));
                }
                _ => return self.error()
            }
        }

        self.alloc_at_loc(0, 0, SwitchStatement {
            discriminant,
            cases: cases.into_list()
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use parser::parse;
    use parser::mock::Mock;
    use ast::{List, Literal, ObjectMember, Function, Class, OperatorKind};
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
    fn try_statement_empty() {
        let src = "try {} catch (err) {}";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            TryStatement {
                body: mock.empty_block(),
                error: mock.ptr("err"),
                handler: mock.empty_block()
            }
        ]);

        assert_eq!(module.body(), expected);
    }

    #[test]
    fn try_statement() {
        let src = "try { foo; } catch (err) { bar; }";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            TryStatement {
                body: mock.block([
                    mock.ptr("foo")
                ]),
                error: mock.ptr("err"),
                handler: mock.block([
                    mock.ptr("bar")
                ]),
            }
        ]);

        assert_eq!(module.body(), expected);
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
                body: List::empty(),
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
                cases: mock.list([
                    SwitchCaseStatement {
                        test: Some(mock.number("3")),
                        consequent: mock.list([
                            BreakStatement { label: None }
                        ])
                    },
                    SwitchCaseStatement {
                        test: Some(mock.number("2")),
                        consequent: mock.list([
                            ReturnStatement { value: Some(mock.ptr("b")) }
                        ])
                    },
                    SwitchCaseStatement {
                        test: Some(mock.ptr(Expression::Literal(Literal::String("\"1\"")))),
                        consequent: List::empty()
                    },
                    SwitchCaseStatement {
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
