tauri_includedir
===========

## Deprecation notice

This crate was deprecated. It is now part of the `tauri-codegen` crate.

# tauri-includedir

Include a directory in your Rust binary, e.g. static files for your web server or assets for your game.

## Features

* [x] Automatically compile data into binary
* [x] Use [rust-phf](https://github.com/sfackler/rust-phf) for efficient lookup
* [x] Wrapping API around the phf map, to abstract away additional features
* [x] Compression, with optional crate "flate2"
* [x] Reading from source files for debug builds

## Example

**Cargo.toml**
```toml
[package]
name = "example"
version = "0.1.0"

build = "build.rs"
include = ["data"]

[dependencies]
phf = "0.8.0"
tauri_includedir = "0.5.0"

[build-dependencies]
tauri_includedir_codegen = "0.5.0"
```

**build.rs**

```rust
extern crate tauri_includedir_codegen;

use tauri_includedir_codegen::Compression;

fn main() {
    tauri_includedir_codegen::start("FILES")
        .dir("data", Compression::Gzip)
        .build("data.rs")
        .unwrap();
}
```

**src/main.rs**

```rust
extern crate tauri_includedir;
extern crate phf;

use std::env;

include!(concat!(env!("OUT_DIR"), "/data.rs"));

fn main() {
    FILES.set_passthrough(env::var_os("PASSTHROUGH").is_some());

    println!("{:?}", FILES.get("data/foo"))
}
```
