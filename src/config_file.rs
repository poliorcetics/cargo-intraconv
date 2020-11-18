use serde::Deserialize;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct FileConfig(RawFileConfig);

impl FileConfig {
    pub fn is_ignored(&self, file: &Path, name: &str, value: &Path) -> bool {
        self.0.is_globally_ignored(name, value) || self.0.is_locally_ignored(file, name, value)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default, Deserialize)]
pub struct RawFileConfig {
    #[serde(rename = "ignore")]
    /// Form:
    ///
    /// ```toml
    /// [ignore]
    /// "name" = [ "link" ]
    /// ```
    globals: HashMap<String, BTreeSet<PathBuf>>,

    /// Form:
    ///
    /// ```toml
    /// # Canonicalized to match exactly
    /// ["tracing/src/lib.rs"]
    /// "`downcast_ref`" = [ "#method.downcast_ref", ]
    ///
    /// # Not canonicalized, suffix check
    /// ["lib.rs"]
    /// "`downcast_ref`" = [ "#method.downcast_ref", ]
    /// ```
    #[serde(flatten)]
    per_file: HashMap<PathBuf, HashMap<String, BTreeSet<PathBuf>>>,
}

impl RawFileConfig {
    pub fn finish(mut self) -> std::io::Result<FileConfig> {
        let mut canonicalized = self.per_file.clone();
        canonicalized.retain(|k, _| {
            let mut c = k.components();
            c.next().is_some() && c.next().is_none()
        });
        for (k, v) in self.per_file.into_iter() {
            let mut c = k.components();
            // Testing if the path has two components without checking it in
            // its entirety.
            if c.next().is_some() && c.next().is_some() {
                let canon = k.canonicalize()?;
                canonicalized.insert(canon, v);
            }
        }
        self.per_file = canonicalized;
        Ok(FileConfig(self))
    }

    #[inline]
    fn is_globally_ignored(&self, name: &str, value: &Path) -> bool {
        contains_ignored(&self.globals, name, value)
    }

    /// Returns `true` if any file stored in the configuration is a suffix
    /// of the passed file and has a link `(name, value)`.
    #[inline]
    fn is_locally_ignored(&self, file: &Path, name: &str, value: &Path) -> bool {
        self.per_file.iter().any(|(f, names)| {
            if !file.ends_with(f) {
                return false;
            }
            contains_ignored(names, name, value)
        })
    }
}

#[inline]
fn contains_ignored(map: &HashMap<String, BTreeSet<PathBuf>>, name: &str, value: &Path) -> bool {
    map.get(name).map_or(false, |values| values.contains(value))
}
