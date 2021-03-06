use std::path::Path;

/// Options specific to the conversion of a file only.
///
/// See the `Args` struct for more information.
///
/// The `'cf` lifetime is for the configuration file.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ConversionOptions<'cf> {
    /// Crate name.
    ///
    /// The `Krate` type will ensure this is a valid Rust identifier.
    pub krate: Krate,

    /// When `true` disambiguators will be prepended to the item when
    /// appropriate, like `mod@item`.
    ///
    /// Suffix-disambiguation (`my_function()` or  `my_macro!`) are always
    /// active since they don't bring much noise.
    pub disambiguate: bool,

    /// When `true` favored links will be checked for which means some `https`
    /// links may be transformed.
    pub favored_links: bool,

    /// Contains the links that have been marked as ignored through a
    /// configuration file.
    ///
    /// Paths that had two components or more must have been canonicalized
    /// before being saved here.
    pub ignored_links: &'cf crate::FileConfig,

    ///
    pub current_path: &'cf Path,
}

impl<'cf> ConversionOptions<'cf> {
    pub fn is_ignored(&self, name: &str, value: &Path) -> bool {
        self.ignored_links
            .is_ignored(self.current_path, name, value)
    }
}

/// A valid Rust identifier for a crate.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Krate(String);

impl Krate {
    /// Given some unchecked string this function will return `None` when the
    /// passed crate name is not a valid Rust identifier.
    ///
    /// No correction will be attempted.
    pub fn new(name: &str) -> Option<Self> {
        if crate::RUST_IDENTIFIER_RE.is_match(name) {
            Some(Self(name.into()))
        } else {
            None
        }
    }

    /// Reference to the stored crate name.
    pub fn name(&self) -> &str {
        &self.0
    }
}

#[test]
fn new_valid() {
    let name = "_";
    assert_eq!(name, Krate::new(name).unwrap().0);

    let name = "a";
    assert_eq!(name, Krate::new(name).unwrap().0);

    let name = "A";
    assert_eq!(name, Krate::new(name).unwrap().0);

    let name = "std";
    assert_eq!(name, Krate::new(name).unwrap().0);

    let name = "cargo_intraconv";
    assert_eq!(name, Krate::new(name).unwrap().0);

    let name = "a_09";
    assert_eq!(name, Krate::new(name).unwrap().0);
}

#[test]
fn new_invalid() {
    let name = "";
    assert!(Krate::new(name).is_none());

    let name = "-";
    assert!(Krate::new(name).is_none());

    let name = "a-";
    assert!(Krate::new(name).is_none());

    let name = "A-";
    assert!(Krate::new(name).is_none());

    let name = "-A";
    assert!(Krate::new(name).is_none());

    let name = "0";
    assert!(Krate::new(name).is_none());

    let name = "a&A";
    assert!(Krate::new(name).is_none());

    let name = "a A";
    assert!(Krate::new(name).is_none());

    let name = "a 0";
    assert!(Krate::new(name).is_none());

    let name = "a-0";
    assert!(Krate::new(name).is_none());

    let name = "test_name_00-";
    assert!(Krate::new(name).is_none());
}

#[test]
fn name() {
    // This also tests invalid names to ensure correctness.

    let name = "";
    assert_eq!(name, Krate(name.into()).name());

    let name = "-";
    assert_eq!(name, Krate(name.into()).name());

    let name = "krate";
    assert_eq!(name, Krate(name.into()).name());

    let name = "0937864";
    assert_eq!(name, Krate(name.into()).name());

    let name = "a";
    assert_eq!(name, Krate(name.into()).name());

    let name = "krate_A_12";
    assert_eq!(name, Krate(name.into()).name());
}
