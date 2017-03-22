use ast::Program;
use std::fmt;

impl<'src> fmt::Debug for Program<'src> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {


        f.write_str("program!");
    }
}
