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

## Usage

### Basic Inline Hash Expression

You can use #[hash_with(expr)] to specify a custom hashing expression for a field:
```rust
use hash_with::HashWith;

#[derive(HashWith)]
struct Brightness {
    /// Use a custom hash expression for this field.
    #[hash_with(self.inner.to_bits())]
    inner: f64,
}

let b1 = Brightness { inner: 1.1 };
let b2 = Brightness { inner: 2.2 };

assert_ne!(b1.get_hash(), b2.get_hash());
```

## Custom Hash Function

You can also specify a custom function for hashing a field:

```rust
use hash_with::HashWith;
use std::hash::Hash;

/// Custom hash function for f64
fn hash_f64_bits<H: std::hash::Hasher>(val: &f64, state: &mut H) {
    val.to_bits().hash(state);
}

#[derive(HashWith)]
struct Config {
    name: String,
    /// Hash using a custom function
    #[hash_with = "hash_f64_bits"]
    brightness: f64,
}
```

## Ignoring Fields

Fields can be excluded from the hash calculation using #[hash_without]:

```rust
use hash_with::HashWith;

#[derive(HashWith)]
struct User {
    id: u32,
    /// This field is ignored in the hash
    #[hash_without]
    session_token: String,
}

let user1 = User { id: 1, session_token: "abc".into() };
let user2 = User { id: 1, session_token: "xyz".into() };

// The hash ignores `session_token`
assert_eq!(user1.get_hash(), user2.get_hash());
```

# Why Use `hash_with`?
 - Rust's default Hash implementation does not support some primitive types like `f64` directly.
 - Simplifies implementing `Hash` for structs with attributes from external or third-party types that you cannot modify.
 - Lets you ignore irrelevant fields without implementing Hash manually.

