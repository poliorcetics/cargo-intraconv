// Code originally from:
// https://github.com/deadlinks/cargo-deadlinks/blob/5af27cd5b4a2ce9c21b38053461ae007e645192f/src/main.rs#L130-L174
use std::path::PathBuf;
use std::process;

use cargo_metadata::Metadata;

pub fn determine_dir() -> Vec<(PathBuf, String)> {
    let manifest = metadata_run(None).unwrap_or_else(|()| {
        eprintln!("This is not a cargo directory, pass the files explicitely");
        ::std::process::exit(1);
    });

    let root = manifest.workspace_root;

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

pub fn metadata_run(additional_args: Option<String>) -> Result<Metadata, ()> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| String::from("cargo"));
    let mut cmd = std::process::Command::new(cargo);
    cmd.arg("metadata");
    cmd.args(&["--format-version", "1"]);
    if let Some(additional_args) = additional_args {
        cmd.arg(&additional_args);
    }

    let fail_msg = "failed to run `cargo metadata` - do you have cargo installed?";
    let output = cmd
        .stdout(process::Stdio::piped())
        .spawn()
        .expect(fail_msg)
        .wait_with_output()
        .expect(fail_msg);

    if !output.status.success() {
        // don't need more info because we didn't capture stderr;
        // hopefully `cargo metadata` gave a useful error, but if not we can't do
        // anything
        return Err(());
    }

    let stdout = std::str::from_utf8(&output.stdout).expect("invalid UTF8");
    Ok(serde_json::from_str(stdout).expect("invalid JSON"))
}
