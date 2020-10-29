[![CI](https://github.com/poliorcetics/cargo-intraconv/workflows/ci/badge.svg)](https://github.com/poliorcetics/cargo-intraconv/actions)
![crates.io](https://img.shields.io/crates/v/cargo-intraconv)
![crates.io](https://img.shields.io/crates/l/cargo-intraconv)

# Cargo intraconv

`cargo-intraconv` is a simple helper which will transform Markdown links to
[intra-doc links] in Rust projects when appropriate.

> Note: you will need you need beta/nightly rustdoc or to wait until
> stabilization of intra-doc links, which is underway for **1.48.0** !
>
> This crate can still be used to help updating the documentation for
> `rust-lang/rust` itself and it is its intended usage right now. You can
> also use it for projects depending on beta/nightly.

[intra-doc links]: https://doc.rust-lang.org/nightly/rustdoc/unstable-features.html#linking-to-items-by-type

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
the prelude are just a `[String]` away). Those links are clearer for both
the person writing them in the first place and subsequent readers reviewing them.
They are also easier to reason about since file hierachy does not affect them.

```rust
/// [`make_ascii_uppercase`]: u8::make_ascii_uppercase()

/// [`f32::classify`]: std::f32::classify()
```

## Why this crate ?

Changing all the existing links can be tedious and can be automated. This crate
is a proof-of-concept of the feasibility and it is my hope to include a similar
tool in `cargo fix` soon. The goal of this crate is to help you while the
`cargo fix` version is not available.

## Usage

By default the binary produced by the crate will not modify the given files,
only show what would change:

```shell
$ cargo intraconv path/to/std/file.rs

$ cargo intraconv path/to/core/file.rs -c core # Specifying the root crate

$ cargo intraconv path/to/std/file.rs -a # Applying the changes

$ cargo intraconv path/to/my/file.rs -d # Disambiguate links by prefixing them
                                        # with their rustdoc group ('type@', ...)

$ cargo intravonc path/to/my/file.rs -f # Do not transform favored links to
                                        # intra-doc links (see below for more)

$ cargo intraconv path/to/my/file.rs -q # Do not display changes, only errors
```

It is possible to give multiple paths to files. Note that directories will not
work. Giving no paths will produce an error.

> Note: `intraconv` will accept any file, no just `.rs` ones: you can use it
> on markdown files that are included as docs in Rust files for example.

### Favored links

By default the crate will transform favored `http(s)://` links to intra-doc
links (like those from [`docs.rs`](https://docs.rs)). To disable this behaviour
use the `-f` (`--no-favored`) flag.

## Known issues

Both intra-doc links and this crate have several known issues, most of which
should be adressed in future versions of either the crate or Rust itself.

For issues about intra-doc links you should look-up [the issues at `rust-lang/rust`].

For issues about this crate, here are a few:

  - `#method.method_name` links will sometimes be transformed to point to the
    wrong item. This is because `intraconv` uses regexes to find links and the
    types related to them, which is not perfect.
  - `[Item](link)` links are not transformed. There are a lot more false 
    positives with those so, as long as no one wants them, I will not add them.
    I'm still open to a PR adding them if you wish to do so and will gladly
    help you if needed !

[the issues at `rust-lang/rust`]: https://github.com/rust-lang/rust/issues?q=is%3Aopen+label%3AA-intra-doc-links+label%3AC-bug

## Drawbacks

It is **not** an official tool and the way it works right now is based on regexes.
This approach means it is simple to understand but it has several drawbacks.
For example `cargo-intraconv` is not aware of `use`s and will happily ignore them,
even when they could shorten or remove links.

## License

See `LICENSE` at the repository's root.
