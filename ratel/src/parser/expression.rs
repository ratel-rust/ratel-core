use toolshed::list::ListBuilder;
use parser::{Parser, Parse, BindingPower, ANY, B0, B15};
use lexer::Token::*;
use ast::{Node, NodeList, Expression, ExpressionNode, IdentifierNode, ExpressionList};
use ast::{Property, PropertyKey, OperatorKind, Literal, Function, Class, StatementNode};
use ast::expression::*;


type ExpressionHandler = for<'ast> fn(&mut Parser<'ast>) -> ExpressionNode<'ast>;

pub type Context = &'static [ExpressionHandler; 108];

static DEF_CONTEXT: Context = &[
    ____, ____, ____, ____, PRN,  ____, ARR,  ____, OBJ,  ____, ____, NEW,
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

// Adds handlers for VoidExpression and SpreadExpression
pub static ARRAY_CONTEXT: Context = &[
    ____, ____, ____, VOID, PRN,  ____, ARR,  VOID, OBJ,  ____, ____, NEW,
    OP,   OP,   OP,   OP,   OP,   OP,   OP,   ____, REG,  ____, ____, OP,
    OP,   ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, SPRD, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, CLAS, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, FUNC, THIS, ____, ____, ____,
    ____, ____, ____, TRUE, FALS, NULL, UNDE, STR,  NUM,  BIN,  ____, ____,
    ____, ____, ____, ____, ____, ____, IDEN, ____, TPLE, TPLS, ____, ____,
];

// Adds handler for SpreadExpression
pub static CALL_CONTEXT: Context = &[
    ____, ____, ____, ____, PRN,  ____, ARR,  ____, OBJ,  ____, ____, NEW,
    OP,   OP,   OP,   OP,   OP,   OP,   OP,   ____, REG,  ____, ____, OP,
    OP,   ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, SPRD, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, CLAS, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, FUNC, THIS, ____, ____, ____,
    ____, ____, ____, TRUE, FALS, NULL, UNDE, STR,  NUM,  BIN,  ____, ____,
    ____, ____, ____, ____, ____, ____, IDEN, ____, TPLE, TPLS, ____, ____,
];

macro_rules! create_handlers {
    ($( const $name:ident = |$par:ident| $code:expr; )* $( pub const $pname:ident = |$ppar:ident| $pcode:expr; )*) => {
        $(
            #[allow(non_snake_case)]
            fn $name<'ast>($par: &mut Parser<'ast>) -> ExpressionNode<'ast> {
                $code
            }
        )*

        pub(crate) mod handlers {
            use super::*;

            $(
                #[allow(non_snake_case)]
                pub fn $pname<'ast>($ppar: &mut Parser<'ast>) -> StatementNode<'ast> {
                    let expression = $pcode;
                    $ppar.expression_statement(expression)
                }
            )*
        }

        $(
            #[allow(non_snake_case)]
            fn $pname<'ast>($ppar: &mut Parser<'ast>) -> ExpressionNode<'ast> {
                $pcode
            }
        )*
    };
}

create_handlers! {
    const ____ = |par| {
        let loc = par.lexer.start();
        par.error::<()>();
        par.alloc_at_loc(loc, loc, Expression::Void)
    };

    const VOID = |par| par.void_expression();

    const OBJ = |par| par.object_expression();

    const CLAS = |par| par.class_expression();

    const FUNC = |par| par.function_expression();

    const IDEN = |par| {
        let ident = par.lexer.token_as_str();
        let expr = par.alloc_in_loc(ident);

        par.lexer.consume();
        expr
    };

    const SPRD = |par| {
        let start = par.lexer.start_then_consume();
        let argument = par.expression::<B0>();

        par.alloc_at_loc(start, argument.end, SpreadExpression { argument })
    };

    pub const THIS = |par| {
        let expr = par.alloc_in_loc(ThisExpression);
        par.lexer.consume();

        expr
    };

    pub const OP = |par| {
        let start = par.lexer.start();
        let op = OperatorKind::from_token(par.lexer.token).expect("Must be a prefix operator");
        par.lexer.consume();
        let expression = par.prefix_expression(op);
        let end = par.lexer.end();
        par.alloc_at_loc(start, end, expression)
    };

    pub const NEW = |par| {
        let (start, op_end) = par.lexer.loc();

        par.lexer.consume();

        if par.lexer.token == Accessor {
            let meta = par.alloc_at_loc(start, op_end, "new");
            let expression = par.meta_property_expression(meta);
            let end = par.lexer.end();
            par.lexer.consume();
            par.alloc_at_loc(start, end, expression)
        } else {
            let expression = par.prefix_expression(OperatorKind::New);
            let end = par.lexer.end();
            par.alloc_at_loc(start, end, expression)
        }
    };

    pub const PRN = |par| {
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
        let quasi = par.alloc_in_loc(quasi);

        par.lexer.consume();

        par.alloc_at_loc(quasi.start, quasi.end, TemplateLiteral {
            expressions: NodeList::empty(),
            quasis: NodeList::from(par.arena, quasi)
        })
    };

    pub const TPLE = |par| par.template_expression();
}

impl<'ast> Parser<'ast> {
    #[inline]
    fn bound_expression(&mut self) -> ExpressionNode<'ast> {
        unsafe { (*(DEF_CONTEXT as *const ExpressionHandler).offset(self.lexer.token as isize))(self) }
    }

    #[inline]
    fn context_bound_expression(&mut self, context: Context) -> ExpressionNode<'ast> {
        unsafe { (*(context as *const ExpressionHandler).offset(self.lexer.token as isize))(self) }
    }

    #[inline]
    pub fn expression<B>(&mut self) -> ExpressionNode<'ast>
    where
        B: BindingPower
    {
        let left = self.bound_expression();

        self.nested_expression::<B>(left)
    }

    #[inline]
    pub fn expression_in_context<B>(&mut self, context: Context) -> ExpressionNode<'ast>
    where
        B: BindingPower
    {
        let left = self.context_bound_expression(context);

        self.nested_expression::<B>(left)
    }

    #[inline]
    pub fn arrow_function_expression(&mut self, params: ExpressionList<'ast>) -> ArrowExpression<'ast> {
        let params = self.params_from_expressions(params);

        let body = match self.lexer.token {
            BraceOpen => ArrowBody::Block(self.unchecked_block()),
            _         => ArrowBody::Expression(self.expression::<B0>()),
        };

        ArrowExpression {
            params,
            body,
        }
    }

    #[inline]
    pub fn call_arguments(&mut self) -> ExpressionList<'ast> {
        if self.lexer.token == ParenClose {
            return NodeList::empty();
        }

        let expression = self.expression_in_context::<B0>(CALL_CONTEXT);
        let builder = ListBuilder::new(self.arena, expression);

        loop {
            let expression = match self.lexer.token {
                ParenClose => break,
                Comma      => {
                    self.lexer.consume();

                    if self.lexer.token == ParenClose {
                        break
                    }

                    self.expression_in_context::<B0>(CALL_CONTEXT)
                }
                _ => {
                    self.error::<()>();
                    break;
                }
            };

            builder.push(self.arena, expression);
        }

        builder.as_list()
    }

    #[inline]
    pub fn paren_expression(&mut self) -> ExpressionNode<'ast> {
        let start = self.lexer.start_then_consume();
        match self.lexer.token {
            ParenClose => {
                self.lexer.consume();
                expect!(self, OperatorFatArrow);
                let expression = self.arrow_function_expression(NodeList::empty());
                let end = self.lexer.end();
                self.alloc_at_loc(start, end, expression)
            },
            _ => {
                let expression = self.expression::<ANY>();

                expect!(self, ParenClose);

                expression
            }
        }
    }

    #[inline]
    pub fn prefix_expression(&mut self, operator: OperatorKind) -> PrefixExpression<'ast> {
        let operand = self.expression::<B15>();

        PrefixExpression {
            operator,
            operand,
        }
    }

    #[inline]
    pub fn object_expression(&mut self) -> ExpressionNode<'ast> {
        let start = self.lexer.start_then_consume();
        let body = self.property_list();
        let end = self.lexer.end_then_consume();

        self.alloc_at_loc(start, end, ObjectExpression {
            body
        })
    }

    #[inline]
    pub fn meta_property_expression(&mut self, meta: IdentifierNode<'ast>) -> MetaPropertyExpression<'ast> {
        let property = self.lexer.accessor_as_str();

        // Only `NewTarget` is a valid MetaProperty.
        if property != "target" {
            self.error::<()>();
        }

        let property = self.alloc_in_loc(property);

        MetaPropertyExpression {
            meta,
            property,
        }
    }

    #[inline]
    pub fn property_list(&mut self) -> NodeList<'ast, Property<'ast>> {
        if self.lexer.token == BraceClose {
            return NodeList::empty();
        }

        let builder = ListBuilder::new(self.arena, self.property());
        loop {
            match self.lexer.token {
                BraceClose => break,
                Comma      => self.lexer.consume(),
                _          => {
                    self.error::<()>();
                    break;
                }
            }

            match self.lexer.token {
                BraceClose => break,
                _          => builder.push(self.arena, self.property()),
            }
        }

        builder.as_list()
    }

    #[inline]
    pub fn property(&mut self) -> Node<'ast, Property<'ast>> {
        let start = self.lexer.start();

        let key = match self.lexer.token {
            _ if self.lexer.token.is_word() => {
                let (start, end) = self.lexer.loc();
                let label = self.lexer.token_as_str();

                self.lexer.consume();

                match self.lexer.token {
                    Colon | ParenOpen => self.alloc_at_loc(start, end, PropertyKey::Literal(label)),

                    _ => return self.alloc_at_loc(start, end, Property::Shorthand(label)),
                }
            },
            OperatorSpread => {
                let start = self.lexer.start_then_consume();
                let argument = self.expression::<B0>();
                let end = self.lexer.end();
                return self.alloc_at_loc(start, end, Property::Spread { argument });
            },
            LiteralString |
            LiteralNumber => {
                let num = self.lexer.token_as_str();
                let key = self.alloc_in_loc(PropertyKey::Literal(num));

                self.lexer.consume();

                key
            },
            LiteralBinary => {
                let num = self.lexer.token_as_str();
                let key = self.alloc_in_loc(PropertyKey::Binary(num));

                self.lexer.consume();

                key
            },
            BracketOpen => {
                let start = self.lexer.start_then_consume();
                let expression = self.expression::<ANY>();
                let end = self.lexer.end();

                expect!(self, BracketClose);

                self.alloc_at_loc(start, end, PropertyKey::Computed(expression))
            },
            _ => return self.error(),
        };

        match self.lexer.token {
            Colon => {
                self.lexer.consume();

                let value = self.expression::<B0>();

                self.alloc_at_loc(start, value.end, Property::Literal {
                    key,
                    value,
                })
            },
            ParenOpen => {
                let value = Node::parse(self);

                self.alloc_at_loc(start, value.end, Property::Method {
                    key,
                    value,
                })
            },
            _ => self.error()
        }
    }

    #[inline]
    pub fn array_expression(&mut self) -> ExpressionNode<'ast> {
        let start = self.lexer.start_then_consume();
        let body = self.array_elements(|par| par.expression_in_context::<B0>(ARRAY_CONTEXT));
        let end = self.lexer.end_then_consume();

        self.alloc_at_loc(start, end, ArrayExpression { body })
    }

    #[inline]
    /// Only in ArrayExpression
    pub fn void_expression(&mut self) -> ExpressionNode<'ast> {
        let loc = self.lexer.start();
        self.alloc_at_loc(loc, loc, Expression::Void)
    }

    #[inline]
    pub fn array_elements<F, I>(&mut self, get: F) -> NodeList<'ast, I> where
        F: Fn(&mut Parser<'ast>) -> Node<'ast, I>,
        I: 'ast + Copy,
    {
        let item = match self.lexer.token {
            BracketClose => return NodeList::empty(),
            _            => get(self),
        };

        let builder = ListBuilder::new(self.arena, item);

        loop {
            match self.lexer.token {
                Comma        => self.lexer.consume(),
                BracketClose => break,
                _            => {
                    self.error::<()>();
                    break;
                }
            }

            builder.push(self.arena, get(self))
        }

        builder.as_list()
    }

    #[inline]
    pub fn regular_expression(&mut self) -> ExpressionNode<'ast> {
        let start = self.lexer.start();
        let value = self.lexer.read_regular_expression();
        let end = self.lexer.end();

        expect!(self, LiteralRegEx);

        self.alloc_at_loc(start, end, Literal::RegEx(value))
    }

    #[inline]
    pub fn template_string<T>(&mut self) -> Node<'ast, T>
    where
        T: Copy + From<TemplateLiteral<'ast>>,
    {
        let quasi = self.lexer.quasi;
        let quasi = self.alloc_in_loc(quasi);

        self.lexer.consume();

        self.alloc_at_loc(quasi.start, quasi.end, TemplateLiteral {
            expressions: NodeList::empty(),
            quasis: NodeList::from(self.arena, quasi)
        })
    }

    #[inline]
    pub fn template_literal<T>(&mut self) -> Node<'ast, T>
    where
        T: Copy + From<TemplateLiteral<'ast>>,
    {
        let quasi = self.lexer.quasi;
        let quasi = self.alloc_in_loc(quasi);

        let start = self.lexer.start_then_consume();
        let end;

        let expression = self.expression::<ANY>();

        match self.lexer.token {
            BraceClose => self.lexer.read_template_kind(),
            _          => self.error(),
        }

        let quasis = ListBuilder::new(self.arena, quasi);
        let expressions = ListBuilder::new(self.arena, expression);

        loop {
            match self.lexer.token {
                TemplateOpen => {
                    let quasi = self.lexer.quasi;
                    quasis.push(self.arena, self.alloc_in_loc(quasi));
                    self.lexer.consume();
                    expressions.push(self.arena, self.expression::<ANY>());

                    match self.lexer.token {
                        BraceClose => self.lexer.read_template_kind(),
                        _          => {
                            end = self.lexer.end();
                            self.error::<()>();
                            break;
                        }
                    }
                },
                TemplateClosed => {
                    let quasi = self.lexer.quasi;
                    quasis.push(self.arena, self.alloc_in_loc(quasi));
                    end = self.lexer.end_then_consume();
                    break;
                },
                _ => {
                    end = self.lexer.end();
                    self.error::<()>();
                    break;
                }
            }
        }

        self.alloc_at_loc(start, end, TemplateLiteral {
            expressions: expressions.as_list(),
            quasis: quasis.as_list(),
        })
    }

    #[inline]
    pub fn template_expression(&mut self) -> ExpressionNode<'ast> {
        self.template_literal()
    }

    #[inline]
    pub fn tagged_template_expression(&mut self, tag: ExpressionNode<'ast>) -> ExpressionNode<'ast> {
        let quasi = self.template_literal();

        self.alloc_at_loc(tag.start, quasi.end, TaggedTemplateExpression {
            tag,
            quasi,
        })
    }

    #[inline]
    pub fn function_expression(&mut self) -> ExpressionNode<'ast> {
        let start = self.lexer.start_then_consume();
        let function = Function::parse(self);

        self.alloc_at_loc(start, function.body.end, function)
    }

    #[inline]
    pub fn class_expression(&mut self) -> ExpressionNode<'ast> {
        let start = self.lexer.start_then_consume();
        let class = Class::parse(self);

        self.alloc_at_loc(start, class.body.end, class)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ast::{OperatorKind, Literal, Statement, Function, Pattern, Class};
    use ast::expression::*;
    use ast::statement::*;
    use parser::parse;
    use parser::mock::Mock;

    #[test]
    fn ident_expression() {
        let expected = Expression::Identifier("foobar");

        assert_expr!("foobar;", expected);
    }

    #[test]
    fn value_expression() {
        let expected_a = Literal::String(r#""foobar""#);
        let expected_b = Literal::Number("100");
        let expected_c = Literal::True;

        assert_expr!(r#""foobar";"#, expected_a);
        assert_expr!("100;", expected_b);
        assert_expr!("true;", expected_c);
    }

    #[test]
    fn template_expression() {
        let src = "`foobar`;";
        let mock = Mock::new();

        let expected = TemplateLiteral {
            expressions: NodeList::empty(),
            quasis: mock.list(["foobar"]),
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn tagged_template_expression() {
        let src = "foo`bar`;";
        let mock = Mock::new();

        let expected = TaggedTemplateExpression {
            tag: mock.ptr("foo"),
            quasi: mock.ptr(TemplateLiteral {
                expressions: NodeList::empty(),
                quasis: mock.list(["bar"]),
            })
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn complex_template_expression() {
        let src = "`foo${ 10 }bar${ 20 }baz`;";
        let mock = Mock::new();

        let expected = TemplateLiteral {
            expressions: mock.list([
                Literal::Number("10"),
                Literal::Number("20"),
            ]),
            quasis: mock.list(["foo", "bar", "baz" ]),
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn tagged_complex_template_expression() {
        let src = "foo`bar${ 42 }baz`;";
        let mock = Mock::new();

        let expected = TaggedTemplateExpression {
            tag: mock.ptr("foo"),
            quasi: mock.ptr(TemplateLiteral {
                expressions: mock.list([
                    Literal::Number("42"),
                ]),
                quasis: mock.list(["bar", "baz"]),
            })
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn sequence_expression() {
        let src = "foo, bar, baz;";
        let mock = Mock::new();

        let expected = SequenceExpression {
            body: mock.list(["foo", "bar", "baz"]),
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn binary_expression() {
        let src = "foo + bar;";
        let mock = Mock::new();

        let expected = BinaryExpression {
            operator: OperatorKind::Addition,
            left: mock.ptr("foo"),
            right: mock.ptr("bar"),
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn parenthesized_binary_expression() {
        let src = "(2 + 2);";
        let mock = Mock::new();

        let expected = BinaryExpression {
            operator: OperatorKind::Addition,
            left: mock.number("2"),
            right: mock.number("2"),
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn conditional_expression() {
        let src = "true ? foo : bar";

        let mock = Mock::new();

        let expected = ConditionalExpression {
            test: mock.ptr(Expression::Literal(Literal::True)),
            consequent: mock.ptr("foo"),
            alternate: mock.ptr("bar"),
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn complex_conditional_expression() {
        let src = "true ? foo = bar : baz";

        let mock = Mock::new();

        let expected = ConditionalExpression {
            test: mock.ptr(Expression::Literal(Literal::True)),
            consequent: mock.ptr(BinaryExpression {
                operator: OperatorKind::Assign,
                left: mock.ptr("foo"),
                right: mock.ptr("bar"),
            }),
            alternate: mock.ptr("baz"),
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn postfix_expression() {
        let src = "baz++;";
        let mock = Mock::new();

        let expected = PostfixExpression {
            operator: OperatorKind::Increment,
            operand: mock.ptr("baz"),
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn call_expression() {
        {
            let src = "foo();";
            let mock = Mock::new();

            let expected = CallExpression {
                callee: mock.ptr("foo"),
                arguments: NodeList::empty(),
            };

            assert_expr!(src, expected);
        }

        {
            let src = "foo(1);";
            let mock = Mock::new();

            let expected = CallExpression {
                callee: mock.ptr("foo"),
                arguments: mock.list([
                    Literal::Number("1"),
                ]),
            };

            assert_expr!(src, expected);
        }

        {
            let src = "foo(1,2);";
            let mock = Mock::new();

            let expected = CallExpression {
                callee: mock.ptr("foo"),
                arguments: mock.list([
                    Literal::Number("1"),
                    Literal::Number("2"),
                ]),
            };

            assert_expr!(src, expected);
        }

        {
            let src = "foo(1,);";
            let mock = Mock::new();

            let expected = CallExpression {
                callee: mock.ptr("foo"),
                arguments: mock.list([
                    Literal::Number("1"),
                ]),
            };

            assert_expr!(src, expected);
        }

        {
            let src = "foo(1,2,);";
            let mock = Mock::new();

            let expected = CallExpression {
                callee: mock.ptr("foo"),
                arguments: mock.list([
                    Literal::Number("1"),
                    Literal::Number("2"),
                ]),
            };

            assert_expr!(src, expected);
        }
    }

    #[test]
    fn member_expression() {
        let src = "foo.bar";
        let mock = Mock::new();

        let expected = MemberExpression {
            object: mock.ptr("foo"),
            property: mock.ptr("bar"),
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn keyword_member_expression() {
        let src = "foo.function";
        let mock = Mock::new();

        let expected = MemberExpression {
            object: mock.ptr("foo"),
            property: mock.ptr("function"),
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn computed_member_expression() {
        let src = "foo[10]";
        let mock = Mock::new();

        let expected = ComputedMemberExpression {
            object: mock.ptr("foo"),
            property: mock.number("10"),
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn meta_property_expression() {
        let src = "new.target";
        let mock = Mock::new();
        let expected = MetaPropertyExpression {
            meta: mock.ptr("new"),
            property: mock.ptr("target"),
        };
        assert_expr!(src, expected);
    }

    #[test]
    fn meta_property_expression_throws() {
        assert!(parse("new.callee").is_err());
    }

    #[test]
    fn regular_expression() {
        let src = r#"/^[A-Z]+\/[\d]+/g"#;

        let expected = Literal::RegEx("/^[A-Z]+\\/[\\d]+/g");

        assert_expr!(src, expected);
    }

    #[test]
    fn array_expression() {
        let src = "[0, 1, 2]";
        let mock = Mock::new();

        let expected = ArrayExpression {
            body: mock.list([
                Literal::Number("0"),
                Literal::Number("1"),
                Literal::Number("2"),
            ])
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn sparse_array_expression() {
        let src = "[,,foo,bar,,]";
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

        assert_expr!(src, expected);
    }

    #[test]
    fn spread_expression_in_array() {
        let src = "[a, b, ...c]";
        let mock = Mock::new();

        let expected = ArrayExpression {
            body: mock.list([
                Expression::Identifier("a"),
                Expression::Identifier("b"),
                Expression::Spread(SpreadExpression {
                    argument: mock.ptr("c")
                })
            ])
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn spread_expression_in_call() {
        let src = "foo(a, b, ...c)";
        let mock = Mock::new();

        let expected = CallExpression {
            callee: mock.ptr("foo"),
            arguments: mock.list([
                Expression::Identifier("a"),
                Expression::Identifier("b"),
                Expression::Spread(SpreadExpression {
                    argument: mock.ptr("c")
                })
            ])
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn spread_expression_illegal_bare() {
        assert!(parse("let foo = ...c;").is_err());
    }

    #[test]
    fn function_expression() {
        let src = "(function () {})";
        let mock = Mock::new();

        let expected = Function {
            name: None.into(),
            generator: false,
            params: NodeList::empty(),
            body: mock.empty_block()
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn named_function_expression() {
        let src = "(function foo () {})";
        let mock = Mock::new();

        let expected = Function {
            name: mock.name("foo"),
            generator: false,
            params: NodeList::empty(),
            body: mock.empty_block()
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn arrow_function_expression() {
        let src = "() => bar";
        let mock = Mock::new();

        let expected = ArrowExpression {
            params: NodeList::empty(),
            body: ArrowBody::Expression(mock.ptr("bar")),
        };
        assert_expr!(src, expected);
    }

    #[test]
    fn arrow_function_shorthand() {
        let src = "n => n * n";
        let mock = Mock::new();

        let expected = ArrowExpression {
            params: mock.list([
                Pattern::Identifier("n")
            ]),

            body: ArrowBody::Expression(mock.ptr(BinaryExpression {
                operator: OperatorKind::Multiplication,
                left: mock.ptr("n"),
                right: mock.ptr("n"),
            }))

        };
        assert_expr!(src, expected);
    }

    #[test]
    fn arrow_function_with_params() {
        let src = "(a, b, c) => bar";
        let mock = Mock::new();

        let expected = ArrowExpression {
            params: mock.list([
                Pattern::Identifier("a"),
                Pattern::Identifier("b"),
                Pattern::Identifier("c")
            ]),
            body: ArrowBody::Expression(mock.ptr("bar"))
        };
        assert_expr!(src, expected);
    }

    #[test]
    fn arrow_function_invalid_params_throws() {
        assert!(parse("(a, b, c * 2) => bar").is_err());
    }

    #[test]
    fn arrow_function_with_default_params() {
        let src = "(a, b, c = 2) => bar";
        let mock = Mock::new();

        let expected = ArrowExpression {
            params: mock.list([
                Pattern::Identifier("a"),
                Pattern::Identifier("b"),
                Pattern::AssignmentPattern {
                    left: mock.ptr(Pattern::Identifier("c")),
                    right: mock.number("2")
                }
            ]),
            body: ArrowBody::Expression(mock.ptr("bar"))
        };
        assert_expr!(src, expected);
    }

    #[test]
    fn class_expression() {
        let src = "(class {})";
        let mock = Mock::new();

        let expected = Class {
            name: None.into(),
            extends: None,
            body: mock.empty_block()
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn named_class_expression() {
        let src = "(class Foo {})";
        let mock = Mock::new();

        let expected = Class {
            name: mock.name("Foo"),
            extends: None,
            body: mock.empty_block()
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn named_child_class_expression() {
        let src = "(class Foo extends Bar {})";
        let mock = Mock::new();

        let expected = Class {
            name: mock.name("Foo"),
            extends: Some(mock.ptr("Bar")),
            body: mock.empty_block()
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn regression_operator_precedence() {
        let src = "true === true && false === false";
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

        assert_expr!(src, expected);
    }

    #[test]
    fn arrow_function_in_sequence() {
        let src = "(() => {}, foo)";
        let mock = Mock::new();

        let expected = SequenceExpression {
            body: mock.list([
                Expression::Arrow(ArrowExpression {
                    params: NodeList::empty(),
                    body: ArrowBody::Block(mock.ptr(BlockStatement {
                        body: NodeList::empty()
                    }))
                }),
                Expression::Identifier("foo"),
            ])
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn regression_increments() {
        let src = "x++ + ++y";
        let mock = Mock::new();

        let expected = BinaryExpression {
            operator: OperatorKind::Addition,
            left: mock.ptr(PostfixExpression {
                operator: OperatorKind::Increment,
                operand: mock.ptr("x"),
            }),
            right: mock.ptr(PrefixExpression {
                operator: OperatorKind::Increment,
                operand: mock.ptr("y"),
            })
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn regression_decrements() {
        let src = "x-- - --y";
        let mock = Mock::new();

        let expected = BinaryExpression {
            operator: OperatorKind::Subtraction,
            left: mock.ptr(PostfixExpression {
                operator: OperatorKind::Decrement,
                operand: mock.ptr("x"),
            }),
            right: mock.ptr(PrefixExpression {
                operator: OperatorKind::Decrement,
                operand: mock.ptr("y"),
            })
        };

        assert_expr!(src, expected);
    }

    #[test]
    fn assignment_to_lvalue() {
        assert!(parse("(x++)++").is_err());
        assert!(parse("x+++++y").is_err());
    }

    #[test]
    fn regression_asi_increments() {
        let src = r#"x
        ++
        y"#;
        let mock = Mock::new();

        let expected = mock.list([
            mock.ptr(Expression::Identifier("x")),
            mock.ptr(PrefixExpression {
                operator: OperatorKind::Increment,
                operand: mock.ptr("y"),
            }),
        ]);
        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn regression_asi_decrements() {
        let src = r#"x
        --
        y"#;
        let mock = Mock::new();

        let expected = mock.list([
            mock.ptr(Expression::Identifier("x")),
            mock.ptr(PrefixExpression {
                operator: OperatorKind::Decrement,
                operand: mock.ptr("y"),
            }),
        ]);
        assert_eq!(parse(src).unwrap().body(), expected);
    }

    #[test]
    fn regression_asi_safe() {
        let src = r#"foo
        .bar"#;
        let mock = Mock::new();

        let expected = MemberExpression {
            object: mock.ptr("foo"),
            property: mock.ptr("bar"),
        };

        assert_expr!(src, expected);
    }
}
