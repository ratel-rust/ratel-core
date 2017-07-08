use ast::{Function, Class, ClassMember, Name, MandatoryName, OptionalName};
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
