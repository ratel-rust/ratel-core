extern crate ratel;

use ratel::{ transformer, parser, codegen };


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
fn keyword_prefix_operators() {
    assert_compile!("delete a;", "delete a;");
    assert_compile!("new a;", "new a;");
    assert_compile!("typeof a;", "typeof a;");
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
fn exponent_in_sequence() {
    assert_compile!("(1, 2 ** 2)", "(1,Math.pow(2,2));");
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

#[test]
fn class_expression() {
    assert_compile!("

    const Foo = class {
        constructor() {
            console.log('to the moon');
        }
    };

    ", "var Foo=function(){console.log('to the moon');};");
}

#[test]
fn named_class_expression() {
    assert_compile!("

    const Foo = class Bar {
        constructor() {
            console.log('to the moon');
        }
    };

    ", "var Foo=function Bar(){console.log('to the moon');};");
}

#[test]
fn class_expression_with_a_method() {
    assert_compile!("

    const Foo = class {
        bar() {
            console.log('to the moon');
        }
    };

    ", "var Foo=function(){function ___(){}___.prototype.bar=function(){console.log('to the moon');};return ___;}();")
}

#[test]
fn function_statement_default_parameters() {
    assert_compile!("function foo(a,b=1){}", "function foo(a,b){b===undefined&&(b=1);}");
}

#[test]
fn function_expression_default_parameters() {
    assert_compile!("(function(a,b=1){})", "(function(a,b){b===undefined&&(b=1);});");
}

#[test]
fn arrow_function_default_parameters() {
    assert_compile!("(a,b=1)=>{}", "(function(a,b){b===undefined&&(b=1);});");
}

#[test]
fn object_method_default_parameters() {
    assert_compile!("({foo(a,b=1){}})", "({foo:function(a,b){b===undefined&&(b=1);}});");
}

#[test]
fn class_method_default_parameters() {
    assert_compile!("class Foo{bar(a,b=1){}}", "function Foo(){}Foo.prototype.bar=function(a,b){b===undefined&&(b=1);};")
}

#[test]
fn try_catch() {
    assert_compile!("try{foo();}catch(err){console.error(err);}", "try{foo();}catch(err){console.error(err);}");
}

#[test]
fn ternary_expression() {
    assert_compile!("const a=1?console.log('foo'):null;", "var a=1?console.log('foo'):null;")
}

#[test]
fn sparse_array_expression() {
    assert_compile!("[,,1];", "[,,1];");
    assert_compile!("[,,];", "[,,];");
}

#[test]
fn destructing_array() {
    assert_compile!("var [ x, y, c ] = [ 1, 2 ]", "var _ref=[1,2],x=_ref[0],y=_ref[1],c=_ref[2];");
}

#[test]
fn destructing_array_elision() {
    assert_compile!("const [,, x, y] = ['a', 'b', 'c', 'd'];", "var _ref=['a','b','c','d'],x=_ref[2],y=_ref[3];")
}