use grammar::OwnedSlice;
use grammar::LiteralValue;
use grammar::OperatorType;
use grammar::VariableDeclarationKind;

#[derive(Debug, PartialEq)]
pub enum ReservedKind {
    Enum,
    Implements,
    Package,
    Protected,
    Interface,
    Private,
    Public,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    LineTermination,
    Semicolon,
    Comma,
    Colon,
    Operator(OperatorType),
    Declaration(VariableDeclarationKind),
    ParenOn,
    ParenOff,
    BracketOn,
    BracketOff,
    BraceOn,
    BraceOff,
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
    Await,
    Static,
    Reserved(ReservedKind),
    Identifier(OwnedSlice),
    Literal(LiteralValue),
}
