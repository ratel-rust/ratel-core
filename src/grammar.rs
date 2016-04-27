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
pub enum MemberKey {
    Literal(String),
    Computed(Expression),
}

#[derive(Debug, PartialEq)]
pub enum ObjectMember {
    Shorthand {
        key: String,
    },
    Literal {
        key: String,
        value: Expression,
    },
    Computed {
        key: Expression,
        value: Expression,
    },
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
use self::OperatorType::*;

impl OperatorType {
    /// According to the Operator Precedence Table
    pub fn binding_power(&self, prefix: bool) -> u8 {
        match *self {
            Accessor         => 18,

            New              => 17,

            Increment        |
            Decrement        => if prefix { 15 } else { 16 },

            LogicalNot       |
            BitwiseNot       |
            Typeof           |
            Void             |
            Delete           => 15,

            Multiply         |
            Divide           |
            Modulo           |
            Exponent         => 14,

            Add              |
            Substract        => if prefix { 15 } else { 13 },

            Lesser           |
            LesserEquals     |
            Greater          |
            GreaterEquals    |
            Instanceof       |
            In               => 11,

            StrictEquality   |
            StrictInequality |
            Equality         |
            Inequality       => 10,

            Assign           => 3,

            // _                => 0,
        }
    }

    pub fn prefix(&self) -> bool {
        match *self {
            LogicalNot |
            BitwiseNot |
            Typeof     |
            Void       |
            Delete     |
            New        |
            Increment  |
            Decrement  |
            Add        |
            Substract  => true,

            _          => false
        }
    }

    pub fn infix(&self) -> bool {
        match *self {
            Assign           |
            Accessor         |
            Multiply         |
            Divide           |
            Modulo           |
            Exponent         |
            StrictEquality   |
            StrictInequality |
            Equality         |
            Inequality       |
            Lesser           |
            LesserEquals     |
            Greater          |
            GreaterEquals    |
            Instanceof       |
            In               |
            Add              |
            Substract        => true,

            _                => false
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    IdentifierExpression(String),
    LiteralExpression(LiteralValue),
    ArrayExpression(Vec<Expression>),
    ObjectExpression(Vec<ObjectMember>),
    MemberExpression {
        object: Box<Expression>,
        property: Box<MemberKey>,
    },
    CallExpression {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },
    BinaryExpression {
        left: Box<Expression>,
        operator: OperatorType,
        right: Box<Expression>,
    },
    PrefixExpression {
        operator: OperatorType,
        operand: Box<Expression>,
    },
    PostfixExpression {
        operator: OperatorType,
        operand: Box<Expression>,
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
