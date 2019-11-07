extern crate neon;
extern crate ratel;
extern crate ratel_codegen;
extern crate serde;
extern crate serde_json;

use neon::prelude::*;

use ratel::Module;
use ratel::error::{Error, ParseError};

#[inline]
fn format_errors(errors: Vec<Error>, source: Handle<JsString>) -> Vec<String> {
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
    if minify {
        serde_json::to_string(&module)
    } else {
        serde_json::to_string_pretty(&module)
    }
}

fn ast(mut cx: FunctionContext) -> JsResult<JsString> {
    if cx.len() == 0 {
        return cx.throw_type_error("First argument must be a string")
    }

    let source = cx.argument::<JsString>(0)?;
    let minify = cx.argument::<JsBoolean>(1)?;

    let module = match ratel::parse(&source.value()) {
        Err(errors) => {
            let str = format_errors(errors, source).join("\n");
            return cx.throw_type_error(&str)
        },
        Ok(module) => module,
    };

    let result = generate_ast(&module, minify.value()).unwrap();

    Ok(cx.string(&result))
}

fn transform(mut cx: FunctionContext) -> JsResult<JsString> {
    if cx.len() == 0 {
        return cx.throw_type_error("First argument must be a string")
    }

    let source = cx.argument::<JsString>(0)?;
    let minify = cx.argument::<JsBoolean>(1)?;

    let module = match ratel::parse(&source.value()) {
        Err(errors) => {
            let str = format_errors(errors, source).join("\n");
            return cx.throw_type_error(&str)
        },
        Ok(module) => module,
    };
    // transformer::transform(&mut module, transformer::Settings::target_es5());
    let out = ratel_codegen::codegen(&module, minify.value());

    Ok(cx.string(&out))
}

fn parse(mut cx: FunctionContext) -> JsResult<JsString> {
    if cx.len() == 0 {
        return cx.throw_type_error("First argument must be a string")
    }

    let source = cx.argument::<JsString>(0)?;

    let module = match ratel::parse(&source.value()) {
        Err(errors) => {
            let str = format_errors(errors, source).join("\n");
            return cx.throw_type_error(&str)
        },
        Ok(module) => module,
    };

    let out = format!("{:?}", module.body());

    Ok(cx.string(&out))
}

register_module!(mut cx, {
    cx.export_function("transform", transform)?;
    cx.export_function("parse", parse)?;
    cx.export_function("ast", ast)?;
    Ok(())
});
