use ast::{Loc, List, DeclarationKind, Function, Class, MandatoryName};
use ast::{ExpressionPtr, StatementPtr, StatementList, Block, BlockPtr};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Declarator<'ast> {
    pub name: DeclaratorId<'ast>,
    pub value: Option<ExpressionPtr<'ast>>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DeclaratorId<'ast> {
    Identifier(&'ast str),
    Pattern(ExpressionPtr<'ast>)
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DeclarationStatement<'ast> {
    pub kind: DeclarationKind,
    pub declarators: List<'ast, Declarator<'ast>>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ReturnStatement<'ast> {
    pub value: Option<ExpressionPtr<'ast>>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct BreakStatement<'ast> {
    pub label: Option<ExpressionPtr<'ast>>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ThrowStatement<'ast> {
    pub value: ExpressionPtr<'ast>
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct IfStatement<'ast> {
    pub test: ExpressionPtr<'ast>,
    pub consequent: StatementPtr<'ast>,
    pub alternate: Option<StatementPtr<'ast>>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct WhileStatement<'ast> {
    pub test: ExpressionPtr<'ast>,
    pub body: StatementPtr<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DoStatement<'ast> {
    pub body: StatementPtr<'ast>,
    pub test: ExpressionPtr<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ForStatement<'ast> {
    pub init: Option<StatementPtr<'ast>>,
    pub test: Option<ExpressionPtr<'ast>>,
    pub update: Option<ExpressionPtr<'ast>>,
    pub body: StatementPtr<'ast>
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ForInStatement<'ast> {
    pub left: StatementPtr<'ast>,
    pub right: ExpressionPtr<'ast>,
    pub body: StatementPtr<'ast>
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ForOfStatement<'ast> {
    pub left: StatementPtr<'ast>,
    pub right: ExpressionPtr<'ast>,
    pub body: StatementPtr<'ast>
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TryStatement<'ast> {
    pub body: BlockPtr<'ast, Statement<'ast>>,
    pub error: ExpressionPtr<'ast>,
    pub handler: BlockPtr<'ast, Statement<'ast>>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct LabeledStatement<'ast> {
    pub label: &'ast str,
    pub body: StatementPtr<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ContinueStatement<'ast> {
    pub label: Option<ExpressionPtr<'ast>>
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SwitchStatement<'ast> {
    pub discriminant: ExpressionPtr<'ast>,
    pub cases: StatementList<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SwitchCaseStatement<'ast> {
    pub test: Option<ExpressionPtr<'ast>>,
    pub consequent: StatementList<'ast>,
}

pub type BlockStatement<'ast> = Block<'ast, Statement<'ast>>;
pub type FunctionStatement<'ast> = Function<'ast, MandatoryName<'ast>>;
pub type ClassStatement<'ast> = Class<'ast, MandatoryName<'ast>>;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Statement<'ast> {
    Error,
    Empty,
    Expression(ExpressionPtr<'ast>),
    Declaration(DeclarationStatement<'ast>),
    Return(ReturnStatement<'ast>),
    Break(BreakStatement<'ast>),
    Throw(ThrowStatement<'ast>),
    If(IfStatement<'ast>),
    While(WhileStatement<'ast>),
    Do(DoStatement<'ast>),
    For(ForStatement<'ast>),
    ForIn(ForInStatement<'ast>),
    ForOf(ForOfStatement<'ast>),
    Try(TryStatement<'ast>),
    Block(BlockStatement<'ast>),
    Labeled(LabeledStatement<'ast>),
    Function(FunctionStatement<'ast>),
    Class(ClassStatement<'ast>),
    Continue(ContinueStatement<'ast>),
    Switch(SwitchStatement<'ast>),
    SwitchCase(SwitchCaseStatement<'ast>)
}

macro_rules! impl_from {
    ($( $type:ident => $variant:ident ),*) => ($(
        impl<'ast> From<$type<'ast>> for Statement<'ast> {
            #[inline]
            fn from(val: $type<'ast>) -> Statement<'ast> {
                Statement::$variant(val)
            }
        }
    )*)
}

impl_from! {
    ExpressionPtr => Expression,
    DeclarationStatement => Declaration,
    ReturnStatement => Return,
    BreakStatement => Break,
    ThrowStatement => Throw,
    IfStatement => If,
    WhileStatement => While,
    DoStatement => Do,
    ForStatement => For,
    ForInStatement => ForIn,
    ForOfStatement => ForOf,
    TryStatement => Try,
    BlockStatement => Block,
    LabeledStatement => Labeled,
    ContinueStatement => Continue,
    FunctionStatement => Function,
    ClassStatement => Class,
    SwitchStatement => Switch,
    SwitchCaseStatement => SwitchCase
}

impl<'ast> Statement<'ast> {
    #[inline]
    pub fn is_block(&self) -> bool {
        match *self {
            Statement::Block(_) => true,
            _                   => false,
        }
    }
}
