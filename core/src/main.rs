extern crate docopt;
extern crate rustc_serialize;

use std::process;
use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::time::{ Instant, Duration };
use docopt::Docopt;
use error::ParseError;

pub mod error;
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
  -h --help                    Show this screen.
  --version                    Show version.
  -e SCRIPT, --string=SCRIPT   Specifies an input string.
  -f FILE, --file=FILE         Specifies the input file.
  -o FILE, --output=FILE       Specifies the output file.
  --pretty                     Don't minify the output.
  --ast                        Print out the Abstract Syntax Tree of the input.
";

fn read_file(path: &str) -> Result<String, io::Error> {
    let mut f = try!(File::open(path));
    let mut s = String::new();
    match f.read_to_string(&mut s) {
        Ok(_) => Ok(s),
        Err(err) => Err(err)
    }
}

fn write_file(filename: &str, program: String) -> Result<(), io::Error> {
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
    flag_string: Option<String>,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("v{}", VERSION);
        process::exit(0);
    }

    let source = match args.flag_string {
        Some(source) => source,
        None => {
            match args.flag_file {
                Some(path) => {
                    read_file(&path).unwrap_or_else(|err| {
                        println!("ERR Couldn't read file: {:?}", err);
                        process::exit(1);
                    })
                },
                None => {
                    println!("{}", USAGE);
                    process::exit(0);
                }
            }
        }
    };

    let start = Instant::now();
    let result = parser::parse(source);
    let parse_duration = Instant::now().duration_since(start);

    let mut ast = match result {
        Err(ParseError::UnexpectedEndOfProgram) => {
            println!("Unexpected end of program");

            process::exit(1);
        },
        Err(ParseError::UnexpectedToken { source, start, end }) => {
            let (lineno, line) = source[..start]
                                   .lines()
                                   .enumerate()
                                   .last()
                                   .expect("Must always have at least one line.");

            let colno = line.chars().count();
            let token_len = source[start..end].chars().count();

            println!("Unexpected token at {}:{}\n", lineno + 1, colno + 1);

            let iter = source
                        .lines()
                        .enumerate()
                        .skip_while(|&(index, _)| index < lineno.saturating_sub(2))
                        .take_while(|&(index, _)| index < lineno + 3);

            for (index, line) in iter {
                if index == lineno {
                    println!("> {:4} | {}", index+1, line);

                    let mut marker = String::with_capacity(line.len() + 9);

                    marker.push_str("       | ");

                    for _ in 0..colno {
                        marker.push(' ');
                    }

                    for _ in 0..token_len {
                        marker.push('^');
                    }

                    println!("{}", marker);

                } else {
                    println!("{:6} | {}", index+1, line);
                }
            }

            process::exit(1);
        },
        Ok(ast) => ast,
    };

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
