# HashWith

`HashWith` is a Rust procedural macro crate that allows you to automatically implement `Hash` for structs, overriding fields that don’t natively implement `Hash` (like `f64` or structs which use them). It supports custom hash functions per field.

## Features

- Derive `Hash` on structs with fields that normally cannot be hashed.
- Support for custom hash functions per field via the `#[hash_with = "..."]` attribute.
- Inline closures for per-field hashing.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
hash_with = { git = "https://github.com/some1and2-xc/hash-with" }
```
