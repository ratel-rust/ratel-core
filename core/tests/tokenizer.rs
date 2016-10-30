extern crate ratel;

use ratel::grammar::*;
use ratel::tokenizer::*;
use ratel::operator::OperatorKind;
use ratel::lexicon::Token;
use ratel::lexicon::Token::*;
use ratel::lexicon::ReservedKind;

fn test_token(input: &str, expected: Token) -> bool {
    let mut tokenizer = Tokenizer::new(&input);
    let tok = tokenizer.get_token().unwrap();
    if tok != expected {
        println!("\n{:?}\n", tok);
    }
    tok == expected
}

macro_rules! lit_num {
    ($num:expr) => (Literal(Value::Number($num.into())))
}

macro_rules! lit_bin {
    ($num:expr) => (Literal(Value::Binary($num)))
}

macro_rules! assert_token {
    ($string:expr, $token:expr, $descr:expr) => {
        assert_eq!(test_token($string, $token), true, $descr);
    }
}

#[test]
fn test_tokenizer_chars() {
    assert_token!(";", Semicolon, "read a Semicolon");
    assert_token!(":", Colon, "read a Colon");
    assert_token!(",", Comma, "read a Comma");
    assert_token!("(", ParenOpen, "read a ParenOpen");
    assert_token!(")", ParenClose, "read a ParenClose");
    assert_token!("[", BracketOpen, "read a BracketOpen");
    assert_token!("]", BracketClose, "read a BracketClose");
    assert_token!("{", BraceOpen, "read a BraceOpen");
    assert_token!("}", BraceClose, "read a BraceClose");
}

#[test]
fn test_tokenizer_control_statements() {
    assert_token!("break", Break, "read a Break");
    assert_token!("do", Do, "read a Do");
    assert_token!("case", Case, "read a Case");
    assert_token!("else", Else, "read a Else");
    assert_token!("catch", Catch, "read a Catch");
    assert_token!("export", Export, "read a Export");
    assert_token!("class", Class, "read a Class");
    assert_token!("extends", Extends, "read a Extends");
    assert_token!("return", Return, "read a Return");
    assert_token!("while", While, "read a While");
    assert_token!("finally", Finally, "read a Finally");
    assert_token!("super", Super, "read a Super");
    assert_token!("with", With, "read a With");
    assert_token!("continue", Continue, "read a Continue");
    assert_token!("for", For, "read a For");
    assert_token!("switch", Switch, "read a Switch");
    assert_token!("yield", Yield, "read a Yield");
    assert_token!("debugger", Debugger, "read a Debugger");
    assert_token!("function", Function, "read a Function");
    assert_token!("this", This, "read a This");
    assert_token!("default", Default, "read a Default");
    assert_token!("if", If, "read a If");
    assert_token!("throw", Throw, "read a Throw");
    assert_token!("import", Import, "read a Import");
    assert_token!("try", Try, "read a Try");
}

#[test]
fn test_tokenizer_operators() {
    assert_token!("=>", Operator(OperatorKind::FatArrow), "OperatorKind::FatArrow");
    assert_token!(".", Operator(OperatorKind::Accessor), "OperatorKind::Accessor");
    assert_token!("new", Operator(OperatorKind::New), "OperatorKind::New");
    assert_token!("++", Operator(OperatorKind::Increment), "OperatorKind::Increment");
    assert_token!("--", Operator(OperatorKind::Decrement), "OperatorKind::Decrement");
    assert_token!("!", Operator(OperatorKind::LogicalNot), "OperatorKind::LogicalNot");
    assert_token!("~", Operator(OperatorKind::BitwiseNot), "OperatorKind::BitwiseNot");
    assert_token!("typeof", Operator(OperatorKind::Typeof), "OperatorKind::Typeof");
    assert_token!("void", Operator(OperatorKind::Void), "OperatorKind::Void");
    assert_token!("delete", Operator(OperatorKind::Delete), "OperatorKind::Delete");
    assert_token!("*", Operator(OperatorKind::Multiplication), "OperatorKind::Multiplication");
    assert_token!("/", Operator(OperatorKind::Division), "OperatorKind::Division");
    assert_token!("%", Operator(OperatorKind::Remainder), "OperatorKind::Remainder");
    assert_token!("**", Operator(OperatorKind::Exponent), "OperatorKind::Exponent");
    assert_token!("+", Operator(OperatorKind::Addition), "OperatorKind::Addition");
    assert_token!("-", Operator(OperatorKind::Substraction), "OperatorKind::Substraction");
    assert_token!("<<", Operator(OperatorKind::BitShiftLeft), "OperatorKind::BitShiftLeft");
    assert_token!(">>", Operator(OperatorKind::BitShiftRight), "OperatorKind::BitShiftRight");
    assert_token!(">>>", Operator(OperatorKind::UBitShiftRight), "OperatorKind::UBitShiftRight");
    assert_token!("<", Operator(OperatorKind::Lesser), "OperatorKind::Lesser");
    assert_token!("<=", Operator(OperatorKind::LesserEquals), "OperatorKind::LesserEquals");
    assert_token!(">", Operator(OperatorKind::Greater), "OperatorKind::Greater");
    assert_token!(">=", Operator(OperatorKind::GreaterEquals), "OperatorKind::GreaterEquals");
    assert_token!("instanceof", Operator(OperatorKind::Instanceof), "OperatorKind::Instanceof");
    assert_token!("in", Operator(OperatorKind::In), "OperatorKind::In");
    assert_token!("===", Operator(OperatorKind::StrictEquality), "OperatorKind::StrictEquality");
    assert_token!("!==", Operator(OperatorKind::StrictInequality), "OperatorKind::StrictInequality");
    assert_token!("==", Operator(OperatorKind::Equality), "OperatorKind::Equality");
    assert_token!("!=", Operator(OperatorKind::Inequality), "OperatorKind::Inequality");
    assert_token!("&", Operator(OperatorKind::BitwiseAnd), "OperatorKind::BitwiseAnd");
    assert_token!("^", Operator(OperatorKind::BitwiseXor), "OperatorKind::BitwiseXor");
    assert_token!("|", Operator(OperatorKind::BitwiseOr), "OperatorKind::BitwiseOr");
    assert_token!("&&", Operator(OperatorKind::LogicalAnd), "OperatorKind::LogicalAnd");
    assert_token!("||", Operator(OperatorKind::LogicalOr), "OperatorKind::LogicalOr");
    assert_token!("?", Operator(OperatorKind::Conditional), "OperatorKind::Conditional");
    assert_token!("=", Operator(OperatorKind::Assign), "OperatorKind::Assign");
    assert_token!("+=", Operator(OperatorKind::AddAssign), "OperatorKind::AddAssign");
    assert_token!("-=", Operator(OperatorKind::SubstractAssign), "OperatorKind::SubstractAssign");
    assert_token!("**=", Operator(OperatorKind::ExponentAssign), "OperatorKind::ExponentAssign");
    assert_token!("*=", Operator(OperatorKind::MultiplyAssign), "OperatorKind::MultiplyAssign");
    assert_token!("/=", Operator(OperatorKind::DivideAssign), "OperatorKind::DivideAssign");
    assert_token!("%=", Operator(OperatorKind::RemainderAssign), "OperatorKind::RemainderAssign");
    assert_token!("<<=", Operator(OperatorKind::BSLAssign), "OperatorKind::BSLAssign");
    assert_token!(">>=", Operator(OperatorKind::BSRAssign), "OperatorKind::BSRAssign");
    assert_token!(">>>=", Operator(OperatorKind::UBSRAssign), "OperatorKind::UBSRAssign");
    assert_token!("&=", Operator(OperatorKind::BitAndAssign), "OperatorKind::BitAndAssign");
    assert_token!("^=", Operator(OperatorKind::BitXorAssign), "OperatorKind::BitXorAssign");
    assert_token!("|=", Operator(OperatorKind::BitOrAssign), "OperatorKind::BitOrAssign");
    assert_token!("...", Operator(OperatorKind::Spread), "OperatorKind::Spread");
}

#[test]
fn test_tokenizer_literals() {
    assert_token!("undefined", Literal(Value::Undefined), "Value::Undefined");
    assert_token!("null", Literal(Value::Null), "Value::Null");
    assert_token!("true", Literal(Value::True), "Value::True");
    assert_token!("false", Literal(Value::False), "Value::False");

    assert_token!("'foo'", Literal(Value::String("'foo'".into())), "Value::String");
    assert_token!("\"foo\"", Literal(Value::String("\"foo\"".into())), "Value::String");

    assert_token!("2.2", Literal(Value::Number("2.2".into())), "Value::Number");
    assert_token!("2", Literal(Value::Number("2".into())), "Value::Number");

    assert_token!("0xff", Literal(Value::Number("0xff".into())), "Value::Number");
    assert_token!("0xff", lit_num!("0xff"), "Value::Number");
    assert_token!("0XFF", lit_num!("0XFF"), "Value::Number");
    assert_token!("0b01001011", lit_bin!(75u64), "Value::Number");
    assert_token!("0B01001011", lit_bin!(75u64), "Value::Number");
    assert_token!("0o113", lit_num!("0o113"), "Value::Number");
    assert_token!("0O113", lit_num!("0O113"), "Value::Number");
}

#[test]
fn test_scientifix_numbers() {
    assert_token!("0e-2", Literal(Value::Number("0e-2".into())), "Value::Number");
    assert_token!("0e2", Literal(Value::Number("0e2".into())), "Value::Number");
    assert_token!("2e3", Literal(Value::Number("2e3".into())), "Value::Number");
    assert_token!("2e-3", Literal(Value::Number("2e-3".into())), "Value::Number");
    assert_token!("2e+3", Literal(Value::Number("2e+3".into())), "Value::Number");
    assert_token!("0.2e3", Literal(Value::Number("0.2e3".into())), "Value::Number");
    assert_token!("0.2e-3", Literal(Value::Number("0.2e-3".into())), "Value::Number");
}

#[test]
fn test_tokenizer_reserved() {
    assert_token!("enum", Reserved(ReservedKind::Enum), "ReservedKind::Enum");
    assert_token!("implements", Reserved(ReservedKind::Implements), "ReservedKind::Implements");
    assert_token!("package", Reserved(ReservedKind::Package), "ReservedKind::Package");
    assert_token!("protected", Reserved(ReservedKind::Protected), "ReservedKind::Protected");
    assert_token!("interface", Reserved(ReservedKind::Interface), "ReservedKind::Interface");
    assert_token!("private", Reserved(ReservedKind::Private), "ReservedKind::Private");
    assert_token!("public", Reserved(ReservedKind::Public), "ReservedKind::Public");
}


#[test]
fn test_tokenizer_whitespace() {
    assert_token!("", EndOfProgram, "empty string");
    assert_token!("  ", EndOfProgram, "whitespaces");
    assert_token!("\n\n\n  ", EndOfProgram, "newlines");
    assert_token!("//Comment\n//Comment", EndOfProgram, "single-line comment");
    assert_token!("/**\n  * Comment\n  */", EndOfProgram, "multi-line comment");
}
