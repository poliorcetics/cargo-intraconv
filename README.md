# Cargo intraconv

`cargo-intraconv` is a simple helper which will transform Markdown links to
intra-doc links in Rust projects when appropriate.

> Note: you will need `rustdoc` 1.48 (or maybe 1.49) at least, which are not
> available yet. This crate can still be used to help updating the documentation
> for `rust-lang/rust` itself.

## What are intra-doc links ?

Previously the only way to write links to other elements of your crate (or other
crates) was the following, the path depending on the current and target files:

```rust
// In the `u8` impl in `core`
/// [`make_ascii_uppercase`]: #method.make_ascii_uppercase

/// [`f32::classify`]: ../../std/primitive.f32.html#method.classify
```

It is now possible to write them with Rust paths, depending on the path of the
targeted item and what's in scope (which means items like `String` which are in
the prelude are just a ```[`String`]``` away). Those links are clearer for both
the person writing them in the first place and subsequent readers reviewing them.
They are also easier to reason about since file hierachy does not affect them.

```rust
/// [`make_ascii_uppercase`]: u8::make_ascii_uppercase

/// [`f32::classify`]: std::f32::classify
```

## Why this crate ?

Changing all the existing links can be tedious and can be automated. This crate
is a proof-of-concept of the feasibility and it is my hope to include a similar
tool in `cargo fix` soon. The goal of this crate is to help you while the
`cargo fix` version is not available.

## Known issues

Both intra-doc links and this crate have several known issues, most of which
should be adressed in future versions of either the crate or Rust itself.

For issues about intra-doc links you should look-up the issues at `rust-lang/rust`.

For issues about this crate, here are a few:

  - `#method.method_name` links outside of an `impl` block are not transformed
    right now, this is a bug and will be fixed in a future version.
  - `[Item](link)` links are not transformed. This is also a bug and will be
    fixed in a future version.

## Drawbacks

It is **not** an official tool and the way it works right now is based on regexes.
This approach means it is simple to understand but it has several drawbacks.
For example `cargo-intraconv` is not aware of `use`s and will happily ignore them,
even when they could shorten or remove links.

## License

See `LICENSE` at the repository's root.
