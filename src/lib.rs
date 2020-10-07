mod action;
use action::Action;

mod transform;
use transform::Context;

use argh::FromArgs;
use regex::Regex;
use std::fmt::Write;
use std::fs::{File, OpenOptions};
use std::io::BufReader;
use std::io::Write as _;
use std::path::PathBuf;
use std::process;

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
    /// root crate (examples: `std`, `core`, `my_crate`, ...).
    #[argh(
        option,
        long = "crate",
        short = 'c',
        from_str_fn(check_krate),
        default = "\"std\".into()"
    )]
    krate: String,

    /// apply the proposed changes.
    #[argh(switch, short = 'a')]
    apply: bool,

    /// files to search links in.
    #[argh(positional)]
    paths: Vec<PathBuf>,
}

/// Takes an `Args` instance to transform the paths it contains accordingly
/// with its stored parameters.
pub fn run(args: Args) {
    if args.paths.is_empty() {
        eprintln!("No paths were passed as arguments.");
        eprintln!("usage: cargo-intraconv [<paths...>] [-c <crate>] [-a]");
        process::exit(1);
    }

    let mut ctx = Context::new(args.krate);
    for path in args.paths {
        // First display the path of the file that is about to be opened and tested.
        let path_display = path.display().to_string();
        println!("{}", &path_display);
        // TODO: Not always perfect because of unicode, fix this.
        println!("{}\n", "=".repeat(path_display.len()));

        // Then open the file, reporting if it fails.
        let file = match File::open(&path) {
            Ok(file) => BufReader::new(file),
            Err(err) => {
                eprintln!("Failed to open file '{}' for read: {}", &path_display, err);
                continue;
            }
        };

        let actions = match ctx.transform_file(file) {
            Ok(actions) => actions,
            Err(err) => {
                eprintln!("Failed to transform file '{}': {}", &path_display, err);
                continue;
            }
        };

        // Do not allocate when unecessary.
        let mut updated_content = if args.apply {
            String::with_capacity(64 * actions.len())
        } else {
            String::new()
        };

        // Display the changes that can be made.
        for l in actions {
            if !l.is_unchanged() {
                println!("{}\n", l);
            }
            if args.apply {
                write!(updated_content, "{}", l.as_new_line()).unwrap();
            }
        }

        if !args.apply {
            return;
        }

        let mut file = match OpenOptions::new().write(true).truncate(true).open(path) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("Failed to open file '{}' for write: {}", &path_display, err);
                return;
            }
        };

        match write!(file, "{}", updated_content) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("Failed to write to '{}': {}", &path_display, err);
                return;
            }
        };
    }
}

/// Check the given `krate` is exactly one of `std`, `core` or `alloc`.
/// In any other case it will return an error message.
fn check_krate(krate: &str) -> Result<String, String> {
    let krate_regex = Regex::new(r"^[\w_]+$").unwrap();
    if krate_regex.is_match(krate) {
        Ok(krate.into())
    } else {
        Err(format!(
            "The passed crate identifier '{}' is not valid.\n",
            krate
        ))
    }
}
