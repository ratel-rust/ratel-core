use transformer::{Transformer, Transformable};
use ast::Statement;

impl<'ast> Transformable<'ast> for Statement<'ast> {
    fn transform(&self, t: &Transformer) {
        use self::Statement::*;

        match *self {
            Error => panic!("Module contains errors"),
            Empty => {},
            Expression {
                ref expression
            } => {
                unimplemented!();
            },
            Declaration {
                ref kind,
                ref declarators,
            } => {
                unimplemented!();
            },
            Return {
                ref value,
            } => {
                unimplemented!();
            },
            Break {
                ref label,
            } => {
                unimplemented!();
            },
            Throw {
                ref value,
            } => {
                unimplemented!();
            },
            If {
                ref test,
                ref consequent,
                ref alternate,
            } => {
                unimplemented!();
            },
            While {
                ref test,
                ref body,
            } => {
                unimplemented!();
            },
            Do {
                ref test,
                ref body,
            } => {
                unimplemented!();
            },
            For {
                ref init,
                ref test,
                ref update,
                ref body,
            } => {
                unimplemented!();
            },
            ForIn {
                ref left,
                ref right,
                ref body,
            } => {
                unimplemented!();
            },
            ForOf {
                ref left,
                ref right,
                ref body,
            } => {
                unimplemented!();
            },
            Try {
                ref body,
                ref error,
                ref handler,
            } => {
                unimplemented!();
            },
            Labeled {
                ref label,
                ref body,
            } => {
                unimplemented!();
            },
            Block {
                ref body,
            } => {
                body.transform(t);
            },
            Function {
                ref function,
            } => {
                unimplemented!();
            },
            Class {
                ref class,
            } => {
                unimplemented!();
            }
        }
    }
}
