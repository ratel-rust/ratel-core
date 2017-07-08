use ast::{Expression, Statement, Value, OperatorKind, ObjectMember, Property};
use codegen::{ToCode, Generator};

impl<'ast, G: Generator> ToCode<G> for Value<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        use ast::Value::*;

        match *self {
            Undefined         => gen.write_bytes(b"undefined"),
            Null              => gen.write_bytes(b"null"),
            True              => gen.write_bytes(b"true"),
            False             => gen.write_bytes(b"false"),
            Binary(n)         => gen.write(&format!("{}", n).as_str()),
            Number(ref val)   |
            String(ref val)   |
            RawQuasi(ref val) |
            RegEx(ref val)    => gen.write(val),
        }
    }
}

impl<G: Generator> ToCode<G> for OperatorKind {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write_bytes(self.as_str().as_bytes())
    }
}

impl<'ast, G: Generator> ToCode<G> for Property<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        use ast::Property::*;

        match *self {
            Computed(ref val) => {
                gen.write_byte(b'[');
                gen.write(val);
                gen.write_byte(b']');
            },
            Literal(ref val) => gen.write(val),
            Binary(ref val) => gen.write(val),
        }
    }
}

impl<'ast, G: Generator> ToCode<G> for ObjectMember<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        use ast::ObjectMember::*;

        match *self {
            Shorthand(ref label) => gen.write(label),
            Value {
                ref property,
                ref value,
            } => {
                gen.write(property);
                gen.write_byte(b':');
                gen.write_pretty(b' ');
                gen.write(value);
            },
            Method {
                ref property,
                ref params,
                ref body,
            } => {
                gen.write(property);
                gen.write_byte(b'(');
                gen.write_list(params);
                gen.write_byte(b')');
                gen.write_pretty(b' ');
                gen.write_byte(b'{');
                gen.write_block(body);
                gen.write_byte(b'}');
            }
        }
    }
}

impl<'ast, G: Generator> ToCode<G> for Expression<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        use ast::Expression::*;

        match *self {
            Error => panic!("Module contains errors"),
            This => gen.write_bytes(b"this"),
            Identifier(ref ident) => gen.write(ident),
            Value(ref value) => gen.write(value),
            Sequence {
                ref body
            } => {
                gen.write_list(body.iter());
            },
            Array {
                ref body
            } => {
                gen.write_byte(b'[');
                gen.write_list(body.iter());
                gen.write_byte(b']');
            },
            Member {
                ref object,
                ref property,
            } => {
                gen.write(object);
                gen.write_byte(b'.');
                gen.write(property);
            },
            ComputedMember {
                ref object,
                ref property,
            } => {
                gen.write(object);
                gen.write_byte(b'[');
                gen.write(property);
                gen.write_byte(b']');
            },
            Call {
                ref callee,
                ref arguments,
            } => {
                gen.write(callee);
                gen.write_byte(b'(');
                gen.write_list(arguments);
                gen.write_byte(b')');
            },
            Binary {
                ref operator,
                ref left,
                ref right,
                ..
            } => {
                let bp = self.binding_power();
                let spacing = operator.is_word();

                if left.binding_power() < bp {
                    gen.write_byte(b'(');
                    gen.write(left);
                    gen.write_byte(b')');
                } else {
                    gen.write(left);
                }

                match spacing {
                    true  => gen.write_byte(b' '),
                    false => gen.write_pretty(b' '),
                }
                gen.write(operator);
                match spacing {
                    true  => gen.write_byte(b' '),
                    false => gen.write_pretty(b' '),
                }

                if right.binding_power() < bp {
                    gen.write_byte(b'(');
                    gen.write(right);
                    gen.write_byte(b')');
                } else {
                    gen.write(right);
                }
            },
            Prefix {
                ref operator,
                ref operand,
            } => {
                gen.write(operator);
                if operator.is_word() {
                    gen.write_byte(b' ');
                }
                gen.write(operand);
            },
            Postfix {
                ref operator,
                ref operand,
            } => {
                gen.write(operand);
                gen.write(operator);
            },
            Conditional {
                ref test,
                ref consequent,
                ref alternate,
            } => {
                gen.write(test);
                gen.write_pretty(b' ');
                gen.write_byte(b'?');
                gen.write_pretty(b' ');
                gen.write(consequent);
                gen.write_pretty(b' ');
                gen.write_byte(b':');
                gen.write_pretty(b' ');
                gen.write(alternate);
            },
            Arrow {
                ref params,
                ref body,
            } => {
                match params.only_element() {
                    Some(param) => gen.write(param),
                    None        => {
                        gen.write_byte(b'(');
                        gen.write_list(params);
                        gen.write_byte(b')');
                    }
                }
                gen.write_pretty(b' ');
                gen.write_bytes(b"=>");
                gen.write_pretty(b' ');
                match body.item {
                    Statement::Expression {
                        ref expression,
                    } => gen.write(expression),
                    _ => gen.write(body),
                }
            },
            Object {
                ref body,
            } => {
                if body.is_empty() {
                    gen.write_bytes(b"{}");
                    return;
                }

                gen.write_byte(b'{');
                gen.indent();

                let mut iter = body.iter();

                for member in iter.next() {
                    gen.new_line();
                    gen.write(member);
                }

                for member in iter {
                    gen.write_byte(b',');
                    gen.new_line();
                    gen.write(member);
                }

                gen.dedent();
                gen.new_line();
                gen.write_byte(b'}');
            },
            Function {
                ref function,
            } => {
                gen.write(function);
            },
            Class {
                ref class,
            } => {
                gen.write(class);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use codegen::assert_parse;

    #[test]
    fn values() {
        assert_parse("null", "null;");
        assert_parse("undefined", "undefined;");
        assert_parse("true", "true;");
        assert_parse("false", "false;");
        assert_parse("42", "42;");
        assert_parse("3.14", "3.14;");
        assert_parse(r#" "foobar" "#, r#""foobar";"#);
        assert_parse(r#" 'foobar' "#, r#"'foobar';"#);
    }

    #[test]
    fn sequence_expression() {
        assert_parse("foo, bar, baz;", "foo,bar,baz;");
        assert_parse("1, 2, 3;", "1,2,3;");
        assert_parse("1,2,3+4;", "1,2,3+4;");
        assert_parse("1+(2,3,4);", "1+(2,3,4);");
        assert_parse("(1,2,3)+4;", "(1,2,3)+4;");
    }

    #[test]
    fn binary_expression() {
        assert_parse("a = 10", "a=10;");
        assert_parse("a == 10", "a==10;");
        assert_parse("a === 10", "a===10;");
        assert_parse("a != 10", "a!=10;");
        assert_parse("a !== 10", "a!==10;");
        assert_parse("a += 10", "a+=10;");
        assert_parse("a -= 10", "a-=10;");
        assert_parse("a <<= 10", "a<<=10;");
        assert_parse("a >>= 10", "a>>=10;");
        assert_parse("a >>>= 10", "a>>>=10;");
        assert_parse("2 + 2", "2+2;");
        assert_parse("2 - 2", "2-2;");
        assert_parse("2 * 2", "2*2;");
        assert_parse("2 / 2", "2/2;");
        assert_parse("2 % 2", "2%2;");
        assert_parse("2 ** 2", "2**2;");
        assert_parse("2 << 2", "2<<2;");
        assert_parse("2 >> 2", "2>>2;");
        assert_parse("2 >>> 2", "2>>>2;");
        assert_parse("foo in bar", "foo in bar;");
        assert_parse("foo instanceof Foo", "foo instanceof Foo;");
    }

    #[test]
    fn binary_expression_precedence() {
        assert_parse("2 + 2 * 2", "2+2*2;");
        assert_parse("2 + (2 * 2)", "2+2*2;");
        assert_parse("(2 + 2) * 2", "(2+2)*2;");
    }

    #[test]
    fn prefix_expression() {
        assert_parse("+foo", "+foo;");
        assert_parse("-foo", "-foo;");
        assert_parse("!foo", "!foo;");
        assert_parse("~foo", "~foo;");
        assert_parse("++foo", "++foo;");
        assert_parse("--foo", "--foo;");
        assert_parse("new foo", "new foo;");
        assert_parse("void foo", "void foo;");
        assert_parse("typeof foo", "typeof foo;");
    }

    #[test]
    fn postfix_expression() {
        assert_parse("foo++", "foo++;");
        assert_parse("foo--", "foo--;");
    }

    #[test]
    fn conditional_expression() {
        assert_parse("true ? foo : bar", "true?foo:bar;")
    }

    #[test]
    fn function_expression() {
        assert_parse("(function () {})", "(function(){});");
        assert_parse("(function foo() {})", "(function foo(){});");
    }

    #[test]
    fn call_expression() {
        assert_parse("foobar();", "foobar();");
        assert_parse("foobar(1, 2, 3);", "foobar(1,2,3);");
    }

    #[test]
    fn member_expression() {
        assert_parse("foo.bar", "foo.bar;");
        assert_parse("this.bar", "this.bar;");
        assert_parse("10..fooz", "10..fooz;");
        assert_parse("foo[10]", "foo[10];");
        assert_parse(r#"foo["bar"]"#, r#"foo["bar"];"#);
    }

    #[test]
    fn object_expression() {
        assert_parse("({});", "({});");
        assert_parse("({ foo });", "({foo});");
        assert_parse("({ foo: 10 });", "({foo:10});");
        assert_parse("({ foo, bar });", "({foo,bar});");
        assert_parse("({ foo: 10, bar: 20 });", "({foo:10,bar:20});");
        assert_parse("({ foo: 10, bar() {} });", "({foo:10,bar(){}});");
        assert_parse("({ foo(bar, baz) {} });", "({foo(bar,baz){}});");
    }
}
