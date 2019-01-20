use ast::{Node, NodeList, DeclarationKind, Function, Class, MandatoryName, IdentifierNode};
use ast::{ExpressionNode, StatementNode, StatementList, Block, BlockNode, Pattern};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Declarator<'ast> {
    pub id: Node<'ast, Pattern<'ast>>,
    pub init: Option<ExpressionNode<'ast>>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DeclarationStatement<'ast> {
    pub kind: DeclarationKind,
    pub declarators: NodeList<'ast, Declarator<'ast>>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ReturnStatement<'ast> {
    pub value: Option<ExpressionNode<'ast>>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct BreakStatement<'ast> {
    // TODO: This should be a `LabelNode`, with `Label` being a newtype for &str.
    pub label: Option<IdentifierNode<'ast>>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ContinueStatement<'ast> {
    // TODO: This should be a `LabelNode`, with `Label` being a newtype for &str.
    pub label: Option<IdentifierNode<'ast>>
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ThrowStatement<'ast> {
    pub value: ExpressionNode<'ast>
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct IfStatement<'ast> {
    pub test: ExpressionNode<'ast>,
    pub consequent: StatementNode<'ast>,
    pub alternate: Option<StatementNode<'ast>>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct WhileStatement<'ast> {
    pub test: ExpressionNode<'ast>,
    pub body: StatementNode<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DoStatement<'ast> {
    pub body: StatementNode<'ast>,
    pub test: ExpressionNode<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ForInit<'ast> {
    Declaration(DeclarationStatement<'ast>),
    Expression(ExpressionNode<'ast>)
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ForStatement<'ast> {
    pub init: Option<Node<'ast, ForInit<'ast>>>,
    pub test: Option<ExpressionNode<'ast>>,
    pub update: Option<ExpressionNode<'ast>>,
    pub body: StatementNode<'ast>
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ForInStatement<'ast> {
    pub left: Node<'ast, ForInit<'ast>>,
    pub right: ExpressionNode<'ast>,
    pub body: StatementNode<'ast>
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ForOfStatement<'ast> {
    pub left: Node<'ast, ForInit<'ast>>,
    pub right: ExpressionNode<'ast>,
    pub body: StatementNode<'ast>
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct CatchClause<'ast> {
    pub param: Node<'ast, Pattern<'ast>>,
    pub body: BlockNode<'ast, Statement<'ast>>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TryStatement<'ast> {
    pub block: BlockNode<'ast, Statement<'ast>>,
    pub handler: Option<Node<'ast, CatchClause<'ast>>>,
    pub finalizer: Option<BlockNode<'ast, Statement<'ast>>>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct LabeledStatement<'ast> {
    pub label: &'ast str,
    pub body: StatementNode<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SwitchStatement<'ast> {
    pub discriminant: ExpressionNode<'ast>,
    pub cases: BlockNode<'ast, SwitchCase<'ast>>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SwitchCase<'ast> {
    pub test: Option<ExpressionNode<'ast>>,
    pub consequent: StatementList<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ImportDeclaration<'ast> {
    pub specifiers: NodeList<'ast, ForImportSpecifier<'ast>>,
    pub source: &'ast str,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ImportSpecifier<'ast> {
    pub imported: IdentifierNode<'ast>,
    pub local: IdentifierNode<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ImportDefaultSpecifier<'ast> {
    pub local: IdentifierNode<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ImportNamespaceSpecifier<'ast> {
    pub local: IdentifierNode<'ast>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ForImportSpecifier<'ast> {
    ImportSpecifier(ImportSpecifier<'ast>),
    ImportDefaultSpecifier(ImportDefaultSpecifier<'ast>),
    ImportNamespaceSpecifier(ImportNamespaceSpecifier<'ast>),
}



pub type BlockStatement<'ast> = Block<'ast, Statement<'ast>>;
pub type FunctionStatement<'ast> = Function<'ast, MandatoryName<'ast>>;
pub type ClassStatement<'ast> = Class<'ast, MandatoryName<'ast>>;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Statement<'ast> {
    Empty,
    Expression(ExpressionNode<'ast>),
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
    Switch(SwitchStatement<'ast>),
    Import(ImportDeclaration<'ast>)
}

macro_rules! impl_from {
    ($( $type:ident => $variant:ident ),*) => ($(
        impl<'ast> From<$type<'ast>> for Statement<'ast> {
            #[inline]
            fn from(val: $type<'ast>) -> Self {
                Statement::$variant(val)
            }
        }
    )*)
}

impl_from! {
    ExpressionNode => Expression,
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
    ImportDeclaration => Import
}

impl<'ast> From<DeclarationStatement<'ast>> for ForInit<'ast> {
    #[inline]
    fn from(val: DeclarationStatement<'ast>) -> Self {
        ForInit::Declaration(val)
    }
}

impl<'ast> From<ExpressionNode<'ast>> for ForInit<'ast> {
    #[inline]
    fn from(val: ExpressionNode<'ast>) -> Self {
        ForInit::Expression(val)
    }
}

impl<'ast> From<ImportSpecifier<'ast>> for ForImportSpecifier<'ast> {
    #[inline]
    fn from(val: ImportSpecifier<'ast>) -> Self {
        ForImportSpecifier::ImportSpecifier(val)
    }
}
impl<'ast> From<ImportDefaultSpecifier<'ast>> for ForImportSpecifier<'ast> {
    #[inline]
    fn from(val: ImportDefaultSpecifier<'ast>) -> Self {
        ForImportSpecifier::ImportDefaultSpecifier(val)
    }
}
impl<'ast> From<ImportNamespaceSpecifier<'ast>> for ForImportSpecifier<'ast> {
    #[inline]
    fn from(val: ImportNamespaceSpecifier<'ast>) -> Self {
        ForImportSpecifier::ImportNamespaceSpecifier(val)
    }
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
