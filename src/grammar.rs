use std::{ str, ptr, slice };

const SMART_STRING_CAP: usize = 8;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SmartString {
    Literal {
        buf: [u8; SMART_STRING_CAP],
        len: usize,
    },
    InSitu {
        start: usize,
        end: usize,
    }
}

impl SmartString {
    #[inline]
    pub fn in_situ(start: usize, end: usize) -> Self {
        SmartString::InSitu {
            start: start,
            end: end,
        }
    }

    pub fn from_str(source: &str) -> Self {
        debug_assert!(
            source.len() <= SMART_STRING_CAP,
            "Tried to create smart string from literal that's too long!"
        );

        let mut buf = [0u8; SMART_STRING_CAP];
        let len = source.len();

        unsafe {
            ptr::copy_nonoverlapping(
                source.as_ptr(),
                buf.as_mut_ptr(),
                len
            );
        }

        SmartString::Literal {
            buf: buf,
            len: len,
        }
    }

    #[inline]
    pub fn as_str<'a>(&self, source: &'a str) -> &'a str {
        match *self {
            SmartString::Literal { ref buf, len } => unsafe {
                str::from_utf8_unchecked(
                    slice::from_raw_parts(buf.as_ptr(), len)
                )
            },
            SmartString::InSitu { start, end } => &source[start..end]
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum LiteralValue {
    LiteralUndefined,
    LiteralNull,
    LiteralTrue,
    LiteralFalse,
    LiteralInteger(u64),
    LiteralFloat(f64),
    LiteralString(SmartString),
}
pub use self::LiteralValue::*;

impl LiteralValue {
    pub fn float_from_string(string: &str) -> Self {
        match string.parse::<f64>() {
            Ok(float) => LiteralFloat(float),
            _         => panic!("Couldn't parse float from {}", string),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum MemberKey {
    Literal(SmartString),
    Computed(Expression),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Parameter {
    pub name: SmartString,
}

#[derive(Debug, PartialEq, Clone, Copy)]
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
    AddAssign,        //   …  += …
    SubstractAssign,  //   …  -= …
    ExponentAssign,   //   … **= …
    MultiplyAssign,   //   …  *= …
    DivideAssign,     //   …  /= …
    RemainderAssign,  //   …  %= …
    BSLAssign,        //   … <<= …
    BSRAssign,        //   … >>= …
    UBSRAssign,       //   … >>>= …
    BitAndAssign,     //   …  &= …
    BitXorAssign,     //   …  ^= …
    BitOrAssign,      //   …  |= …
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

            Assign           |
            AddAssign        |
            SubstractAssign  |
            ExponentAssign   |
            MultiplyAssign   |
            DivideAssign     |
            RemainderAssign  |
            BSLAssign        |
            BSRAssign        |
            UBSRAssign       |
            BitAndAssign     |
            BitXorAssign     |
            BitOrAssign      => 3,

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
            Substraction     |
            Assign           |
            AddAssign        |
            SubstractAssign  |
            ExponentAssign   |
            MultiplyAssign   |
            DivideAssign     |
            RemainderAssign  |
            BSLAssign        |
            BSRAssign        |
            UBSRAssign       |
            BitAndAssign     |
            BitXorAssign     |
            BitOrAssign      => true,

            _                => false
        }
    }

    pub fn assignment(&self) -> bool {
        match *self {
            Assign           |
            AddAssign        |
            SubstractAssign  |
            ExponentAssign   |
            MultiplyAssign   |
            DivideAssign     |
            RemainderAssign  |
            BSLAssign        |
            BSRAssign        |
            UBSRAssign       |
            BitAndAssign     |
            BitXorAssign     |
            BitOrAssign      => true,

            _                => false
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    This,
    Identifier(SmartString),
    Literal(LiteralValue),
    Array(Vec<Expression>),
    Sequence(Vec<Expression>),
    Object(Vec<ObjectMember>),
    Member {
        object: Box<Expression>,
        property: Box<MemberKey>,
    },
    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },
    Binary {
        left: Box<Expression>,
        operator: OperatorType,
        right: Box<Expression>,
    },
    Prefix {
        operator: OperatorType,
        operand: Box<Expression>,
    },
    Postfix {
        operator: OperatorType,
        operand: Box<Expression>,
    },
    Conditional {
        test: Box<Expression>,
        consequent: Box<Expression>,
        alternate: Box<Expression>,
    },
    ArrowFunction {
        params: Vec<Parameter>,
        body: Box<Statement>,
    },
    Function {
        name: Option<SmartString>,
        params: Vec<Parameter>,
        body: Vec<Statement>,
    }
}

impl Expression {
    pub fn binding_power(&self) -> u8 {
        match *self {
            Expression::Member {
                ..
            }
            |
            Expression::ArrowFunction {
                ..
            } => 18,

            Expression::Call {
                ..
            } => 17,

            Expression::Prefix {
                ref operator,
                ..
            } => operator.binding_power(true),

            Expression::Binary {
                ref operator,
                ..
            }
            |
            Expression::Postfix {
                ref operator,
                ..
            } => operator.binding_power(false),

            Expression::Conditional {
                ..
            } => 4,

            _  => 100,
        }
    }

    #[inline(always)]
    pub fn ident(name: &str) -> Self {
        Expression::Identifier(SmartString::from_str(name))
    }

    #[inline(always)]
    pub fn member(object: Expression, property: &str) -> Self {
        Expression::Member {
            object: Box::new(object),
            property: Box::new(
                MemberKey::Literal(SmartString::from_str(property))
            )
        }
    }

    #[inline(always)]
    pub fn call(callee: Expression, arguments: Vec<Expression>) -> Self {
        Expression::Call {
            callee: Box::new(callee),
            arguments: arguments,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ObjectMember {
    Shorthand {
        key: SmartString,
    },
    Literal {
        key: SmartString,
        value: Expression,
    },
    Computed {
        key: Expression,
        value: Expression,
    },
    Method {
        name: SmartString,
        params: Vec<Parameter>,
        body: Vec<Statement>,
    },
    ComputedMethod {
        name: Expression,
        params: Vec<Parameter>,
        body: Vec<Statement>,
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ClassMember {
    Constructor {
        params: Vec<Parameter>,
        body: Vec<Statement>,
    },
    Method {
        is_static: bool,
        name: SmartString,
        params: Vec<Parameter>,
        body: Vec<Statement>,
    },
    Property {
        is_static: bool,
        name: SmartString,
        value: Expression,
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum VariableDeclarationKind {
    Var,
    Let,
    Const,
}

#[derive(Debug, PartialEq, Clone)]
pub struct VariableDeclarator {
    pub name: SmartString,
    pub value: Option<Expression>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Block {
        body: Vec<Statement>,
    },
    Labeled {
        label: SmartString,
        body: Box<Statement>,
    },
    VariableDeclaration {
        kind: VariableDeclarationKind,
        declarators: Vec<VariableDeclarator>,
    },
    Expression {
        value: Expression
    },
    Return {
        value: Option<Expression>,
    },
    Break {
        label: Option<SmartString>,
    },
    Function {
        name: SmartString,
        params: Vec<Parameter>,
        body: Vec<Statement>,
    },
    If {
        test: Expression,
        consequent: Box<Statement>,
        alternate: Option<Box<Statement>>,
    },
    While {
        test: Expression,
        body: Box<Statement>,
    },
    For {
        init: Option<Box<Statement>>,
        test: Option<Expression>,
        update: Option<Expression>,
        body: Box<Statement>,
    },
    ForIn {
        left: Box<Statement>,
        right: Expression,
        body: Box<Statement>,
    },
    ForOf {
        left: Box<Statement>,
        right: Expression,
        body: Box<Statement>,
    },
    Class {
        name: SmartString,
        extends: Option<SmartString>,
        body: Vec<ClassMember>,
    },
}

#[derive(Debug, PartialEq)]
pub struct Program {
    pub body: Vec<Statement>,
}
