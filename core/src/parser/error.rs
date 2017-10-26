use error::Error;

use ast::{Ptr, Loc, List, Statement, Expression, Declarator, ObjectMember};
use ast::{Name, Function, Class, ClassMember};
use parser::Parser;

pub trait Handle<'ast> {
    fn handle_error(parser: &mut Parser<'ast>, err: Error) -> Self;
}

pub trait ToError {
    fn to_error() -> Self;
}

impl<'ast> ToError for Statement<'ast> {
    fn to_error() -> Self {
        Statement::Error
    }
}

impl<'ast> ToError for Expression<'ast> {
    fn to_error() -> Self {
        Expression::Error
    }
}

impl<'ast> ToError for Declarator<'ast> {
    fn to_error() -> Self {
        Declarator {
            name: Ptr::new(&Loc {
                start: 0,
                end: 0,
                item: Expression::Error
            }),
            value: None,
        }
    }
}

impl<'ast> ToError for ObjectMember<'ast> {
    fn to_error() -> Self {
        ObjectMember::Shorthand("")
    }
}

impl<'ast> ToError for ClassMember<'ast> {
    fn to_error() -> Self {
        ClassMember::Constructor {
            params: List::empty(),
            body: List::empty()
        }
    }
}

impl<'ast, N: Name<'ast>> Handle<'ast> for Function<'ast, N> {
    fn handle_error(parser: &mut Parser<'ast>, err: Error) -> Self {
        parser.errors.push(err);

        Function {
            name: N::empty(),
            params: List::empty(),
            body: List::empty(),
        }
    }
}

impl<'ast, N: Name<'ast>> Handle<'ast> for Class<'ast, N> {
    fn handle_error(parser: &mut Parser<'ast>, err: Error) -> Self {
        parser.errors.push(err);

        Class {
            name: N::empty(),
            extends: None,
            body: List::empty(),
        }
    }
}

impl<'ast, T: 'ast + ToError> Handle<'ast> for Loc<T> {
    fn handle_error(parser: &mut Parser<'ast>, err: Error) -> Self {
        parser.errors.push(err);

        parser.in_loc(ToError::to_error())
    }
}

impl<'ast, T: 'ast + Copy> Handle<'ast> for List<'ast, Loc<T>> {
    fn handle_error(parser: &mut Parser<'ast>, err: Error) -> Self {
        parser.errors.push(err);

        List::empty()
    }
}
