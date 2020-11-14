// Code originally from:
// https://github.com/deadlinks/cargo-deadlinks/blob/5af27cd5b4a2ce9c21b38053461ae007e645192f/src/main.rs#L130-L174
use std::path::PathBuf;

use cargo_metadata::MetadataCommand;

pub fn determine_dir() -> Vec<(PathBuf, String)> {
    let manifest = MetadataCommand::new().no_deps().exec().unwrap_or_else(|_| {
        eprintln!("This is not a cargo directory, pass the files explicitly");
        ::std::process::exit(1);
    });

    let root = manifest.workspace_root;

    // FIXME: This doesn't handle renamed packages for renames other than `-` -> `_`.
    manifest
        .packages
        .into_iter()
        .flat_map(|package| {
            vec![
                (
                    root.join(&package.name).join("src"),
                    package.name.replace("-", "_"),
                ),
                (root.join("src"), package.name.replace("-", "_")),
            ]
            .into_iter()
        })
        .filter(|(path, _)| path.is_dir())
        .collect()
}
