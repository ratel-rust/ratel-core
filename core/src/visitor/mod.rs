use ast::{ExpressionPtr, StatementPtr};
use ast::{OptionalName, MandatoryName, Function, Class, Literal};

use ast::expression::{PrefixExpression, ArrowExpression, ArrayExpression};
use ast::expression::{ObjectExpression, TemplateExpression, CallExpression, BinaryExpression};
use ast::expression::{SequenceExpression, MemberExpression, ComputedMemberExpression};
use ast::expression::{PostfixExpression, ConditionalExpression, SpreadExpression};

use ast::statement::{ThrowStatement, ContinueStatement, BreakStatement, ReturnStatement};
use ast::statement::{TryStatement, IfStatement, WhileStatement, DoStatement};
use ast::statement::{DeclarationStatement, ForStatement, ForInStatement, ForOfStatement};
use ast::statement::{SwitchStatement, SwitchCase, LabeledStatement, ForInit};

type FunctionExpression<'ast> = Function<'ast, OptionalName<'ast>>;
type FunctionStatement<'ast> = Function<'ast, MandatoryName<'ast>>;
type ClassExpression<'ast> = Class<'ast, OptionalName<'ast>>;
type ClassStatement<'ast> = Class<'ast, MandatoryName<'ast>>;
type Identifier<'ast> = &'ast str;

#[macro_use]
mod build;

build! {
    #[expressions]
    on_identifier                 => Identifier;
    on_literal                    => Literal;

    on_array_expression           => ArrayExpression;
    on_arrow_expression           => ArrowExpression;
    on_binary_expression          => BinaryExpression;
    on_call_expression            => CallExpression;
    on_class_expression           => ClassExpression;
    on_computed_member_expression => ComputedMemberExpression;
    on_conditional_expression     => ConditionalExpression;
    on_function_expression        => FunctionExpression;
    on_member_expression          => MemberExpression;
    on_object_expression          => ObjectExpression;
    on_postfix_expression         => PostfixExpression;
    on_prefix_expression          => PrefixExpression;
    on_sequence_expression        => SequenceExpression;
    on_spread_expression          => SpreadExpression;
    on_template_expression        => TemplateExpression;

    #[statements]
    on_break_statement       => BreakStatement;
    on_continue_statement    => ContinueStatement;
    on_class_statement       => ClassStatement;
    on_declaration_statement => DeclarationStatement;
    on_do_statement          => DoStatement;
    on_for_in_statement      => ForInStatement;
    on_for_init              => ForInit;
    on_for_of_statement      => ForOfStatement;
    on_for_statement         => ForStatement;
    on_function_statement    => FunctionStatement;
    on_if_statement          => IfStatement;
    on_labeled_statement     => LabeledStatement;
    on_return_statement      => ReturnStatement;
    on_switch_case           => SwitchCase;
    on_switch_statement      => SwitchStatement;
    on_throw_statement       => ThrowStatement;
    on_try_statement         => TryStatement;
    on_while_statement       => WhileStatement;
}
