use error::Error;

use ast::{Ptr, Loc, List, Statement, Expression, Declarator, Function, Name};
use parser::Parser;
use arena::Arena;

pub trait Handle<'ast> {
    fn handle_error(parser: &mut Parser<'ast>, err: Error) -> Self;
}

pub trait ToError<'ast> {
    fn to_error(&'ast Arena) -> Self;
}

impl<'ast> ToError<'ast> for Statement<'ast> {
    fn to_error(_: &'ast Arena) -> Self {
        Statement::Error
    }
}

impl<'ast> ToError<'ast> for Expression<'ast> {
    fn to_error(_: &'ast Arena) -> Self {
        Expression::Error
    }
}

impl<'a, 'ast: 'a> ToError<'ast> for Declarator<'a> {
    fn to_error(arena: &'ast Arena) -> Self {
        Declarator {
            name: Ptr::new(arena.alloc(Loc::new(0, 0, Expression::Error))),
            value: None,
        }
    }
}

impl<'ast, N: Name<'ast>> Handle<'ast> for Function<'ast, N> {
    fn handle_error(parser: &mut Parser<'ast>, err: Error) -> Self {
        parser.errors.push(err);

        Function {
            name: N::empty(&parser.arena),
            params: List::empty(),
            body: List::empty(),
        }
    }
}

impl<'ast, T: 'ast + ToError<'ast>> Handle<'ast> for Loc<T> {
    fn handle_error(parser: &mut Parser<'ast>, err: Error) -> Self {
        parser.errors.push(err);

        parser.in_loc(ToError::to_error(parser.arena))
    }
}

impl<'ast, T: 'ast> Handle<'ast> for List<'ast, Loc<T>> {
    fn handle_error(parser: &mut Parser<'ast>, err: Error) -> Self {
        parser.errors.push(err);

        List::empty()
    }
}
