use ast::DeclarationKind;
use parser::expression::{
    ExpressionHandler,
    ParenHandler,
    ArrayExpressionHandler,
    ObjectExpressionHandler,
    NewHandler,
    IncrementPrefixHandler,
    DecrementPrefixHandler,
    LogicalNotPrefixHandler,
    BitwiseNotPrefixHandler,
    TypeofPrefixHandler,
    VoidPrefixHandler,
    DeletePrefixHandler,
    AdditionPrefixHandler,
    SubtractionPrefixHandler,
    RegExHandler,
    ThisHandler,
    TrueLiteralHandler,
    FalseLiteralHandler,
    NullLiteralHandler,
    UndefinedLiteralHandler,
    StringLiteralHandler,
    NumberLiteralHandler,
    BinaryLiteralHandler,
    TemplateStringLiteralHandler,
    TemplateExpressionHandler,
    SpreadExpressionHandler,
};
use lexer::Token::*;

use super::Feature;

pub static ES5: Feature = |set| {
    set.set_expression(ParenOpen, ParenHandler);
    set.set_expression(BracketOpen, ArrayExpressionHandler);
    set.set_expression(BraceOpen, ObjectExpressionHandler);
    set.set_expression(OperatorNew, NewHandler);
    set.set_expression(OperatorIncrement, IncrementPrefixHandler);
    set.set_expression(OperatorDecrement, DecrementPrefixHandler);
    set.set_expression(OperatorLogicalNot, LogicalNotPrefixHandler);
    set.set_expression(OperatorBitwiseNot, BitwiseNotPrefixHandler);
    set.set_expression(OperatorTypeof, TypeofPrefixHandler);
    set.set_expression(OperatorVoid, VoidPrefixHandler);
    set.set_expression(OperatorDelete, DeletePrefixHandler);
    set.set_expression(OperatorAddition, AdditionPrefixHandler);
    set.set_expression(OperatorSubtraction, SubtractionPrefixHandler);
    set.set_expression(OperatorDivision, RegExHandler);
    set.set_expression(This, ThisHandler);
    set.set_expression(LiteralTrue, TrueLiteralHandler);
    set.set_expression(LiteralFalse, FalseLiteralHandler);
    set.set_expression(LiteralNull, NullLiteralHandler);
    set.set_expression(LiteralUndefined, UndefinedLiteralHandler);
    set.set_expression(LiteralString, StringLiteralHandler);
    set.set_expression(LiteralNumber, NumberLiteralHandler);
    set.set_expression(LiteralBinary, BinaryLiteralHandler);
    set.set_expression(TemplateOpen, TemplateExpressionHandler);
    set.set_expression(TemplateClosed, TemplateStringLiteralHandler);

    set.statements.extend(&[
        (Semicolon,           ::parser::statement::empty),
        (BraceOpen,           |par| par.block_statement()),
        (DeclarationVar,      |par| par.variable_declaration_statement(DeclarationKind::Var)),
        (DeclarationLet,      |par| par.variable_declaration_statement(DeclarationKind::Let)),
        (DeclarationConst,    |par| par.variable_declaration_statement(DeclarationKind::Const)),
        (Break,               |par| par.break_statement()),
        (Do,                  |par| par.do_statement()),
        (Class,               |par| par.class_statement()),
        (Return,              |par| par.return_statement()),
        (While,               |par| par.while_statement()),
        (Continue,            |par| par.continue_statement()),
        (For,                 |par| par.for_statement()),
        (Switch,              |par| par.switch_statement()),
        (Function,            |par| par.function_statement()),
        (If,                  |par| par.if_statement()),
        (Throw,               |par| par.throw_statement()),
        (Try,                 |par| par.try_statement()),
        (Identifier,          |par| par.labeled_or_expression_statement()),
    ]);

    set.expressions.default.extend(&[
        (Class,               |par| par.class_expression()),
        (Function,            |par| par.function_expression()),
        (Identifier,          |par| par.node_consume_str(|ident| ident)),
    ]);

    // Adds handler for SpreadExpression
    set.expressions.call = set.expressions.default.extend_copy(&[
        (OperatorSpread, SpreadExpressionHandler::expression),
    ]);

    // Adds handlers for VoidExpression and SpreadExpression
    set.expressions.array = set.expressions.default.extend_copy(&[
        (BracketClose, ::parser::expression::void),
        (Comma, ::parser::expression::void),
        (OperatorSpread, SpreadExpressionHandler::expression),
    ]);
};
