use parser::{Parser, B0, B1, B15};
use lexer::Token::*;
use ast::{Ptr, Loc, List, ListBuilder, Expression, ExpressionPtr, ExpressionList, StatementPtr};
use ast::{ObjectMember, Property, OperatorKind, Value, Parameter, ParameterKey, ParameterList};


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
    const ____ = |par| unexpected_token!(par);

    const OBJ = |par| par.object_expression();

    const CLAS = |par| par.class_expression();

    const FUNC = |par| par.function_expression();

    const IDEN = |par| {
        let value = par.lexer.token_as_str();
        let expr = par.alloc_in_loc(Expression::Identifier(value));

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
        let expr = par.alloc_in_loc(Expression::Value(Value::True));
        par.lexer.consume();

        expr
    };

    pub const FALS = |par| {
        let expr = par.alloc_in_loc(Expression::Value(Value::False));

        par.lexer.consume();
        expr
    };

    pub const NULL = |par| {
        let expr = par.alloc_in_loc(Expression::Value(Value::Null));

        par.lexer.consume();
        expr
    };

    pub const UNDE = |par| {
        let expr = par.alloc_in_loc(Expression::Value(Value::Undefined));

        par.lexer.consume();
        expr
    };

    pub const STR = |par| {
        let value = par.lexer.token_as_str();
        let expr = par.alloc_in_loc(Expression::Value(Value::String(value)));

        par.lexer.consume();
        expr
    };

    pub const NUM = |par| {
        let value = par.lexer.token_as_str();
        let expr = par.alloc_in_loc(Expression::Value(Value::Number(value)));

        par.lexer.consume();
        expr
    };

    pub const BIN = |par| {
        let value = par.lexer.token_as_str();
        let expr = par.alloc_in_loc(Expression::Value(Value::Binary(value)));

        par.lexer.consume();
        expr
    };

    pub const TPLS = |par| {
        let quasi = par.lexer.quasi;
        let expr = par.alloc_in_loc(Expression::Value(Value::Template(quasi)));

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
            BraceOpen => {
                self.lexer.consume();
                self.block_statement()
            },
            _ => {
                let expression = self.expression(B1);
                self.wrap_expression(expression)
            }
        };

        self.alloc(Expression::Arrow {
            params,
            body,
        }.at(0, 0))
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
                _          => unexpected_token!(self),
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

        self.alloc(Expression::Prefix {
            operator: operator,
            operand: operand,
        }.at(0, 0))
    }

    #[inline]
    pub fn object_expression(&mut self) -> ExpressionPtr<'ast> {
        let start = self.lexer.start();
        self.lexer.consume();

        if self.lexer.token == BraceClose {
            self.lexer.consume();
            return self.alloc_in_loc(Expression::Object {
                body: List::empty()
            });
        }

        let member = self.object_member();

        let mut builder = ListBuilder::new(self.arena, member);

        loop {
            match self.lexer.token {
                BraceClose => {
                    self.lexer.consume();
                    break;
                },
                Comma      => {
                    self.lexer.consume();
                },
                _          => unexpected_token!(self)
            }

            match self.lexer.token {
                BraceClose => {
                    self.lexer.consume();
                    break;
                },
                _ => builder.push(self.object_member()),
            }
        }

        self.alloc(Expression::Object {
            body: builder.into_list()
        }.at(start, 0))
    }

    #[inline]
    pub fn object_member(&mut self) -> Ptr<'ast, Loc<ObjectMember<'ast>>> {
        let property = match self.lexer.token {
            Identifier => {
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
            _ => {
                // Allow word tokens such as "null" and "typeof" as identifiers
                match self.lexer.token.as_word() {
                    Some(label) => {
                        self.lexer.consume();
                        self.in_loc(Property::Literal(label))
                    }
                    None        => unexpected_token!(self)
                }
            }
        };

        let property = self.alloc(property);

        match self.lexer.token {
            Colon => {
                self.lexer.consume();

                let value = self.expression(B1);

                self.alloc(Loc::new(0, 0, ObjectMember::Value {
                    property,
                    value,
                }))
            },
            ParenOpen => {
                self.lexer.consume();

                let params = self.parameter_list();
                let body = self.block_body();

                self.alloc(Loc::new(0, 0, ObjectMember::Method {
                    property,
                    params,
                    body,
                }))
            },
            _ => unexpected_token!(self)
        }
    }

    #[inline]
    pub fn array_expression(&mut self) -> ExpressionPtr<'ast> {
        let start = self.lexer.start();
        self.lexer.consume();

        let expression = match self.lexer.token {
            Comma        => {
                self.lexer.consume();
                self.alloc_in_loc(Expression::Void)
            },
            BracketClose => {
                self.lexer.consume();
                return self.alloc(Expression::Array { body: List::empty() }.at(0,0))
            },
            _            => {
                let expression = self.expression(B1);

                match self.lexer.token {
                    BracketClose => {
                        self.lexer.consume();

                        let body = List::from(self.arena, expression);

                        return self.alloc(Expression::Array { body }.at(0, 0));
                    },
                    Comma        => {
                        self.lexer.consume();
                        expression
                    },
                    _            => unexpected_token!(self),
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
                    self.lexer.consume();

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
                    self.lexer.consume();
                    break;
                }
                Comma        => self.lexer.consume(),
                _            => unexpected_token!(self),
            }
        }

        self.alloc(Expression::Array {
            body: builder.into_list()
        }.at(start, 0))
    }

    #[inline]
    pub fn regular_expression(&mut self) -> ExpressionPtr<'ast> {
        let value = self.lexer.read_regular_expression();

        expect!(self, LiteralRegEx);

        self.alloc(Expression::Value(Value::RegEx(value)).at(0, 0))
    }

    #[inline]
    pub fn template_string(&mut self, tag: Option<ExpressionPtr<'ast>>) -> ExpressionPtr<'ast> {
        let quasi = self.lexer.quasi;
        let quasi = self.alloc_in_loc(quasi);

        self.lexer.consume();

        let template = Expression::Template {
            tag,
            expressions: List::empty(),
            quasis: List::from(self.arena, quasi),
        };

        self.alloc_in_loc(template)
    }

    #[inline]
    pub fn template_expression(&mut self, tag: Option<ExpressionPtr<'ast>>) -> ExpressionPtr<'ast> {
        let quasi = self.lexer.quasi;
        let quasi = self.alloc_in_loc(quasi);

        self.lexer.consume();

        let expression = self.expression(B0);

        match self.lexer.token {
            BraceClose => self.lexer.read_template_kind(),
            _          => unexpected_token!(self)
        }

        let mut quasis = ListBuilder::new(self.arena, quasi);
        let mut expressions = ListBuilder::new(self.arena, expression);

        loop {
            match self.lexer.token {
                TemplateOpen => {
                    let quasi = self.lexer.quasi;
                    self.lexer.consume();
                    quasis.push(self.alloc_in_loc(quasi));
                    expressions.push(self.expression(B0));

                    match self.lexer.token {
                        BraceClose => self.lexer.read_template_kind(),
                        _          => unexpected_token!(self)
                    }
                },
                TemplateClosed => {
                    let quasi = self.lexer.quasi;
                    self.lexer.consume();
                    quasis.push(self.alloc_in_loc(quasi));
                    break;
                },
                _ => unexpected_token!(self)
            }
        }

        self.alloc(Expression::Template {
            tag,
            expressions: expressions.into_list(),
            quasis: quasis.into_list(),
        }.at(0, 0))
    }

    #[inline]
    pub fn function_expression(&mut self) -> ExpressionPtr<'ast> {
        let start = self.lexer.start();
        self.lexer.consume();

        let function = self.function();

        self.alloc(Expression::Function { function }.at(start, 0))
    }

    #[inline]
    pub fn class_expression(&mut self) -> ExpressionPtr<'ast> {
        let start = self.lexer.start();
        self.lexer.consume();

        let class = self.class();

        self.alloc(Expression::Class { class }.at(start, 0))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ast::{OperatorKind, Value, Statement, Function, Class};
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

        let expected_a = Expression::Value(Value::String(r#""foobar""#));
        let expected_b = Expression::Value(Value::Number("100"));
        let expected_c = Expression::Value(Value::True);

        assert_expr!(module_a, expected_a);
        assert_expr!(module_b, expected_b);
        assert_expr!(module_c, expected_c);
    }

    #[test]
    fn template_expression() {
        let src = "`foobar`;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Value(Value::Template("foobar"));

        assert_expr!(module, expected);
    }

    #[test]
    fn tagged_template_expression() {
        let src = "foo`bar`;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Template {
            tag: Some(mock.ident("foo")),
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

        let expected = Expression::Template {
            tag: None,
            expressions: mock.list([
                Expression::Value(Value::Number("10")),
                Expression::Value(Value::Number("20")),
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

        let expected = Expression::Template {
            tag: Some(mock.ident("foo")),
            expressions: mock.list([
                Expression::Value(Value::Number("42")),
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

        let expected = Expression::Sequence {
            body: mock.list([
                Expression::Identifier("foo"),
                Expression::Identifier("bar"),
                Expression::Identifier("baz"),
            ])
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn binary_expression() {
        let src = "foo + bar;";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Binary {
            operator: OperatorKind::Addition,
            left: mock.ident("foo"),
            right: mock.ident("bar"),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn parenthesized_binary_expression() {
        let src = "(2 + 2);";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Binary {
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

        let expected = Expression::Conditional {
            test: mock.ptr(Expression::Value(Value::True)),
            consequent: mock.ident("foo"),
            alternate: mock.ident("bar"),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn postfix_expression() {
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
    fn computed_member_expression() {
        let src = "foo[10]";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::ComputedMember {
            object: mock.ident("foo"),
            property: mock.number("10"),
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
    fn sparse_array_expression() {
        let src = "[,,foo,bar,,]";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Array {
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

        let expected = Expression::Function {
            function: Function {
                name: None.into(),
                params: List::empty(),
                body: List::empty()
            }
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn named_function_expression() {
        let src = "(function foo () {})";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Function {
            function: Function {
                name: mock.ptr("foo").into(),
                params: List::empty(),
                body: List::empty()
            }
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn arrow_function_expression() {
        let src = "() => bar";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Arrow {
            params: ParameterList::empty(),
            body: mock.ptr(Statement::Expression {
                expression: mock.ident("bar")
            })
        };
        assert_expr!(module, expected);
    }

    #[test]
    fn arrow_function_shorthand() {
        let src = "n => n* n";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Arrow {
            params: mock.list([
                Parameter {
                    key: ParameterKey::Identifier("n"),
                    value: None,
                },
            ]),

            body: mock.ptr(Statement::Expression {
                expression: mock.ptr(Expression::Binary {
                    operator: OperatorKind::Multiplication,
                    left: mock.ident("n"),
                    right: mock.ident("n"),
                })
            })

        };
        assert_expr!(module, expected);
    }

    #[test]
    fn arrow_function_with_params() {
        let src = "(a, b, c) => bar";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Arrow {
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

            body: mock.ptr(Statement::Expression {
                expression: mock.ident("bar")
            })

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

        let expected = Expression::Arrow {
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
                    value: Some(mock.ptr(Expression::Value(Value::Number("2"))))
                }
            ]),
            body: mock.ptr(Statement::Expression {
                expression: mock.ident("bar")
            })
        };
        assert_expr!(module, expected);
    }

    #[test]
    fn class_expression() {
        let src = "(class {})";
        let module = parse(src).unwrap();

        let expected = Expression::Class {
            class: Class {
                name: None.into(),
                extends: None,
                body: List::empty()
            }
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn named_class_expression() {
        let src = "(class Foo {})";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Class {
            class: Class {
                name: mock.ptr("Foo").into(),
                extends: None,
                body: List::empty()
            }
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn named_child_class_expression() {
        let src = "(class Foo extends Bar {})";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Class {
            class: Class {
                name: mock.ptr("Foo").into(),
                extends: Some(mock.ptr(Expression::Identifier("Bar"))),
                body: List::empty()
            }
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn regression_operator_precedence() {
        let src = "true === true && false === false";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Binary {
            operator: OperatorKind::LogicalAnd,
            left: mock.ptr(Expression::Binary {
                operator: OperatorKind::StrictEquality,
                left: mock.ptr(Expression::Value(Value::True)),
                right: mock.ptr(Expression::Value(Value::True)),
            }),
            right: mock.ptr(Expression::Binary {
                operator: OperatorKind::StrictEquality,
                left: mock.ptr(Expression::Value(Value::False)),
                right: mock.ptr(Expression::Value(Value::False)),
            }),
        };

        assert_expr!(module, expected);
    }

    #[test]
    fn arrow_function_in_sequence() {
        let src = "(() => {}, foo)";
        let module = parse(src).unwrap();
        let mock = Mock::new();

        let expected = Expression::Sequence {
            body: mock.list([
                Expression::Arrow {
                    params: List::empty(),
                    body: mock.ptr(Statement::Block {
                        body: List::empty()
                    })
                },
                Expression::Identifier("foo"),
            ])
        };

        assert_expr!(module, expected);
    }
}
