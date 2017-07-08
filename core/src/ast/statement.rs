use ast::{Loc, List, DeclarationKind, Function, Class, MandatoryName};
use ast::{ExpressionPtr, StatementPtr, StatementList};

#[derive(Debug, PartialEq, Clone)]
pub struct Declarator<'ast> {
    pub name: ExpressionPtr<'ast>,
    pub value: Option<ExpressionPtr<'ast>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement<'ast> {
    Error,
    Empty,
    Expression {
        expression: ExpressionPtr<'ast>
    },
    Declaration {
        kind: DeclarationKind,
        declarators: List<'ast, Loc<Declarator<'ast>>>,
    },
    Return {
        value: Option<ExpressionPtr<'ast>>,
    },
    Break {
        label: Option<ExpressionPtr<'ast>>,
    },
    Throw {
        value: ExpressionPtr<'ast>
    },
    If {
        test: ExpressionPtr<'ast>,
        consequent: StatementPtr<'ast>,
        alternate: Option<StatementPtr<'ast>>,
    },
    While {
        test: ExpressionPtr<'ast>,
        body: StatementPtr<'ast>,
    },
    Do {
        body: StatementPtr<'ast>,
        test: ExpressionPtr<'ast>,
    },
    For {
        init: Option<StatementPtr<'ast>>,
        test: Option<ExpressionPtr<'ast>>,
        update: Option<ExpressionPtr<'ast>>,
        body: StatementPtr<'ast>
    },
    ForIn {
        left: StatementPtr<'ast>,
        right: ExpressionPtr<'ast>,
        body: StatementPtr<'ast>
    },
    ForOf {
        left: StatementPtr<'ast>,
        right: ExpressionPtr<'ast>,
        body: StatementPtr<'ast>
    },
    Try {
        body: StatementList<'ast>,
        error: ExpressionPtr<'ast>,
        handler: StatementList<'ast>,
    },
    Block {
        body: StatementList<'ast>
    },
    Labeled {
        label: &'ast str,
        body: StatementPtr<'ast>,
    },
    Function {
        function: Function<'ast, MandatoryName<'ast>>,
    },
    Class {
        class: Class<'ast, MandatoryName<'ast>>,
    },
}

impl<'ast> Statement<'ast> {
    #[inline]
    pub fn at(self, start: u32, end: u32) -> Loc<Statement<'ast>> {
        Loc::new(start, end, self)
    }

    #[inline]
    pub fn is_block(&self) -> bool {
        match *self {
            Statement::Block { .. } => true,
            _                       => false,
        }
    }
}
