mod args;
use args::Args;

mod action;
use action::Action;

mod candidate;
use candidate::Candidate;

mod consts;
use consts::*;

#[macro_use]
mod error;

mod file_finder;

mod link_parts;

mod options;
use options::{ConversionOptions, Krate};

mod transform;
use transform::ConversionContext;

use std::env;
use std::fmt::Write;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Write as _};
use std::path::Path;

/// Takes an `Args` instance to transform the paths it contains accordingly
/// with its stored parameters.
pub fn run(args: Args) {
    if args.version {
        println!("cargo-intraconv {}", std::env!("CARGO_PKG_VERSION"));
        return;
    }

    let mut true_args = args.clone();
    true_args.paths = vec![];

    let paths = if args.paths.is_empty() || args.paths == [Path::new("intraconv")] {
        file_finder::determine_dir()
    } else {
        args.paths
            .into_iter()
            .map(|p| (p, true_args.krate.clone()))
            .collect()
    };

    let start_dir = code_error!(1, env::current_dir(), "Failed to get current directory");

    let mut visited = ::std::collections::HashSet::new();

    for (path, krate) in paths {
        if visited.contains(&path) {
            continue;
        } else {
            visited.insert(path.clone());
        }

        true_args.krate = krate;

        if !path.is_dir() {
            run_for_file(&path, &true_args);
        } else {
            code_error!(
                1,
                env::set_current_dir(&path),
                "Failed to set current directory to '{:?}'",
                path
            );

            for file in glob::glob("**/*.rs").unwrap() {
                run_for_file(
                    &continue_error!(file, "Failed to access a file in '{:?}'", &path),
                    &true_args,
                );
            }

            code_error!(
                1,
                env::set_current_dir(&start_dir),
                "Failed to set current directory to '{:?}'",
                path
            );
        }
    }
}

fn run_for_file(path: &Path, args: &Args) {
    if path.as_os_str() == "intraconv" && !path.exists() {
        return;
    }

    let path = return_error!(path.canonicalize(), "Failed to canonicalize path");
    let krate = return_error!(
        Krate::new(&args.krate).ok_or("Not a valid Rust identifier"),
        "Invalid crate name: '{}'",
        args.krate
    );

    let opts = ConversionOptions {
        krate,
        disambiguate: args.disambiguate,
        favored_links: !args.no_favored,
    };

    let display_changes = !args.quiet;
    let mut ctx = ConversionContext::with_options(opts);

    // First display the path of the file that is about to be opened and tested.
    let path_display = path.display().to_string();

    // Then open the file, reporting if it fails.
    let file = BufReader::new(return_error!(
        File::open(&path),
        "Failed to open file '{}' for reading",
        &path_display
    ));
    let actions = return_error!(
        ctx.transform_file(file),
        "Failed to transform file '{}'",
        &path_display
    );

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
        return;
    }

    let mut file = return_error!(
        OpenOptions::new().write(true).truncate(true).open(path),
        "Failed to open file '{}' for writing",
        &path_display
    );

    return_error!(
        write!(file, "{}", updated_content),
        "Failed to write changes to '{}'",
        &path_display
    );
}
