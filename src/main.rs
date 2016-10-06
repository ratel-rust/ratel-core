#![feature(box_syntax)]

extern crate docopt;
extern crate rustc_serialize;

use std::process;
use std::io::prelude::*;
use std::io::Error;
use std::fs::File;
use std::time::{ Instant, Duration };
use docopt::Docopt;

pub mod lexicon;
pub mod tokenizer;
pub mod parser;
pub mod grammar;
pub mod transformer;
pub mod codegen;

fn print_ms(label: &str, duration: &Duration) {
    let delta = ((duration.as_secs() as f64) * 1000.0) +
                (duration.subsec_nanos() as f64) / 1_000_000.0;

    println!("{} {}ms", label, delta);
}

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const USAGE: &'static str = "
honeybadger

Usage:
  badger [options]
  badger --version

Options:
  -h --help                 Show this screen.
  --version                 Show version.
  -f FILE, --file=FILE      Specifies the input file.
  -o FILE, --output=FILE    Specifies the output file.
  --pretty                  Don't minify the output.
  --ast                     Print out the Abstract Syntax Tree of the input.
";

fn read_file(path: &str) -> Result<String, Error> {
    let mut f = try!(File::open(path));
    let mut s = String::new();
    match f.read_to_string(&mut s) {
        Ok(_) => Ok(s),
        Err(err) => Err(err)
    }
}

fn write_file(filename: &str, program: String) -> Result<(), Error> {
    let mut f = try!(File::create(filename));
    match f.write_all(&program.into_bytes()[..]) {
        Ok(_) => Ok(()),
        Err(err) => Err(err)
    }
}

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_file: Option<String>,
    flag_output: Option<String>,
    flag_version: bool,
    flag_ast: bool,
    flag_pretty: bool,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("v{}", VERSION);
        process::exit(0);
    }

    if args.flag_file.is_none() {
        println!("{}", USAGE);
        process::exit(0);
    }

    let file = match read_file(&args.flag_file.unwrap()) {
        Ok(file) => file,
        Err(err) => {
            println!("ERR Couldn't read file: {:?}", err);
            process::exit(1)
        }
    };

    let start = Instant::now();
    let mut ast = parser::parse(file);
    let parse_duration = Instant::now().duration_since(start);

    if args.flag_ast {
        println!("{:#?}", ast);
        print_ms("Parsing", &parse_duration);
        process::exit(0);
    }

    let start = Instant::now();
    transformer::transform(&mut ast, transformer::Settings::target_es5());
    let transform_duration = Instant::now().duration_since(start);

    let start = Instant::now();
    let program = codegen::generate_code(ast, !args.flag_pretty);
    let codegen_duration = Instant::now().duration_since(start);

    if args.flag_output.is_none() {
        println!("{}", program);
        print_ms("Parsing        ", &parse_duration);
        print_ms("Transformation ", &transform_duration);
        print_ms("Code generation", &codegen_duration);

        process::exit(0);
    }

    match write_file(&args.flag_output.unwrap(), program) {
        Ok(()) => {
            print_ms("Parsing        ", &parse_duration);
            print_ms("Transformation ", &transform_duration);
            print_ms("Code generation", &codegen_duration);
        },
        Err(err) => {
            println!("ERR Writing out.js {}", err);
            process::exit(1);
        }
    }
}
