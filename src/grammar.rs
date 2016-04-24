#[derive(Debug, PartialEq)]
pub enum LiteralValue {
    LiteralUndefined,
    LiteralNull,
    LiteralTrue,
    LiteralFalse,
    LiteralInteger(i32),
    LiteralFloat(f64),
    LiteralString(String),
    LiteralInvalid,
}
pub use self::LiteralValue::*;

impl LiteralValue {
    pub fn float_from_string(string: String) -> Self {
        match string.parse::<f64>() {
            Ok(float) => LiteralFloat(float),
            _         => LiteralInvalid,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum VariableDeclarationKind {
    Var,
    Let,
    Const,
}

#[derive(Debug, PartialEq)]
pub enum ObjectKey {
    Static(String),
    Computed(Expression),
}

#[derive(Debug, PartialEq)]
pub struct Parameter {
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub enum OptionalBlock {
    Expression(Box<Expression>),
    Block(Vec<Statement>),
}

#[derive(Debug, PartialEq)]
pub enum BinaryOperator {
    Add,
    Substract,
    Multiply,
    Divide,
    Modulo,
    Exponent,
}

#[derive(Debug, PartialEq)]
pub enum UpdateOperator {
    Increment,
    Decrement,
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Variable(String),
    Literal(LiteralValue),
    Array(Vec<Expression>),
    Object(Vec<(ObjectKey, Expression)>),
    Member {
        object: Box<Expression>,
        property: Box<ObjectKey>,
    },
    MethodCall {
        object: Box<Expression>,
        method: Box<ObjectKey>,
        arguments: Vec<Expression>,
    },
    Binary {
        operator: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Update {
        operator: UpdateOperator,
        prefix: bool,
        argument: Box<Expression>,
    },
    ArrowFunction {
        params: Vec<Parameter>,
        body: OptionalBlock,
    },
    Function {
        name: Option<String>,
        params: Vec<Parameter>,
        body: Vec<Statement>,
    }
}

#[derive(Debug, PartialEq)]
pub struct Program {
    pub body: Vec<Statement>,
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    VariableDeclarationStatement {
        kind: VariableDeclarationKind,
        declarations: Vec<(String, Expression)>,
    },
    ExpressionStatement(Expression),
    ReturnStatement(Expression),
    FunctionStatement {
        name: String,
        params: Vec<Parameter>,
        body: Vec<Statement>,
    },
    WhileStatement {
        condition: Expression,
        body: OptionalBlock,
    }
}
