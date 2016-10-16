#[macro_use]
extern crate neon;
extern crate ratel;

use neon::vm::{Call, JsResult};
use neon::js::{JsString, JsBoolean};
use neon::js::error::{JsError, Kind};
use ratel::{parser, transformer, codegen};

fn transform(call: Call) -> JsResult<JsString> {
    let scope = call.scope;

    if call.arguments.len() == 0 {
        return JsError::throw(Kind::TypeError, "First argument must be a string")
    }

    let source = try!(try!(call.arguments.require(scope, 0)).check::<JsString>());
    let minify = try!(try!(call.arguments.require(scope, 1)).check::<JsBoolean>());

    let mut ast = match parser::parse(source.value()) {
        Err(error) => {
            let str = format!("{}", error);
            return JsError::throw(Kind::SyntaxError, &str)
        },
        Ok(ast) => ast,
    };

    transformer::transform(&mut ast, transformer::Settings::target_es5());
    let out = codegen::generate_code(&ast, minify.value());

    Ok(JsString::new(scope, &out).unwrap())
}

fn parse(call: Call) -> JsResult<JsString> {
    let scope = call.scope;

    if call.arguments.len() == 0 {
        return JsError::throw(Kind::TypeError, "First argument must be a string")
    }

    let source = try!(try!(call.arguments.require(scope, 0)).check::<JsString>());

    let ast = match parser::parse(source.value()) {
        Err(error) => {
            let str = format!("{}", error);
            return JsError::throw(Kind::SyntaxError, &str)
        },
        Ok(ast) => ast,
    };

    let out = format!("{:#?}", ast.body);

    Ok(JsString::new(scope, &out).unwrap())
}

register_module!(m, {
    try!(m.export("transform", transform));
    try!(m.export("parse", parse));
    Ok(())
});
