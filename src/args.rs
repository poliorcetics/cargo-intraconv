//! Command lines arguments for `cargo-intraconv`.
use argh::FromArgs;

use std::path::PathBuf;

/// Converter from path-based links to intra-doc links for Rust crates.
///
/// This is not perfect and the modified files should still be reviewed after
/// running it.
///
/// By default it will only print the changes and not apply them, use `-a`
/// (`--apply`) to write them.
///
/// If you are modifying `core` or `alloc` instead of `std`, you can pass the
/// `-c core` (`--crate core`) to mark the change in the root crate.
///
/// By default the crate will output the changes it will do (or did when `-a`
/// is passed). If this is not what you want, use the `-q` (`--quiet`) flag
/// to only show errors.
///
/// By default the crate will transform favored http(s):// links to intra-doc
/// links (like those from `docs.rs`). To disable this behaviour use the `-f`
/// (`--no-favored`) flag.
///
/// When `-q` is not given, only files with changes will be displayed.
#[derive(FromArgs, Debug, Clone)]
pub struct Args {
    /// prints the crate version and exit.
    #[argh(switch)]
    pub version: bool,

    /// root crate (examples: `std`, `core`, `my_crate`, ...).
    #[argh(
        option,
        long = "crate",
        short = 'c',
        from_str_fn(check_krate),
        default = "\"std\".into()"
    )]
    pub krate: String,

    /// apply the proposed changes.
    #[argh(switch, short = 'a')]
    pub apply: bool,

    /// use rustdoc disambiguators in front of the transformed links
    /// ('type@', ...). Ending disambiguators like '()' and '!' are always
    /// added, regardless of this option.
    #[argh(switch, short = 'd')]
    pub disambiguate: bool,

    /// disable transformation of favored links.
    /// Favored links example: https://docs.rs/name/latest/name/span/index.html
    /// will be transformed to `name::span`.
    #[argh(switch, long = "no-favored", short = 'f')]
    pub no_favored: bool,

    /// do not display changes, only errors when they happen.
    #[argh(switch, short = 'q')]
    pub quiet: bool,

    /// files to search links in.
    #[argh(positional)]
    pub paths: Vec<PathBuf>,
}

/// Returns `Ok(krate.into())` when the passed string is a valid Rust
/// identifier for a crate name.
fn check_krate(krate: &str) -> Result<String, String> {
    if crate::RUST_IDENTIFIER_RE.is_match(krate) {
        Ok(krate.into())
    } else {
        Err(format!(
            "The passed crate identifier '{}' is not valid.\n",
            krate
        ))
    }
}

#[test]
fn test_check_krate() {
    assert_eq!(check_krate("regex"), Ok("regex".into()));
    assert_eq!(check_krate("regex_2"), Ok("regex_2".into()));

    assert!(check_krate("invalid-krate").is_err());
    assert!(check_krate("0invalidkrate").is_err());
}
