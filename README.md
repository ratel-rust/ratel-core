# ratel-core

[![Travis CI](https://travis-ci.org/ratel-rust/ratel-core.svg)](https://travis-ci.org/ratel-rust/ratel-core)
[![Crates.io](https://img.shields.io/crates/v/ratel.svg)](https://crates.io/crates/ratel)
[![Gitter](https://img.shields.io/gitter/room/ratel-rust/Lobby.svg)](https://gitter.im/ratel-rust/Lobby)

**Ratel** is a high performance JavaScript to JavaScript compiler with a Rust core. It's goal is to take newest versions of JavaScript as input, and produce output that's compatible with older versions of the language.

This repo is split in two separate folders:

- `core` contains the main Rust codebase that does all the heavy lifting.
- `ffi` contains the Node.js wrapper around the Rust core with [Neon](http://neon.rustbridge.io/) bindings.

For common usage checkout the [ratel-cli](https://github.com/ratel-rust/ratel-cli) repo.

## Performance

While still incomplete, the Parser part of **Ratel** can run circles around even the fastest parsers built in JavaScript, here it is compared to [Esprima](http://esprima.org/).

![ratel vs esprima chart](http://terhix.com/ratel-perf-1.png)

## Contributors

This project is created and maintained by [Maciej Hirsz](https://github.com/maciejhirsz) with the help of awesome [contributors](https://github.com/ratel-rust/ratel-core/graphs/contributors). Extended thanks to:

- [cmtt](https://github.com/cmtt) for work on the Node.js FFI and testing.
- [Jan Schulte](https://github.com/schultyy) for the initial version of transformer and codegen.

## License

This code is distributed under the terms of both the MIT license
and the Apache License (Version 2.0), choose whatever works for you.

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
