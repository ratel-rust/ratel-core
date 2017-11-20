use ast::{Ptr, List, DeclarationKind, Function, Class, MandatoryName, IdentifierPtr};
use ast::{ExpressionPtr, StatementPtr, StatementList, Block, BlockPtr, Pattern};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Declarator<'ast> {
    pub id: Ptr<'ast, Pattern<'ast>>,
    pub init: Option<ExpressionPtr<'ast>>,
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
    pub label: Option<IdentifierPtr<'ast>>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ContinueStatement<'ast> {
    pub label: Option<IdentifierPtr<'ast>>
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
pub struct CatchClause<'ast> {
    pub param: Ptr<'ast, Pattern<'ast>>,
    pub body: BlockPtr<'ast, Statement<'ast>>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TryStatement<'ast> {
    pub block: BlockPtr<'ast, Statement<'ast>>,
    pub handler: Option<Ptr<'ast, CatchClause<'ast>>>,
    pub finalizer: Option<BlockPtr<'ast, Statement<'ast>>>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct LabeledStatement<'ast> {
    pub label: &'ast str,
    pub body: StatementPtr<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SwitchStatement<'ast> {
    pub discriminant: ExpressionPtr<'ast>,
    pub cases: BlockPtr<'ast, SwitchCase<'ast>>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SwitchCase<'ast> {
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
    Continue(ContinueStatement<'ast>),
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
    Switch(SwitchStatement<'ast>)
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
    SwitchStatement => Switch
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
