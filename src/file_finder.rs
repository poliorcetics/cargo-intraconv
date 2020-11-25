// Code originally from:
// https://github.com/deadlinks/cargo-deadlinks/blob/5af27cd5b4a2ce9c21b38053461ae007e645192f/src/main.rs#L130-L174
use cargo_metadata::MetadataCommand;

use std::path::PathBuf;

pub fn crate_and_src() -> impl Iterator<Item = (String, PathBuf)> {
    let manifest = crate::code_error!(
        1,
        MetadataCommand::new().no_deps().exec(),
        "This is not a cargo directory, pass the files explicitly"
    );

    // FIXME: This doesn't handle renamed packages for renames other than `-` -> `_`.
    manifest.packages.into_iter().filter_map(|package| {
        let path = package
            .manifest_path
            .parent()
            .expect("A Cargo.toml cannot be the root")
            .join("src");
        let name = package.name.replace("-", "_");

        if path.is_dir() {
            Some((name, path))
        } else {
            None
        }
    })
}
