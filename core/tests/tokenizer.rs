extern crate ratel;

pub use ratel::*;
pub use ratel::grammar::*;
pub use ratel::tokenizer::*;
pub use ratel::lexicon::Token;
pub use ratel::lexicon::Token::*;
pub use ratel::lexicon::ReservedKind;
pub use ratel::owned_slice::OwnedSlice;

fn test_token(input: &str, expected: Token) -> bool {
    let mut tokenizer = Tokenizer::new(&input);
    let tok = tokenizer.get_token().unwrap();
    if tok != expected {
        println!("\n{:?}\n", tok);
    }
    tok == expected
}

macro_rules! num {
    ($num:expr) => (Literal(Value::Number($num.into())))
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
    assert_token!("=>", Operator(OperatorType::FatArrow), "OperatorType::FatArrow");
    assert_token!(".", Operator(OperatorType::Accessor), "OperatorType::Accessor");
    assert_token!("new", Operator(OperatorType::New), "OperatorType::New");
    assert_token!("++", Operator(OperatorType::Increment), "OperatorType::Increment");
    assert_token!("--", Operator(OperatorType::Decrement), "OperatorType::Decrement");
    assert_token!("!", Operator(OperatorType::LogicalNot), "OperatorType::LogicalNot");
    assert_token!("~", Operator(OperatorType::BitwiseNot), "OperatorType::BitwiseNot");
    assert_token!("typeof", Operator(OperatorType::Typeof), "OperatorType::Typeof");
    assert_token!("void", Operator(OperatorType::Void), "OperatorType::Void");
    assert_token!("delete", Operator(OperatorType::Delete), "OperatorType::Delete");
    assert_token!("*", Operator(OperatorType::Multiplication), "OperatorType::Multiplication");
    assert_token!("/", Operator(OperatorType::Division), "OperatorType::Division");
    assert_token!("%", Operator(OperatorType::Remainder), "OperatorType::Remainder");
    assert_token!("**", Operator(OperatorType::Exponent), "OperatorType::Exponent");
    assert_token!("+", Operator(OperatorType::Addition), "OperatorType::Addition");
    assert_token!("-", Operator(OperatorType::Substraction), "OperatorType::Substraction");
    assert_token!("<<", Operator(OperatorType::BitShiftLeft), "OperatorType::BitShiftLeft");
    assert_token!(">>", Operator(OperatorType::BitShiftRight), "OperatorType::BitShiftRight");
    assert_token!(">>>", Operator(OperatorType::UBitShiftRight), "OperatorType::UBitShiftRight");
    assert_token!("<", Operator(OperatorType::Lesser), "OperatorType::Lesser");
    assert_token!("<=", Operator(OperatorType::LesserEquals), "OperatorType::LesserEquals");
    assert_token!(">", Operator(OperatorType::Greater), "OperatorType::Greater");
    assert_token!(">=", Operator(OperatorType::GreaterEquals), "OperatorType::GreaterEquals");
    assert_token!("instanceof", Operator(OperatorType::Instanceof), "OperatorType::Instanceof");
    assert_token!("in", Operator(OperatorType::In), "OperatorType::In");
    assert_token!("===", Operator(OperatorType::StrictEquality), "OperatorType::StrictEquality");
    assert_token!("!==", Operator(OperatorType::StrictInequality), "OperatorType::StrictInequality");
    assert_token!("==", Operator(OperatorType::Equality), "OperatorType::Equality");
    assert_token!("!=", Operator(OperatorType::Inequality), "OperatorType::Inequality");
    assert_token!("&", Operator(OperatorType::BitwiseAnd), "OperatorType::BitwiseAnd");
    assert_token!("^", Operator(OperatorType::BitwiseXor), "OperatorType::BitwiseXor");
    assert_token!("|", Operator(OperatorType::BitwiseOr), "OperatorType::BitwiseOr");
    assert_token!("&&", Operator(OperatorType::LogicalAnd), "OperatorType::LogicalAnd");
    assert_token!("||", Operator(OperatorType::LogicalOr), "OperatorType::LogicalOr");
    assert_token!("?", Operator(OperatorType::Conditional), "OperatorType::Conditional");
    assert_token!("=", Operator(OperatorType::Assign), "OperatorType::Assign");
    assert_token!("+=", Operator(OperatorType::AddAssign), "OperatorType::AddAssign");
    assert_token!("-=", Operator(OperatorType::SubstractAssign), "OperatorType::SubstractAssign");
    assert_token!("**=", Operator(OperatorType::ExponentAssign), "OperatorType::ExponentAssign");
    assert_token!("*=", Operator(OperatorType::MultiplyAssign), "OperatorType::MultiplyAssign");
    assert_token!("/=", Operator(OperatorType::DivideAssign), "OperatorType::DivideAssign");
    assert_token!("%=", Operator(OperatorType::RemainderAssign), "OperatorType::RemainderAssign");
    assert_token!("<<=", Operator(OperatorType::BSLAssign), "OperatorType::BSLAssign");
    assert_token!(">>=", Operator(OperatorType::BSRAssign), "OperatorType::BSRAssign");
    assert_token!(">>>=", Operator(OperatorType::UBSRAssign), "OperatorType::UBSRAssign");
    assert_token!("&=", Operator(OperatorType::BitAndAssign), "OperatorType::BitAndAssign");
    assert_token!("^=", Operator(OperatorType::BitXorAssign), "OperatorType::BitXorAssign");
    assert_token!("|=", Operator(OperatorType::BitOrAssign), "OperatorType::BitOrAssign");
    assert_token!("...", Operator(OperatorType::Spread), "OperatorType::Spread");
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

    assert_token!("0xff", Literal(Value::Number("255".into())), "Value::Number");
    assert_token!("0xff", num!("255"), "Value::Number");
    assert_token!("0XFF", num!("255"), "Value::Number");
    assert_token!("0b01001011", num!("75"), "Value::Number");
    assert_token!("0B01001011", num!("75"), "Value::Number");
    assert_token!("0o113", num!("75"), "Value::Number");
    assert_token!("0O113", num!("75"), "Value::Number");
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
