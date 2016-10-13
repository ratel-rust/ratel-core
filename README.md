# ratel-core

JavaScript compiler core written in Rust.

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

## Performance

Parser part of `ratel` is doing exceptionally well when compared to even the fastest parsers built in JavaScript, such as [Esprima](http://esprima.org/).

![ratel vs esprima chart](http://terhix.com/ratel-perf-1.png)
