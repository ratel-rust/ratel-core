use ast::{Index, OptIndex, Item, Program};
use ast::Item::*;

use std::fmt::{Debug, Formatter, Result};

trait AstDebug<'src> {
    fn ast_fmt(&self, gen: &mut DebugGen<'src>, f: &mut Formatter) -> Result;
}


/// The `Generator` is a wrapper around an owned `String` that's used to
/// stringify the AST. There is a bunch of useful methods here to manage
/// things like indentation and automatically producing minified code.
struct DebugGen<'src> {
    program: &'src Program<'src>,
    dent: u16,
}

impl<'src> DebugGen<'src> {
    pub fn new(program: &'src Program<'src>) -> Self {
        DebugGen {
            program: program,
            dent: 0,
        }
    }

    #[inline]
    pub fn write<T>(&mut self, item: &T, f: &mut Formatter) -> Result
        where T: AstDebug<'src>
    {
        item.ast_fmt(self, f)
    }

    #[inline]
    fn write_from_index(&mut self, index: Index, f: &mut Formatter) -> Result {
        self.program[index].ast_fmt(self, f)
    }

    #[inline]
    fn write_from_optindex(&mut self, opt: OptIndex, f: &mut Formatter) -> Result {
        if !opt.is_null() {
            self.write_from_index(opt.unwrap(), f)?;
        }

        Ok(())
    }

    #[inline]
    pub fn indent(&mut self) {
        self.dent += 1;
    }

    #[inline]
    pub fn dedent(&mut self) {
        self.dent -= 1;
    }
}

impl<'src> AstDebug<'src> for Item<'src> {
    fn ast_fmt(&self, gen: &mut DebugGen<'src>, f: &mut Formatter) -> Result {
        match *self {
            _ => f.write_str("💀")?
        }

        Ok(())
    }
}

impl<'src> Debug for Program<'src> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut gen = DebugGen::new(self);

        gen.write_from_optindex(self.root, f)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use parser::parse;

    #[test]
    fn should_die() {
        let src = "break foo;";
        let program = parse(src).unwrap();

        panic!("{:?}", program);
    }
}
