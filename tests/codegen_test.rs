#[path="../src/grammar.rs"]
mod grammar;
#[path="../src/parser.rs"]
mod parser;
#[path="../src/codegen.rs"]
mod codegen;
#[path="../src/lexicon.rs"]
mod lexicon;
#[path="../src/tokenizer.rs"]
mod tokenizer;

#[test]
fn convert_const_to_var() {
    let program = parser::parse("const foo = \"bar\";".into());
    let output_program = codegen::generate_code(program);
    assert!(output_program.is_ok());
    assert_eq!(output_program, Ok("var foo = \"bar\";".into()));
}

#[test]
fn convert_var_to_var() {
    let program = parser::parse("var foo = \"bar\";".into());
    let output_program = codegen::generate_code(program);
    assert!(output_program.is_ok());
    assert_eq!(output_program, Ok("var foo = \"bar\";".into()));
}
