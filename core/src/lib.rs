extern crate serde;
extern crate toolshed;

pub mod ast;
pub mod module;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod codegen;
// pub mod visitor;
// pub mod transformer;
// pub mod astgen;

#[cfg(test)]
extern crate serde_json;

#[cfg(test)]
#[allow(unused_imports)]
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
extern crate pretty_assertions;
