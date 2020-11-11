use regex::Regex;
use std::path::{Component, Path};

/// A `Link` is a candidate for transformation to an intra-doc link.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Link<'a>(&'a Path);

impl<'a> Link<'a> {
    /// `true` if the link is an HTTP one. This includes favored links.
    fn is_http(&self) -> bool {
        (self.0.starts_with("http:") || self.0.starts_with("https:"))
            && self.0.components().count() >= 2
    }

    /// `true` if the link is a favored HTTP link.
    fn is_favored(&self) -> bool {
        use std::ffi::OsStr;

        if !self.is_http() {
            return false;
        }

        let comp_count = self.0.components().count();

        let mut comps = self.0.components();
        let _http = comps.next();
        let domain = comps.next().expect("At least 2 components");

        let docs_rs = OsStr::new("docs.rs");
        let doc_rust_lang_org = OsStr::new("doc.rust-lang.org");

        // Checking the domain for favored links patterns.
        match domain {
            Component::Normal(dom) if dom == docs_rs && comp_count >= 3 => true,
            Component::Normal(dom) if dom == doc_rust_lang_org && comp_count >= 3 => {
                use regex::Regex;
                lazy_static::lazy_static! {
                    static ref VERSION_REGEX: Regex = Regex::new(r"1\.\d+\.\d+").unwrap();
                }

                // Ensure the channel can be converted to a valid "&str".
                let channel = match comps
                    .next()
                    .expect("At least 3 components")
                    .as_os_str()
                    .to_str()
                {
                    Some(c) => c,
                    None => return false,
                };

                // Check if the channel is a valid one.
                if !(["nightly", "beta", "stable"].contains(&channel)
                    || VERSION_REGEX.is_match(channel))
                {
                    return false;
                }

                // Get the crate in the link. If it is not there, this is not a favored link.
                let krate = match comps.next().map(|c| c.as_os_str().to_str()) {
                    Some(Some(k)) => k,
                    _ => return false,
                };

                // Final check: the crate in the link must be a valid one.
                ["std", "alloc", "core", "test", "proc_macro"].contains(&krate)
            }
            _ => false,
        }
    }

    /// `true` if the link ends with an associated item alone.
    fn is_associated_item(&self) -> bool {
        // It is not invalid to have './' before the associated item when it
        // points to a module-level item.
        let mut comps = self.0.components().skip_while(|c| c == &Component::CurDir);
        let assoc_item = match comps.next().map(|ai| ai.as_os_str().to_str()) {
            Some(Some(ai)) => ai,
            _ => return false,
        };

        // The associoted item MUST be the last element.
        if comps.next().is_some() {
            return false;
        }

        lazy_static::lazy_static! {
            static ref ASSOC_ITEM: Regex = Regex::new(&format!(
                r"^#{}\.[a-zA-Z_][a-zA-Z0-9_]*$",
                crate::link_analyser::consts::ITEM_TYPES.as_str()
            )).unwrap();
        }

        ASSOC_ITEM.is_match(assoc_item)
    }

    /// `true` if the link ends with a section alone.
    fn is_section(&self) -> bool {
        if self.is_associated_item() {
            return false;
        }

        // It is not invalid to have './' before the section when it points to
        // a module-level item.
        let mut comps = self.0.components().skip_while(|c| c == &Component::CurDir);
        let section = match comps.next().map(|s| s.as_os_str().to_str()) {
            Some(Some(s)) => s,
            _ => return false,
        };

        // The section MUST be the last element.
        if comps.next().is_some() {
            return false;
        }

        lazy_static::lazy_static! {
            static ref SECTION: Regex = Regex::new(
                r"^#[a-zA-Z0-9_\-\.]+$"
            ).unwrap();
        }

        SECTION.is_match(section)
    }

    /// `true` if the link ends with an item. It can have a section or
    /// associated item tacked on like `struct.String.html#section`.
    fn is_item(&self) -> bool {
        if self.is_http() || self.is_favored() {
            return false;
        }

        lazy_static::lazy_static! {
            static ref SIMPLE_ITEM: Regex = Regex::new(
                &format!(r"^{}\.[a-zA-Z_][a-zA-Z0-9_]*\.html$",
                         crate::link_analyser::consts::ITEM_TYPES.as_str())
            ).unwrap();

            static ref WITH_ASSOCIATED: Regex = Regex::new(
                &format!(r"^{it}\.[a-zA-Z_][a-zA-Z0-9_]*\.html#{it}\.[a-zA-Z_][a-zA-Z0-9_]*$",
                         it = crate::link_analyser::consts::ITEM_TYPES.as_str())
            ).unwrap();

            static ref WITH_SECTION: Regex = Regex::new(
                &format!(r"^{it}\.[a-zA-Z_][a-zA-Z0-9_]*\.html#[a-zA-Z0-9_\-\.]+$",
                         it = crate::link_analyser::consts::ITEM_TYPES.as_str())
            ).unwrap();

            static ref RUST_IDENTIFIER: Regex = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
        }

        // Reverse the iterator to have easy access to the last element.
        let mut comps = self.0.components().rev();
        // If the last element is incorrect this cannot be an item.
        match comps.next().expect("The last element").as_os_str().to_str() {
            Some(last_comp) => {
                if !(SIMPLE_ITEM.is_match(last_comp)
                    || WITH_ASSOCIATED.is_match(last_comp)
                    || WITH_SECTION.is_match(last_comp))
                {
                    return false;
                }
            }
            _ => return false,
        }

        // Check all the other component to ensure they are either `.` or `..`
        // or a valid rust identifier (a module name). If not, return `false`.
        for comp in comps {
            match comp {
                Component::CurDir | Component::ParentDir => (),
                Component::Normal(path) => match path.to_str() {
                    Some(path) => {
                        if !RUST_IDENTIFIER.is_match(path) {
                            return false;
                        }
                    }
                    None => return false,
                },
                _ => return false,
            }
        }

        true
    }

    /// `true` if the link ends with a module. It can have a section or
    /// associated item tacked on like `index.html#section` or `mod1#section`.
    fn is_module(&self) -> bool {
        if self.is_http() || self.is_favored() {
            return false;
        }

        lazy_static::lazy_static! {
            static ref LONG_FORM: Regex = Regex::new(r"^(?:index.html|[a-zA-Z_][a-zA-Z0-9_]*)(?:#[a-zA-Z0-9_\-\.]+)?$").unwrap();
            static ref RUST_IDENTIFIER: Regex = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
        }

        // Reverse the iterator to have easy access to the last element.
        let mut comps = self.0.components().rev();
        // If the last element is incorrect this cannot be an item.
        match comps.next().expect("The last element").as_os_str().to_str() {
            Some(last_comp) => {
                if !LONG_FORM.is_match(last_comp) {
                    return false;
                }
            }
            _ => return false,
        }

        // Check all the other component to ensure they are either `.` or `..`
        // or a valid rust identifier (a module name). If not, return `false`.
        for comp in comps {
            match comp {
                Component::CurDir | Component::ParentDir => (),
                Component::Normal(path) => match path.to_str() {
                    Some(path) => {
                        if !RUST_IDENTIFIER.is_match(path) {
                            return false;
                        }
                    }
                    None => return false,
                },
                _ => return false,
            }
        }

        true
    }
}

impl<'a> From<&'a Path> for Link<'a> {
    /// Wraps a `std::path::Path` in a `Link` to check if it can be transformed
    /// or not.
    fn from(path: &'a Path) -> Self {
        Self(path)
    }
}

#[test]
fn from_path() {
    assert_eq!(Link::from(Path::new("test")).0, Path::new("test"));
    assert_ne!(Link::from(Path::new("test")).0, Path::new("not/test"));

    assert_eq!(Link::from(Path::new("mod1/mod2")).0, Path::new("mod1/mod2"));
    assert_ne!(Link::from(Path::new("mod1/mod2")).0, Path::new("mod2/mod1"));
}

#[test]
fn is_http() {
    let link = Link(&Path::new("http/docs.rs"));
    assert!(!link.is_http());

    let link = Link(&Path::new("http//docs.rs"));
    assert!(!link.is_http());

    let link = Link(&Path::new("http://docs.rs"));
    assert!(link.is_http());

    let link = Link(&Path::new("http://example.com"));
    assert!(link.is_http());

    let link = Link(&Path::new("https://example.com"));
    assert!(link.is_http());

    let link = Link(&Path::new(
        "https://example.com/sub1/sub2/elem.html#section",
    ));
    assert!(link.is_http());
}

#[test]
fn is_favored() {
    // None.
    let link = Link(&Path::new("https://example.com"));
    assert!(!link.is_favored());

    // doc.rs
    let link = Link(&Path::new("https://docs.rs"));
    assert!(!link.is_favored());

    let link = Link(&Path::new("https://docs.rs/regex/"));
    assert!(link.is_favored());

    let link = Link(&Path::new("https://docs.rs/regex/1.4.2"));
    assert!(link.is_favored());

    let link = Link(&Path::new("https://docs.rs/regex/1.4.2/regex"));
    assert!(link.is_favored());

    let link = Link(&Path::new(
        "https://docs.rs/regex/1.4.2/regex/struct.Regex.html",
    ));
    assert!(link.is_favored());

    let link = Link(&Path::new(
        "https://docs.rs/regex/1.4.2/regex/struct.Regex.html#examples",
    ));
    assert!(link.is_favored());

    // doc.rust-lang.org
    let link = Link(&Path::new("https://doc.rust-lang.org/"));
    assert!(!link.is_favored());

    let link = Link(&Path::new("https://doc.rust-lang.org/other"));
    assert!(!link.is_favored());

    let link = Link(&Path::new("https://doc.rust-lang.org/nightly"));
    assert!(!link.is_favored());

    let link = Link(&Path::new("https://doc.rust-lang.org/beta"));
    assert!(!link.is_favored());

    let link = Link(&Path::new("https://doc.rust-lang.org/stable"));
    assert!(!link.is_favored());

    let link = Link(&Path::new("https://doc.rust-lang.org/1.42.0"));
    assert!(!link.is_favored());

    let link = Link(&Path::new(
        "https://doc.rust-lang.org/nightly/nightly-rustc",
    ));
    assert!(!link.is_favored());

    let link = Link(&Path::new("https://doc.rust-lang.org/nightly/std"));
    assert!(link.is_favored());

    let link = Link(&Path::new("https://doc.rust-lang.org/nightly/alloc"));
    assert!(link.is_favored());

    let link = Link(&Path::new("https://doc.rust-lang.org/nightly/core"));
    assert!(link.is_favored());

    let link = Link(&Path::new("https://doc.rust-lang.org/nightly/test"));
    assert!(link.is_favored());

    let link = Link(&Path::new("https://doc.rust-lang.org/nightly/proc_macro"));
    assert!(link.is_favored());

    let link = Link(&Path::new(
        "https://doc.rust-lang.org/nightly/std/string/index.html",
    ));
    assert!(link.is_favored());

    let link = Link(&Path::new(
        "https://doc.rust-lang.org/nightly/std/string/struct.String.html",
    ));
    assert!(link.is_favored());

    let link = Link(&Path::new(
        "https://doc.rust-lang.org/nightly/std/string/struct.String.html#examples",
    ));
    assert!(link.is_favored());

    let link = Link(&Path::new(
        "https://doc.rust-lang.org/nightly/std/string/struct.String.html#method.drain",
    ));
    assert!(link.is_favored());
}

#[test]
fn is_associated_item() {
    let mut assoc_item = String::with_capacity(40);

    for item in crate::link_analyser::consts::ALL_ITEM_TYPES {
        assoc_item.clear();
        assoc_item.push('#');
        assoc_item.push_str(item);
        assoc_item.push_str(".Item");

        assert!(Link(Path::new(&assoc_item)).is_associated_item());
    }

    assert!(Link(Path::new("./#struct.Item")).is_associated_item());
    assert!(Link(Path::new("././#struct.Item")).is_associated_item());

    assert!(!Link(Path::new("struct.Item")).is_associated_item());
    assert!(!Link(Path::new(".#struct.Item")).is_associated_item());
    assert!(!Link(Path::new("#struct.Item.html")).is_associated_item());
    assert!(!Link(Path::new("../#struct.Item.html")).is_associated_item());
    assert!(!Link(Path::new("#struct.Item/rest")).is_associated_item());
    assert!(!Link(Path::new("#struct.0Item")).is_associated_item());
}

#[test]
fn is_section() {
    assert!(!Link(Path::new("#struct.Item")).is_section());
    assert!(!Link(Path::new("./#struct.Item")).is_section());
    assert!(!Link(Path::new("././#struct.Item")).is_section());

    assert!(!Link(Path::new("../#section")).is_section());
    assert!(!Link(Path::new("#section/rest")).is_section());

    assert!(Link(Path::new("#section-a")).is_section());
    assert!(Link(Path::new("#section-1")).is_section());
    assert!(Link(Path::new("#section-A")).is_section());
    assert!(Link(Path::new("#section_a")).is_section());
    assert!(Link(Path::new("#section.a")).is_section());
    assert!(Link(Path::new("#Section.a")).is_section());
    assert!(Link(Path::new("#rection.a")).is_section());
    assert!(Link(Path::new("#_ection.a")).is_section());
}

#[test]
fn is_item() {
    let mut rust_item = String::with_capacity(40);

    for item in crate::link_analyser::consts::ALL_ITEM_TYPES {
        rust_item.clear();
        rust_item.push_str(item);
        rust_item.push_str(".Type.html");
        assert!(Link(Path::new(&rust_item)).is_item());

        rust_item.clear();
        rust_item.push_str(item);
        rust_item.push_str(".Type.html#method.call");
        assert!(Link(Path::new(&rust_item)).is_item());

        rust_item.clear();
        rust_item.push_str(item);
        rust_item.push_str(".Type.html#section-name");
        assert!(Link(Path::new(&rust_item)).is_item());

        rust_item.clear();
        rust_item.push_str("./");
        rust_item.push_str(item);
        rust_item.push_str(".Type.html");
        assert!(Link(Path::new(&rust_item)).is_item());

        rust_item.clear();
        rust_item.push_str("../");
        rust_item.push_str(item);
        rust_item.push_str(".Type.html");
        assert!(Link(Path::new(&rust_item)).is_item());

        rust_item.clear();
        rust_item.push_str("../mod1/mod2/");
        rust_item.push_str(item);
        rust_item.push_str(".Type.html");
        assert!(Link(Path::new(&rust_item)).is_item());
    }

    assert!(!Link(Path::new("#section")).is_item());
    assert!(!Link(Path::new("#fn.associated_item")).is_item());
    assert!(!Link(Path::new("https://docs.rs/regex")).is_item());
    assert!(!Link(Path::new("http://example.com")).is_item());
    assert!(!Link(Path::new("mod1")).is_item());
    assert!(!Link(Path::new("../mod1/mod2/index.html#section")).is_item());
}

#[test]
fn is_module() {
    assert!(Link(Path::new("mod1")).is_module());
    assert!(Link(Path::new("mod1#section")).is_module());
    assert!(Link(Path::new("index.html")).is_module());
    assert!(Link(Path::new("index.html#section")).is_module());
    assert!(Link(Path::new("mod1/mod2")).is_module());
    assert!(Link(Path::new("./mod1/mod2")).is_module());
    assert!(Link(Path::new("../mod1/mod2")).is_module());
    assert!(Link(Path::new("../mod1/mod2#section")).is_module());
    assert!(Link(Path::new("../mod1/mod2/index.html")).is_module());
    assert!(Link(Path::new("../mod1/mod2/index.html#section")).is_module());

    assert!(!Link(Path::new("#section")).is_module());
    assert!(!Link(Path::new("#fn.associated_item")).is_module());
    assert!(!Link(Path::new("struct.Type.html#fn.associated_item")).is_module());
    assert!(!Link(Path::new("struct.Type.html#section")).is_module());
    assert!(!Link(Path::new("https://docs.rs/regex/latest/regex/index.html")).is_module());
    assert!(!Link(Path::new("http://example.com")).is_module());
}
