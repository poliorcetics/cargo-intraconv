use regex::Regex;
use std::borrow::Cow;
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

    /// Returns `None` when `self` is not an associated item link.
    ///
    /// When it is the parts are found while limiting allocations to the
    /// strict minimum.
    fn associated_item_parts(&self) -> Option<LinkParts<'a>> {
        // It is not invalid to have './' before the associated item when it
        // points to a module-level item.
        let mut comps = self.0.components().skip_while(|c| c == &Component::CurDir);
        let assoc_item = comps.next()?.as_os_str().to_str()?;
        // The associated item MUST be the last element.
        if comps.next().is_some() {
            return None;
        }

        lazy_static::lazy_static! {
            static ref ASSOC_ITEM: Regex = Regex::new(&format!(
                r"^#(?P<dis>{})\.(?P<name>{})$",
                crate::link_analyser::consts::ITEM_TYPES.as_str(),
                crate::link_analyser::consts::RUST_IDENTIFIER,
            )).unwrap();
        }

        let captures = ASSOC_ITEM.captures(assoc_item)?;
        let dis = Disambiguator::from(captures.name("dis")?.as_str());
        let name = captures.name("name")?.as_str().into();

        let start = LinkStart::Local;
        let modules = None;
        let end = LinkEnd::Assoc(AssociatedItem { dis, name });

        Some(LinkParts {
            start,
            modules,
            end,
        })
    }

    /// Returns `None` when `self` is not a section link.
    ///
    /// When it is the parts are found while limiting allocations to the
    /// strict minimum.
    fn section_parts(&self) -> Option<LinkParts<'a>> {
        if self.associated_item_parts().is_some() {
            return None;
        }

        // It is not invalid to have './' before the section when it points to
        // a module-level item.
        let mut comps = self.0.components().skip_while(|c| c == &Component::CurDir);
        let section = comps.next()?.as_os_str().to_str()?;
        // The section MUST be the last element.
        if comps.next().is_some() {
            return None;
        }

        lazy_static::lazy_static! {
            static ref SECTION: Regex = Regex::new(
                &format!(r"^(?P<name>{})$", crate::link_analyser::consts::HTML_SECTION)
            ).unwrap();
        }

        let captures = SECTION.captures(section)?;
        let name = captures.name("name")?.as_str().strip_prefix('#')?.into();

        let start = LinkStart::Local;
        let modules = None;
        let end = LinkEnd::Section { name };

        Some(LinkParts {
            start,
            modules,
            end,
        })
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum Disambiguator {
    None,
    Start(&'static str),
    End(&'static str),
}

impl From<&'_ str> for Disambiguator {
    fn from(s: &'_ str) -> Self {
        match s {
            "struct" | "enum" | "trait" | "union" | "type" => Self::Start("type@"),
            "const" | "static" | "value" => Self::Start("value@"),
            "derive" | "attr" => Self::Start("macro@"),
            "primitive" => Self::Start("prim@"),
            "mod" => Self::Start("mod@"),
            "fn" | "method" => Self::End("()"),
            "macro" => Self::End("!"),
            _ => Self::None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum LinkStart<'a> {
    Local,
    Mod(Cow<'a, str>),
    Crate(Cow<'a, str>),
    Supers(usize),
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct Modules<'a>(&'a Path);

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum LinkEnd<'a> {
    Section {
        name: Cow<'a, str>,
    },
    Assoc(AssociatedItem<'a>),
    Item {
        dis: Disambiguator,
        name: Cow<'a, str>,
        assoc: Option<AssociatedItem<'a>>,
    },
    Module {
        name: Cow<'a, str>,
    },
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct AssociatedItem<'a> {
    dis: Disambiguator,
    name: Cow<'a, str>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct LinkParts<'a> {
    start: LinkStart<'a>,
    modules: Option<Modules<'a>>,
    end: LinkEnd<'a>,
}

impl<'a> LinkParts<'a> {
    fn dis(&self) -> Disambiguator {
        match self.end {
            LinkEnd::Section { name: _ } | LinkEnd::Module { name: _ } => {
                Disambiguator::from("mod")
            }
            LinkEnd::Assoc(ref assoc) => assoc.dis,
            LinkEnd::Item {
                dis,
                name: _,
                ref assoc,
            } => assoc.as_ref().map_or(dis, |a| a.dis),
        }
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
fn associated_item_parts() {
    let mut assoc_item = String::with_capacity(40);

    for &item in crate::link_analyser::consts::ALL_ITEM_TYPES {
        assoc_item.clear();
        assoc_item.push('#');
        assoc_item.push_str(item);
        assoc_item.push_str(".Item");

        assert_eq!(
            Link(Path::new(&assoc_item)).associated_item_parts(),
            Some(LinkParts {
                start: LinkStart::Local,
                modules: None,
                end: LinkEnd::Assoc(AssociatedItem {
                    dis: Disambiguator::from(item),
                    name: Cow::from("Item")
                }),
            })
        );
    }

    assert_eq!(
        Link(Path::new("./#struct.Item")).associated_item_parts(),
        Some(LinkParts {
            start: LinkStart::Local,
            modules: None,
            end: LinkEnd::Assoc(AssociatedItem {
                dis: Disambiguator::from("struct"),
                name: Cow::from("Item")
            }),
        })
    );
    assert_eq!(
        Link(Path::new("././#struct.Item")).associated_item_parts(),
        Some(LinkParts {
            start: LinkStart::Local,
            modules: None,
            end: LinkEnd::Assoc(AssociatedItem {
                dis: Disambiguator::from("struct"),
                name: Cow::from("Item")
            }),
        })
    );

    assert_eq!(Link(Path::new("struct.Item")).associated_item_parts(), None);
    assert_eq!(
        Link(Path::new(".#struct.Item")).associated_item_parts(),
        None
    );
    assert_eq!(
        Link(Path::new("#struct.Item.html")).associated_item_parts(),
        None
    );
    assert_eq!(
        Link(Path::new("../#struct.Item.html")).associated_item_parts(),
        None
    );
    assert_eq!(
        Link(Path::new("#struct.Item/rest")).associated_item_parts(),
        None
    );
    assert_eq!(
        Link(Path::new("#struct.0Item")).associated_item_parts(),
        None
    );
}

#[test]
fn section_parts() {
    assert_eq!(Link(Path::new("#struct.Item")).section_parts(), None);
    assert_eq!(Link(Path::new("./#struct.Item")).section_parts(), None);
    assert_eq!(Link(Path::new("././#struct.Item")).section_parts(), None);

    assert_eq!(Link(Path::new("../#section")).section_parts(), None);
    assert_eq!(Link(Path::new("#section/rest")).section_parts(), None);

    assert_eq!(
        Link(Path::new("#section-a")).section_parts(),
        Some(LinkParts {
            start: LinkStart::Local,
            modules: None,
            end: LinkEnd::Section {
                name: "section-a".into()
            },
        })
    );
    assert_eq!(
        Link(Path::new("#section-1")).section_parts(),
        Some(LinkParts {
            start: LinkStart::Local,
            modules: None,
            end: LinkEnd::Section {
                name: "section-1".into()
            },
        })
    );
    assert_eq!(
        Link(Path::new("#section-A")).section_parts(),
        Some(LinkParts {
            start: LinkStart::Local,
            modules: None,
            end: LinkEnd::Section {
                name: "section-A".into()
            },
        })
    );
    assert_eq!(
        Link(Path::new("#section_a")).section_parts(),
        Some(LinkParts {
            start: LinkStart::Local,
            modules: None,
            end: LinkEnd::Section {
                name: "section_a".into()
            },
        })
    );
    assert_eq!(
        Link(Path::new("#section.a")).section_parts(),
        Some(LinkParts {
            start: LinkStart::Local,
            modules: None,
            end: LinkEnd::Section {
                name: "section.a".into()
            },
        })
    );
    assert_eq!(
        Link(Path::new("#Section.a")).section_parts(),
        Some(LinkParts {
            start: LinkStart::Local,
            modules: None,
            end: LinkEnd::Section {
                name: "Section.a".into()
            },
        })
    );
    assert_eq!(
        Link(Path::new("#rection.a")).section_parts(),
        Some(LinkParts {
            start: LinkStart::Local,
            modules: None,
            end: LinkEnd::Section {
                name: "rection.a".into()
            },
        })
    );
    assert_eq!(
        Link(Path::new("#0ection.a")).section_parts(),
        Some(LinkParts {
            start: LinkStart::Local,
            modules: None,
            end: LinkEnd::Section {
                name: "0ection.a".into()
            },
        })
    );
    assert_eq!(
        Link(Path::new("#_ection.a")).section_parts(),
        Some(LinkParts {
            start: LinkStart::Local,
            modules: None,
            end: LinkEnd::Section {
                name: "_ection.a".into()
            },
        })
    );
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

#[test]
fn disambiguate_from() {
    use Disambiguator::*;

    assert_eq!(Start("type@"), Disambiguator::from("struct"));
    assert_eq!(Start("type@"), Disambiguator::from("enum"));
    assert_eq!(Start("type@"), Disambiguator::from("trait"));
    assert_eq!(Start("type@"), Disambiguator::from("union"));
    assert_eq!(Start("type@"), Disambiguator::from("type"));

    assert_eq!(Start("value@"), Disambiguator::from("const"));
    assert_eq!(Start("value@"), Disambiguator::from("static"));
    assert_eq!(Start("value@"), Disambiguator::from("value"));

    assert_eq!(Start("macro@"), Disambiguator::from("derive"));
    assert_eq!(Start("macro@"), Disambiguator::from("attr"));

    assert_eq!(Start("prim@"), Disambiguator::from("primitive"));

    assert_eq!(Start("mod@"), Disambiguator::from("mod"));

    assert_eq!(End("()"), Disambiguator::from("fn"));
    assert_eq!(End("()"), Disambiguator::from("method"));

    assert_eq!(End("!"), Disambiguator::from("macro"));

    assert_eq!(None, Disambiguator::from("other"));
    assert_eq!(None, Disambiguator::from("soomething else"));
}

#[test]
fn link_parts_dis() {
    let mut lp = LinkParts {
        start: LinkStart::Local,
        modules: None,
        end: LinkEnd::Section {
            name: "name".into(),
        },
    };
    assert_eq!(lp.dis(), Disambiguator::Start("mod@"));

    lp.end = LinkEnd::Module {
        name: "name".into(),
    };
    assert_eq!(lp.dis(), Disambiguator::Start("mod@"));

    lp.end = LinkEnd::Assoc(AssociatedItem {
        dis: Disambiguator::from("fn"),
        name: "name".into(),
    });
    assert_eq!(lp.dis(), Disambiguator::End("()"));

    lp.end = LinkEnd::Item {
        dis: Disambiguator::from("struct"),
        name: "name".into(),
        assoc: None,
    };
    assert_eq!(lp.dis(), Disambiguator::Start("type@"));

    lp.end = LinkEnd::Item {
        dis: Disambiguator::from("struct"),
        name: "name".into(),
        assoc: Some(AssociatedItem {
            dis: Disambiguator::from("fn"),
            name: "name_2".into(),
        }),
    };
    assert_eq!(lp.dis(), Disambiguator::End("()"));
}
