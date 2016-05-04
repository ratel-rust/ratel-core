# HoneyBadger

WIP ES2015+ to ES5 transpiler + bundler + minifier in Rust.

Because Webpack+Babel+UglifyJS are both awesome and terrible at the same time.

## Requirements

- Rust 1.8.0

## Usage

```
$ cargo run -- -f input.js
```
This generates a new file called `out.js`.
