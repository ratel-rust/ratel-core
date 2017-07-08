use ast::{Function, Class, Name, MandatoryName, OptionalName};
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
