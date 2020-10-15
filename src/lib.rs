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
/// When `-q` is not given, only files with changes will be displayed.
#[derive(FromArgs, Debug)]
pub struct Args {
    /// prints the crate version and exit.
    #[argh(switch)]
    version: bool,

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

    /// use rustdoc disambiguators in front of the transformed links
    /// ('type@', ...). Ending disambiguators like '()' and '!' are always
    /// added, regardless of this option.
    #[argh(switch, short = 'd')]
    disambiguate: bool,

    /// do not display changes, only errors when they happen.
    #[argh(switch, short = 'q')]
    quiet: bool,

    /// files to search links in.
    #[argh(positional)]
    paths: Vec<PathBuf>,
}

/// Takes an `Args` instance to transform the paths it contains accordingly
/// with its stored parameters.
pub fn run(args: Args) {
    if args.version {
        println!("cargo-intraconv {}", std::env!("CARGO_PKG_VERSION"));
        return;
    }

    if args.paths.is_empty() {
        eprintln!("No paths were passed as arguments.");
        eprintln!("Usage: target/debug/cargo-intraconv [<paths...>] [--version] [-c <crate>] [-a] [-d] [-q]");
        process::exit(1);
    }

    let display_changes = !args.quiet;

    let mut ctx = Context::new(args.krate, args.disambiguate);
    for path in args.paths {
        if path.as_os_str() == "intraconv" && !path.exists() {
            continue;
        }

        // First display the path of the file that is about to be opened and tested.
        let path_display = path.display().to_string();

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

        // Only display the filename when -q is not set and there are changes.
        if display_changes && actions.iter().any(|a| !a.is_unchanged()) {
            println!("{}", &path_display);
            // TODO: Not always perfect because of unicode, fix this.
            println!("{}\n", "=".repeat(path_display.len()));
        }

        // Display the changes that can be made.
        for l in actions {
            if !l.is_unchanged() && display_changes {
                println!("{}\n", l);
            }
            if args.apply {
                write!(updated_content, "{}", l.as_new_line()).unwrap();
            }
        }

        if !args.apply {
            continue;
        }

        let mut file = match OpenOptions::new().write(true).truncate(true).open(path) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("Failed to open file '{}' for write: {}", &path_display, err);
                continue;
            }
        };

        match write!(file, "{}", updated_content) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("Failed to write to '{}': {}", &path_display, err);
                continue;
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
