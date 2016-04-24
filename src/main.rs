use std::io::prelude::*;
use std::io::Error;
use std::fs::File;
// use tokenizer::Tokenizer;
use parser::parse;
use std::time::Instant;

pub mod lexicon;
pub mod tokenizer;
pub mod parser;
pub mod grammar;

fn parse_file(path: &str) -> Result<(), Error> {
    let mut f = try!(File::open(path));
    let mut s = String::new();
    try!(f.read_to_string(&mut s));
    let start = Instant::now();
    let program = parse(s);
    let delta = {
        let duration = Instant::now().duration_since(start);
        ((duration.as_secs() as f64) * 1000.) +
        (duration.subsec_nanos() as f64) / 1_000_000.
    };
    println!("{:#?}", program);
    println!("Took {}ms", delta);
    Ok(())
}

fn main() {
    parse_file("test.js").unwrap();
}
