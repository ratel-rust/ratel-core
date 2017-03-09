use error::Error;

use ast::{Index, OptIndex, Node, Item};
use parser::Parser;

pub trait Handle<'src> {
    fn handle_error(parser: &mut Parser<'src>, err: Error) -> Self;
}

impl<'src> Handle<'src> for Node<'src> {
    fn handle_error(parser: &mut Parser<'src>, err: Error) -> Self {
        parser.program.errors += 1;

        parser.in_loc(Item::Error(err))
    }
}

impl<'src> Handle<'src> for Index {
    fn handle_error(parser: &mut Parser<'src>, err: Error) -> Self {
        parser.program.errors += 1;

        let node = parser.in_loc(Item::Error(err));

        parser.store(node)
    }
}

impl<'src> Handle<'src> for OptIndex {
    fn handle_error(parser: &mut Parser<'src>, err: Error) -> Self {
        parser.program.errors += 1;

        let node = parser.in_loc(Item::Error(err));

        parser.store(node).into()
    }
}
