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
pub enum OperatorType {
    LogicalNot,       //      ! ...
    BitwiseNot,       //      ~ ...
    Typeof,           // typeof ...
    Void,             //   void ...
    Delete,           // delete ...
    New,              //    new ...
    Increment,        // ++ ... | ... ++
    Decrement,        // -- ... | ... --
    Assign,           // ...  =  ...
    Accessor,         // ...  .  ...
    Multiply,         // ...  *  ...
    Divide,           // ...  /  ...
    Modulo,           // ...  %  ...
    Exponent,         // ...  ** ...
    StrictEquality,   // ... === ...
    StrictInequality, // ... !== ...
    Equality,         // ...  == ...
    Inequality,       // ...  != ...
    Lesser,           // ...  <  ...
    LesserEquals,     // ...  <= ...
    Greater,          // ...  >  ...
    GreaterEquals,    // ...  >= ...
    Instanceof,       // ... instanceof ...
    In,               // ... in ...
    Add,              // + ... | ... + ...
    Substract,        // - ... | ... - ...
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Identifier(String),
    Literal(LiteralValue),
    ArrayExpression(Vec<Expression>),
    ObjectExpression(Vec<(ObjectKey, Expression)>),
    MemberExpression {
        object: Box<Expression>,
        property: Box<ObjectKey>,
    },
    CallExpression {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },
    BinaryExpression {
        operator: OperatorType,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    PrefixExpression {
        operator: OperatorType,
        argument: Box<Expression>,
    },
    PostfixExpression {
        operator: OperatorType,
        argument: Box<Expression>,
    },
    ArrowFunctionExpression {
        params: Vec<Parameter>,
        body: OptionalBlock,
    },
    FunctionExpression {
        name: Option<String>,
        params: Vec<Parameter>,
        body: Vec<Statement>,
    }
}

#[derive(Debug, PartialEq)]
pub enum ClassMember {
    ClassConstructor {
        params: Vec<Parameter>,
        body: Vec<Statement>,
    },
    ClassMethod {
        is_static: bool,
        name: String,
        params: Vec<Parameter>,
        body: Vec<Statement>,
    },
    ClassProperty {
        is_static: bool,
        name: String,
        value: Expression,
    }
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
    ClassStatement {
        name: String,
        extends: Option<String>,
        body: Vec<ClassMember>,
    },
    WhileStatement {
        condition: Expression,
        body: OptionalBlock,
    }
}

#[derive(Debug, PartialEq)]
pub struct Program {
    pub body: Vec<Statement>,
}
