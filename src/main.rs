use std::io::prelude::*;
use std::io::Error;
use std::fs::File;
use tokenizer::Tokenizer;

pub mod tokenizer;
pub mod lexicon;

fn parse_file(path: &str) -> Result<(), Error> {
    let mut f = try!(File::open(path));
    let mut s = String::new();
    try!(f.read_to_string(&mut s));
    let mut tokenizer = Tokenizer::new(&mut s);
    while let Some(token) = tokenizer.next() {
        println!("{:?}", token);
    }
    Ok(())
}

fn main() {
    parse_file("test.js").unwrap();
}
