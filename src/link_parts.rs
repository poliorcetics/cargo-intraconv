use crate::ConversionOptions;
use crate::Krate;

use regex::Regex;
use std::borrow::Cow;
use std::path::{Component, Path};

pub fn link_parts<'a>(
    path: &'a Path,
    opts: &ConversionOptions,
) -> Result<LinkParts<'a>, &'a std::ffi::OsStr> {
    favored_parts(path, opts)
        .or_else(|| start_middle_end(path, &opts.krate))
        .ok_or_else(|| path.as_os_str())
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LinkParts<'a> {
    start: Start<'a>,
    modules: Option<&'a Path>,
    end: End<'a>,
}

impl<'a> LinkParts<'a> {
    pub fn transform(self, ctx: &crate::ConversionContext) -> String {
        let mut result = String::with_capacity(60);

        match self.start {
            Start::Empty => (),
            Start::Local => {
                let needs_type_block = matches!((&self.end, self.modules), (End::Assoc(_), None));
                if needs_type_block {
                    result.push_str(ctx.current_type_block().unwrap_or("Self"))
                }
            }
            Start::Crate => result.push_str("crate"),
            Start::Mod(s) => result.push_str(s),
            Start::Supers(n) => {
                for _ in 0..(n - 1) {
                    result.push_str("super::");
                }
                result.push_str("super");
            }
        }

        for c in self.modules.unwrap_or_else(|| Path::new("")).components() {
            let c = c.as_os_str().to_str().expect("Has been checked already");
            if !result.is_empty() {
                result.push_str("::");
            }
            result.push_str(c);
        }

        match &self.end {
            End::Section(Section { name }) => {
                result.push('#');
                result.push_str(name);
            }
            End::Assoc(AssociatedItem { dis: _, name }) => {
                if !result.is_empty() {
                    result.push_str("::");
                }
                result.push_str(name);
            }
            End::Module { name, section } => {
                if !result.is_empty() {
                    result.push_str("::");
                }

                result.push_str(name.as_ref());

                if let Some(Section { name }) = section {
                    result.push('#');
                    result.push_str(name);
                }
            }
            End::Item {
                dis: _,
                name,
                added,
            } => {
                if !result.is_empty() {
                    result.push_str("::");
                }

                result.push_str(name);

                match added {
                    None => (),
                    Some(AssocOrSection::Section(Section { name })) => {
                        // Put the suffix disambiguator before the section when
                        // there is one.
                        if let Disambiguator::Suffix(s) = self.dis() {
                            result.push_str(s);
                        }

                        result.push('#');
                        result.push_str(name);
                    }
                    Some(AssocOrSection::Assoc(AssociatedItem { dis: _, name })) => {
                        result.push_str("::");
                        result.push_str(name);
                    }
                }
            }
        }

        if let Disambiguator::Prefix(s) = self.dis() {
            if ctx.options().disambiguate {
                result.insert_str(0, s);
            }
        } else if let Disambiguator::Suffix(s) = self.dis() {
            let disambiguation_already_done = matches!(&self.end, End::Item {
                dis: _,
                name: _,
                added: Some(AssocOrSection::Section(_)),
            });

            if !disambiguation_already_done {
                result.push_str(s);
            }
        }

        result
    }

    fn dis(&self) -> Disambiguator {
        match self.end {
            // NOTE: maybe this could use a context to see if it should point
            // to a module or type ?
            End::Section(_) => Disambiguator::Empty,
            End::Module {
                name: _,
                section: _,
            } => Disambiguator::from("mod"),
            End::Assoc(ref assoc) => assoc.dis,
            End::Item {
                dis: _,
                name: _,
                added: Some(AssocOrSection::Assoc(ref assoc)),
            } => assoc.dis,
            End::Item {
                dis,
                name: _,
                added: _,
            } => dis,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum Disambiguator {
    Empty,
    Prefix(&'static str),
    Suffix(&'static str),
}

impl From<&'_ str> for Disambiguator {
    fn from(s: &'_ str) -> Self {
        match s {
            "struct" | "enum" | "trait" | "union" | "type" => Self::Prefix("type@"),
            "const" | "static" | "value" => Self::Prefix("value@"),
            "derive" | "attr" => Self::Prefix("macro@"),
            // See consts.rs for more information on this being commented out.
            // "primitive" => Self::Prefix("prim@"),
            "mod" => Self::Prefix("mod@"),
            "fn" | "method" => Self::Suffix("()"),
            "macro" => Self::Suffix("!"),
            _ => Self::Empty,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum Start<'a> {
    Empty,
    Local,
    Crate,
    Mod(&'a str),
    Supers(usize),
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum End<'a> {
    Section(Section<'a>),
    Assoc(AssociatedItem<'a>),
    Item {
        dis: Disambiguator,
        name: &'a str,
        added: Option<AssocOrSection<'a>>,
    },
    Module {
        name: Cow<'a, str>,
        section: Option<Section<'a>>,
    },
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum AssocOrSection<'a> {
    Assoc(AssociatedItem<'a>),
    Section(Section<'a>),
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct AssociatedItem<'a> {
    dis: Disambiguator,
    name: &'a str,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct Section<'a> {
    name: &'a str,
}

fn favored_parts<'a>(path: &'a Path, opts: &ConversionOptions) -> Option<LinkParts<'a>> {
    fn is_http(path: &Path) -> bool {
        (path.starts_with("http:") || path.starts_with("https:")) && path.components().count() >= 2
    }

    if !is_http(path) || !opts.favored_links {
        return None;
    }

    let comp_count = path.components().count();

    let mut comps = path.components();
    let _http = comps.next()?;
    let domain = comps.next()?;

    const DOCS_RS: &str = "docs.rs";
    const DOC_RUST_LANG_ORG: &str = "doc.rust-lang.org";

    // Checking the domain for favored links patterns.
    match domain {
        // https://docs.rs/regex
        Component::Normal(dom) if dom == DOCS_RS && comp_count >= 3 => {
            favored_docs_rs(path, &opts.krate)
        }
        Component::Normal(dom) if dom == DOC_RUST_LANG_ORG && comp_count >= 3 => {
            favored_doc_rust_lang_org(path, &opts.krate)
        }
        _ => None,
    }
}

fn favored_docs_rs<'a>(path: &'a Path, krate: &Krate) -> Option<LinkParts<'a>> {
    let mut comps = path.components();
    let http = comps.next()?;
    debug_assert!([
        Component::Normal("http:".as_ref()),
        Component::Normal("https:".as_ref())
    ]
    .contains(&http));
    let domain = comps.next()?;
    debug_assert_eq!(Component::Normal("docs.rs".as_ref()), domain);

    let untreated = path
        .strip_prefix(http)
        .expect("Stripping http")
        .strip_prefix(domain)
        .expect("Stripping domain name");

    let crate_name = comps.next()?.as_os_str();

    // https://docs.rs/crate/regex/ is NOT a link to the documentation for
    // the `regex` crate.
    if crate_name == "crate" {
        return None;
    }

    let crate_only_link_parts = || {
        let start = Start::Empty;
        let modules = None;

        // Early return that avoids making a conversion to UTF-8 when
        // possible.
        if crate_name == krate.name() {
            return Some(LinkParts {
                start,
                modules,
                end: End::Module {
                    name: "crate".into(),
                    section: None,
                },
            });
        }

        let crate_name = crate_name.to_str()?;
        if crate::RUST_IDENTIFIER_RE.is_match(crate_name) {
            Some(LinkParts {
                start,
                modules,
                end: End::Module {
                    name: crate_name.into(),
                    section: None,
                },
            })
        } else {
            // Attempts to fix the crate name to be a valid Rust
            // identifier.
            let crate_name = crate_name.replace('-', "_");
            if crate::RUST_IDENTIFIER_RE.is_match(&crate_name) {
                Some(LinkParts {
                    start,
                    modules,
                    end: End::Module {
                        name: crate_name.into(),
                        section: None,
                    },
                })
            } else {
                None
            }
        }
    };

    let version = match comps.next() {
        Some(v) => {
            lazy_static::lazy_static! {
                static ref VERSION_REGEX: Regex = Regex::new(r"(?:\d+\.\d+\.\d+|latest)").unwrap();
            }
            let v = v.as_os_str().to_str().unwrap_or("");
            if !VERSION_REGEX.is_match(v) {
                return None;
            }

            v
        }
        None => return crate_only_link_parts(),
    };

    match comps.next().map(|x| x.as_os_str()) {
        Some(_) => (),
        None => return crate_only_link_parts(),
    }

    let untreated = untreated
        .strip_prefix(crate_name)
        .expect("Removing crate first identifier")
        .strip_prefix(version)
        .expect("Removing version");

    start_middle_end(untreated, krate)
}

fn favored_doc_rust_lang_org<'a>(path: &'a Path, krate: &Krate) -> Option<LinkParts<'a>> {
    let mut comps = path.components();
    let http = comps.next()?;
    debug_assert!([
        Component::Normal("http:".as_ref()),
        Component::Normal("https:".as_ref())
    ]
    .contains(&http));
    let domain = comps.next()?;
    debug_assert_eq!(Component::Normal("doc.rust-lang.org".as_ref()), domain);

    let untreated = path
        .strip_prefix(http)
        .expect("Stripping http")
        .strip_prefix(domain)
        .expect("Stripping domain name");

    // Ensure the channel can be converted to a valid "&str".
    let channel_or_crate = comps.next()?.as_os_str();

    const CRATES: [&str; 6] = [
        "std",
        "alloc",
        "core",
        "test",
        "proc_macro",
        "nightly-rustc",
    ];
    const CHANNELS: [&str; 3] = ["nightly", "beta", "stable"];
    lazy_static::lazy_static! {
        static ref VERSION_REGEX: Regex = Regex::new(r"1\.\d+\.\d+").unwrap();
    }

    // Check if the channel is a valid one.
    let (linked_crate, untreated) = if CHANNELS.iter().any(|c| c == &channel_or_crate)
        || VERSION_REGEX.is_match(channel_or_crate.to_str()?)
    {
        (
            comps.next()?.as_os_str(),
            untreated
                .strip_prefix(channel_or_crate)
                .expect("Stripping channel from path"),
        )
    } else {
        (channel_or_crate, untreated)
    };

    if !CRATES.iter().any(|c| c == &linked_crate) {
        return None;
    }

    if untreated.components().next().is_none() {
        let k = if linked_crate == krate.name() {
            "crate"
        } else {
            linked_crate.to_str()?
        };

        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Module {
                name: k.into(),
                section: None,
            },
        })
    } else {
        start_middle_end(untreated, krate)
    }
}

fn start_middle_end<'a>(path: &'a Path, krate: &Krate) -> Option<LinkParts<'a>> {
    associated_item_parts(path)
        .or_else(|| section_parts(path, krate))
        .or_else(|| item_parts(path, krate))
        .or_else(|| module_parts(path, krate))
}

fn associated_item_parts(path: &Path) -> Option<LinkParts<'_>> {
    // It is not invalid to have './' before the associated item when it
    // points to a module-level item.
    let mut comps = path.components().skip_while(|c| c == &Component::CurDir);
    let assoc_item = comps.next()?.as_os_str();
    // The associated item MUST be the last element.
    if comps.next().is_some() {
        return None;
    }

    lazy_static::lazy_static! {
        static ref ASSOC_ITEM: Regex = Regex::new(&format!(
            r"^#(?P<dis>{})\.(?P<name>{})$",
            crate::ITEM_TYPES.as_str(),
            crate::RUST_IDENTIFIER,
        )).unwrap();
    }

    let captures = ASSOC_ITEM.captures(assoc_item.to_str()?)?;
    let dis = Disambiguator::from(captures.name("dis")?.as_str());
    let name = captures.name("name")?.as_str();

    let start = Start::Local;
    let modules = None;
    let end = End::Assoc(AssociatedItem { dis, name });

    Some(LinkParts {
        start,
        modules,
        end,
    })
}

fn section_parts<'a>(path: &'a Path, krate: &Krate) -> Option<LinkParts<'a>> {
    if associated_item_parts(path).is_some() {
        return None;
    }

    let section = path.file_name()?;

    lazy_static::lazy_static! {
        static ref SECTION: Regex = Regex::new(
            &format!(r"^(?P<name>{})$", crate::HTML_SECTION)
        ).unwrap();
    }

    let captures = SECTION.captures(section.to_str()?)?;
    let name = captures.name("name")?.as_str().strip_prefix('#')?;

    let end = End::Section(Section { name });

    match path.parent() {
        Some(untreated) if !untreated.as_os_str().is_empty() => {
            start_and_middle(untreated, end, krate)
        }
        _ => Some(LinkParts {
            start: Start::Local,
            modules: None,
            end,
        }),
    }
}

fn item_parts<'a>(path: &'a Path, krate: &Krate) -> Option<LinkParts<'a>> {
    lazy_static::lazy_static! {
        static ref ITEM: Regex = Regex::new(&format!(
            r"^(?P<i_ty>{ty})\.(?P<i_name>{rid})\.html(?:#(?P<ai_ty>{ty})\.(?P<ai_name>{rid})|(?P<section>{sec}))?$",
            ty = crate::ITEM_TYPES.as_str(),
            rid = crate::RUST_IDENTIFIER,
            sec = crate::HTML_SECTION,
        )).unwrap();
    }

    let last_comp = path.file_name()?.to_str()?;
    let captures = ITEM.captures(last_comp)?;

    let item_type = captures.name("i_ty")?.as_str();
    let item_name = captures.name("i_name")?.as_str();

    let assoc_type = captures.name("ai_ty").map(|x| x.as_str());
    let assoc_name = captures.name("ai_name").map(|x| x.as_str());
    let section = captures.name("section").map(|x| {
        x.as_str()
            .strip_prefix('#')
            .expect("section should have # prefix")
    });

    let untreated = path.parent().unwrap_or_else(|| Path::new(""));

    let added = if let Some(section) = section {
        Some(AssocOrSection::Section(Section { name: section }))
    } else if let (Some(assoc_type), Some(assoc_name)) = (assoc_type, assoc_name) {
        Some(AssocOrSection::Assoc(AssociatedItem {
            dis: Disambiguator::from(assoc_type),
            name: assoc_name,
        }))
    } else {
        None
    };

    let end = End::Item {
        dis: Disambiguator::from(item_type),
        name: item_name,
        added,
    };

    start_and_middle(untreated, end, krate)
}

fn module_parts<'a>(path: &'a Path, krate: &Krate) -> Option<LinkParts<'a>> {
    lazy_static::lazy_static! {
        static ref LONG_FORM: Regex = Regex::new(
            &format!(
                r"^(?:index.html|(?P<name>{}))(?P<section>{})?$",
                crate::RUST_IDENTIFIER,
                crate::HTML_SECTION,
            )
        ).unwrap();
    }

    let last = path.file_name()?.to_str()?;
    let captures = LONG_FORM.captures(last)?;

    let end = if let Some(section) = captures.name("section") {
        // - mod#section
        // - index.html#section
        // - path/to/mod#section
        // - path/to/mod/index.html#section
        let section = section
            .as_str()
            .strip_prefix('#')
            .expect("Matched in the regex for the section name");

        // Ensuring the module name is not lost when it is present.
        // - mod#section
        // - path/to/mod#section
        if let Some(name) = captures.name("name") {
            End::Module {
                name: name.as_str().into(),
                section: Some(Section { name: section }),
            }
        } else if last.starts_with("index.html") && path.components().count() == 1 {
            End::Module {
                name: "self".into(),
                section: Some(Section { name: section }),
            }
        } else {
            End::Section(Section { name: section })
        }
    } else if let Some(name) = captures.name("name") {
        // - mod
        // - path/to/mod
        let name = name.as_str();
        End::Module {
            name: name.into(),
            section: None,
        }
    } else {
        // - index.html
        // - ./index.html
        // - ../index.html
        // - path/to/mod/index.html
        match path.parent().map(|p| p.components().next_back()).flatten() {
            // - index.html
            // - ./index.html
            Some(Component::CurDir) | None => End::Module {
                name: "self".into(),
                section: None,
            },
            // - ../index.html
            Some(Component::ParentDir) => End::Module {
                name: "super".into(),
                section: None,
            },
            // - path/to/mod/index.html
            Some(Component::Normal(os)) => {
                let s = os.to_str()?;
                if !crate::RUST_IDENTIFIER_RE.is_match(s) {
                    return None;
                }

                End::Module {
                    name: if s == krate.name() { "crate" } else { s }.into(),
                    section: None,
                }
            }
            _ => return None,
        }
    };

    let untreated = path.parent().unwrap_or_else(|| Path::new(""));
    let untreated = if (None, None) == (captures.name("name"), captures.name("section")) {
        // - index.html
        // - ./index.html
        // - ../index.html
        // - path/to/mod/index.html
        untreated.parent().unwrap_or_else(|| Path::new(""))
    } else {
        untreated
    };

    start_and_middle(untreated, end, krate)
}

fn start_and_middle<'a>(untreated: &'a Path, end: End<'a>, krate: &Krate) -> Option<LinkParts<'a>> {
    if untreated.components().next().is_none() {
        return Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end,
        });
    }

    let supers = untreated
        .components()
        .take_while(|p| p == &Component::ParentDir)
        .count();
    let untreated = (0..supers).fold(untreated, |acc, _| {
        acc.strip_prefix(Component::ParentDir)
            .expect("supers was checked before")
    });

    let (start, untreated) = match untreated.components().next() {
            None if supers == 0 => (Start::Empty, Path::new("")),
            None /* if supers > 0 */ => (Start::Supers(supers), Path::new("")),
            Some(Component::CurDir) => (
                Start::Local,
                untreated
                    .strip_prefix(Component::CurDir)
                    .expect("CurDir component"),
            ),
            Some(Component::Normal(os)) if os == krate.name() => (
                Start::Crate,
                untreated
                    .strip_prefix(Component::Normal(os))
                    .expect("Normal component"),
            ),
            Some(Component::Normal(os)) if supers == 0 => {
                let s = os.to_str()?;
                if !crate::RUST_IDENTIFIER_RE.is_match(s) {
                    return None;
                }
                (Start::Mod(s), untreated.strip_prefix(os).expect("Stripping first module"))
            }
            Some(Component::Normal(_)) /* if supers > 0 */ => {
                (Start::Supers(supers), untreated)
            }
            Some(Component::Prefix(_)) | Some(Component::RootDir) => return None,
            Some(Component::ParentDir) => unreachable!(),
        };

    // Check all the other component to ensure they are either `.` or `..`
    // or a valid rust identifier (a module name).
    if untreated.components().all(|c| match c {
        Component::CurDir | Component::ParentDir => true,
        Component::Normal(mod_name) => {
            let mod_name = match mod_name.to_str() {
                Some(mn) => mn,
                None => return false,
            };
            crate::RUST_IDENTIFIER_RE.is_match(mod_name)
        }
        _ => false,
    }) {
        Some(LinkParts {
            start,
            modules: if untreated.components().next().is_some() {
                Some(untreated)
            } else {
                None
            },
            end,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests;
