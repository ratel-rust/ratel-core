use ast::{Function, Class, ClassMember, Name, EmptyName, MandatoryName, OptionalName, MethodKind};
use codegen::{ToCode, Generator};

impl<G: Generator> ToCode<G> for EmptyName {
    #[inline]
    fn to_code(&self, _: &mut G) {}
}

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

pub trait ClassFunctionDeclaration<G: Generator> {
    #[inline]
    fn write_class(gen: &mut G) {
        gen.write_bytes(b"class");
    }

    #[inline]
    fn write_function(gen: &mut G) {
        gen.write_bytes(b"function");
    }
}

impl<G: Generator> ClassFunctionDeclaration<G> for EmptyName {
    #[inline]
    fn write_class(_: &mut G) {}

    #[inline]
    fn write_function(_: &mut G) {}
}

impl<'ast, G: Generator> ClassFunctionDeclaration<G> for OptionalName<'ast> {}
impl<'ast, G: Generator> ClassFunctionDeclaration<G> for MandatoryName<'ast> {}

impl<'ast, G, N> ToCode<G> for Function<'ast, N> where
    G: Generator,
    N: Name<'ast> + ToCode<G> + ClassFunctionDeclaration<G>,
{
    #[inline]
    fn to_code(&self, gen: &mut G) {
        N::write_function(gen);
        gen.write(&self.name);
        gen.write_byte(b'(');
        gen.write_list(&self.params);
        gen.write_byte(b')');
        gen.write_pretty(b' ');
        gen.write(&self.body);
    }
}

impl<'ast, G: Generator> ToCode<G> for ClassMember<'ast> {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        use ast::ClassMember::*;

        match *self {
            Error => panic!("Module contains errors"),
            Method {
                is_static,
                kind,
                ref key,
                ref value,
            } => {
                if is_static {
                    gen.write_bytes(b"static ");
                }
                match kind {
                    MethodKind::Get => gen.write_bytes(b"get "),
                    MethodKind::Set => gen.write_bytes(b"set "),
                    _               => {},
                }
                gen.write(key);
                gen.write(value);
            },
            Literal {
                is_static,
                ref key,
                ref value,
            } => {
                if is_static {
                    gen.write_bytes(b"static ");
                }
                gen.write(key);
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
    N: Name<'ast> + ToCode<G> + ClassFunctionDeclaration<G>,
{
    #[inline]
    fn to_code(&self, gen: &mut G) {
        N::write_class(gen);
        gen.write(&self.name);
        if let Some(ref super_class) = self.extends {
            gen.write_bytes(b" extends ");
            gen.write(super_class);
        }
        gen.write_pretty(b' ');
        gen.write(&self.body);
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
    fn rest_and_spread() {
        assert_min("function foo(...things) { bar(...things); }", "function foo(...things){bar(...things);}");
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
