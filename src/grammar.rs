#[derive(Debug, PartialEq, Clone)]
pub enum LiteralValue {
    LiteralUndefined,
    LiteralNull,
    LiteralTrue,
    LiteralFalse,
    LiteralInteger(i32),
    LiteralFloat(f64),
    LiteralString(String),
}
pub use self::LiteralValue::*;

impl LiteralValue {
    pub fn float_from_string(string: String) -> Self {
        match string.parse::<f64>() {
            Ok(float) => LiteralFloat(float),
            _         => panic!("Coudln't parse float from {}", string),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum VariableDeclarationKind {
    Var,
    Let,
    Const,
}

#[derive(Debug, PartialEq, Clone)]
pub enum MemberKey {
    Literal(String),
    Computed(Expression),
}

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
pub struct Parameter {
    pub name: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum OperatorType {
    FatArrow,         //   …  => …
    Accessor,         //   …  .  …
    New,              //     new …
    Increment,        //      ++ … | … ++
    Decrement,        //      -- … | … --
    LogicalNot,       //       ! …
    BitwiseNot,       //       ~ …
    Typeof,           //  typeof …
    Void,             //    void …
    Delete,           //  delete …
    Multiplication,   //   …  *  …
    Division,         //   …  /  …
    Remainder,        //   …  %  …
    Exponent,         //   …  ** …
    Addition,         //   …  +  … | + …
    Substraction,     //   …  -  … | - …
    BitShiftLeft,     //   …  << …
    BitShiftRight,    //   …  >> …
    UBitShiftRight,   //   … >>> …
    Lesser,           //   …  <  …
    LesserEquals,     //   …  <= …
    Greater,          //   …  >  …
    GreaterEquals,    //   …  >= …
    Instanceof,       //   … instanceof …
    In,               //   …  in …
    StrictEquality,   //   … === …
    StrictInequality, //   … !== …
    Equality,         //   …  == …
    Inequality,       //   …  != …
    BitwiseAnd,       //   …  &  …
    BitwiseXor,       //   …  ^  …
    BitwiseOr,        //   …  |  …
    LogicalAnd,       //   …  && …
    LogicalOr,        //   …  || …
    Conditional,      //   …  ?  …  :  …
    Assign,           //   …  =  …
    Spread,           //     ... …
}
use self::OperatorType::*;

impl OperatorType {
    /// According to the Operator Precedence Table
    pub fn binding_power(&self, prefix: bool) -> u8 {
        match *self {
            FatArrow         |
            Accessor         => 18,

            New              => 17,

            Increment        |
            Decrement        => if prefix { 15 } else { 16 },

            LogicalNot       |
            BitwiseNot       |
            Typeof           |
            Void             |
            Delete           => 15,

            Multiplication   |
            Division         |
            Remainder        |
            Exponent         => 14,

            Addition         |
            Substraction     => if prefix { 15 } else { 13 },

            BitShiftLeft     |
            BitShiftRight    |
            UBitShiftRight   => 12,

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

            BitwiseAnd       => 9,
            BitwiseXor       => 8,
            BitwiseOr        => 7,
            LogicalAnd       => 6,
            LogicalOr        => 5,
            Conditional      => 4,
            Assign           => 3,
            Spread           => 1,
        }
    }

    pub fn prefix(&self) -> bool {
        match *self {
            LogicalNot       |
            BitwiseNot       |
            Typeof           |
            Void             |
            Delete           |
            New              |
            Spread           |
            Increment        |
            Decrement        |
            Addition         |
            Substraction     => true,

            _                => false
        }
    }

    pub fn infix(&self) -> bool {
        match *self {
            FatArrow         |
            Assign           |
            Accessor         |
            Multiplication   |
            Division         |
            Remainder        |
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
            BitShiftLeft     |
            BitShiftRight    |
            UBitShiftRight   |
            BitwiseAnd       |
            BitwiseXor       |
            BitwiseOr        |
            LogicalAnd       |
            LogicalOr        |
            Conditional      |
            Addition         |
            Substraction     => true,

            _                => false
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
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
    ConditionalExpression {
        test: Box<Expression>,
        consequent: Box<Expression>,
        alternate: Box<Expression>,
    },
    ArrowFunctionExpression {
        params: Vec<Parameter>,
        body: Box<Statement>,
    },
    FunctionExpression {
        name: Option<String>,
        params: Vec<Parameter>,
        body: Vec<Statement>,
    }
}

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
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
    IfStatement {
        test: Expression,
        consequent: Box<Statement>,
        alternate: Option<Box<Statement>>,
    },
    WhileStatement {
        test: Expression,
        body: Box<Statement>,
    },
    BlockStatement {
        body: Vec<Statement>,
    },
}

#[derive(Debug, PartialEq)]
pub struct Program {
    pub body: Vec<Statement>,
}
