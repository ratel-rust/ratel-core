#![recursion_limit="128"]
#![cfg_attr(feature = "cargo-clippy", allow(trivially_copy_pass_by_ref))]

extern crate serde;
extern crate toolshed;

#[macro_use]
extern crate serde_derive;

#[cfg(test)]
#[macro_use]
extern crate serde_json;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

pub mod ast;
pub mod error;
pub mod lexer;

mod module;
mod parser;
mod astgen;

pub use parser::parse;
pub use module::Module;
