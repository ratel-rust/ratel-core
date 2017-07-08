use ast::{Value, OperatorKind, DeclarationKind};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ReservedKind {
    Enum,
    Implements,
    Package,
    Protected,
    Interface,
    Private,
    Public,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TemplateKind<'src> {
    Open(&'src str),
    Closed(&'src str),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Token<'src> {
    EndOfProgram,
    Semicolon,
    Colon,
    Comma,
    ParenOpen,
    ParenClose,
    BracketOpen,
    BracketClose,
    BraceOpen,
    BraceClose,
    Operator(OperatorKind),
    Declaration(DeclarationKind),
    Break,
    Do,
    Case,
    Else,
    Catch,
    Export,
    Class,
    Extends,
    Return,
    While,
    Finally,
    Super,
    With,
    Continue,
    For,
    Switch,
    Yield,
    Debugger,
    Function,
    This,
    Default,
    If,
    Throw,
    Import,
    Try,
    Static,
    Literal(Value<'src>),
    Reserved(ReservedKind),
    Identifier(&'src str),
    Accessor(&'src str),
    Template(TemplateKind<'src>),
    UnexpectedToken,
    UnexpectedEndOfProgram,
}
