use ast::{Function, Class, ClassMember, Name, MandatoryName, OptionalName};
use ast::{Parameter, ParameterKey};
use codegen::{ToCode, Generator};

impl<'ast, G: Generator> ToCode<G> for MandatoryName<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write_byte(b' ');
        gen.write(&self.0);
    }
}

impl<'ast, G: Generator> ToCode<G> for OptionalName<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        match self.0 {
            Some(ref name) => {
                gen.write_byte(b' ');
                gen.write(name);
            },
            None => gen.write_pretty(b' '),
        }
    }
}

impl<'ast, G: Generator> ToCode<G> for ParameterKey<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        use ast::ParameterKey::*;

        match *self {
            Identifier(ref label) => gen.write(label),
            Pattern(ref pattern) => gen.write(pattern),
        }
    }
}

impl<'ast, G: Generator> ToCode<G> for Parameter<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write(&self.key);

        if let Some(ref value) = self.value {
            gen.write_pretty(b' ');
            gen.write_byte(b'=');
            gen.write_pretty(b' ');
            gen.write(value);
        }
    }
}

impl<'ast, G, N> ToCode<G> for Function<'ast, N> where
    G: Generator,
    N: Name<'ast> + ToCode<G>,
{
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write_bytes(b"function");
        gen.write(&self.name);
        gen.write_byte(b'(');
        gen.write_list(&self.params);
        gen.write_byte(b')');
        gen.write_pretty(b' ');
        gen.write_byte(b'{');
        gen.write_block(&self.body);
        gen.write_byte(b'}');
    }
}

impl<'ast, G: Generator> ToCode<G> for ClassMember<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        use ast::ClassMember::*;

        match *self {
            Constructor {
                ref params,
                ref body,
            } => {
                gen.write_bytes(b"constructor(");
                gen.write_list(params);
                gen.write_byte(b')');
                gen.write_pretty(b' ');
                gen.write_byte(b'{');
                gen.write_block(body);
                gen.write_byte(b'}');
            },
            Method {
                is_static,
                ref property,
                ref params,
                ref body,
            } => {
                if is_static {
                    gen.write_bytes(b"static ");
                }
                gen.write(property);
                gen.write_byte(b'(');
                gen.write_list(params);
                gen.write_byte(b')');
                gen.write_pretty(b' ');
                gen.write_byte(b'{');
                gen.write_block(body);
                gen.write_byte(b'}');
            },
            Value {
                is_static,
                ref property,
                ref value,
            } => {
                if is_static {
                    gen.write_bytes(b"static ");
                }
                gen.write(property);
                gen.write_pretty(b' ');
                gen.write_byte(b'=');
                gen.write_pretty(b' ');
                gen.write(value);
                gen.write_byte(b';');
            }
        }
    }
}

impl<'ast, G, N> ToCode<G> for Class<'ast, N> where
    G: Generator,
    N: Name<'ast> + ToCode<G>,
{
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write_bytes(b"class");
        gen.write(&self.name);
        if let Some(ref super_class) = self.extends {
            gen.write_bytes(b" extends ");
            gen.write(super_class);
        }
        gen.write_pretty(b' ');
        gen.write_byte(b'{');
        gen.write_block(&self.body);
        gen.write_byte(b'}');
    }
}

#[cfg(test)]
mod test {
    use codegen::assert_min;

    #[test]
    fn function() {
        assert_min("function foo() {}", "function foo(){}");
        assert_min("function foo(a) {}", "function foo(a){}");
        assert_min("function foo(a, b, c) {}", "function foo(a,b,c){}");
        assert_min("function foo(bar) { return 10; }", "function foo(bar){return 10;}");
    }

    #[test]
    fn class() {
        assert_min("class Foo {}", "class Foo{}");
        assert_min("class Foo extends Bar {}", "class Foo extends Bar{}");
        assert_min("class Foo { constructor(a, b) { debug; } }", "class Foo{constructor(a,b){debug;}}");
        assert_min("class Foo { static constructor(a, b) { debug; } }", "class Foo{static constructor(a,b){debug;}}");
        assert_min("class Foo { method(a, b) { debug; } }", "class Foo{method(a,b){debug;}}");
        assert_min("class Foo { static method(a, b) { debug; } }", "class Foo{static method(a,b){debug;}}");
        assert_min("class Foo { a = 10; b = 20; }", "class Foo{a=10;b=20;}");
        assert_min("class Foo { static a = 10; b = 20; }", "class Foo{static a=10;b=20;}");
    }
}
