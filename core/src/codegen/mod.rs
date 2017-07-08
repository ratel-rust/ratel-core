use ast::{Ptr, Loc, Statement, StatementPtr};
use parser::Module;

mod expression;
mod statement;
mod function;

pub trait Generator: Sized {
    type Output;

    fn consume(self) -> Self::Output;
    fn write_byte(&mut self, u8);
    fn write_bytes(&mut self, &[u8]);
    fn write_pretty(&mut self, u8);

    #[inline]
    fn write<T>(&mut self, item: &T) where
        T: ToCode<Self>,
    {
        item.to_code(self);
    }

    #[inline]
    fn write_list<T, I>(&mut self, items: I) where
        T: ToCode<Self>,
        I: IntoIterator<Item = T>,
    {
        let mut items = items.into_iter();

        for item in items.next() {
            self.write(&item);
        }

        for item in items {
            self.write_byte(b',');
            self.write_pretty(b' ');
            self.write(&item);
        }
    }

    #[inline]
    fn write_block<T, I>(&mut self, items: I) where
        T: ToCode<Self>,
        I: IntoIterator<Item = T>,
    {
        let mut items = items.into_iter();

        match items.next() {
            Some(item) => {
                self.indent();
                self.new_line();
                self.write(&item);
            },
            None => return,
        }

        for item in items {
            self.new_line();
            self.write(&item);
        }
        self.dedent();
        self.new_line();
    }

    fn write_declaration_or_expression(&mut self, statement: &StatementPtr) {
        match ***statement {
            Statement::Declaration {
                ref kind,
                ref declarators,
            } => {
                self.write(kind);
                self.write_byte(b' ');
                self.write_list(declarators);
            },

            Statement::Expression {
                ref expression,
            } => {
                self.write(expression);
            },

            _ => panic!("Invalid AST structure!"),
        }
    }

    #[inline]
    fn new_line(&mut self) {}

    #[inline]
    fn indent(&mut self) {}

    #[inline]
    fn dedent(&mut self) {}
}

pub struct MinifyingGenerator {
    code: Vec<u8>
}

impl MinifyingGenerator {
    fn new() -> Self {
        MinifyingGenerator {
            code: Vec::with_capacity(128)
        }
    }
}

impl Generator for MinifyingGenerator {
    type Output = String;

    fn consume(self) -> String {
        unsafe { String::from_utf8_unchecked(self.code) }
    }

    #[inline]
    fn write_byte(&mut self, ch: u8) {
        self.code.push(ch);
    }

    #[inline]
    fn write_pretty(&mut self, _: u8) {}

    #[inline]
    fn write_bytes(&mut self, slice: &[u8]) {
        extend_from_slice(&mut self.code, slice);
    }
}

struct PrettyGenerator {
    code: Vec<u8>,
    dent: usize,
}

impl PrettyGenerator {
    fn new() -> Self {
        PrettyGenerator {
            code: Vec::with_capacity(128),
            dent: 0,
        }
    }
}

impl Generator for PrettyGenerator {
    type Output = String;

    fn consume(self) -> String {
        unsafe { String::from_utf8_unchecked(self.code) }
    }

    #[inline]
    fn write_byte(&mut self, ch: u8) {
        self.code.push(ch);
    }

    #[inline]
    fn write_pretty(&mut self, ch: u8) {
        self.code.push(ch);
    }

    #[inline]
    fn write_bytes(&mut self, slice: &[u8]) {
        extend_from_slice(&mut self.code, slice);
    }

    #[inline]
    fn new_line(&mut self) {
        self.write_byte(b'\n');
        for _ in 0..self.dent {
            self.write_bytes(b"    ");
        }
    }

    #[inline]
    fn indent(&mut self) {
        self.dent += 1;
    }

    #[inline]
    fn dedent(&mut self) {
        self.dent -= 1;
    }
}

pub fn codegen<'ast>(module: &Module, minify: bool) -> String {
    if minify {
        let mut gen = MinifyingGenerator::new();

        for statement in module.body() {
            gen.write(&statement);
            gen.new_line();
        }

        gen.consume()
    } else {
        let mut gen = PrettyGenerator::new();

        for statement in module.body() {
            gen.write(&statement);
            gen.new_line();
        }

        gen.consume()
    }
}

/// The `ToCode` trait provides an interface to pieces of grammar, that allows
/// to efficiently write characters and string slices to the code `Generator`.
pub trait ToCode<G: Generator> {
    fn to_code(&self, gen: &mut G);
}

impl<'ast, G, T> ToCode<G> for Ptr<'ast, T> where
    G: Generator,
    T: 'ast + ToCode<G>,
{
    #[inline]
    fn to_code(&self, gen: &mut G) {
        (**self).to_code(gen)
    }
}

impl<G, T> ToCode<G> for Loc<T> where
    G: Generator,
    T: ToCode<G>,
{
    #[inline]
    fn to_code(&self, gen: &mut G) {
        self.item.to_code(gen)
    }
}

impl<'a, G, T> ToCode<G> for &'a Loc<T> where
    G: Generator,
    T: ToCode<G>,
{
    #[inline]
    fn to_code(&self, gen: &mut G) {
        self.item.to_code(gen)
    }
}

impl<'a, G> ToCode<G> for &'a str where
    G: Generator,
{
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write_bytes(self.as_bytes())
    }
}

impl<G: Generator> ToCode<G> for u64 {
    #[inline]
    fn to_code(&self, gen: &mut G) {
        gen.write_bytes(format!("{}", self).as_str().as_bytes())
    }
}

// From: https://github.com/dtolnay/fastwrite/blob/master/src/lib.rs#L68
//
// LLVM is not able to lower `Vec::extend_from_slice` into a memcpy, so this
// helps eke out that last bit of performance.
#[inline]
fn extend_from_slice(dst: &mut Vec<u8>, src: &[u8]) {
    let dst_len = dst.len();
    let src_len = src.len();

    dst.reserve(src_len);

    unsafe {
        // We would have failed if `reserve` overflowed
        dst.set_len(dst_len + src_len);

        ::std::ptr::copy_nonoverlapping(
            src.as_ptr(),
            dst.as_mut_ptr().offset(dst_len as isize),
            src_len);
    }
}

#[cfg(test)]
fn assert_parse(source: &str, expected: &str) {
    use parser::parse;

    let module = parse(source).unwrap();

    assert_eq!(codegen(&module, true).as_str(), expected);
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use parser::parse;

//     #[test]
//     fn should_die() {
//         let src = "break foo;";
//         let program = parse(src).unwrap();

//         let code = codegen(&program, true);

//         assert_eq!(code, "break foo;");
//     }
// }
