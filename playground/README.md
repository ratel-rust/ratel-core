# Ratel playground

This is the source for the Ratel playground. It currently supports trying out parsing and transpilation inside the browser, using WebAssembly.

## Setup

You should have Node.js, the Rust toolchain and [wasm-pack](https://rustwasm.github.io/wasm-pack/) installed.

```
npm install
```

## Developing

To start a development server which automatically reloads:

```
npm run start
```

To build a static version in `dist/`:

```
npm run build
```
