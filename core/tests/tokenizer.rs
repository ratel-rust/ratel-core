extern crate ratel;

pub use ratel::*;
pub use ratel::grammar::*;
pub use ratel::tokenizer::*;
pub use ratel::lexicon::Token;
pub use ratel::lexicon::ReservedKind;
pub use ratel::lexicon::Token::*;
pub use ratel::owned_slice::OwnedSlice;

fn test_token(input: &str, expected: Token) -> bool {
    let mut tokenizer = Tokenizer::new(&input);
    let tok = tokenizer.get_token().unwrap();
    if tok != expected {
        println!("\n{:?}\n", tok);
    }
    tok == expected
}

macro_rules! assert_token {
    ($string:expr, $token:expr, $descr:expr) => {
        assert_eq!(test_token($string, $token), true, $descr);
    }
}

#[test]
fn test_tokenizer_chars() {
    assert_token!(";", Token::Semicolon, "read a Token::Semicolon");
    assert_token!(":", Token::Colon, "read a Token::Colon");
    assert_token!(",", Token::Comma, "read a Token::Comma");
    assert_token!("(", Token::ParenOpen, "read a Token::ParenOpen");
    assert_token!(")", Token::ParenClose, "read a Token::ParenClose");
    assert_token!("[", Token::BracketOpen, "read a Token::BracketOpen");
    assert_token!("]", Token::BracketClose, "read a Token::BracketClose");
    assert_token!("{", Token::BraceOpen, "read a Token::BraceOpen");
    assert_token!("}", Token::BraceClose, "read a Token::BraceClose");
}

#[test]
fn test_tokenizer_control_statements() {
    assert_token!("break", Token::Break, "read a Token::Break");
    assert_token!("do", Token::Do, "read a Token::Do");
    assert_token!("case", Token::Case, "read a Token::Case");
    assert_token!("else", Token::Else, "read a Token::Else");
    assert_token!("catch", Token::Catch, "read a Token::Catch");
    assert_token!("export", Token::Export, "read a Token::Export");
    assert_token!("class", Token::Class, "read a Token::Class");
    assert_token!("extends", Token::Extends, "read a Token::Extends");
    assert_token!("return", Token::Return, "read a Token::Return");
    assert_token!("while", Token::While, "read a Token::While");
    assert_token!("finally", Token::Finally, "read a Token::Finally");
    assert_token!("super", Token::Super, "read a Token::Super");
    assert_token!("with", Token::With, "read a Token::With");
    assert_token!("continue", Token::Continue, "read a Token::Continue");
    assert_token!("for", Token::For, "read a Token::For");
    assert_token!("switch", Token::Switch, "read a Token::Switch");
    assert_token!("yield", Token::Yield, "read a Token::Yield");
    assert_token!("debugger", Token::Debugger, "read a Token::Debugger");
    assert_token!("function", Token::Function, "read a Token::Function");
    assert_token!("this", Token::This, "read a Token::This");
    assert_token!("default", Token::Default, "read a Token::Default");
    assert_token!("if", Token::If, "read a Token::If");
    assert_token!("throw", Token::Throw, "read a Token::Throw");
    assert_token!("import", Token::Import, "read a Token::Import");
    assert_token!("try", Token::Try, "read a Token::Try");
}

#[test]
fn test_tokenizer_operators() {
    assert_token!("=>", Token::Operator(OperatorType::FatArrow), "OperatorType::FatArrow");
    assert_token!(".", Token::Operator(OperatorType::Accessor), "OperatorType::Accessor");
    assert_token!("new", Token::Operator(OperatorType::New), "OperatorType::New");
    assert_token!("++", Token::Operator(OperatorType::Increment), "OperatorType::Increment");
    assert_token!("--", Token::Operator(OperatorType::Decrement), "OperatorType::Decrement");
    assert_token!("!", Token::Operator(OperatorType::LogicalNot), "OperatorType::LogicalNot");
    assert_token!("~", Token::Operator(OperatorType::BitwiseNot), "OperatorType::BitwiseNot");
    assert_token!("typeof", Token::Operator(OperatorType::Typeof), "OperatorType::Typeof");
    assert_token!("void", Token::Operator(OperatorType::Void), "OperatorType::Void");
    assert_token!("delete", Token::Operator(OperatorType::Delete), "OperatorType::Delete");
    assert_token!("*", Token::Operator(OperatorType::Multiplication), "OperatorType::Multiplication");
    assert_token!("/", Token::Operator(OperatorType::Division), "OperatorType::Division");
    assert_token!("%", Token::Operator(OperatorType::Remainder), "OperatorType::Remainder");
    assert_token!("**", Token::Operator(OperatorType::Exponent), "OperatorType::Exponent");
    assert_token!("+", Token::Operator(OperatorType::Addition), "OperatorType::Addition");
    assert_token!("-", Token::Operator(OperatorType::Substraction), "OperatorType::Substraction");
    assert_token!("<<", Token::Operator(OperatorType::BitShiftLeft), "OperatorType::BitShiftLeft");
    assert_token!(">>", Token::Operator(OperatorType::BitShiftRight), "OperatorType::BitShiftRight");
    assert_token!(">>>", Token::Operator(OperatorType::UBitShiftRight), "OperatorType::UBitShiftRight");
    assert_token!("<", Token::Operator(OperatorType::Lesser), "OperatorType::Lesser");
    assert_token!("<=", Token::Operator(OperatorType::LesserEquals), "OperatorType::LesserEquals");
    assert_token!(">", Token::Operator(OperatorType::Greater), "OperatorType::Greater");
    assert_token!(">=", Token::Operator(OperatorType::GreaterEquals), "OperatorType::GreaterEquals");
    assert_token!("instanceof", Token::Operator(OperatorType::Instanceof), "OperatorType::Instanceof");
    assert_token!("in", Token::Operator(OperatorType::In), "OperatorType::In");
    assert_token!("===", Token::Operator(OperatorType::StrictEquality), "OperatorType::StrictEquality");
    assert_token!("!==", Token::Operator(OperatorType::StrictInequality), "OperatorType::StrictInequality");
    assert_token!("==", Token::Operator(OperatorType::Equality), "OperatorType::Equality");
    assert_token!("!=", Token::Operator(OperatorType::Inequality), "OperatorType::Inequality");
    assert_token!("&", Token::Operator(OperatorType::BitwiseAnd), "OperatorType::BitwiseAnd");
    assert_token!("^", Token::Operator(OperatorType::BitwiseXor), "OperatorType::BitwiseXor");
    assert_token!("|", Token::Operator(OperatorType::BitwiseOr), "OperatorType::BitwiseOr");
    assert_token!("&&", Token::Operator(OperatorType::LogicalAnd), "OperatorType::LogicalAnd");
    assert_token!("||", Token::Operator(OperatorType::LogicalOr), "OperatorType::LogicalOr");
    assert_token!("?", Token::Operator(OperatorType::Conditional), "OperatorType::Conditional");
    assert_token!("=", Token::Operator(OperatorType::Assign), "OperatorType::Assign");
    assert_token!("+=", Token::Operator(OperatorType::AddAssign), "OperatorType::AddAssign");
    assert_token!("-=", Token::Operator(OperatorType::SubstractAssign), "OperatorType::SubstractAssign");
    assert_token!("**=", Token::Operator(OperatorType::ExponentAssign), "OperatorType::ExponentAssign");
    assert_token!("*=", Token::Operator(OperatorType::MultiplyAssign), "OperatorType::MultiplyAssign");
    assert_token!("/=", Token::Operator(OperatorType::DivideAssign), "OperatorType::DivideAssign");
    assert_token!("%=", Token::Operator(OperatorType::RemainderAssign), "OperatorType::RemainderAssign");
    assert_token!("<<=", Token::Operator(OperatorType::BSLAssign), "OperatorType::BSLAssign");
    assert_token!(">>=", Token::Operator(OperatorType::BSRAssign), "OperatorType::BSRAssign");
    assert_token!(">>>=", Token::Operator(OperatorType::UBSRAssign), "OperatorType::UBSRAssign");
    assert_token!("&=", Token::Operator(OperatorType::BitAndAssign), "OperatorType::BitAndAssign");
    assert_token!("^=", Token::Operator(OperatorType::BitXorAssign), "OperatorType::BitXorAssign");
    assert_token!("|=", Token::Operator(OperatorType::BitOrAssign), "OperatorType::BitOrAssign");
    assert_token!("...", Token::Operator(OperatorType::Spread), "OperatorType::Spread");
}

#[test]
fn test_tokenizer_literals() {
    assert_token!("undefined", Token::Literal(Value::Undefined), "Token::Value::Undefined");
    assert_token!("null", Token::Literal(Value::Null), "Token::Value::Null");
    assert_token!("true", Token::Literal(Value::True), "Token::Value::True");
    assert_token!("false", Token::Literal(Value::False), "Token::Value::False");

    assert_token!("'foo'", Token::Literal(Value::String("'foo'".into())), "Token::Value::String");
    assert_token!("\"foo\"", Token::Literal(Value::String("\"foo\"".into())), "Token::Value::String");

    assert_token!("2.2", Token::Literal(Value::Number("2.2".into())), "Token::Value::Number");
    assert_token!("2", Token::Literal(Value::Number("2".into())), "Token::Value::Number");

    assert_token!("0xff", Token::Literal(Value::Integer(255)), "Token::Value::Integer");
    assert_token!("0XFF", Token::Literal(Value::Integer(255)), "Token::Value::Integer");
    assert_token!("0b01001011", Token::Literal(Value::Integer(75)), "Token::Value::Integer");
    assert_token!("0B01001011", Token::Literal(Value::Integer(75)), "Token::Value::Integer");
    assert_token!("0o113", Token::Literal(Value::Integer(75)), "Token::Value::Integer");
    assert_token!("0O113", Token::Literal(Value::Integer(75)), "Token::Value::Integer");
}

#[test]
fn test_tokenizer_reserved() {
    assert_token!("enum", Token::Reserved(ReservedKind::Enum), "ReservedKind::Enum");
    assert_token!("implements", Token::Reserved(ReservedKind::Implements), "ReservedKind::Implements");
    assert_token!("package", Token::Reserved(ReservedKind::Package), "ReservedKind::Package");
    assert_token!("protected", Token::Reserved(ReservedKind::Protected), "ReservedKind::Protected");
    assert_token!("interface", Token::Reserved(ReservedKind::Interface), "ReservedKind::Interface");
    assert_token!("private", Token::Reserved(ReservedKind::Private), "ReservedKind::Private");
    assert_token!("public", Token::Reserved(ReservedKind::Public), "ReservedKind::Public");
}


#[test]
fn test_tokenizer_whitespace() {
    assert_token!("", Token::EndOfProgram, "empty string");
    assert_token!("  ", Token::EndOfProgram, "whitespaces");
    assert_token!("\n\n\n  ", Token::EndOfProgram, "newlines");
    assert_token!("//Comment\n//Comment", Token::EndOfProgram, "single-line comment");
    assert_token!("/**\n  * Comment\n  */", Token::EndOfProgram, "multi-line comment");
}
