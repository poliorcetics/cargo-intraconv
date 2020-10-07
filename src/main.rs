use std::process;
use cargo_intraconv::Args;

fn main() {
    let args: Args = argh::from_env();
    // TODO(https://github.com/google/argh/issues/61): this shouldn't be necessary
    if args.paths.is_empty() {
        // TODO: it seems like `argh` should expose a way to do this ...
        eprintln!("usage: cargo-intraconv [<paths...>] [-c <crate>] [-a]");
        process::exit(1);
    }
    cargo_intraconv::run(args)
}
