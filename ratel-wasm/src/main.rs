extern crate ratel;
extern crate ratel_codegen;
extern crate serde_json;

use std::os::raw::c_char;
use std::ffi::CStr;
use std::ffi::CString;
use ratel::error::{Error, ParseError};

fn main() {}

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

// TODO: convert to unsafe function
#[cfg_attr(feature = "cargo-clippy", allow(not_unsafe_ptr_arg_deref))]

#[no_mangle]
pub fn transform(i: *const c_char, minify: bool) -> *const c_char {
	let data = unsafe {
        CStr::from_ptr(i).to_str().unwrap()
    };

	let result = match ratel::parse(&data) {
		Ok(module) => {
			ratel_codegen::codegen(&module, minify)
		},
		Err(errors) => format_errors(errors, data)
	};

	CString::new(result.as_str()).unwrap().into_raw()
}

// TODO: convert to unsafe function
#[cfg_attr(feature = "cargo-clippy", allow(not_unsafe_ptr_arg_deref))]

#[no_mangle]
pub fn generate_ast(i: *const c_char, minify: bool) -> *const c_char {
	let data = unsafe {
        CStr::from_ptr(i).to_str().unwrap()
    };

	let result = match ratel::parse(&data) {
		Ok(module) => {
			if minify {
		    	format!("{:?}", module.body())
			} else {
		    	format!("{:#?}", module.body())
			}
		},
		Err(errors) => format_errors(errors, data)
	};

	CString::new(result.as_str()).unwrap().into_raw()
}

// TODO: convert to unsafe function
#[cfg_attr(feature = "cargo-clippy", allow(not_unsafe_ptr_arg_deref))]

#[no_mangle]
pub fn generate_ast_estree(i: *const c_char, minify: bool) -> *const c_char {
	let data = unsafe {
        CStr::from_ptr(i).to_str().unwrap()
    };

	let result = match ratel::parse(&data) {
		Ok(module) => {
			if minify {
				serde_json::to_string(&module).unwrap()
			} else {
				serde_json::to_string_pretty(&module).unwrap()
			}
		},
		Err(errors) => format_errors(errors, data)
	};

	CString::new(result.as_str()).unwrap().into_raw()
}
