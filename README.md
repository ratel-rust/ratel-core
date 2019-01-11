![Ratel](http://maciej.codes/things/ratel-400.png)

# ratel-core

[![Travis CI](https://travis-ci.org/ratel-rust/ratel-core.svg)](https://travis-ci.org/ratel-rust/ratel-core)
[![Crates.io](https://img.shields.io/crates/v/ratel.svg)](https://crates.io/crates/ratel)
[![Discord](https://img.shields.io/discord/530727712969588746.svg?logo=discord)](https://discord.gg/5YmRBvu)
[![Gitter](https://img.shields.io/gitter/room/ratel-rust/Lobby.svg?logo=gitter)](https://gitter.im/ratel-rust/Lobby)

**Ratel** is a high performance JavaScript to JavaScript compiler with a Rust core. Its goal is to take newest versions of JavaScript as input, and produce output that's compatible with older versions of the language.

[**Online REPL with Wasm**](http://maciej.codes/ratel-wasm/), courtesy of [cmtt](https://github.com/cmtt).

This repo is split in two separate folders:

- `core` contains the main Rust codebase that does all the heavy lifting.
- `ffi` contains the Node.js wrapper around the Rust core with [Neon](http://neon.rustbridge.io/) bindings.

For common usage checkout the [ratel-cli](https://github.com/ratel-rust/ratel-cli) repo.

## Performance

While still incomplete, the Parser part of **Ratel** can run circles around even the fastest parsers built in JavaScript, here it is compared to [Esprima](http://esprima.org/) using the ratel FFI.

![ratel chart](https://user-images.githubusercontent.com/787228/46786973-beee0c80-cd36-11e8-989a-62b92d624d38.png)

## Contributors

This project is created and maintained by [Maciej Hirsz](https://github.com/maciejhirsz) with the help of awesome [contributors](https://github.com/ratel-rust/ratel-core/graphs/contributors). Extended thanks to:

- [cmtt](https://github.com/cmtt) for work on the Node.js FFI and testing.
- [Jan Schulte](https://github.com/schultyy) for the initial version of transformer and codegen.

## Logo

The smirky **Ratel** by the courtesy of [A. L. Palmer](https://www.behance.net/alpalmer60b4).

## License

This code is distributed under the terms of both the MIT license
and the Apache License (Version 2.0), choose whatever works for you.

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
