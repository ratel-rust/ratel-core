extern crate ratel;

pub use ratel::*;
pub use ratel::grammar::*;
pub use ratel::parser::parse;
pub use ratel::grammar::OperatorType::*;

fn output_program(input_program: &str) -> String {
    let mut ast = parser::parse(input_program.to_string()).expect("Must compile");
    transformer::transform(&mut ast, transformer::Settings::target_es5());
    codegen::generate_code(&ast, true)
}

macro_rules! assert_compile {
    ($string:expr, $expect:expr) => {
        assert_eq!(output_program($string), $expect.to_string());
    }
}

#[test]
fn convert_const_to_var_in_global_scope() {
    assert_compile!("const pi = 3.14;", "var pi=3.14;");
}

#[test]
fn convert_let_to_var_in_global_scope() {
    assert_compile!("let pi = 3.14;", "var pi=3.14;");
}

#[test]
fn dont_touch_var_in_global_scope() {
    assert_compile!("var pi = 3.14;", "var pi=3.14;");
}

#[test]
fn convert_let_to_var_in_block() {
    let program = "if(true) {
      let pi = 3.14;
    }";

    let expected = "if(!0){var pi=3.14;}";

    assert_compile!(program, expected);
}

#[test]
fn regex() {
    assert_compile!("/foo/gi.test();", "/foo/gi.test();");
}

#[test]
fn template_strings_plain() {
    assert_compile!("`foobar`;", r#""foobar";"#);
    assert_compile!("`foo\\`bar`;", r#""foo`bar";"#);
    assert_compile!("`foo\nbar`;", "\"foo\\nbar\";");
}

#[test]
fn operator_precedence_and_parens() {
    // Preserve parens when necessary
    assert_compile!("'foo'+(1+2);", r#"'foo'+(1+2);"#);

    // Should strip parens when not necessary
    assert_compile!("(1+2)+'foo';", r#"1+2+'foo';"#);
    assert_compile!("'foo'+(1*2);", r#"'foo'+1*2;"#);
    assert_compile!("(1*2)+'foo';", r#"1*2+'foo';"#);
}

#[test]
fn template_strings_interpolation() {
    assert_compile!("`foo${1}bar`;", r#""foo"+1+"bar";"#);
    assert_compile!("`foo${1+2}bar`;", r#""foo"+(1+2)+"bar";"#);
    assert_compile!("`foo${1*2}bar`;", r#""foo"+1*2+"bar";"#);
    assert_compile!("`foo${1}${2**2}bar`;", r#""foo"+1+Math.pow(2,2)+"bar";"#);
    assert_compile!("`foo${1}bar${2**2}`;", r#""foo"+1+"bar"+Math.pow(2,2);"#);
}

#[test]
fn template_strings_tagged() {
    assert_compile!("foo`bar`;", r#"foo(["bar"]);"#);
    assert_compile!("foo`bar${1}baz`;", r#"foo(["bar","baz"],1);"#);
    assert_compile!("foo`bar${1}${2}baz`;", r#"foo(["bar","","baz"],1,2);"#);
    assert_compile!("foo`bar${1}baz${2}`;", r#"foo(["bar","baz",""],1,2);"#);
}

#[test]
fn empty_class() {
    assert_compile!("class Foo {}", "function Foo(){}");
}

#[test]
fn class_with_constructor() {
    assert_compile!(r#"

    class Foo {
        constructor() {
            console.log("Hello world!");
        }
    }

    "#, r#"function Foo(){console.log("Hello world!");}"#);
}

#[test]
fn class_with_props() {
    assert_compile!("

    class Foo {
        foo = 100;
        bar = 200;
    }

    ", "function Foo(){this.foo=100;this.bar=200;}");
}

#[test]
fn class_with_constructor_and_props() {
    assert_compile!(r#"

    class Foo {
        foo = 100;
        bar = 200;

        constructor() {
            console.log("Hello world!");
        }
    }

    "#, r#"function Foo(){this.foo=100;this.bar=200;console.log("Hello world!");}"#);
}

#[test]
fn class_with_methods() {
    assert_compile!("

    class Foo {
        foo() {}
        bar() {}
    }

    ", "function Foo(){}Foo.prototype.foo=function(){};Foo.prototype.bar=function(){};");
}

#[test]
fn class_with_static_methods() {
    assert_compile!("

    class Foo {
        static foo() {}
        static bar() {}
    }

    ", "function Foo(){}Foo.foo=function(){};Foo.bar=function(){};");
}
