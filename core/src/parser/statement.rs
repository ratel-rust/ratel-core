use parser::Parser;
use lexer::Token::*;
use lexer::Asi;
use ast::{Ptr, Loc, List, ListBuilder, Declarator, DeclaratorId, DeclarationKind};
use ast::{Statement, StatementPtr, Expression, ExpressionPtr, Value};
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

const ____: StatementHandler = |par| unexpected_token!(par);

/// Shared expression handlers that produce StatementPtr<'ast>
use parser::expression::handlers::{
    PRN, ARR, OP, REG, THIS, TRUE, FALS, NULL, UNDE, STR, NUM, BIN, TPLS, TPLE
};

const EMPT : StatementHandler = |par| {
    let stmt = par.alloc_in_loc(Statement::Empty);
    par.lexer.consume();

    stmt
};

const BLCK : StatementHandler = |par| {
    par.lexer.consume();
    par.block_statement()
};

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

const CLAS: StatementHandler = |par| {
    par.lexer.consume();
    par.class_statement()
};

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

const FUNC: StatementHandler = |par| {
    par.lexer.consume();
    par.function_statement()
};

const LABL: StatementHandler = |par| {
    let label = par.lexer.token_as_str();
    par.lexer.consume();
    par.labeled_or_expression_statement(label)
};

impl<'ast> Parser<'ast> {
    #[inline]
    pub fn statement(&mut self) -> StatementPtr<'ast> {
        unsafe { (*(&STMT_HANDLERS as *const StatementHandler).offset(self.lexer.token as isize))(self) }
    }

    #[inline]
    pub fn block_statement(&mut self) -> StatementPtr<'ast> {
        let body = self.block_body_tail();

        self.alloc(Statement::Block { body }.at(0, 0))
    }

    #[inline]
    pub fn expression_statement(&mut self, expression: ExpressionPtr<'ast>) -> StatementPtr<'ast> {
        let expression = self.nested_expression(expression, 0);

        self.wrap_expression(expression)
    }

    #[inline]
    pub fn wrap_expression(&mut self, expression: ExpressionPtr<'ast>) -> StatementPtr<'ast> {
        let start = expression.start;
        let end = expression.end;

        expect_semicolon!(self);

        self.alloc(Statement::Expression { expression }.at(start, end))
    }

    #[inline]
    pub fn labeled_or_expression_statement(&mut self, label: &'ast str) -> StatementPtr<'ast> {
        if let Colon = self.lexer.token {
            self.lexer.consume();

            let body = self.statement();

            return self.alloc(Statement::Labeled {
                label,
                body,
            }.at(0, 0))
        }

        let expression = self.alloc_in_loc(Expression::Identifier(label));
        let expression = self.nested_expression(expression, 0);

        expect_semicolon!(self);

        self.alloc(Statement::Expression { expression }.at(0, 0))
    }

    #[inline]
    pub fn function_statement(&mut self) -> StatementPtr<'ast> {
        let name = expect_identifier!(self);
        let name = self.alloc_in_loc(name);
        let function = self.function(name);

        self.alloc(Statement::Function { function }.at(0, 0))
    }

    #[inline]
    fn class_statement(&mut self) -> StatementPtr<'ast> {
        let name = expect_identifier!(self);
        let name = self.alloc_in_loc(name);
        let class = self.class(name);

        self.alloc(Statement::Class { class }.at(0, 0))
    }

    #[inline]
    pub fn return_statement(&mut self) -> StatementPtr<'ast> {
        let value = match self.asi() {
            Asi::NoSemicolon => {
                let expression = self.expression(0);

                expect_semicolon!(self);

                Some(expression)
            }

            Asi::ImplicitSemicolon => None,
            Asi::ExplicitSemicolon => {
                self.lexer.consume();

                None
            }
        };

        self.alloc(Statement::Return { value }.at(0, 0))
    }

    #[inline]
    pub fn variable_declaration_statement(&mut self, kind: DeclarationKind) -> StatementPtr<'ast> {
        let declarators = self.variable_declarators();

        let declaration = self.alloc(Statement::Declaration {
            kind: kind,
            declarators
        }.at(0, 0));

        expect_semicolon!(self);

        declaration
    }

    #[inline]
    pub fn variable_declarator(&mut self) -> Ptr<'ast, Loc<Declarator<'ast>>> {
        let name = match self.lexer.token {
            BraceOpen   => {
                self.lexer.consume();
                DeclaratorId::Pattern(self.object_expression())
            },
            BracketOpen => {
                self.lexer.consume();
                DeclaratorId::Pattern(self.array_expression())
            },
            Identifier  => {
                let name = self.lexer.token_as_str();
                self.lexer.consume();
                DeclaratorId::Identifier(name)
            },
            _                => unexpected_token!(self),
        };

        let value = match self.lexer.token {
            OperatorAssign => {
                self.lexer.consume();
                Some(self.expression(1))
            },
            _ => None
        };

        self.alloc(Loc::new(0, 0, Declarator {
            name,
            value,
        }))
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

                Some(self.alloc_in_loc(Expression::Identifier(label)))
            }
        };

        self.alloc(Statement::Break { label }.at(0, 0))
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

                Some(self.alloc_in_loc(Expression::Identifier(label)))
            }
        };

        self.alloc(Statement::Continue { label }.at(0, 0))
    }

    #[inline]
    pub fn throw_statement(&mut self) -> StatementPtr<'ast> {
        let value = self.expression(0);

        expect_semicolon!(self);

        self.alloc(Statement::Throw { value }.at(0, 0))
    }

    #[inline]
    pub fn try_statement(&mut self) -> StatementPtr<'ast> {
        let body = self.block_body();
        expect!(self, Catch);
        expect!(self, ParenOpen);

        let error = expect_identifier!(self);
        let error = self.alloc_in_loc(Expression::Identifier(error));
        expect!(self, ParenClose);

        let handler = self.block_body();
        expect_semicolon!(self);

        self.alloc(Statement::Try {
            body,
            error,
            handler,
        }.at(0, 0))
    }

    #[inline]
    pub fn if_statement(&mut self) -> StatementPtr<'ast> {
        expect!(self, ParenOpen);
        let test = self.expression(0);
        expect!(self, ParenClose);

        let consequent = self.statement();

        let alternate = match self.lexer.token {
            Else => {
                self.lexer.consume();
                Some(self.statement())
            },
            _ => None
        };

        self.alloc(Statement::If {
            test,
            consequent,
            alternate,
        }.at(0, 0))
    }

    #[inline]
    pub fn while_statement(&mut self) -> StatementPtr<'ast> {
        expect!(self, ParenOpen);
        let test = self.expression(0);
        expect!(self, ParenClose);

        let body = self.statement();

        self.alloc(Statement::While {
            test,
            body,
        }.at(0, 0))
    }

    #[inline]
    pub fn do_statement(&mut self) -> StatementPtr<'ast> {
        let body = self.statement();
        expect!(self, While);

        let test = self.expression(0);

        self.alloc(Statement::Do {
            body,
            test,
        }.at(0, 0))
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
                    if let Expression::Binary { operator: In, right, .. } = value.item {
                        let left = self.alloc(Statement::Declaration {
                            kind,
                            declarators,
                        }.at(0, 0));

                        return self.for_in_statement_from_parts(left, right);
                    }
                }

                Some(self.alloc(Statement::Declaration {
                    kind,
                    declarators,
                }.at(0, 0)))
            }

            _ => {
                let expression = self.expression(0);

                if let Expression::Binary {
                    operator: In,
                    left,
                    right,
                    ..
                } = expression.item {
                    let left = self.alloc(Statement::Expression {
                        expression: left
                    }.at(0, 0));

                    return self.for_in_statement_from_parts(left, right);
                }

                Some(self.alloc(Statement::Expression {
                    expression
                }.at(0, 0)))
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
                        unexpected_token!(self);
                    }
                    self.lexer.consume();
                    return self.for_of_statement(init);
                },
                Semicolon => self.lexer.consume(),
                _         => unexpected_token!(self),
            }
        }

        let test = match self.lexer.token {
            Semicolon => {
                self.lexer.consume();
                None
            },
            _         => {
                let test = self.expression(0);
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
                let update = self.expression(0);
                expect!(self, ParenClose);

                Some(update)
            }
        };

        let body = self.statement();

        self.alloc(Statement::For {
            init,
            test,
            update,
            body,
        }.at(0, 0))
    }

    fn for_in_statement_from_parts(&mut self, left: StatementPtr<'ast>, right: ExpressionPtr<'ast>) -> StatementPtr<'ast> {
        expect!(self, ParenClose);

        let body = self.statement();

        self.alloc(Statement::ForIn {
            left,
            right,
            body,
        }.at(0, 0))
    }

    fn for_in_statement(&mut self, left: StatementPtr<'ast>) -> StatementPtr<'ast> {
        let right = self.expression(0);

        expect!(self, ParenClose);

        let body = self.statement();

        self.alloc(Statement::ForIn {
            left,
            right,
            body,
        }.at(0, 0))
    }

    fn for_of_statement(&mut self, left: StatementPtr<'ast>) -> StatementPtr<'ast> {
        let right = self.expression(0);

        expect!(self, ParenClose);

        let body = self.statement();

        self.alloc(Statement::ForOf {
            left,
            right,
            body,
        }.at(0, 0))
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

        self.alloc(Statement::SwitchCase {
            test: expr,
            consequent: consequent.into_list()
        }.at(0, 0))
    }

    fn switch_statement(&mut self) -> StatementPtr<'ast> {
        expect!(self, ParenOpen);

        let discriminant = self.expression(0);

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
                    let expr = self.expression(0);
                    cases.push(self.case_statement(Some(expr)));
                },
                Default => {
                    self.lexer.consume();
                    cases.push(self.case_statement(None));
                }
                _ => unexpected_token!(self)
            }
        }

        self.alloc(Statement::Switch {
            discriminant,
            cases: cases.into_list()
        }.at(0, 0))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use parser::parse;
    use parser::mock::Mock;
    use ast::{List, Value, ObjectMember, Function, Class, OperatorKind};

    #[test]
    fn block_statement() {
        let src = "{ true }";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Block {
                body: mock.list([
                    Statement::Expression {
                        expression: mock.ptr(Expression::Value(Value::True))
                    }
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
            Statement::Labeled {
                label: "foobar",
                body: mock.ptr(Statement::Block {
                    body: mock.list([
                        Statement::Expression {
                            expression: mock.ptr(Expression::Value(Value::True))
                        }
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
            Statement::If {
                test: mock.ptr(Expression::Value(Value::True)),
                consequent: mock.ptr(Statement::Expression {
                    expression: mock.ident("foo")
                }),
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
            Statement::If {
                test: mock.ptr(Expression::Value(Value::True)),
                consequent: mock.ptr(Statement::Expression {
                    expression: mock.ident("foo")
                }),
                alternate: Some(mock.ptr(Statement::Block {
                    body: mock.list([
                        Statement::Expression {
                            expression: mock.ident("bar")
                        }
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
            Statement::While {
                test: mock.ptr(Expression::Value(Value::True)),
                body: mock.ptr(Statement::Expression {
                    expression: mock.ident("foo")
                })
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
            Statement::While {
                test: mock.ptr(Expression::Value(Value::True)),
                body: mock.ptr(Statement::Block {
                    body: mock.list([
                        Statement::Expression {
                            expression: mock.ident("foo")
                        }
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
            Statement::Do {
                body: mock.ptr(Statement::Expression {
                    expression: mock.ident("foo")
                }),
                test: mock.ptr(Expression::Value(Value::True))
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
            Statement::Break {
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
            Statement::Break {
                label: Some(mock.ident("foo")),
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
            Statement::Throw {
                value: mock.ptr(Expression::Value(Value::String("'3'"))),
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
            Statement::Try {
                body: List::empty(),
                error: mock.ident("err"),
                handler: List::empty()
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
            Statement::Try {
                body: mock.list([
                    Statement::Expression {
                        expression: mock.ident("foo")
                    }
                ]),
                error: mock.ident("err"),
                handler: mock.list([
                    Statement::Expression {
                        expression: mock.ident("bar")
                    }
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
            Statement::Declaration {
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
            Statement::Declaration {
                kind: DeclarationKind::Let,
                declarators: mock.list([
                    Declarator {
                        name: DeclaratorId::Pattern(mock.ptr(Expression::Array {
                            body: mock.list([
                                Expression::Identifier("x"),
                                Expression::Identifier("y"),
                            ])
                        })),
                        value: Some(mock.ptr(Expression::Array {
                            body: mock.list([
                                Expression::Value(Value::Number("1")),
                                Expression::Value(Value::Number("2")),
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
            Statement::Declaration {
                kind: DeclarationKind::Const,
                declarators: mock.list([
                    Declarator {
                        name: DeclaratorId::Pattern(mock.ptr(Expression::Object {
                            body: mock.list([
                                ObjectMember::Shorthand("x"),
                                ObjectMember::Shorthand("y"),
                            ])
                        })),
                        value: Some(mock.ptr(Expression::Object {
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
            Statement::For {
                init: Some(mock.ptr(Statement::Declaration {
                    kind: DeclarationKind::Let,
                    declarators: mock.list([
                        Declarator {
                            name: DeclaratorId::Identifier("i"),
                            value: Some(mock.number("0")),
                        }
                    ]),
                })),
                test: Some(mock.ptr(Expression::Binary {
                    operator: OperatorKind::Lesser,
                    left: mock.ident("i"),
                    right: mock.number("10"),
                })),
                update: Some(mock.ptr(Expression::Postfix {
                    operator: OperatorKind::Increment,
                    operand: mock.ident("i")
                })),
                body: mock.ptr(Statement::Block {
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
            Statement::For {
                init: None,
                test: None,
                update: None,
                body: mock.ptr(Statement::Block {
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
            Statement::For {
                init: Some(mock.ptr(Statement::Declaration {
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
                test: Some(mock.ptr(Expression::Binary {
                    operator: OperatorKind::Lesser,
                    left: mock.ident("i"),
                    right: mock.number("10"),
                })),
                update: Some(mock.ptr(Expression::Sequence {
                    body: mock.list([
                        Expression::Postfix {
                            operator: OperatorKind::Increment,
                            operand: mock.ident("i")
                        },
                        Expression::Postfix {
                            operator: OperatorKind::Decrement,
                            operand: mock.ident("j")
                        }
                    ])
                })),
                body: mock.ptr(Statement::Block {
                    body: mock.list([
                        Statement::Continue {
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
            Statement::For {
                init: Some(mock.ptr(Statement::Declaration {
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
                test: Some(mock.ptr(Expression::Binary {
                    operator: OperatorKind::Lesser,
                    left: mock.ident("i"),
                    right: mock.number("10"),
                })),
                update: Some(mock.ptr(Expression::Sequence {
                    body: mock.list([
                        Expression::Postfix {
                            operator: OperatorKind::Increment,
                            operand: mock.ident("i")
                        },
                        Expression::Postfix {
                            operator: OperatorKind::Decrement,
                            operand: mock.ident("j")
                        }
                    ])
                })),
                body: mock.ptr(Statement::Block {
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
            Statement::Function {
                function: Function {
                    name: mock.ptr("foo").into(),
                    params: List::empty(),
                    body: List::empty(),
                }
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
            Statement::Class {
                class: Class {
                    name: mock.ptr("Foo").into(),
                    extends: None,
                    body: List::empty(),
                }
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
            Statement::Switch {
                discriminant: mock.ident("times"),
                cases: mock.list([
                    Statement::SwitchCase {
                        test: Some(mock.number("3")),
                        consequent: mock.list([
                            Statement::Break { label: None }
                        ])
                    },
                    Statement::SwitchCase {
                        test: Some(mock.number("2")),
                        consequent: mock.list([
                            Statement::Return { value: Some(mock.ident("b")) }
                        ])
                    },
                    Statement::SwitchCase {
                        test: Some(mock.ptr(Expression::Value(Value::String("\"1\"")))),
                        consequent: mock.list([])
                    },
                    Statement::SwitchCase {
                        test: None,
                        consequent: mock.list([
                            Statement::Return { value: Some(mock.ptr(Expression::Value(Value::False))) }
                        ])
                    },
                ])
            }
        ]);

        let module = parse(src).unwrap();
        assert_eq!(module.body(), expected);
    }
}
