mod action;
mod candidate;
mod cli_args;
mod config_file;
mod consts;
#[macro_use]
mod error;
mod file_finder;
mod link_parts;
mod options;
mod transform;

use action::Action;
use candidate::Candidate;
use cli_args::CliArgs;
use config_file::{FileConfig, RawFileConfig};
use consts::*;
use options::{ConversionOptions, Krate};
use transform::ConversionContext;

use std::env;
use std::fmt::Write;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Write as _};
use std::path::Path;

/// Takes an `CliArgs` instance to transform the paths it contains accordingly
/// with its stored parameters.
pub fn run(mut args: CliArgs) {
    if args.version {
        println!(
            "{} {}",
            std::env!("CARGO_PKG_NAME"),
            std::env!("CARGO_PKG_VERSION")
        );
        return;
    }

    let start_dir = code_error!(1, env::current_dir(), "Failed to get current directory");

    let file_config: FileConfig = if let Some(conf_file) = &args.config_file {
        let cf = code_error!(
            1,
            std::fs::read(conf_file),
            "Failed to read the given configuration file"
        );
        let content = code_error!(
            1,
            String::from_utf8(cf),
            "The given configuration file is not readable as UTF-8"
        );
        let fc: RawFileConfig = code_error!(
            1,
            toml::from_str(&content),
            "Failed to parse the content of the configuration file"
        );
        code_error!(
            1,
            fc.finish(),
            "Failed to canonicalize a path from the configuration file, \
            check it is correct when starting from the directory on which \
            you called `{}` ({})",
            std::env!("CARGO_PKG_NAME"),
            start_dir.display(),
        )
    } else {
        Default::default()
    };

    let default_crate = code_error!(
        1,
        Krate::new(&args.krate).ok_or("Invalid Rust identifier"),
        "Invalid crate name: '{}'",
        &args.krate
    )
    .name()
    .to_string();

    if args.paths.is_empty() || args.paths == &[Path::new("intraconv")] {
        args.paths.push(Path::new(".").into());
    }

    let mut paths = args.paths.iter();

    if args
        .paths
        .iter()
        .next()
        .map_or(false, |p| p.as_os_str() == "intraconv")
    {
        paths.next();
    }

    for path in paths {
        if path.is_dir() {
            for (maybe_crate_id, src_dir) in file_finder::crate_and_src() {
                code_error!(
                    1,
                    env::set_current_dir(&start_dir),
                    "Failed to set current directory back to '{:?}'",
                    start_dir
                );
                match Krate::new(&maybe_crate_id) {
                    Some(_) => args.krate = maybe_crate_id,
                    None => {
                        eprintln!(
                            "Invalid crate identifier '{}', using the default '{}'",
                            maybe_crate_id, &default_crate
                        );
                        args.krate = default_crate.clone();
                    }
                }
                code_error!(
                    1,
                    env::set_current_dir(&src_dir),
                    "Failed to set current directory to '{:?}'",
                    src_dir
                );

                for file in glob::glob("**/*.rs").unwrap() {
                    run_for_file(
                        &continue_error!(&file, "Failed to access '{:?}' in '{:?}'", &file, &path),
                        &args,
                        &file_config,
                    );
                }
            }
        } else {
            if args.krate != default_crate {
                args.krate = default_crate.clone();
            }
            run_for_file(&path, &args, &file_config);
        }
    }
}

fn run_for_file(path: &Path, args: &CliArgs, file_config: &FileConfig) {
    let krate = Krate::new(&args.krate).expect("Not a valid Rust identifier");

    let opts = ConversionOptions {
        krate,
        disambiguate: args.disambiguate,
        favored_links: !args.no_favored,
        ignored_links: file_config,
        current_path: &path,
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
        println!(
            "{}: {}\n{}\n",
            &args.krate,
            &path_display,
            "=".repeat(args.krate.len() + 1)
        );
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
