// mod action;
// use action::Action;

// mod consts;
pub mod new;
// mod transform;
// pub mod updater;

use argh::FromArgs;
use std::path::PathBuf;

/// Converter from path-based links to intra-doc links for the `rust-lang/rust`
/// project.
///
/// This is not perfect and the modified files should still be reviewed after
/// running it.
///
/// By default it will only print the changes and not apply them, use `-a`
/// (`--apply`) to write them.
///
/// If you are modifying `core` or `alloc` instead of `std`, you can pass the
/// `-c core` (`--crate core`) to mark the change in the root crate.
#[derive(FromArgs, Debug)]
pub struct Args {
    /// root crate (one of `std`, `core` or `alloc`).
    #[argh(
        option,
        long = "crate",
        short = 'c',
        from_str_fn(check_krate),
        default = "\"std\".into()"
    )]
    krate: String,

    /// apply the proposed changes. Does nothing for now.
    #[argh(switch, short = 'a')]
    apply: bool,

    /// files to search links in.
    #[argh(positional)]
    paths: Vec<PathBuf>,
}

/// Takes an `Args` instance to transform the paths it contains accordingly
/// with its stored parameters.
// pub fn run(args: Args) {
//     args.paths
//         .iter()
//         .for_each(|path| transform::handle_path(path, args.krate, args.apply))
// }

/// Check the given `krate` is exactly one of `std`, `core` or `alloc`.
/// In any other case it will return an error message.
fn check_krate(krate: &str) -> Result<String, String> {
    match krate {
        "std" | "core" | "alloc" => Ok(krate.into()),
        _ => Err("Valid crate options are `std`, `core` and `alloc`.".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_krate() {
        assert_eq!(check_krate("std"), Ok("std".into()));
        assert_eq!(check_krate("core"), Ok("core".into()));
        assert_eq!(check_krate("alloc"), Ok("alloc".into()));

        // The error text is not what's important here.
        assert!(check_krate("Alloc").is_err());
        assert!(check_krate("sTD").is_err());
        assert!(check_krate("CoRe").is_err());
        assert!(check_krate("abc").is_err());
    }
}
