mod action;
use action::Action;

mod consts;
mod transform;

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
    /// root crate
    #[argh(
        option,
        long = "crate",
        short = 'c',
        default = "\"std\".to_string()"
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
pub fn run(args: Args) {
    args.paths
        .iter()
        .for_each(|path| transform::handle_path(path, &args.krate, args.apply))
}
