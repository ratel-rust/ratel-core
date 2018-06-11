use ratel::ast::{Expression, Literal, OperatorKind, Property, PropertyKey, Pattern};
use ratel::ast::expression::*;

use {ToCode, Generator};


impl<'ast, G: Generator> ToCode<G> for Expression<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        use ratel::ast::Expression::*;

        match *self {
            Void                         => {},
            This(_)                      => gen.write_bytes(b"this"),
            Identifier(ref ident)        => gen.write(ident),
            Literal(ref value)           => gen.write(value),
            Sequence(ref sequence)       => gen.write(sequence),
            Array(ref array)             => gen.write(array),
            Member(ref member)           => gen.write(member),
            ComputedMember(ref computed) => gen.write(computed),
            MetaProperty(ref property)   => gen.write(property),
            Call(ref call)               => gen.write(call),
            Binary(ref binary)           => gen.write(binary),
            Prefix(ref prefix)           => gen.write(prefix),
            Postfix(ref postfix)         => gen.write(postfix),
            Conditional(ref conditional) => gen.write(conditional),
            Template(ref template)       => gen.write(template),
            TaggedTemplate(ref tagged)   => gen.write(tagged),
            Spread(ref spread)           => gen.write(spread),
            Arrow(ref arrow)             => gen.write(arrow),
            Object(ref object)           => gen.write(object),
            Function(ref function)       => gen.write(function),
            Class(ref class)             => gen.write(class),
        }
    }
}

impl<'ast, G: Generator> ToCode<G> for Literal<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        use ratel::ast::Literal::*;

        match *self {
            Undefined         => gen.write_bytes(b"undefined"),
            Null              => gen.write_bytes(b"null"),
            True              => gen.write_bytes(b"true"),
            False             => gen.write_bytes(b"false"),
            Binary(n)         => gen.write(&format!("{}", n).as_str()),
            Number(ref val)   |
            String(ref val)   |
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

impl<'ast, G: Generator> ToCode<G> for PropertyKey<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        use ratel::ast::PropertyKey::*;

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

impl<'ast, G: Generator> ToCode<G> for Property<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        use ratel::ast::Property::*;

        match *self {
            Shorthand(ref label) => gen.write(label),
            Literal {
                ref key,
                ref value,
            } => {
                gen.write(key);
                gen.write_byte(b':');
                gen.write_pretty(b' ');
                gen.write(value);
            },
            Method {
                ref key,
                ref value,
            } => {
                gen.write(key);
                gen.write(value);
            }
        }
    }
}

impl<'ast, G: Generator> ToCode<G> for SequenceExpression<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write_list(&self.body);
    }
}

impl<'ast, G: Generator> ToCode<G> for ArrayExpression<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write_byte(b'[');
        gen.write_list(&self.body);
        gen.write_byte(b']');
    }
}

impl<'ast, G: Generator> ToCode<G> for MemberExpression<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write_expression(&self.object, 19);
        gen.write_byte(b'.');
        gen.write(&self.property);
    }
}

impl<'ast, G: Generator> ToCode<G> for ComputedMemberExpression<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write_expression(&self.object, 19);
        gen.write_byte(b'[');
        gen.write(&self.property);
        gen.write_byte(b']');
    }
}

impl<'ast, G: Generator> ToCode<G> for MetaPropertyExpression<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write(&self.meta);
        gen.write_byte(b'.');
        gen.write(&self.property);
    }
}

impl<'ast, G: Generator> ToCode<G> for CallExpression<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write(&self.callee);
        gen.write_byte(b'(');
        gen.write_list(&self.arguments);
        gen.write_byte(b')');
    }
}

impl<'ast, G: Generator> ToCode<G> for BinaryExpression<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        let bp = self.operator.binding_power();
        let spacing = self.operator.is_word();

        gen.write_expression(&self.left, bp);

        if spacing {
            gen.write_byte(b' ');
        } else {
            gen.write_pretty(b' ');
        }
        gen.write(&self.operator);
        if spacing {
            gen.write_byte(b' ');
        } else {
            gen.write_pretty(b' ');
        }

        // `2 / 2 * 2` and `2 / (2 * 2)` are different expressions,
        // hence the need for parenthesis in a right-balanced tree
        // even if binding power of operators is exactly the same.
        gen.write_expression(&self.right, bp + 1);
    }
}

impl<'ast, G: Generator> ToCode<G> for PrefixExpression<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write(&self.operator);
        if self.operator.is_word() {
            gen.write_byte(b' ');
        }
        gen.write(&self.operand);
    }
}

impl<'ast, G: Generator> ToCode<G> for PostfixExpression<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write(&self.operand);
        gen.write(&self.operator);
    }
}

impl<'ast, G: Generator> ToCode<G> for ConditionalExpression<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write(&self.test);
        gen.write_pretty(b' ');
        gen.write_byte(b'?');
        gen.write_pretty(b' ');
        gen.write(&self.consequent);
        gen.write_pretty(b' ');
        gen.write_byte(b':');
        gen.write_pretty(b' ');
        gen.write(&self.alternate);
    }
}

impl<'ast, G: Generator> ToCode<G> for TemplateLiteral<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write_byte(b'`');

        match self.quasis.only_element() {
            Some(quasi) => gen.write(quasi),
            None => {
                let mut quasis = self.quasis.iter();

                if let Some(quasi) = quasis.next() {
                    gen.write(quasi);
                }

                for (quasi, expression) in quasis.zip(&self.expressions) {
                    gen.write_bytes(b"${");
                    gen.write_pretty(b' ');
                    gen.write(expression);
                    gen.write_pretty(b' ');
                    gen.write_byte(b'}');
                    gen.write(quasi);
                }
            }
        }

        gen.write_byte(b'`');
    }
}

impl<'ast, G: Generator> ToCode<G> for TaggedTemplateExpression<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write(&self.tag);
        gen.write(&self.quasi);
    }
}

impl<'ast, G: Generator> ToCode<G> for SpreadExpression<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write_bytes(b"...");
        gen.write(&self.argument);
    }
}

impl<'ast, G: Generator> ToCode<G> for ArrowBody<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        match *self {
            ArrowBody::Expression(ref expression) => gen.write(expression),
            ArrowBody::Block(ref block)           => gen.write(block),
        }
    }
}

impl<'ast, G: Generator> ToCode<G> for ArrowExpression<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        match self.params.only_element().map(|el| &el.item) {
            Some(&Pattern::Identifier(ref ident)) => gen.write(ident),
            _ => {
                gen.write_byte(b'(');
                gen.write_list(&self.params);
                gen.write_byte(b')');
            }
        }
        gen.write_pretty(b' ');
        gen.write_bytes(b"=>");
        gen.write_pretty(b' ');
        gen.write(&self.body);
    }
}

impl<'ast, G: Generator> ToCode<G> for ObjectExpression<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        let mut properties = self.body.iter();

        match properties.next() {
            Some(property) => {
                gen.write_byte(b'{');
                gen.indent();
                gen.new_line();
                gen.write(property);
            },
            None => {
                gen.write_bytes(b"{}");
                return;
            }
        }

        for property in properties {
            gen.write_byte(b',');
            gen.new_line();
            gen.write(property);
        }
        gen.dedent();
        gen.new_line();
        gen.write_byte(b'}');
    }
}

#[cfg(test)]
mod test {
    use {assert_min, assert_pretty};

    #[test]
    fn values() {
        assert_min("null", "null;");
        assert_min("undefined", "undefined;");
        assert_min("true", "true;");
        assert_min("false", "false;");
        assert_min("42", "42;");
        assert_min("3.14", "3.14;");
        assert_min(r#" "foobar" "#, r#""foobar";"#);
        assert_min(r#" 'foobar' "#, r#"'foobar';"#);
    }

    #[test]
    fn template_expression() {
        assert_min("``", "``;");
        assert_min("foo``", "foo``;");
        assert_min("`foobar`", "`foobar`;");
        assert_min("foo`bar`", "foo`bar`;");
        assert_min("`foo${ 10 }bar${ 20 }baz`", "`foo${10}bar${20}baz`;");
        assert_min("foo`bar${ 10 }baz`", "foo`bar${10}baz`;");
        assert_min("foo`${ 10 }`", "foo`${10}`;");
    }

    #[test]
    fn sequence_expression() {
        assert_min("foo, bar, baz;", "foo,bar,baz;");
        assert_min("1, 2, 3;", "1,2,3;");
        assert_min("1,2,3+4;", "1,2,3+4;");
        assert_min("1,2,(3+4);", "1,2,3+4;");
        assert_min("1+2,3,4;", "1+2,3,4;");
        assert_min("(1+2),3,4;", "1+2,3,4;");
        assert_min("1+(2,3,4);", "1+(2,3,4);");
        assert_min("(1,2,3)+4;", "(1,2,3)+4;");
    }

    #[test]
    fn binary_expression() {
        assert_min("a = 10", "a=10;");
        assert_min("a == 10", "a==10;");
        assert_min("a === 10", "a===10;");
        assert_min("a != 10", "a!=10;");
        assert_min("a !== 10", "a!==10;");
        assert_min("a += 10", "a+=10;");
        assert_min("a -= 10", "a-=10;");
        assert_min("a <<= 10", "a<<=10;");
        assert_min("a >>= 10", "a>>=10;");
        assert_min("a >>>= 10", "a>>>=10;");
        assert_min("2 + 2", "2+2;");
        assert_min("2 - 2", "2-2;");
        assert_min("2 * 2", "2*2;");
        assert_min("2 / 2", "2/2;");
        assert_min("2 % 2", "2%2;");
        assert_min("2 ** 2", "2**2;");
        assert_min("2 << 2", "2<<2;");
        assert_min("2 >> 2", "2>>2;");
        assert_min("2 >>> 2", "2>>>2;");
        assert_min("foo in bar", "foo in bar;");
        assert_min("foo instanceof Foo", "foo instanceof Foo;");
    }

    #[test]
    fn binary_expression_precedence() {
        assert_min("2 + 2 * 2", "2+2*2;");
        assert_min("2 + (2 * 2)", "2+2*2;");
        assert_min("(2 + 2) * 2", "(2+2)*2;");
    }

    #[test]
    fn prefix_expression() {
        assert_min("+foo", "+foo;");
        assert_min("-foo", "-foo;");
        assert_min("!foo", "!foo;");
        assert_min("~foo", "~foo;");
        assert_min("++foo", "++foo;");
        assert_min("--foo", "--foo;");
        assert_min("new foo", "new foo;");
        assert_min("void foo", "void foo;");
        assert_min("typeof foo", "typeof foo;");
    }

    #[test]
    fn postfix_expression() {
        assert_min("foo++", "foo++;");
        assert_min("foo--", "foo--;");
    }

    #[test]
    fn conditional_expression() {
        assert_min("true ? foo : bar", "true?foo:bar;")
    }

    #[test]
    fn function_expression() {
        assert_min("(function () {})", "(function(){});");
        assert_min("(function foo() {})", "(function foo(){});");
    }

    #[test]
    fn class_expression() {
        assert_min("(class {})", "(class{});");
        assert_min("(class Foo {})", "(class Foo{});");
        assert_min("(class extends Foo {})", "(class extends Foo{});");
        assert_min("(class Foo extends Bar {})", "(class Foo extends Bar{});");

    }

    #[test]
    fn call_expression() {
        assert_min("foobar();", "foobar();");
        assert_min("foobar(1, 2, 3);", "foobar(1,2,3);");
    }

    #[test]
    fn member_expression() {
        assert_min("foo.bar", "foo.bar;");
        assert_min("this.bar", "this.bar;");
        assert_min("10..fooz", "10..fooz;");
        assert_min("foo[10]", "foo[10];");
        assert_min(r#"foo["bar"]"#, r#"foo["bar"];"#);
    }

    #[test]
    fn array_expression() {
        assert_min("[]", "[];");
        assert_min("[foo]", "[foo];");
        assert_min("[foo,bar]", "[foo,bar];");
        assert_min("[foo,bar,baz]", "[foo,bar,baz];");
    }

    #[test]
    fn array_spread() {
        assert_min("[...foo,...bar]", "[...foo,...bar];");
    }

    #[test]
    fn sparse_array_expression() {
        assert_min("[]", "[];");
        assert_min("[,]", "[,];");
        assert_min("[1,]", "[1,];");
        assert_min("[,1]", "[,1];");
        assert_min("[,,];", "[,,];");
        assert_min("[1,,];", "[1,,];");
        assert_min("[,,1];", "[,,1];");
    }

    #[test]
    fn sparse_array_expression_pretty() {
        assert_pretty("[]", "[];");
        assert_pretty("[,]", "[, ];");
        assert_pretty("[1,]", "[1, ];");
        assert_pretty("[,1]", "[, 1];");
        assert_pretty("[,,];", "[, , ];");
        assert_pretty("[1,,];", "[1, , ];");
        assert_pretty("[,,1];", "[, , 1];");
    }

    #[test]
    fn object_expression() {
        assert_min("({});", "({});");
        assert_min("({ foo });", "({foo});");
        assert_min("({ foo: 10 });", "({foo:10});");
        assert_min("({ foo, bar });", "({foo,bar});");
        assert_min("({ foo: 10, bar: 20 });", "({foo:10,bar:20});");
        assert_min("({ foo: 10, bar() {} });", "({foo:10,bar(){}});");
        assert_min("({ foo(bar, baz) {} });", "({foo(bar,baz){}});");
        let expected = "({\n    foo: true,\n    bar: false\n});";
        assert_pretty("({ foo: true, bar: false })", expected);
    }

    #[test]
    fn binding_power() {
        assert_min("1 + 2 * 3;", "1+2*3;");
        assert_min("1 + (2 * 3);", "1+2*3;");
        assert_min("(1 + 2) * 3;", "(1+2)*3;");
        assert_min("(denominator / divider * 100).toFixed(2);", "(denominator/divider*100).toFixed(2);");
        assert_min("(1 + 1)[0];", "(1+1)[0];");
        assert_min("2 * 2 / 2;", "2*2/2;");
        assert_min("2 * (2 / 2);", "2*(2/2);");
        assert_min("(2 * 2) / 2;", "2*2/2;");
    }
}
