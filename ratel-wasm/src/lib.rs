extern crate ratel;
extern crate ratel_codegen;
extern crate serde_json;
extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

use ratel::error::{Error, ParseError};

fn format_errors(errors: Vec<Error>, source: &str) -> String {
    let error = errors
    .into_iter()
    .map(|err| {
        match err {
            Error { start, end, .. } => {
               ParseError::UnexpectedToken { start, end, source: source.to_string() }
            }
        }
    })
    .map(|err| format!("{}", err))
    .collect::<Vec<String>>()
    .join("\n");

	format!("Error: {}", error)
}

#[wasm_bindgen]
pub fn transform(data: &str, minify: bool) -> String {
	match ratel::parse(&data) {
		Ok(module) => {
			ratel_codegen::codegen(&module, minify)
		},
		Err(errors) => format_errors(errors, data)
	}
}

#[wasm_bindgen(js_name = generateAST)]
pub fn generate_ast(data: &str, minify: bool) -> String {
	match ratel::parse(&data) {
		Ok(module) => {
			if minify {
		    	format!("{:?}", module.body())
			} else {
		    	format!("{:#?}", module.body())
			}
		},
		Err(errors) => format_errors(errors, data)
	}
}

#[wasm_bindgen(js_name = generateASTEstree)]
pub fn generate_ast_estree(data: &str, minify: bool) -> String {
	match ratel::parse(&data) {
		Ok(module) => {
			if minify {
				serde_json::to_string(&module).unwrap()
			} else {
				serde_json::to_string_pretty(&module).unwrap()
			}
		},
		Err(errors) => format_errors(errors, data)
	}
}
