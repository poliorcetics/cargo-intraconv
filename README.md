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

/// See [the `Rotation` type](../struct.Rotation.html)
```

It is now possible to write them with Rust paths, depending on the path of the
targeted item and what's in scope (which means items like `String` which are in
the prelude are just a `[String]` away). Those links are clearer for both
the person writing them in the first place and subsequent readers reviewing them.
They are also easier to reason about since file hierachy does not affect them.

```rust
/// [`make_ascii_uppercase`]: u8::make_ascii_uppercase()

/// [`f32::classify`]: std::f32::classify()

/// See [the `Rotation` type](super::Rotation)
```

When both side of the link are the same, it is possible to be even shorter:

```diff
/// See [`Rotation`]
///
- /// [`Rotation`]: struct.Rotation.html

- /// See [`Rotation`](struct.Rotation.html)
+ /// See [`Rotation`]
```

## Why this crate ?

Changing all the existing links can be tedious and can be automated. This crate
is here to help you update your documentation to intra-doc links as painlessly
as possible.

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

$ cargo intraconv path/to/my/file.rs -i intraconv.toml # Give a file containing
                                                       # links to ignore 
```

It is possible to give multiple paths to files or directories.

> Note: `intraconv` will accept any file, no just `.rs` ones: you can use it
> on markdown files that are included as docs in Rust files for example.

### Favored links

By default the crate will transform favored `http(s)://` links to intra-doc
links (like those from [`docs.rs`](https://docs.rs)). To disable this behaviour
use the `-f` (`--no-favored`) flag.

### Ignoring links

`cargo-intraconv` is not perfect and will sometimes wrongly transform links,
as in #31. To fix that you can either do it manually if you run it only once
but if you start to run it several times because the changes are significative
it will quickly become repetitive and error-prone. For a tool designed to
reduce repetitive and error prone work, this is a sad thing !

To fix this, you can write a file in the TOML format with links to ignore:

```toml
# Global ignores, applied everywhere.
[ignore]
# Will be deleted by the second instance of `"name"`.
"name" = [ "link 1" ] # Must be an array, a single value will not work,
                      # this allows for multiples values easily.
"name" = [ "link 2" ]
# Will only keep one instance of 'link 3'.
"other name" = [ "link 3", "link 3", "link 1" ]

# Will match exactly the lib.rs file in tracing-core.
# NOTE: this path must either be absolute OR relative to the EXACT directory
# from which `cargo-intraconv` is called.
#
# Paths must be unique whether canonicalized (for paths with more than one
# component) or not (paths with a single component, as `lib.rs` below).
["tracing-core/src/lib.rs"]
"`downcast_ref`" = [ "#method.downcast_ref" ] # Must be an array.

# Will match EVERY lib.rs file found.
["lib.rs"]
"`downcast_ref`" = [ "#method.downcast_ref" ]
```

## Known issues

Both intra-doc links and this crate have several known issues, most of which
should be adressed in future versions of either the crate or Rust itself.

For issues about intra-doc links you should look-up [the issues at `rust-lang/rust`].

For issues about this crate, here are a few:

  - `#method.method_name` links will sometimes be transformed to point to the
    wrong item. This is because `intraconv` uses regexes to find links and the
    types related to them, which is not perfect.
  - Crate names are not detected very well at the moment and if you don't use
    `--crate` (`-c`) will sometime fail in strange ways. Be sure to check !
    By default `cargo-intraconv` will use `my_krate` to avoid false `crate::`
    replacements.

[the issues at `rust-lang/rust`]: https://github.com/rust-lang/rust/issues?q=is%3Aopen+label%3AA-intra-doc-links+label%3AC-bug

## Drawbacks

It is **not** an official tool and the way it works right now is based on regexes.
This approach means it is simple to understand but it has several drawbacks.
For example `cargo-intraconv` is not aware of `use`s and will happily ignore them,
even when they could shorten or remove links.

## Contributing

Any form of contribution is appreciated ! You can open issues or PR and I'll
try to answer you quickly !

## License

See `LICENSE` at the repository's root.
