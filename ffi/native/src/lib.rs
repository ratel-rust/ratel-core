#[macro_use]
extern crate neon;
extern crate ratel;
extern crate serde;
extern crate serde_json;

use neon::vm::{Call, JsResult};
use neon::js::{JsString, JsBoolean};
use neon::js::error::{JsError, Kind};
use ratel::{parser, codegen, error, transformer};
use error::{Error, ParseError};
use ratel::module::Module;
use ratel::astgen::*;

#[inline]
fn format_errors(errors: Vec<Error>, source: neon::mem::Handle<JsString>) -> Vec<String> {
    errors
    .into_iter()
    .map(|err| {
        match err {
            Error { start, end, .. } => {
               ParseError::UnexpectedToken { start, end, source: source.value() }
            }
        }
    })
    .map(|err| format!("{}", err))
    .collect()
}

#[inline]
fn generate_ast(module: &Module, minify: bool) -> Result<String, serde_json::Error> {
    let program = Program {
        body: &module.body()
    };
    if minify {
        serde_json::to_string(&program)
    } else {
        serde_json::to_string_pretty(&program)
    }
}

fn ast(call: Call) -> JsResult<JsString> {
    let scope = call.scope;

    if call.arguments.len() == 0 {
        return JsError::throw(Kind::TypeError, "First argument must be a string")
    }

    let source = call.arguments.require(scope, 0)?.check::<JsString>()?;
    let minify = call.arguments.require(scope, 1)?.check::<JsBoolean>()?;

    let module = match parser::parse(&source.value()) {
        Err(errors) => {
            let str = format_errors(errors, source).join("\n");
            return JsError::throw(Kind::SyntaxError, &str)
        },
        Ok(module) => module,
    };

    let result = generate_ast(&module, minify.value()).unwrap();

    Ok(JsString::new(scope, &result).unwrap())
}

fn transform(call: Call) -> JsResult<JsString> {
    let scope = call.scope;

    if call.arguments.len() == 0 {
        return JsError::throw(Kind::TypeError, "First argument must be a string")
    }

    let source = call.arguments.require(scope, 0)?.check::<JsString>()?;
    let minify = call.arguments.require(scope, 1)?.check::<JsBoolean>()?;

    let mut module = match parser::parse(&source.value()) {
        Err(errors) => {
            let str = format_errors(errors, source).join("\n");
            return JsError::throw(Kind::SyntaxError, &str)
        },
        Ok(module) => module,
    };
    // transformer::transform(&mut module, transformer::Settings::target_es5());
    let out = codegen::codegen(&module, minify.value());

    Ok(JsString::new(scope, &out).unwrap())
}

fn parse(call: Call) -> JsResult<JsString> {
    let scope = call.scope;

    if call.arguments.len() == 0 {
        return JsError::throw(Kind::TypeError, "First argument must be a string")
    }

    let source = call.arguments.require(scope, 0)?.check::<JsString>()?;

    let module = match parser::parse(&source.value()) {
        Err(errors) => {
            let str = format_errors(errors, source).join("\n");
            return JsError::throw(Kind::SyntaxError, &str)
        },
        Ok(module) => module,
    };

    let out = format!("{:?}", module.body());

    Ok(JsString::new(scope, &out).unwrap())
}

register_module!(m, {
    m.export("transform", transform)?;
    m.export("parse", parse)?;
    m.export("ast", ast)?;
    Ok(())
});
