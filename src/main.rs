use std::io::prelude::*;
use std::io::Error;
use std::fs::File;
use parser::parse;
use std::time::Instant;
use docopt::Docopt;
use std::process;

pub mod lexicon;
pub mod tokenizer;
pub mod parser;
pub mod grammar;
pub mod transformer;
extern crate docopt;
extern crate rustc_serialize;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const USAGE: &'static str = "
honeybadger

Usage:
  badger [options]
  badger --version

Options:
  -h --help              Show this screen.
  --version              Show version.
  -f FILE, --file=FILE   Specifies the input file
";

fn read_file(path: &str) -> Result<String, Error> {
    let mut f = try!(File::open(path));
    let mut s = String::new();
    match f.read_to_string(&mut s) {
        Ok(_) => Ok(s),
        Err(err) => Err(err)
    }
}

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_file: String,
    flag_version: bool
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("v{}", VERSION);
        process::exit(0);
    }

    let file = match read_file(&args.flag_file) {
        Ok(file) => file,
        Err(err) => {
            println!("ERR Couldn't read file: {:?}", err);
            process::exit(1)
        }
    };

    let ast = parser::parse(file);

    let transformed_ast = transformer::traverse(ast);
    println!("{:?}", transformed_ast);
}
