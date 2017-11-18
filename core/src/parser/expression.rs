use parser::{Parser, Parse, B0, B1, B15};
use lexer::Token::*;
use ast::{Ptr, Loc, List, ListBuilder, Expression, ExpressionPtr, ExpressionList, StatementPtr, Function};
use ast::{ObjectMember, Property, OperatorKind, Literal, Parameter, ParameterKey, ParameterList, Class};
use ast::expression::{PrefixExpression, ArrowExpression, ArrowBody, ArrayExpression, ObjectExpression, TemplateExpression};


type ExpressionHandler = for<'ast> fn(&mut Parser<'ast>) -> ExpressionPtr<'ast>;

static EXPR_HANDLERS: [ExpressionHandler; 108] = [
    ____, ____, ____, ____, PRN,  ____, ARR,  ____, OBJ,  ____, ____, OP,
//  EOF   ;     :     ,     (     )     [     ]     {     }     =>    NEW

    OP,   OP,   OP,   OP,   OP,   OP,   OP,   ____, REG,  ____, ____, OP,
//  ++    --    !     ~     TYPOF VOID  DELET *     /     %     **    +

    OP,   ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//  -     <<    >>    >>>   <     <=    >     >=    INSOF IN    ===   !==

    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//  ==    !=    &     ^     |     &&    ||    ?     =     +=    -=    **=

    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//  *=    /=    %=    <<=   >>=   >>>=  &=    ^=    |=    ...   VAR   LET

    ____, ____, ____, ____, ____, ____, ____, CLAS, ____, ____, ____, ____,
//  CONST BREAK DO    CASE  ELSE  CATCH EXPRT CLASS EXTND RET   WHILE FINLY

    ____, ____, ____, ____, ____, ____, ____, FUNC, THIS, ____, ____, ____,
//  SUPER WITH  CONT  FOR   SWTCH YIELD DBGGR FUNCT THIS  DEFLT IF    THROW

    ____, ____, ____, TRUE, FALS, NULL, UNDE, STR,  NUM,  BIN,  ____, ____,
//  IMPRT TRY   STATI TRUE  FALSE NULL  UNDEF STR   NUM   BIN   REGEX ENUM

    ____, ____, ____, ____, ____, ____, IDEN, ____, TPLE, TPLS, ____, ____,
//  IMPL  PCKG  PROT  IFACE PRIV  PUBLI IDENT ACCSS TPL_O TPL_C ERR_T ERR_E
];

macro_rules! create_handlers {
    ($( const $name:ident = |$par:ident| $code:expr; )* $( pub const $pname:ident = |$ppar:ident| $pcode:expr; )*) => {
        $(
            #[allow(non_snake_case)]
            fn $name<'ast>($par: &mut Parser<'ast>) -> ExpressionPtr<'ast> {
                $code
            }
        )*

        pub(crate) mod handlers {
            use super::*;

            $(
                #[allow(non_snake_case)]
                pub fn $pname<'ast>($ppar: &mut Parser<'ast>) -> StatementPtr<'ast> {
                    let expression = $pcode;
                    $ppar.expression_statement(expression)
                }
            )*
        }

        $(
            #[allow(non_snake_case)]
            fn $pname<'ast>($ppar: &mut Parser<'ast>) -> ExpressionPtr<'ast> {
                $pcode
            }
        )*
    };
}

create_handlers! {
    const ____ = |par| return par.error();

    const OBJ = |par| par.object_expression();

    const CLAS = |par| par.class_expression();

    const FUNC = |par| par.function_expression();

    const IDEN = |par| {
        let ident = par.lexer.token_as_str();
        let expr = par.alloc_in_loc(ident);

        par.lexer.consume();
        expr
    };

    pub const THIS = |par| {
        let expr = par.alloc_in_loc(Expression::This);
        par.lexer.consume();

        expr
    };

    pub const OP = |par| {
        let op = OperatorKind::from_token(par.lexer.token).expect("Must be a prefix operator");
        par.lexer.consume();
        par.prefix_expression(op)
    };

    pub const PRN = |par| {
        par.lexer.consume();
        par.paren_expression()
    };

    pub const ARR = |par| par.array_expression();

    pub const REG = |par| par.regular_expression();

    pub const TRUE = |par| {
        let expr = par.alloc_in_loc(Literal::True);
        par.lexer.consume();

        expr
    };

    pub const FALS = |par| {
        let expr = par.alloc_in_loc(Literal::False);

        par.lexer.consume();
        expr
    };

    pub const NULL = |par| {
        let expr = par.alloc_in_loc(Literal::Null);

        par.lexer.consume();
        expr
    };

    pub const UNDE = |par| {
        let expr = par.alloc_in_loc(Literal::Undefined);

        par.lexer.consume();
        expr
    };

    pub const STR = |par| {
        let value = par.lexer.token_as_str();
        let expr = par.alloc_in_loc(Literal::String(value));

        par.lexer.consume();
        expr
    };

    pub const NUM = |par| {
        let value = par.lexer.token_as_str();
        let expr = par.alloc_in_loc(Literal::Number(value));

        par.lexer.consume();
        expr
    };

    pub const BIN = |par| {
        let value = par.lexer.token_as_str();
        let expr = par.alloc_in_loc(Literal::Binary(value));

        par.lexer.consume();
        expr
    };

    pub const TPLS = |par| {
        let quasi = par.lexer.quasi;
        let expr = par.alloc_in_loc(Literal::Template(quasi));

        par.lexer.consume();
        expr
    };

    pub const TPLE = |par| par.template_expression(None);
}

impl<'ast> Parser<'ast> {
    #[inline]
    pub fn bound_expression(&mut self) -> ExpressionPtr<'ast> {
        unsafe { (*(&EXPR_HANDLERS as *const ExpressionHandler).offset(self.lexer.token as isize))(self) }
    }

    #[inline]
    pub fn arrow_function_expression(&mut self, params: ExpressionList<'ast>) -> ExpressionPtr<'ast> {
        let params = self.params_from_expressions(params);

        let body = match self.lexer.token {
            BraceOpen => ArrowBody::Block(self.unchecked_block()),
            _         => ArrowBody::Expression(self.expression(B1)),
        };

        self.alloc_at_loc(0, 0, ArrowExpression {
            params,
            body,
        })
    }

    #[inline]
    pub fn expression_list(&mut self) -> ExpressionList<'ast> {
        if self.lexer.token == ParenClose {
            self.lexer.consume();
            return List::empty();
        }

        let expression = self.expression(B1);
        let mut builder = ListBuilder::new(self.arena, expression);

        loop {
            let expression = match self.lexer.token {
                ParenClose => {
                    self.lexer.consume();
                    break;
                },
                Comma      => {
                    self.lexer.consume();
                    self.expression(B1)
                }
                _ => return self.error(),
            };

            builder.push(expression);
        }

        builder.into_list()
    }

    #[inline]
    pub fn paren_expression(&mut self) -> ExpressionPtr<'ast> {
        match self.lexer.token {
            ParenClose => {
                self.lexer.consume();
                expect!(self, OperatorFatArrow);
                self.arrow_function_expression(List::empty())
            },
            _ => {
                let expression = self.expression(B0);

                expect!(self, ParenClose);

                expression
            }
        }
    }

    #[inline]
    pub fn prefix_expression(&mut self, operator: OperatorKind) -> ExpressionPtr<'ast> {
        let operand = self.expression(B15);

        self.alloc_at_loc(0, 0, PrefixExpression {
            operator: operator,
            operand: operand,
        })
    }

    #[inline]
    pub fn object_expression(&mut self) -> ExpressionPtr<'ast> {
        let start = self.lexer.start();
        let end;
        self.lexer.consume();

        if self.lexer.token == BraceClose {
            end = self.lexer.end_then_consume();
            return self.alloc_at_loc(start, end, Expression::Object(ObjectExpression {
                body: List::empty()
            }));
        }

        let member = self.object_member();

        let mut builder = ListBuilder::new(self.arena, member);

        loop {
            match self.lexer.token {
                BraceClose => {
                    end = self.lexer.end_then_consume();
                    break;
                },
                Comma      => {
                    self.lexer.consume();
                },
                _ => return self.error()
            }

            match self.lexer.token {
                BraceClose => {
                    end = self.lexer.end_then_consume();
                    break;
                },
                _ => builder.push(self.object_member()),
            }
        }

        self.alloc_at_loc(start, end, ObjectExpression {
            body: builder.into_list()
        })
    }

    #[inline]
    pub fn object_member(&mut self) -> Ptr<'ast, ObjectMember<'ast>> {
        let property = match self.lexer.token {
            _ if self.lexer.token.is_word() => {
                let label = self.lexer.token_as_str();
                self.lexer.consume();

                match self.lexer.token {
                    Colon | ParenOpen => self.in_loc(Property::Literal(label)),

                    _ => return self.alloc_in_loc(ObjectMember::Shorthand(label)),
                }
            },
            LiteralString |
            LiteralNumber => {
                let key = self.lexer.token_as_str();
                self.lexer.consume();
                self.in_loc(Property::Literal(key))
            },
            LiteralBinary => {
                let num = self.lexer.token_as_str();
                self.lexer.consume();
                self.in_loc(Property::Binary(num))
            },
            BracketOpen => {
                self.lexer.consume();

                let expression = self.expression(B0);
                let property = Loc::new(0, 0, Property::Computed(expression));

                expect!(self, BracketClose);

                property
            },
            _ => return self.error(),
        };

        let property = self.alloc(property);

        match self.lexer.token {
            Colon => {
                self.lexer.consume();

                let value = self.expression(B1);

                self.alloc_at_loc(0, 0,  ObjectMember::Literal {
                    property,
                    value,
                })
            },
            ParenOpen => {
                self.lexer.consume();

                let params = self.parameter_list();
                let body = self.block();

                self.alloc_at_loc(0, 0, ObjectMember::Method {
                    property,
                    params,
                    body,
                })
            },
            _ => return self.error()
        }
    }

    #[inline]
    pub fn array_expression(&mut self) -> ExpressionPtr<'ast> {
        let start = self.lexer.start();
        let end;
        self.lexer.consume();

        let expression = match self.lexer.token {
            Comma => {
                self.lexer.consume();
                self.alloc_in_loc(Expression::Void)
            },
            BracketClose => {
                end = self.lexer.end_then_consume();
                return self.alloc_at_loc(start, end, ArrayExpression { body: List::empty() });
            },
            _ => {
                let expression = self.expression(B1);

                match self.lexer.token {
                    BracketClose => {
                        end = self.lexer.end_then_consume();

                        let body = List::from(self.arena, expression);

                        return self.alloc_at_loc(start, end, ArrayExpression { body });
                    },
                    Comma => {
                        self.lexer.consume();
                        expression
                    },
                    _ => return self.error(),
                }
            }
        };

        let mut builder = ListBuilder::new(self.arena, expression);

        loop {
            match self.lexer.token {
                Comma => {
                    self.lexer.consume();

                    builder.push(self.alloc_in_loc(Expression::Void));

                    continue;
                },
                BracketClose => {
                    end = self.lexer.end_then_consume();

                    builder.push(self.alloc_in_loc(Expression::Void));

                    break;
                },
                _ => {
                    let expression = self.expression(B1);

                    builder.push(expression);
                }
            }

            match self.lexer.token {
                BracketClose => {
                    end = self.lexer.end_then_consume();
                    break;
                }
                Comma => self.lexer.consume(),
                _     => return self.error(),
            }
        }

        self.alloc_at_loc(start, end, ArrayExpression {
            body: builder.into_list()
        })
    }

    #[inline]
    pub fn regular_expression(&mut self) -> ExpressionPtr<'ast> {
        let value = self.lexer.read_regular_expression();

        expect!(self, LiteralRegEx);

        self.alloc_at_loc(0, 0, Literal::RegEx(value))
    }

    #[inline]
    pub fn template_string(&mut self, tag: ExpressionPtr<'ast>) -> ExpressionPtr<'ast> {
        let start = tag.start;
        let quasi = self.lexer.quasi;
        let quasi = self.alloc_in_loc(quasi);
        let end = self.lexer.end_then_consume();
        let quasis = List::from(self.arena, quasi);

        self.alloc_at_loc(start, end, TemplateExpression {
            tag: Some(tag),
            expressions: List::empty(),
            quasis,
        })
    }

    #[inline]
    pub fn template_expression(&mut self, tag: Option<ExpressionPtr<'ast>>) -> ExpressionPtr<'ast> {
        let quasi = self.lexer.quasi;
        let quasi = self.alloc_in_loc(quasi);
        let start = match tag {
            Some(ref expr) => expr.start,
            _              => self.lexer.start()
        };
        let end;

        self.lexer.consume();

        let expression = self.expression(B0);

        match self.lexer.token {
            BraceClose => self.lexer.read_template_kind(),
            _          => return self.error()
        }

        let mut quasis = ListBuilder::new(self.arena, quasi);
        let mut expressions = ListBuilder::new(self.arena, expression);

        loop {
            match self.lexer.token {
                TemplateOpen => {
                    let quasi = self.lexer.quasi;
                    quasis.push(self.alloc_in_loc(quasi));
                    self.lexer.consume();
                    expressions.push(self.expression(B0));

                    match self.lexer.token {
                        BraceClose => self.lexer.read_template_kind(),
                        _          => return self.error()
                    }
                },
                TemplateClosed => {
                    let quasi = self.lexer.quasi;
                    quasis.push(self.alloc_in_loc(quasi));
                    end = self.lexer.end_then_consume();
                    break;
                },
                _ => return self.error()
            }
        }

        self.alloc_at_loc(start, end, TemplateExpression {
            tag,
            expressions: expressions.into_list(),
            quasis: quasis.into_list(),
        })
    }

    #[inline]
    pub fn function_expression(&mut self) -> ExpressionPtr<'ast> {
        let start = self.lexer.start();
        self.lexer.consume();

        let function = Function::parse(self);

        self.alloc_at_loc(start, function.body.end, function)
    }

    #[inline]
    pub fn class_expression(&mut self) -> ExpressionPtr<'ast> {
        let start = self.lexer.start();
        self.lexer.consume();

        let class = Class::parse(self);

        self.alloc_at_loc(start, class.body.end, class)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ast::{OperatorKind, Literal, Statement, Function, Class};
    use ast::expression::*;
    use ast::statement::*;
    use parser::parse;
    use parser::mock::Mock;

    #[test]
    fn ident_expression() {
        let module = parse("foobar;").unwrap();

        let expected = Expression::Identifier("foobar");

        assert_expr!(module, expected);
    }

    #[test]
    fn value_expression() {
        let module_a = parse(r#""foobar";"#).unwrap();
        let module_b = parse("100;").unwrap();
        let module_c = parse("true;").unwrap();

        let expected_a = Literal::String(r#""foobar""#);
        let expected_b = Literal::Number("100");
        let expected_c = Literal::True;

        assert_expr!(module_a, expected_a);
        assert_expr!(module_b, expected_b);
        assert_expr!(module_c, expected_c);
    }

    #[test]
    fn template_expression() {
        let src = "`foobar`;";
        let module = parse(src).unwrap();

        let expected = Literal::Template("foobar");

        assert_expr!(module, expected);
    }

    #[test]
    fn tagged_template_expression() {
        let src = "foo`bar`;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = TemplateExpression {
            tag: Some(mock.ptr("foo")),
            expressions: List::empty(),
            quasis: mock.list(["bar"]),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn complex_template_expression() {
        let src = "`foo${ 10 }bar${ 20 }baz`;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = TemplateExpression {
            tag: None,
            expressions: mock.list([
                Literal::Number("10"),
                Literal::Number("20"),
            ]),
            quasis: mock.list(["foo", "bar", "baz" ]),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn tagged_complex_template_expression() {
        let src = "foo`bar${ 42 }baz`;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = TemplateExpression {
            tag: Some(mock.ptr("foo")),
            expressions: mock.list([
                Literal::Number("42"),
            ]),
            quasis: mock.list(["bar", "baz"]),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn sequence_expression() {
        let src = "foo, bar, baz;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = SequenceExpression {
            body: mock.list(["foo", "bar", "baz"]),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn binary_expression() {
        let src = "foo + bar;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = BinaryExpression {
            operator: OperatorKind::Addition,
            left: mock.ptr("foo"),
            right: mock.ptr("bar"),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn parenthesized_binary_expression() {
        let src = "(2 + 2);";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = BinaryExpression {
            operator: OperatorKind::Addition,
            left: mock.number("2"),
            right: mock.number("2"),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn conditional_expression() {
        let src = "true ? foo : bar";

        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = ConditionalExpression {
            test: mock.ptr(Expression::Literal(Literal::True)),
            consequent: mock.ptr("foo"),
            alternate: mock.ptr("bar"),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn postfix_expression() {
        let src = "baz++;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = PostfixExpression {
            operator: OperatorKind::Increment,
            operand: mock.ptr("baz"),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn call_expression() {
        let src = "foo();";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = CallExpression {
            callee: mock.ptr("foo"),
            arguments: List::empty(),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn member_expression() {
        let src = "foo.bar";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = MemberExpression {
            object: mock.ptr("foo"),
            property: mock.ptr("bar"),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn keyword_member_expression() {
        let src = "foo.function";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = MemberExpression {
            object: mock.ptr("foo"),
            property: mock.ptr("function"),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn computed_member_expression() {
        let src = "foo[10]";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = ComputedMemberExpression {
            object: mock.ptr("foo"),
            property: mock.number("10"),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn regular_expression() {
        let src = r#"/^[A-Z]+\/[\d]+/g"#;
        let module = parse(src).unwrap();

        let expected = Literal::RegEx("/^[A-Z]+\\/[\\d]+/g");

        assert_expr!(module, expected);
    }

    #[test]
    fn array_expression() {
        let src = "[0, 1, 2]";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = ArrayExpression {
            body: mock.list([
                Literal::Number("0"),
                Literal::Number("1"),
                Literal::Number("2"),
            ])
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn sparse_array_expression() {
        let src = "[,,foo,bar,,]";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = ArrayExpression {
            body: mock.list([
                Expression::Void,
                Expression::Void,
                Expression::Identifier("foo"),
                Expression::Identifier("bar"),
                Expression::Void,
                Expression::Void,
            ])
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn function_expression() {
        let src = "(function () {})";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Function {
            name: None.into(),
            params: List::empty(),
            body: mock.empty_block()
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn named_function_expression() {
        let src = "(function foo () {})";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Function {
            name: mock.name("foo"),
            params: List::empty(),
            body: mock.empty_block()
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn arrow_function_expression() {
        let src = "() => bar";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = ArrowExpression {
            params: ParameterList::empty(),
            body: ArrowBody::Expression(mock.ptr("bar")),
        };
        assert_expr!(module, expected);
    }

    #[test]
    fn arrow_function_shorthand() {
        let src = "n => n* n";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = ArrowExpression {
            params: mock.list([
                Parameter {
                    key: ParameterKey::Identifier("n"),
                    value: None,
                },
            ]),

            body: ArrowBody::Expression(mock.ptr(BinaryExpression {
                operator: OperatorKind::Multiplication,
                left: mock.ptr("n"),
                right: mock.ptr("n"),
            }))

        };
        assert_expr!(module, expected);
    }

    #[test]
    fn arrow_function_with_params() {
        let src = "(a, b, c) => bar";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = ArrowExpression {
            params: mock.list([
                Parameter {
                    key: ParameterKey::Identifier("a"),
                    value: None,
                },
                Parameter {
                    key: ParameterKey::Identifier("b"),
                    value: None,
                },
                Parameter {
                    key: ParameterKey::Identifier("c"),
                    value: None,
                }
            ]),
            body: ArrowBody::Expression(mock.ptr("bar"))
        };
        assert_expr!(module, expected);
    }


    #[test]
    #[should_panic]
    fn arrow_function_invalid_params_throws() {
        parse("(a, b, c * 2) => bar").unwrap();
    }

    #[test]
    fn arrow_function_with_default_params() {
        let src = "(a, b, c = 2) => bar";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = ArrowExpression {
            params: mock.list([
                Parameter {
                    key: ParameterKey::Identifier("a"),
                    value: None,
                },
                Parameter {
                    key: ParameterKey::Identifier("b"),
                    value: None,
                },
                Parameter {
                    key: ParameterKey::Identifier("c"),
                    value: Some(mock.ptr(Literal::Number("2")))
                }
            ]),
            body: ArrowBody::Expression(mock.ptr("bar"))
        };
        assert_expr!(module, expected);
    }

    #[test]
    fn class_expression() {
        let src = "(class {})";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Class {
            name: None.into(),
            extends: None,
            body: mock.empty_block()
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn named_class_expression() {
        let src = "(class Foo {})";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Class {
            name: mock.name("Foo"),
            extends: None,
            body: mock.empty_block()
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn named_child_class_expression() {
        let src = "(class Foo extends Bar {})";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Class {
            name: mock.name("Foo"),
            extends: Some(mock.ptr("Bar")),
            body: mock.empty_block()
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn regression_operator_precedence() {
        let src = "true === true && false === false";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = BinaryExpression {
            operator: OperatorKind::LogicalAnd,
            left: mock.ptr(BinaryExpression {
                operator: OperatorKind::StrictEquality,
                left: mock.ptr(Literal::True),
                right: mock.ptr(Literal::True),
            }),
            right: mock.ptr(BinaryExpression {
                operator: OperatorKind::StrictEquality,
                left: mock.ptr(Literal::False),
                right: mock.ptr(Literal::False),
            }),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn arrow_function_in_sequence() {
        let src = "(() => {}, foo)";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = SequenceExpression {
            body: mock.list([
                Expression::Arrow(ArrowExpression {
                    params: List::empty(),
                    body: ArrowBody::Block(mock.ptr(BlockStatement {
                        body: List::empty()
                    }))
                }),
                Expression::Identifier("foo"),
            ])
        };

        assert_expr!(module, expected);
    }
}
