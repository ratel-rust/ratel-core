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

fn output_program(input_program: &str) -> Result<String, Vec<String>> {
    let program = parser::parse(input_program.into());
    codegen::generate_code(program)
}

#[test]
fn convert_const_str_to_var() {
    assert_eq!(output_program("const foo = \"bar\";"), Ok("var foo = \"bar\";".into()));
}

#[test]
fn convert_var_str_to_var() {
    assert_eq!(output_program("var foo = \"bar\";"), Ok("var foo = \"bar\";".into()));
}
