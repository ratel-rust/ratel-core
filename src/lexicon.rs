use grammar::OwnedSlice;
use grammar::LiteralValue;
use grammar::OperatorType;
use grammar::VariableDeclarationKind;

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
pub enum Token {
    EndOfProgram,
    Control(u8), // One of the control bytes: ( ) [ ] { } ; : ,
    Operator(OperatorType),
    Declaration(VariableDeclarationKind),
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
    Reserved(ReservedKind),
    Identifier(OwnedSlice),
    Literal(LiteralValue),
}
