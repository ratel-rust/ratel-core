extern crate badger;

use badger::grammar::*;
use badger::parser::parse;
use badger::grammar::Statement::*;
use badger::grammar::Expression::*;
use badger::grammar::ClassMember::*;
use badger::grammar::OperatorType::*;

macro_rules! assert_parse {
    ($string:expr, $body:expr) => {
        assert_eq!(parse($string.to_string()).body, $body);
    }
}

macro_rules! assert_expression {
    ($string:expr, $ex:expr) => {
        match parse($string.to_string()).body[0] {
            ExpressionStatement(ref expression) => assert_eq!(*expression, $ex),
            _                                   => panic!("No expression found"),
        }
    }
}

macro_rules! assert_statement {
    ($string:expr, $ex:expr) => (assert_parse!($string, vec![$ex]))
}

macro_rules! num {
    ($num:expr) => (LiteralExpression(LiteralFloat($num)))
}

macro_rules! boxnum {
    ($num:expr) => (Box::new(num!($num)))
}


#[test]
fn var_declare() {
    assert_statement!("var foo = 100;", VariableDeclarationStatement {
        kind: VariableDeclarationKind::Var,
        declarations: vec![(
            "foo".to_string(),
            num!(100.0)
        )]
    });
}

#[test]
fn let_declare() {
    assert_statement!("let foo = 100;", VariableDeclarationStatement {
        kind: VariableDeclarationKind::Let,
        declarations: vec![(
            "foo".to_string(),
            num!(100.0)
        )]
    });
}


#[test]
fn const_declare() {
    assert_statement!("const foo = 100;", VariableDeclarationStatement {
        kind: VariableDeclarationKind::Const,
        declarations: vec![(
            "foo".to_string(),
            num!(100.0)
        )]
    });
}

#[test]
fn var_muliple_declare() {
    assert_statement!("var foo = 100, bar = 200;", VariableDeclarationStatement {
        kind: VariableDeclarationKind::Var,
        declarations: vec![(
            "foo".to_string(),
            num!(100.0)
        ), (
            "bar".to_string(),
            num!(200.0)
        )]
    });
}

#[test]
fn identifier_expression() {
    assert_expression!("foobar", IdentifierExpression("foobar".to_string()))
}

#[test]
fn null_expression() {
    assert_expression!("null", LiteralExpression(LiteralNull));
}

#[test]
fn undefined_expression() {
    assert_expression!("undefined", LiteralExpression(LiteralUndefined));
}

#[test]
fn true_expression() {
    assert_expression!("true", LiteralExpression(LiteralTrue));
}

#[test]
fn false_expression() {
    assert_expression!("false", LiteralExpression(LiteralFalse));
}

#[test]
fn number_expression() {
    assert_expression!("100", num!(100.0));
}

#[test]
fn binary_number_expression() {
    assert_expression!("0b1100100", LiteralExpression(LiteralInteger(100)));
}

#[test]
fn octal_number_expression() {
    assert_expression!("0o144", LiteralExpression(LiteralInteger(100)));
}

#[test]
fn hexdec_number_expression() {
    assert_expression!("0x64", LiteralExpression(LiteralInteger(100)));
}

#[test]
fn floating_number_expression() {
    assert_expression!("3.14", num!(3.14));
}

#[test]
fn binary_expression() {
    assert_expression!("true == 1", BinaryExpression {
        left: Box::new(LiteralExpression(LiteralTrue)),
        operator: Equality,
        right: boxnum!(1.0)
    });
}

#[test]
fn op_precedence_left() {
    assert_expression!("1 + 2 * 3", BinaryExpression {
        left: boxnum!(1.0),
        operator: Addition,
        right: Box::new(BinaryExpression {
            left: boxnum!(2.0),
            operator: Multiplication,
            right: boxnum!(3.0),
        }),
    })
}

#[test]
fn op_precedence_right() {
    assert_expression!("1 * 2 + 3", BinaryExpression {
        left: Box::new(BinaryExpression {
            left: boxnum!(1.0),
            operator: Multiplication,
            right: boxnum!(2.0),
        }),
        operator: Addition,
        right: boxnum!(3.0),
    })
}

#[test]
fn function_statement() {
    assert_statement!("function foo() { return bar; }", FunctionStatement {
        name: "foo".to_string(),
        params: Vec::new(),
        body: vec![
            ReturnStatement(IdentifierExpression("bar".to_string()))
        ]
    })
}

