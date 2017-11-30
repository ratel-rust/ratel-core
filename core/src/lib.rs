#![recursion_limit="128"]

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
pub mod module;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod codegen;
pub mod visitor;
pub mod transformer;
pub mod astgen;
