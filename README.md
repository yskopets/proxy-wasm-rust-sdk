# WebAssembly for Proxies (Rust SDK)

[![Build Status][build-badge]][build-link]
[![Crate][crate-badge]][crate-link]
[![Documentation][docs-badge]][docs-link]
[![Apache 2.0 License][license-badge]][license-link]


[build-badge]: https://github.com/yskopets/proxy-wasm-rust-sdk/workflows/Rust/badge.svg?branch=proxy-wasm-spec-0.1.0
[build-link]: https://github.com/yskopets/proxy-wasm-rust-sdk/actions?query=workflow%3ARust+branch%3Aproxy-wasm-spec-0.1.0
[crate-badge]: https://img.shields.io/crates/v/proxy-wasm-experimental.svg
[crate-link]: https://crates.io/crates/proxy-wasm-experimental
[docs-badge]: https://docs.rs/proxy-wasm-experimental/badge.svg
[docs-link]: https://docs.rs/proxy-wasm-experimental
[license-badge]: https://img.shields.io/github/license/proxy-wasm/proxy-wasm-rust-sdk
[license-link]: https://github.com/proxy-wasm/proxy-wasm-rust-sdk/blob/master/LICENSE

## Examples

+ [Hello World](./examples/hello_world.rs)
+ [HTTP Auth random](./examples/http_auth_random.rs)
+ [HTTP Headers](./examples/http_headers.rs)

## Articles & blog posts from the community

+ [Extending Envoy with WASM and Rust](https://antweiss.com/blog/extending-envoy-with-wasm-and-rust/)
+ [Extending Istio with Rust and WebAssembly](https://blog.red-badger.com/extending-istio-with-rust-and-webassembly)

## Updating dependencies

When updating dependencies, you need to regenerate Bazel `BUILD` files to match updated `Cargo.toml`:
```
cargo install cargo-raze --version 0.11.0
cargo raze --generate-lockfile
```
