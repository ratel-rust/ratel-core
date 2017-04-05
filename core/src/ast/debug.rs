use ast::{Index, OptIndex, Item, Program};
use ast::item::*;
use ast::Item::*;
use ast::Nodes;
use ast::variable::*;
use ast::operator::OperatorKind;
use error::Error;

use std::fmt::{Debug, Formatter, Result};

trait AstDebug<'src> {
    fn ast_fmt(&self, gen: &mut DebugGen<'src>, f: &mut Formatter) -> Result;
}


/// The `Generator` is a wrapper around an owned `String` that's used to
/// stringify the AST. There is a bunch of useful methods here to manage
/// things like indentation and automatically producing minified code.
struct DebugGen<'src> {
    program: &'src Program<'src>,
    dent: u16,
}

impl<'src> DebugGen<'src> {
    pub fn new(program: &'src Program<'src>) -> Self {
        DebugGen {
            program: program,
            dent: 0,
        }
    }

    #[inline]
    pub fn write<T>(&mut self, item: &T, f: &mut Formatter) -> Result
        where T: AstDebug<'src>
    {
        item.ast_fmt(self, f)
    }

    #[inline]
    fn write_from_index(&mut self, index: Index, f: &mut Formatter) -> Result {
        self.program[index].ast_fmt(self, f)
    }

    #[inline]
    fn write_from_optindex(&mut self, opt: OptIndex, f: &mut Formatter) -> Result {
        if !opt.is_null() {
            self.write_from_index(opt.unwrap(), f)?;
        }

        Ok(())
    }

    #[inline]
    fn write_list_from_optindex(&mut self, opt: OptIndex, f: &mut Formatter) -> Result {
        f.write_str("[");
        self.program.store.nodes(opt).items().fold(true, |acc, item| {
          if !acc {
            f.write_str(", ");
          }
          self.write(item, f);
          false
        });
        f.write_str("]");
        Ok(())
    }

    #[inline]
    fn write_list_from_index(&mut self, index: Index, f: &mut Formatter) -> Result {
        f.write_str("[");
        self.program.store.nodes(index).fold(true, |acc, node| {
          if !acc {
            f.write_str(", ");
          }
          self.write(&node.item, f);
          false
        });
        f.write_str("]");
        Ok(())
    }

    #[inline]
    pub fn indent(&mut self) {
        self.dent += 1;
    }

    #[inline]
    pub fn dedent(&mut self) {
        self.dent -= 1;
    }
}

impl<'src> AstDebug<'src> for OperatorKind {
    fn ast_fmt(&self, gen: &mut DebugGen<'src>, f: &mut Formatter) -> Result {
        match *self {
            OperatorKind::FatArrow           => f.write_str("FatArrow")?,
            OperatorKind::New                => f.write_str("New")?,
            OperatorKind::Increment          => f.write_str("Increment")?,
            OperatorKind::Decrement          => f.write_str("Decrement")?,
            OperatorKind::LogicalNot         => f.write_str("LogicalNot")?,
            OperatorKind::BitwiseNot         => f.write_str("BitwiseNot")?,
            OperatorKind::Typeof             => f.write_str("Typeof")?,
            OperatorKind::Void               => f.write_str("Void")?,
            OperatorKind::Delete             => f.write_str("Delete")?,
            OperatorKind::Multiplication     => f.write_str("Multiplication")?,
            OperatorKind::Division           => f.write_str("Division")?,
            OperatorKind::Remainder          => f.write_str("Remainder")?,
            OperatorKind::Exponent           => f.write_str("Exponent")?,
            OperatorKind::Addition           => f.write_str("Addition")?,
            OperatorKind::Substraction       => f.write_str("Substraction")?,
            OperatorKind::BitShiftLeft       => f.write_str("BitShiftLeft")?,
            OperatorKind::BitShiftRight      => f.write_str("BitShiftRight")?,
            OperatorKind::UBitShiftRight     => f.write_str("UBitShiftRight")?,
            OperatorKind::Lesser             => f.write_str("Lesser")?,
            OperatorKind::LesserEquals       => f.write_str("LesserEquals")?,
            OperatorKind::Greater            => f.write_str("Greater")?,
            OperatorKind::GreaterEquals      => f.write_str("GreaterEquals")?,
            OperatorKind::Instanceof         => f.write_str("Instanceof")?,
            OperatorKind::In                 => f.write_str("In")?,
            OperatorKind::StrictEquality     => f.write_str("StrictEquality")?,
            OperatorKind::StrictInequality   => f.write_str("StrictInequality")?,
            OperatorKind::Equality           => f.write_str("Equality")?,
            OperatorKind::Inequality         => f.write_str("Inequality")?,
            OperatorKind::BitwiseAnd         => f.write_str("BitwiseAnd")?,
            OperatorKind::BitwiseXor         => f.write_str("BitwiseXor")?,
            OperatorKind::BitwiseOr          => f.write_str("BitwiseOr")?,
            OperatorKind::LogicalAnd         => f.write_str("LogicalAnd")?,
            OperatorKind::LogicalOr          => f.write_str("LogicalOr")?,
            OperatorKind::Conditional        => f.write_str("Conditional")?,
            OperatorKind::Assign             => f.write_str("Assign")?,
            OperatorKind::AddAssign          => f.write_str("AddAssign")?,
            OperatorKind::SubstractAssign    => f.write_str("SubstractAssign")?,
            OperatorKind::ExponentAssign     => f.write_str("ExponentAssign")?,
            OperatorKind::MultiplyAssign     => f.write_str("MultiplyAssign")?,
            OperatorKind::DivideAssign       => f.write_str("DivideAssign")?,
            OperatorKind::RemainderAssign    => f.write_str("RemainderAssign")?,
            OperatorKind::BSLAssign          => f.write_str("BSLAssign")?,
            OperatorKind::BSRAssign          => f.write_str("BSRAssign")?,
            OperatorKind::UBSRAssign         => f.write_str("UBSRAssign")?,
            OperatorKind::BitAndAssign       => f.write_str("BitAndAssign")?,
            OperatorKind::BitXorAssign       => f.write_str("BitXorAssign")?,
            OperatorKind::BitOrAssign        => f.write_str("BitOrAssign")?,
            OperatorKind::Spread             => f.write_str("Spread")?,
        }
        Ok(())
    }
}

impl<'src> AstDebug<'src> for VariableDeclarationKind {
    fn ast_fmt(&self, gen: &mut DebugGen<'src>, f: &mut Formatter) -> Result {
        match *self {
            VariableDeclarationKind::Var => f.write_str("Var")?,
            VariableDeclarationKind::Let => f.write_str("Let")?,
            VariableDeclarationKind::Const => f.write_str("Const")?,
        }
        Ok(())
    }
}

impl<'src> AstDebug<'src> for Value<'src> {
    fn ast_fmt(&self, gen: &mut DebugGen<'src>, f: &mut Formatter) -> Result {
        match *self {
            Value::Undefined => f.write_str("Undefined")?,
            Value::Null => f.write_str("Null")?,
            Value::True => f.write_str("True")?,
            Value::False => f.write_str("False")?,
            Value::Number(value) => {
                f.write_str("Number(")?;
                f.write_str(value)?;
                f.write_str(")")?;
            },
            Value::Binary(value) => {
                f.write_str("Binary(")?;
                f.write_str(value.to_string().as_str())?;
                f.write_str(")")?;
            },
            Value::String(value) => {
                f.write_str("String(")?;
                f.write_str(value)?;
                f.write_str(")")?;
            },
            Value::RawQuasi(value) => {
                f.write_str("RawQuasi(")?;
                f.write_str(value)?;
                f.write_str(")")?;
            },
            Value::RegEx(value) => {
                f.write_str("RegEx(")?;
                f.write_str(value)?;
                f.write_str(")")?;
            },
        }
        Ok(())
    }
}

impl<'src> AstDebug<'src> for Item<'src> {
    fn ast_fmt(&self, gen: &mut DebugGen<'src>, f: &mut Formatter) -> Result {
        match *self {
            Error(error) => {
                f.write_str("Error(")?;
                let value = match error {
                    Error::UnexpectedEndOfProgram { .. } => "UnexpectedEndOfProgram",
                    Error::UnexpectedToken { .. } => "UnexpectedToken",
                };
                f.write_str(value)?;
                f.write_str(")")?;
            },

            // Identifiers
            Identifier(ident) => f.write_str(ident.as_str())?,
            This => f.write_str("This")?,

            // Expressions
            ValueExpr(value) => {
                f.write_str("ValueExpr(")?;
                value.ast_fmt(gen, f)?;
                f.write_str(")")?;
            },
            ArrayExpr(index) => {
                f.write_str("ArrayExpr[")?;
                gen.write_list_from_optindex(index, f);
                f.write_str("]")?;
            },
            SequenceExpr(index) => {
                f.write_str("SequenceExpr(")?;
                gen.write_list_from_index(index, f);
                f.write_str(")")?;
            },
            MemberExpr { object, property } => {
                f.write_str("MemberExpr { object: ")?;
                gen.write_from_index(object, f);
                f.write_str(", property: ")?;
                gen.write_from_index(property, f);
                f.write_str(" }")?;
            },
            CallExpr { callee, arguments } => {
                f.write_str("CallExpr { callee: ")?;
                gen.write_from_index(callee, f);
                f.write_str(", arguments: ")?;
                gen.write_list_from_optindex(arguments, f);
                f.write_str(" }")?;
            },
            BinaryExpr { parenthesized, operator, left, right } => {
                f.write_str("BinaryExpr { parenthesized: ")?;
                f.write_str(if parenthesized { "true" } else { "false" })?;
                f.write_str(", operator: ")?;
                operator.ast_fmt(gen, f);
                f.write_str(", left: ")?;
                gen.write_from_index(left, f);
                f.write_str(", right: ")?;
                gen.write_from_index(right, f);
                f.write_str(" }")?;
            },
            Prefix { operator, operand } => {
                f.write_str("Prefix { operator: ")?;
                operator.ast_fmt(gen, f);
                f.write_str(", operand: ")?;
                gen.write_from_index(operand, f);
                f.write_str(" }")?;
            },
            PostfixExpr { operator, operand } => {
                f.write_str("PostfixExpr { operator: ")?;
                operator.ast_fmt(gen, f);
                f.write_str(", operand: ")?;
                gen.write_from_index(operand, f);
                f.write_str(" }")?;
            },
            ConditionalExpr { test, consequent, alternate } => {
                f.write_str("ConditionalExpr { test: ")?;
                gen.write_from_index(test, f);
                f.write_str(", consequent: ")?;
                gen.write_from_index(consequent, f);
                f.write_str(", alternate: ")?;
                gen.write_from_index(alternate, f);
                f.write_str(" }")?;
            },
            ArrowExpr { params, body } => {
                f.write_str("ArrowExpr { params: ")?;
                gen.write_list_from_optindex(params, f);
                f.write_str(", body: ")?;
                gen.write_from_optindex(body, f);
                f.write_str(" }")?;
            },
            FunctionExpr { name, params, body } => {
                f.write_str("FunctionExpr { name: ")?;
                gen.write_from_optindex(name, f);
                f.write_str(", params: ")?;
                gen.write_list_from_optindex(params, f);
                f.write_str(", body: ")?;
                gen.write_from_optindex(body, f);
                f.write_str(" }")?;
            },
            ObjectExpr { body } => {
                f.write_str("ObjectExpr { body: ")?;
                gen.write_from_optindex(body, f);
                f.write_str(" }")?;
            },
            ClassExpr { name, extends, body } => {
                f.write_str("ClassExpr { name: ")?;
                gen.write_from_optindex(name, f);
                f.write_str(", extends: ")?;
                gen.write_from_optindex(extends, f);
                f.write_str(", body: ")?;
                gen.write_from_optindex(body, f);
                f.write_str(" }")?;
            },

            // Object
            ShorthandMember(ident) => {
                f.write_str("ShorthandMember(")?;
                f.write_str(ident.as_str())?;
                f.write_str(")")?;
            },
            ObjectMember { key, value } => {
                f.write_str("ObjectMember { key: ")?;
                gen.write_from_index(key, f);
                f.write_str(", value: ")?;
                gen.write_from_index(value, f);
                f.write_str(" }")?;
            },

            // Declaration
            VariableDeclarator { name, value } => {
                f.write_str("VariableDeclarator { name: ")?;
                gen.write_from_index(name, f);
                f.write_str(", value: ")?;
                gen.write_from_optindex(value, f);
                f.write_str(" }")?;
            },

            // Statements

            EmptyStatement => {
                f.write_str("EmptyStatement")?;
            },
            ExpressionStatement(index) => {
                f.write_str("ExpressionStatement(")?;
                gen.write_from_index(index, f);
                f.write_str(")")?;
            },
            DeclarationStatement { kind, declarators } => {
                f.write_str("DeclarationStatement { kind: ")?;
                kind.ast_fmt(gen, f);
                f.write_str(", declarators: ")?;
                gen.write_from_index(declarators, f);
                f.write_str(" }")?;
            },
            FunctionStatement { name, params, body } => {
                f.write_str("FunctionStatement { name: ")?;
                gen.write_from_index(name, f);
                f.write_str(", params: ")?;
                gen.write_from_optindex(params, f);
                f.write_str(", body: ")?;
                gen.write_from_optindex(body, f);
                f.write_str(" }")?;
            },
            ReturnStatement { value } => {
                f.write_str("ReturnStatement { value: ")?;
                gen.write_from_optindex(value, f);
                f.write_str(" }")?;
            },
            BreakStatement { label} => {
                f.write_str("BreakStatement { label: ")?;
                gen.write_from_optindex(label, f);
                f.write_str(" }")?;
            },
            IfStatement { test, consequent, alternate } => {
                f.write_str("IfStatement { test: ")?;
                gen.write_from_index(test, f);
                f.write_str(", consequent: ")?;
                gen.write_from_index(consequent, f);
                f.write_str(", alternate: ")?;
                gen.write_from_optindex(alternate, f);
                f.write_str(" }")?;
            },
            WhileStatement { test, body } => {
                f.write_str("WhileStatement { test: ")?;
                gen.write_from_index(test, f);
                f.write_str(", body: ")?;
                gen.write_from_index(body, f);
                f.write_str(" }")?;
            },
            DoStatement { body, test } => {
                f.write_str("DoStatement { body: ")?;
                gen.write_from_index(body, f);
                f.write_str(", test: ")?;
                gen.write_from_index(test, f);
                f.write_str(" }")?;
            },
            ForStatement { init, test, update, body } => {
                f.write_str("ForStatement { init: ")?;
                gen.write_from_optindex(init, f);
                f.write_str(", test: ")?;
                gen.write_from_optindex(test, f);
                f.write_str(", update: ")?;
                gen.write_from_optindex(update, f);
                f.write_str(", body: ")?;
                gen.write_from_index(body, f);
                f.write_str(" }")?;
            },
            ForIn { left, right, body } => {
                f.write_str("ForIn { left: ")?;
                gen.write_from_index(left, f);
                f.write_str(", right: ")?;
                gen.write_from_index(right, f);
                f.write_str(", body: ")?;
                gen.write_from_index(body, f);
                f.write_str("}")?;
            },
            ForOf { left, right, body } => {
                f.write_str("ForOf { left: ")?;
                gen.write_from_index(left, f);
                f.write_str(", right: ")?;
                gen.write_from_index(right, f);
                f.write_str(", body: ")?;
                gen.write_from_index(body, f);
                f.write_str(" }")?;
            },
            ThrowStatement { value } => {
                f.write_str("ThrowStatement { value: ")?;
                gen.write_from_index(value, f);
                f.write_str(" }")?;
            },
            TryStatement { body, error, handler } => {
                f.write_str("TryStatement { body: ")?;
                gen.write_from_optindex(body, f);
                f.write_str(", error: ")?;
                gen.write_from_index(error, f);
                f.write_str(", handler: ")?;
                gen.write_from_optindex(handler, f);
                f.write_str(" }")?;
            },
            BlockStatement { body } => {
                f.write_str("BlockStatement { body: ")?;
                gen.write_from_optindex(body, f);
                f.write_str(" }")?;
            },
        }

        Ok(())
    }
}

impl<'src> Debug for Program<'src> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut gen = DebugGen::new(self);
        gen.write_from_optindex(self.root, f);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use parser::parse;

    #[test]
    fn should_die() {
        let src = "break foo;";
        let program = parse(src).unwrap();

        panic!("{:?}", program);
    }
}
