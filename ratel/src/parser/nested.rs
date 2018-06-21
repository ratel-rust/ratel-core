use toolshed::list::ListBuilder;
use parser::Parser;
use lexer::Token;
use lexer::Token::*;
use ast::{NodeList, Expression, ExpressionNode};
use ast::expression::*;
use ast::OperatorKind::*;
use lexer::Asi;

type NestedHandler = Option<for<'ast> fn(&mut Parser<'ast>, ExpressionNode<'ast>) -> ExpressionNode<'ast>>;

pub trait BindingPower {
    #[inline]
    fn handler(asi: Asi, token: Token) -> NestedHandler;
}

/// All potential tokens, including Comma for sequence expressions
pub struct ANY;

impl BindingPower for ANY {
    #[inline]
    fn handler(_asi: Asi, token: Token) -> NestedHandler {
        match token {
            Comma => SEQ,
            ParenOpen => CALL,
            BracketOpen => CMEM,
            OperatorFatArrow => ARRW,
            OperatorIncrement => INC,
            OperatorDecrement => DEC,
            OperatorMultiplication => MUL,
            OperatorDivision => DIV,
            OperatorRemainder => REM,
            OperatorExponent => EXPN,
            OperatorAddition => ADD,
            OperatorSubtraction => SUB,
            OperatorBitShiftLeft => BSL,
            OperatorBitShiftRight => BSR,
            OperatorUBitShiftRight => UBSR,
            OperatorLesser => LESS,
            OperatorLesserEquals => LSEQ,
            OperatorGreater => GRTR,
            OperatorGreaterEquals => GREQ,
            OperatorInstanceof => INOF,
            OperatorIn => IN,
            OperatorStrictEquality => STEQ,
            OperatorStrictInequality => SIEQ,
            OperatorEquality => EQ,
            OperatorInequality => INEQ,
            OperatorBitwiseAnd => BWAN,
            OperatorBitwiseXor => BWXO,
            OperatorBitwiseOr => BWOR,
            OperatorLogicalAnd => AND,
            OperatorLogicalOr => OR,
            OperatorConditional => COND,
            OperatorAssign => ASGN,
            OperatorAddAssign => ADDA,
            OperatorSubtractAssign => SUBA,
            OperatorExponentAssign => EXPA,
            OperatorMultiplyAssign => MULA,
            OperatorDivideAssign => DIVA,
            OperatorRemainderAssign => REMA,
            OperatorBSLAssign => BSLA,
            OperatorBSRAssign => BSRA,
            OperatorUBSRAssign => UBSA,
            OperatorBitAndAssign => BWAA,
            OperatorBitXorAssign => XORA,
            OperatorBitOrAssign => BORA,
            Accessor => ACCS,
            TemplateOpen => TPLE,
            TemplateClosed => TPLS,

            _ => None,
        }
    }
}

pub struct B0;

impl BindingPower for B0 {
    #[inline]
    fn handler(_asi: Asi, token: Token) -> NestedHandler {
        match token {
            ParenOpen => CALL,
            BracketOpen => CMEM,
            OperatorFatArrow => ARRW,
            OperatorIncrement => INC,
            OperatorDecrement => DEC,
            OperatorMultiplication => MUL,
            OperatorDivision => DIV,
            OperatorRemainder => REM,
            OperatorExponent => EXPN,
            OperatorAddition => ADD,
            OperatorSubtraction => SUB,
            OperatorBitShiftLeft => BSL,
            OperatorBitShiftRight => BSR,
            OperatorUBitShiftRight => UBSR,
            OperatorLesser => LESS,
            OperatorLesserEquals => LSEQ,
            OperatorGreater => GRTR,
            OperatorGreaterEquals => GREQ,
            OperatorInstanceof => INOF,
            OperatorIn => IN,
            OperatorStrictEquality => STEQ,
            OperatorStrictInequality => SIEQ,
            OperatorEquality => EQ,
            OperatorInequality => INEQ,
            OperatorBitwiseAnd => BWAN,
            OperatorBitwiseXor => BWXO,
            OperatorBitwiseOr => BWOR,
            OperatorLogicalAnd => AND,
            OperatorLogicalOr => OR,
            OperatorConditional => COND,
            OperatorAssign => ASGN,
            OperatorAddAssign => ADDA,
            OperatorSubtractAssign => SUBA,
            OperatorExponentAssign => EXPA,
            OperatorMultiplyAssign => MULA,
            OperatorDivideAssign => DIVA,
            OperatorRemainderAssign => REMA,
            OperatorBSLAssign => BSLA,
            OperatorBSRAssign => BSRA,
            OperatorUBSRAssign => UBSA,
            OperatorBitAndAssign => BWAA,
            OperatorBitXorAssign => XORA,
            OperatorBitOrAssign => BORA,
            Accessor => ACCS,
            TemplateOpen => TPLE,
            TemplateClosed => TPLS,

            _ => None,
        }
    }
}

pub struct B1;

impl BindingPower for B1 {
    #[inline]
    fn handler(_asi: Asi, token: Token) -> NestedHandler {
        match token {
            ParenOpen => CALL,
            BracketOpen => CMEM,
            OperatorFatArrow => ARRW,
            OperatorIncrement => INC,
            OperatorDecrement => DEC,
            OperatorMultiplication => MUL,
            OperatorDivision => DIV,
            OperatorRemainder => REM,
            OperatorExponent => EXPN,
            OperatorAddition => ADD,
            OperatorSubtraction => SUB,
            OperatorBitShiftLeft => BSL,
            OperatorBitShiftRight => BSR,
            OperatorUBitShiftRight => UBSR,
            OperatorLesser => LESS,
            OperatorLesserEquals => LSEQ,
            OperatorGreater => GRTR,
            OperatorGreaterEquals => GREQ,
            OperatorInstanceof => INOF,
            OperatorIn => IN,
            OperatorStrictEquality => STEQ,
            OperatorStrictInequality => SIEQ,
            OperatorEquality => EQ,
            OperatorInequality => INEQ,
            OperatorBitwiseAnd => BWAN,
            OperatorBitwiseXor => BWXO,
            OperatorBitwiseOr => BWOR,
            OperatorLogicalAnd => AND,
            OperatorLogicalOr => OR,
            OperatorConditional => COND,
            Accessor => ACCS,
            TemplateOpen => TPLE,
            TemplateClosed => TPLS,

            _ => None,
        }
    }
}

pub struct B4;

impl BindingPower for B4 {
    #[inline]
    fn handler(_asi: Asi, token: Token) -> NestedHandler {
        match token {
            ParenOpen => CALL,
            BracketOpen => CMEM,
            OperatorFatArrow => ARRW,
            OperatorIncrement => INC,
            OperatorDecrement => DEC,
            OperatorMultiplication => MUL,
            OperatorDivision => DIV,
            OperatorRemainder => REM,
            OperatorExponent => EXPN,
            OperatorAddition => ADD,
            OperatorSubtraction => SUB,
            OperatorBitShiftLeft => BSL,
            OperatorBitShiftRight => BSR,
            OperatorUBitShiftRight => UBSR,
            OperatorLesser => LESS,
            OperatorLesserEquals => LSEQ,
            OperatorGreater => GRTR,
            OperatorGreaterEquals => GREQ,
            OperatorInstanceof => INOF,
            OperatorIn => IN,
            OperatorStrictEquality => STEQ,
            OperatorStrictInequality => SIEQ,
            OperatorEquality => EQ,
            OperatorInequality => INEQ,
            OperatorBitwiseAnd => BWAN,
            OperatorBitwiseXor => BWXO,
            OperatorBitwiseOr => BWOR,
            OperatorLogicalAnd => AND,
            OperatorLogicalOr => OR,
            Accessor => ACCS,
            TemplateOpen => TPLE,
            TemplateClosed => TPLS,

            _ => None,
        }
    }
}

pub struct B5;

impl BindingPower for B5 {
    #[inline]
    fn handler(_asi: Asi, token: Token) -> NestedHandler {
        match token {
            ParenOpen => CALL,
            BracketOpen => CMEM,
            OperatorFatArrow => ARRW,
            OperatorIncrement => INC,
            OperatorDecrement => DEC,
            OperatorMultiplication => MUL,
            OperatorDivision => DIV,
            OperatorRemainder => REM,
            OperatorExponent => EXPN,
            OperatorAddition => ADD,
            OperatorSubtraction => SUB,
            OperatorBitShiftLeft => BSL,
            OperatorBitShiftRight => BSR,
            OperatorUBitShiftRight => UBSR,
            OperatorLesser => LESS,
            OperatorLesserEquals => LSEQ,
            OperatorGreater => GRTR,
            OperatorGreaterEquals => GREQ,
            OperatorInstanceof => INOF,
            OperatorIn => IN,
            OperatorStrictEquality => STEQ,
            OperatorStrictInequality => SIEQ,
            OperatorEquality => EQ,
            OperatorInequality => INEQ,
            OperatorBitwiseAnd => BWAN,
            OperatorBitwiseXor => BWXO,
            OperatorBitwiseOr => BWOR,
            OperatorLogicalAnd => AND,
            Accessor => ACCS,
            TemplateOpen => TPLE,
            TemplateClosed => TPLS,

            _ => None,
        }
    }
}

pub struct B6;

impl BindingPower for B6 {
    #[inline]
    fn handler(_asi: Asi, token: Token) -> NestedHandler {
        match token {
            ParenOpen => CALL,
            BracketOpen => CMEM,
            OperatorFatArrow => ARRW,
            OperatorIncrement => INC,
            OperatorDecrement => DEC,
            OperatorMultiplication => MUL,
            OperatorDivision => DIV,
            OperatorRemainder => REM,
            OperatorExponent => EXPN,
            OperatorAddition => ADD,
            OperatorSubtraction => SUB,
            OperatorBitShiftLeft => BSL,
            OperatorBitShiftRight => BSR,
            OperatorUBitShiftRight => UBSR,
            OperatorLesser => LESS,
            OperatorLesserEquals => LSEQ,
            OperatorGreater => GRTR,
            OperatorGreaterEquals => GREQ,
            OperatorInstanceof => INOF,
            OperatorIn => IN,
            OperatorStrictEquality => STEQ,
            OperatorStrictInequality => SIEQ,
            OperatorEquality => EQ,
            OperatorInequality => INEQ,
            OperatorBitwiseAnd => BWAN,
            OperatorBitwiseXor => BWXO,
            OperatorBitwiseOr => BWOR,
            Accessor => ACCS,
            TemplateOpen => TPLE,
            TemplateClosed => TPLS,

            _ => None,
        }
    }
}

pub struct B7;

impl BindingPower for B7 {
    #[inline]
    fn handler(_asi: Asi, token: Token) -> NestedHandler {
        match token {
            ParenOpen => CALL,
            BracketOpen => CMEM,
            OperatorFatArrow => ARRW,
            OperatorIncrement => INC,
            OperatorDecrement => DEC,
            OperatorMultiplication => MUL,
            OperatorDivision => DIV,
            OperatorRemainder => REM,
            OperatorExponent => EXPN,
            OperatorAddition => ADD,
            OperatorSubtraction => SUB,
            OperatorBitShiftLeft => BSL,
            OperatorBitShiftRight => BSR,
            OperatorUBitShiftRight => UBSR,
            OperatorLesser => LESS,
            OperatorLesserEquals => LSEQ,
            OperatorGreater => GRTR,
            OperatorGreaterEquals => GREQ,
            OperatorInstanceof => INOF,
            OperatorIn => IN,
            OperatorStrictEquality => STEQ,
            OperatorStrictInequality => SIEQ,
            OperatorEquality => EQ,
            OperatorInequality => INEQ,
            OperatorBitwiseAnd => BWAN,
            OperatorBitwiseXor => BWXO,
            Accessor => ACCS,
            TemplateOpen => TPLE,
            TemplateClosed => TPLS,

            _ => None,
        }
    }
}

pub struct B8;

impl BindingPower for B8 {
    #[inline]
    fn handler(_asi: Asi, token: Token) -> NestedHandler {
        match token {
            ParenOpen => CALL,
            BracketOpen => CMEM,
            OperatorFatArrow => ARRW,
            OperatorIncrement => INC,
            OperatorDecrement => DEC,
            OperatorMultiplication => MUL,
            OperatorDivision => DIV,
            OperatorRemainder => REM,
            OperatorExponent => EXPN,
            OperatorAddition => ADD,
            OperatorSubtraction => SUB,
            OperatorBitShiftLeft => BSL,
            OperatorBitShiftRight => BSR,
            OperatorUBitShiftRight => UBSR,
            OperatorLesser => LESS,
            OperatorLesserEquals => LSEQ,
            OperatorGreater => GRTR,
            OperatorGreaterEquals => GREQ,
            OperatorInstanceof => INOF,
            OperatorIn => IN,
            OperatorStrictEquality => STEQ,
            OperatorStrictInequality => SIEQ,
            OperatorEquality => EQ,
            OperatorInequality => INEQ,
            OperatorBitwiseAnd => BWAN,
            Accessor => ACCS,
            TemplateOpen => TPLE,
            TemplateClosed => TPLS,

            _ => None,
        }
    }
}

pub struct B9;

impl BindingPower for B9 {
    #[inline]
    fn handler(_asi: Asi, token: Token) -> NestedHandler {
        match token {
            ParenOpen => CALL,
            BracketOpen => CMEM,
            OperatorFatArrow => ARRW,
            OperatorIncrement => INC,
            OperatorDecrement => DEC,
            OperatorMultiplication => MUL,
            OperatorDivision => DIV,
            OperatorRemainder => REM,
            OperatorExponent => EXPN,
            OperatorAddition => ADD,
            OperatorSubtraction => SUB,
            OperatorBitShiftLeft => BSL,
            OperatorBitShiftRight => BSR,
            OperatorUBitShiftRight => UBSR,
            OperatorLesser => LESS,
            OperatorLesserEquals => LSEQ,
            OperatorGreater => GRTR,
            OperatorGreaterEquals => GREQ,
            OperatorInstanceof => INOF,
            OperatorIn => IN,
            OperatorStrictEquality => STEQ,
            OperatorStrictInequality => SIEQ,
            OperatorEquality => EQ,
            OperatorInequality => INEQ,
            Accessor => ACCS,
            TemplateOpen => TPLE,
            TemplateClosed => TPLS,

            _ => None,
        }
    }
}

pub struct B10;

impl BindingPower for B10 {
    #[inline]
    fn handler(_asi: Asi, token: Token) -> NestedHandler {
        match token {
            ParenOpen => CALL,
            BracketOpen => CMEM,
            OperatorFatArrow => ARRW,
            OperatorIncrement => INC,
            OperatorDecrement => DEC,
            OperatorMultiplication => MUL,
            OperatorDivision => DIV,
            OperatorRemainder => REM,
            OperatorExponent => EXPN,
            OperatorAddition => ADD,
            OperatorSubtraction => SUB,
            OperatorBitShiftLeft => BSL,
            OperatorBitShiftRight => BSR,
            OperatorUBitShiftRight => UBSR,
            OperatorLesser => LESS,
            OperatorLesserEquals => LSEQ,
            OperatorGreater => GRTR,
            OperatorGreaterEquals => GREQ,
            OperatorInstanceof => INOF,
            OperatorIn => IN,
            Accessor => ACCS,
            TemplateOpen => TPLE,
            TemplateClosed => TPLS,

            _ => None,
        }
    }
}

pub struct B11;

impl BindingPower for B11 {
    #[inline]
    fn handler(_asi: Asi, token: Token) -> NestedHandler {
        match token {
            ParenOpen => CALL,
            BracketOpen => CMEM,
            OperatorFatArrow => ARRW,
            OperatorIncrement => INC,
            OperatorDecrement => DEC,
            OperatorMultiplication => MUL,
            OperatorDivision => DIV,
            OperatorRemainder => REM,
            OperatorExponent => EXPN,
            OperatorAddition => ADD,
            OperatorSubtraction => SUB,
            OperatorBitShiftLeft => BSL,
            OperatorBitShiftRight => BSR,
            OperatorUBitShiftRight => UBSR,
            Accessor => ACCS,
            TemplateOpen => TPLE,
            TemplateClosed => TPLS,

            _ => None,
        }
    }
}

pub struct B12;

impl BindingPower for B12 {
    #[inline]
    fn handler(_asi: Asi, token: Token) -> NestedHandler {
        match token {
            ParenOpen => CALL,
            BracketOpen => CMEM,
            OperatorFatArrow => ARRW,
            OperatorIncrement => INC,
            OperatorDecrement => DEC,
            OperatorMultiplication => MUL,
            OperatorDivision => DIV,
            OperatorRemainder => REM,
            OperatorExponent => EXPN,
            OperatorAddition => ADD,
            OperatorSubtraction => SUB,
            Accessor => ACCS,
            TemplateOpen => TPLE,
            TemplateClosed => TPLS,

            _ => None,
        }
    }
}

pub struct B13;

impl BindingPower for B13 {
    #[inline]
    fn handler(_asi: Asi, token: Token) -> NestedHandler {
        match token {
            ParenOpen => CALL,
            BracketOpen => CMEM,
            OperatorFatArrow => ARRW,
            OperatorIncrement => INC,
            OperatorDecrement => DEC,
            OperatorMultiplication => MUL,
            OperatorDivision => DIV,
            OperatorRemainder => REM,
            OperatorExponent => EXPN,
            Accessor => ACCS,
            TemplateOpen => TPLE,
            TemplateClosed => TPLS,

            _ => None,
        }
    }
}

pub struct B14;

impl BindingPower for B14 {
    #[inline]
    fn handler(_asi: Asi, token: Token) -> NestedHandler {
        match token {
            ParenOpen => CALL,
            BracketOpen => CMEM,
            OperatorFatArrow => ARRW,
            OperatorIncrement => INC,
            OperatorDecrement => DEC,
            OperatorExponent => EXPN,
            Accessor => ACCS,
            TemplateOpen => TPLE,
            TemplateClosed => TPLS,

            _ => None,
        }
    }
}

pub struct B15;

impl BindingPower for B15 {
    #[inline]
    fn handler(_asi: Asi, token: Token) -> NestedHandler {
        match token {
            ParenOpen => CALL,
            BracketOpen => CMEM,
            OperatorFatArrow => ARRW,
            OperatorIncrement => INC,
            OperatorDecrement => DEC,
            Accessor => ACCS,
            TemplateOpen => TPLE,
            TemplateClosed => TPLS,

            _ => None,
        }
    }
}

const SEQ: NestedHandler = Some(|par, left| {
    par.lexer.consume();

    let builder = ListBuilder::new(par.arena, left);
    builder.push(par.arena, par.expression::<B0>());

    while let Comma = par.lexer.token {
        par.lexer.consume();
        builder.push(par.arena, par.expression::<B0>());
    }
    let end = par.lexer.end();
    par.alloc_at_loc(left.start, end, SequenceExpression {
        body: builder.as_list()
    })
});


const COND: NestedHandler = Some(|par, left| {
    par.lexer.consume();

    let consequent = par.expression::<B4>();
    expect!(par, Colon);
    let alternate = par.expression::<B4>();

    par.alloc_at_loc(left.start, alternate.end, ConditionalExpression {
        test: left,
        consequent: consequent,
        alternate: alternate,
    })
});

const ARRW: NestedHandler = Some(|par, left| {
    par.lexer.consume();

    let params = match left.item {
        Expression::Sequence(SequenceExpression { body }) => body,
        _ => NodeList::from(par.arena, left)
    };

    let expression = par.arrow_function_expression(params);
    let start = left.start;
    let end = par.lexer.end();
    return par.alloc_at_loc(start, end, expression)
});

const ACCS: NestedHandler = Some(|par, left| {
    let member = par.lexer.accessor_as_str();
    par.lexer.consume();

    let right = par.alloc_in_loc(member);

    par.alloc_at_loc(left.start, right.end, MemberExpression {
        object: left,
        property: right,
    })
});

const CALL: NestedHandler = Some(|par, left| {
    let start = par.lexer.start_then_consume();
    let arguments = par.call_arguments();
    let end = par.lexer.end_then_consume();

    par.alloc_at_loc(start, end, CallExpression {
        callee: left,
        arguments,
    })
});

const CMEM: NestedHandler = Some(|par, left| {
    par.lexer.consume();
    let property = par.expression::<ANY>();

    expect!(par, BracketClose);
    let end = par.lexer.end();

    par.alloc_at_loc(left.start, end, ComputedMemberExpression {
        object: left,
        property: property,
    })
});

const TPLS: NestedHandler = Some(|par, left| {
    let quasi = par.template_string();

    par.alloc_at_loc(left.start, quasi.end, TaggedTemplateExpression {
        tag: left,
        quasi,
    })
});

const TPLE: NestedHandler = Some(|par, left| {
    par.tagged_template_expression(left)
});

macro_rules! postfix {
    ($name:ident => $op:ident) => {
        const $name: NestedHandler = {
            fn handler<'ast>(par: &mut Parser<'ast>, left: ExpressionNode<'ast>) -> ExpressionNode<'ast> {
                let end = par.lexer.end();
                par.lexer.consume();

                if !left.is_lvalue() {
                    par.error::<()>();
                }

                par.alloc_at_loc(left.start, end, PostfixExpression {
                    operator: $op,
                    operand: left,
                })
            }

            Some(handler)
        };
    }
}

macro_rules! assign {
    ($name:ident => $op:ident) => {
        const $name: NestedHandler = {
            fn handler<'ast>(par: &mut Parser<'ast>, left: ExpressionNode<'ast>) -> ExpressionNode<'ast> {
                par.lexer.consume();

                if !left.is_lvalue() {
                    par.error::<()>();
                }

                let right = par.expression::<B1>();

                par.alloc_at_loc(left.start, right.end, BinaryExpression {
                    operator: $op,
                    left,
                    right,
                })
            }

            Some(handler)
        };
    }
}

macro_rules! binary {
    ($name:ident, $bp:ident => $op:ident) => {
        const $name: NestedHandler = {
            fn handler<'ast>(par: &mut Parser<'ast>, left: ExpressionNode<'ast>) -> ExpressionNode<'ast> {
                par.lexer.consume();

                let right = par.expression::<$bp>();

                par.alloc_at_loc(left.start, right.end, BinaryExpression {
                    operator: $op,
                    left,
                    right,
                })
            }

            Some(handler)
        };
    }
}

postfix!(INC => Increment);
postfix!(DEC => Decrement);

assign!(ASGN => Assign);
assign!(ADDA => AddAssign);
assign!(SUBA => SubtractAssign);
assign!(EXPA => ExponentAssign);
assign!(MULA => MultiplyAssign);
assign!(DIVA => DivideAssign);
assign!(REMA => RemainderAssign);
assign!(BSLA => BSLAssign);
assign!(BSRA => BSRAssign);
assign!(UBSA => UBSRAssign);
assign!(BWAA => BitAndAssign);
assign!(XORA => BitXorAssign);
assign!(BORA => BitOrAssign);

binary!(OR   , B5  => LogicalOr);
binary!(AND  , B6  => LogicalAnd);
binary!(BWOR , B7  => BitwiseOr);
binary!(BWXO , B8  => BitwiseXor);
binary!(BWAN , B9  => BitwiseAnd);
binary!(STEQ , B10 => StrictEquality);
binary!(SIEQ , B10 => StrictInequality);
binary!(EQ   , B10 => Equality);
binary!(INEQ , B10 => Inequality);
binary!(LESS , B11 => Lesser);
binary!(LSEQ , B11 => LesserEquals);
binary!(GRTR , B11 => Greater);
binary!(GREQ , B11 => GreaterEquals);
binary!(INOF , B11 => Instanceof);
binary!(IN   , B11 => In);
binary!(BSL  , B12 => BitShiftLeft);
binary!(BSR  , B12 => BitShiftRight);
binary!(UBSR , B12 => UBitShiftRight);
binary!(ADD  , B13 => Addition);
binary!(SUB  , B13 => Subtraction);
binary!(MUL  , B14 => Multiplication);
binary!(DIV  , B14 => Division);
binary!(REM  , B14 => Remainder);
binary!(EXPN , B15 => Exponent);


impl<'ast> Parser<'ast> {
    #[inline]
    pub fn nested_expression<B>(&mut self, mut left: ExpressionNode<'ast>) -> ExpressionNode<'ast>
    where
        B: BindingPower
    {
        while let Some(handler) = B::handler(self.asi(), self.lexer.token) {
            left = handler(self, left);
        }

        left
    }
}
