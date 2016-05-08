# HoneyBadger

WIP ES2015+ to ES5 transpiler + bundler + minifier in Rust.

Because Webpack+Babel+UglifyJS are both awesome and terrible at the same time.

## Requirements

- Rust 1.8.0

## Usage

To print out the compiled code to stdout:
```
$ cargo run -- -f input.js
```

To compile to a file:
```
$ cargo run -- -f input.js -o output.js
```

## Things that work:

* A basic pipeline for parsing, transofrmation and code generation.
* Can parse and code gen a large chunk of ES2015+ syntax (not all yet, but
  getting there).
* The transformer can turn arrow functions into regular function expressions,
  adding `.bind(this)` when necessary.
* Object shorthand as well as computed properties get transmuted to ES5.

## Things that are missing:

* Keep track of location of tokens and later on AST constructs in the original
  source code.
* Meaningful parse error reporting.
* Any sort of bundling.
* A way to configure which transformations to do, and which to skip.
* Interface with external compilers (Sass, Less, Handlebars), maybe use Neon?
* Think of ways to analize function scopes (necessary for variable name scramling).
