use std::io::prelude::*;
use std::io::Error;
use std::fs::File;
// use tokenizer::Tokenizer;
use parser::parse;

pub mod lexicon;
pub mod tokenizer;
pub mod parser;
pub mod literals;

fn parse_file(path: &str) -> Result<(), Error> {
    let mut f = try!(File::open(path));
    let mut s = String::new();
    try!(f.read_to_string(&mut s));
    parse(s);
    Ok(())
}

fn main() {
    parse_file("test.js").unwrap();
}
