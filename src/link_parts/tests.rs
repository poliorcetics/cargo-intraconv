use super::*;

#[test]
fn test_favored_parts() {
    // Wrong version
    let link = Path::new("https://docs.rs/tracing-serde/badge.svg");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_NO_FAV),
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
    );

    // FAVORED INACTIVE
    let link = Path::new("https://docs.rs/regex/1.4.2/regex");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_NO_FAV),
        None
    );

    // SAME CRATE
    let link = Path::new("https://docs.rs/krate-name/1.2.3/krate/struct.Type.html");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://docs.rs/crate/krate-name/1.2.3/krate/struct.Type.html");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://docs.rs");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://docs.rs/crate");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://docs.rs/regex/");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://docs.rs/regex/1.4.2");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://docs.rs/regex/latest");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://docs.rs/crate/regex/");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://docs.rs/crate/regex/latest/");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://docs.rs/crate/regex/1.4.2");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://docs.rs/regex/1.4.2/regex");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://docs.rs/regex/latest/regex");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://docs.rs/crate/regex/1.4.2/regex");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://docs.rs/regex/1.4.2/regex/struct.Regex.html");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://docs.rs/regex/1.4.2/regex/struct.Regex.html#examples");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://docs.rs/regex/1.4.2/regex/struct.Regex.html#method.is_match");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://docs.rs/regex/1.4.2/regex/bytes/index.html");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://docs.rs/regex/1.4.2/regex/bytes/index.html#syntax");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://docs.rs/regex/1.4.2/regex/bytes/struct.Regex.html#examples");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link =
        Path::new("https://docs.rs/regex/1.4.2/regex/bytes/struct.Regex.html#method.is_match");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    // doc.rust-lang.org
    // Missing/Invalid crate
    let link = Path::new("https://doc.rust-lang.org/");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/other");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/nightly");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/beta");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/stable");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/1.42.0");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/nightly/nightly-rustc");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    // Valid crate, short form
    let link = Path::new("https://doc.rust-lang.org/std");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/alloc");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/core");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/test");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/proc_macro");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    // Valid crate, long form
    let link = Path::new("https://doc.rust-lang.org/nightly/std");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/nightly/alloc");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/nightly/core");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/nightly/test");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/nightly/proc_macro");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/nightly/std/string/index.html");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/nightly/std/string/struct.String.html");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link =
        Path::new("https://doc.rust-lang.org/nightly/std/string/struct.String.html#examples");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link =
        Path::new("https://doc.rust-lang.org/nightly/std/string/struct.String.html#method.drain");
    assert_eq!(
        favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate)
    );
}

#[test]
fn test_favored_docs_rs() {
    // Wrong version
    let link = Path::new("https://docs.rs/tracing-serde/badge.svg");
    assert_eq!(
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        None
    );

    // SAME CRATE
    let link = Path::new("https://docs.rs/krate-name/1.2.3/krate/struct.Type.html");
    assert_eq!(
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Crate,
            modules: None,
            end: End::Item {
                dis: Disambiguator::Prefix("type@"),
                name: "Type",
                added: None,
            },
        })
    );

    let link = Path::new("https://docs.rs/crate/krate-name/1.2.3/krate/struct.Type.html");
    assert_eq!(
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Crate,
            modules: None,
            end: End::Item {
                dis: Disambiguator::Prefix("type@"),
                name: "Type",
                added: None,
            },
        })
    );

    let link = Path::new("https://docs.rs");
    assert_eq!(
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        None
    );

    let link = Path::new("https://docs.rs/crate");
    assert_eq!(
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        None
    );

    let link = Path::new("https://docs.rs/regex/");
    assert_eq!(
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Module {
                name: "regex".into(),
                section: None
            },
        })
    );

    let link = Path::new("https://docs.rs/regex/1.4.2");
    assert_eq!(
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Module {
                name: "regex".into(),
                section: None
            },
        })
    );

    let link = Path::new("https://docs.rs/regex/latest");
    assert_eq!(
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Module {
                name: "regex".into(),
                section: None
            },
        })
    );

    let link = Path::new("https://docs.rs/crate/regex/");
    assert_eq!(
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Module {
                name: "regex".into(),
                section: None
            },
        })
    );

    let link = Path::new("https://docs.rs/crate/regex/latest/");
    assert_eq!(
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Module {
                name: "regex".into(),
                section: None
            },
        })
    );

    let link = Path::new("https://docs.rs/crate/regex/1.4.2");
    assert_eq!(
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Module {
                name: "regex".into(),
                section: None
            },
        })
    );

    let link = Path::new("https://docs.rs/regex/1.4.2/regex");
    assert_eq!(
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Module {
                name: "regex".into(),
                section: None
            },
        })
    );

    let link = Path::new("https://docs.rs/regex/latest/regex");
    assert_eq!(
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Module {
                name: "regex".into(),
                section: None
            },
        })
    );

    let link = Path::new("https://docs.rs/crate/regex/1.4.2/regex");
    assert_eq!(
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Module {
                name: "regex".into(),
                section: None
            },
        })
    );

    let link = Path::new("https://docs.rs/regex/1.4.2/regex/struct.Regex.html");
    assert_eq!(
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Mod("regex"),
            modules: None,
            end: End::Item {
                dis: Disambiguator::Prefix("type@"),
                name: "Regex",
                added: None,
            },
        })
    );

    let link = Path::new("https://docs.rs/regex/1.4.2/regex/struct.Regex.html#examples");
    assert_eq!(
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Mod("regex"),
            modules: None,
            end: End::Item {
                dis: Disambiguator::Prefix("type@"),
                name: "Regex",
                added: Some(AssocOrSection::Section(Section { name: "examples" })),
            },
        })
    );

    let link = Path::new("https://docs.rs/regex/1.4.2/regex/struct.Regex.html#method.is_match");
    assert_eq!(
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Mod("regex"),
            modules: None,
            end: End::Item {
                dis: Disambiguator::Prefix("type@"),
                name: "Regex",
                added: Some(AssocOrSection::Assoc(AssociatedItem {
                    dis: Disambiguator::Suffix("()"),
                    name: "is_match"
                })),
            },
        })
    );

    let link = Path::new("https://docs.rs/regex/1.4.2/regex/bytes/index.html");
    assert_eq!(
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Mod("regex"),
            modules: None,
            end: End::Module {
                name: "bytes".into(),
                section: None,
            },
        })
    );

    let link = Path::new("https://docs.rs/regex/1.4.2/regex/bytes/index.html#syntax");
    assert_eq!(
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Mod("regex"),
            modules: Some(Path::new("bytes")),
            end: End::Section(Section { name: "syntax" }),
        })
    );

    let link = Path::new("https://docs.rs/regex/1.4.2/regex/bytes/struct.Regex.html#examples");
    assert_eq!(
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Mod("regex"),
            modules: Some(Path::new("bytes")),
            end: End::Item {
                dis: Disambiguator::Prefix("type@"),
                name: "Regex",
                added: Some(AssocOrSection::Section(Section { name: "examples" })),
            },
        })
    );

    let link =
        Path::new("https://docs.rs/regex/1.4.2/regex/bytes/struct.Regex.html#method.is_match");
    assert_eq!(
        favored_docs_rs(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Mod("regex"),
            modules: Some(Path::new("bytes")),
            end: End::Item {
                dis: Disambiguator::Prefix("type@"),
                name: "Regex",
                added: Some(AssocOrSection::Assoc(AssociatedItem {
                    dis: Disambiguator::Suffix("()"),
                    name: "is_match"
                })),
            },
        })
    );
}

#[test]
fn test_favored_doc_rust_lang_org() {
    // Missing/Invalid crate
    let link = Path::new("https://doc.rust-lang.org/");
    assert_eq!(
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        None
    );

    let link = Path::new("https://doc.rust-lang.org/other");
    assert_eq!(
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        None
    );

    let link = Path::new("https://doc.rust-lang.org/nightly");
    assert_eq!(
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        None
    );

    let link = Path::new("https://doc.rust-lang.org/beta");
    assert_eq!(
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        None
    );

    let link = Path::new("https://doc.rust-lang.org/stable");
    assert_eq!(
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        None
    );

    let link = Path::new("https://doc.rust-lang.org/1.42.0");
    assert_eq!(
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        None
    );

    let link = Path::new("https://doc.rust-lang.org/nightly/nightly-rustc");
    assert_eq!(
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        None
    );

    // Valid crate, short form
    let link = Path::new("https://doc.rust-lang.org/std");
    assert_eq!(
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Module {
                name: "std".into(),
                section: None,
            }
        })
    );

    let link = Path::new("https://doc.rust-lang.org/alloc");
    assert_eq!(
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Module {
                name: "alloc".into(),
                section: None,
            }
        })
    );

    let link = Path::new("https://doc.rust-lang.org/core");
    assert_eq!(
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Module {
                name: "core".into(),
                section: None,
            }
        })
    );

    let link = Path::new("https://doc.rust-lang.org/test");
    assert_eq!(
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Module {
                name: "test".into(),
                section: None,
            }
        })
    );

    let link = Path::new("https://doc.rust-lang.org/proc_macro");
    assert_eq!(
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Module {
                name: "proc_macro".into(),
                section: None,
            }
        })
    );

    // Valid crate, long form
    let link = Path::new("https://doc.rust-lang.org/nightly/std");
    assert_eq!(
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Module {
                name: "std".into(),
                section: None
            },
        }),
    );

    let link = Path::new("https://doc.rust-lang.org/nightly/alloc");
    assert_eq!(
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Module {
                name: "alloc".into(),
                section: None
            },
        }),
    );

    let link = Path::new("https://doc.rust-lang.org/nightly/core");
    assert_eq!(
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Module {
                name: "core".into(),
                section: None
            },
        }),
    );

    let link = Path::new("https://doc.rust-lang.org/nightly/test");
    assert_eq!(
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Module {
                name: "test".into(),
                section: None,
            },
        }),
    );

    let link = Path::new("https://doc.rust-lang.org/nightly/proc_macro");
    assert_eq!(
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Module {
                name: "proc_macro".into(),
                section: None,
            },
        }),
    );

    let link = Path::new("https://doc.rust-lang.org/nightly/std/string/index.html");
    assert_eq!(
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Mod("std"),
            modules: None,
            end: End::Module {
                name: "string".into(),
                section: None,
            },
        }),
    );

    let link = Path::new("https://doc.rust-lang.org/nightly/std/string/struct.String.html");
    assert_eq!(
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Mod("std"),
            modules: Some(Path::new("string")),
            end: End::Item {
                dis: Disambiguator::Prefix("type@"),
                name: "String",
                added: None,
            },
        }),
    );

    let link =
        Path::new("https://doc.rust-lang.org/nightly/std/string/struct.String.html#examples");
    assert_eq!(
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Mod("std"),
            modules: Some(Path::new("string")),
            end: End::Item {
                dis: Disambiguator::Prefix("type@"),
                name: "String",
                added: Some(AssocOrSection::Section(Section { name: "examples" })),
            },
        }),
    );

    let link =
        Path::new("https://doc.rust-lang.org/nightly/std/string/struct.String.html#method.drain");
    assert_eq!(
        favored_doc_rust_lang_org(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Mod("std"),
            modules: Some(Path::new("string")),
            end: End::Item {
                dis: Disambiguator::Prefix("type@"),
                name: "String",
                added: Some(AssocOrSection::Assoc(AssociatedItem {
                    dis: Disambiguator::Suffix("()"),
                    name: "drain",
                })),
            },
        }),
    );
}

#[test]
fn test_associated_item_parts() {
    // Matching items

    let mut assoc_item = String::with_capacity(40);

    for &item in crate::ALL_ITEM_TYPES {
        assoc_item.clear();
        assoc_item.push('#');
        assoc_item.push_str(item);
        assoc_item.push_str(".Item");

        assert_eq!(
            associated_item_parts(Path::new(&assoc_item)),
            Some(LinkParts {
                start: Start::Local,
                modules: None,
                end: End::Assoc(AssociatedItem {
                    dis: Disambiguator::from(item),
                    name: "Item",
                }),
            })
        );
    }

    let assoc_item = "./#struct.Item";
    assert_eq!(
        associated_item_parts(Path::new(assoc_item)),
        Some(LinkParts {
            start: Start::Local,
            modules: None,
            end: End::Assoc(AssociatedItem {
                dis: Disambiguator::Prefix("type@"),
                name: "Item",
            }),
        })
    );

    let assoc_item = "././#struct.Item";
    assert_eq!(
        associated_item_parts(Path::new(assoc_item)),
        Some(LinkParts {
            start: Start::Local,
            modules: None,
            end: End::Assoc(AssociatedItem {
                dis: Disambiguator::Prefix("type@"),
                name: "Item",
            }),
        })
    );

    // Failing items

    let assoc_item = "struct.Item";
    assert_eq!(associated_item_parts(Path::new(assoc_item)), None);

    let assoc_item = "struct.Item.html";
    assert_eq!(associated_item_parts(Path::new(assoc_item)), None);

    let assoc_item = ".#struct.Item";
    assert_eq!(associated_item_parts(Path::new(assoc_item)), None);

    let assoc_item = "#struct.Item.html";
    assert_eq!(associated_item_parts(Path::new(assoc_item)), None);

    let assoc_item = "../#struct.Item.html";
    assert_eq!(associated_item_parts(Path::new(assoc_item)), None);

    let assoc_item = "#struct.Item/rest";
    assert_eq!(associated_item_parts(Path::new(assoc_item)), None);

    let assoc_item = "#struct.0Item";
    assert_eq!(associated_item_parts(Path::new(assoc_item)), None);
}

#[test]
fn test_section_parts() {
    assert_eq!(
        section_parts(
            Path::new("#struct.Item"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        None
    );
    assert_eq!(
        section_parts(
            Path::new("./#struct.Item"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        None
    );
    assert_eq!(
        section_parts(
            Path::new("././#struct.Item"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        None
    );

    assert_eq!(
        section_parts(
            Path::new("#section/rest"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        None
    );

    // Short sections

    assert_eq!(
        section_parts(
            Path::new("#section-a"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        Some(LinkParts {
            start: Start::Local,
            modules: None,
            end: End::Section(Section { name: "section-a" }),
        })
    );
    assert_eq!(
        section_parts(
            Path::new("#section-1"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        Some(LinkParts {
            start: Start::Local,
            modules: None,
            end: End::Section(Section { name: "section-1" }),
        })
    );
    assert_eq!(
        section_parts(
            Path::new("#section-A"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        Some(LinkParts {
            start: Start::Local,
            modules: None,
            end: End::Section(Section { name: "section-A" }),
        })
    );
    assert_eq!(
        section_parts(
            Path::new("#section_a"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        Some(LinkParts {
            start: Start::Local,
            modules: None,
            end: End::Section(Section { name: "section_a" }),
        })
    );
    assert_eq!(
        section_parts(
            Path::new("#section.a"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        Some(LinkParts {
            start: Start::Local,
            modules: None,
            end: End::Section(Section { name: "section.a" }),
        })
    );
    assert_eq!(
        section_parts(
            Path::new("#Section.a"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        Some(LinkParts {
            start: Start::Local,
            modules: None,
            end: End::Section(Section { name: "Section.a" }),
        })
    );
    assert_eq!(
        section_parts(
            Path::new("#rection.a"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        Some(LinkParts {
            start: Start::Local,
            modules: None,
            end: End::Section(Section { name: "rection.a" }),
        })
    );
    assert_eq!(
        section_parts(
            Path::new("#0ection.a"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        Some(LinkParts {
            start: Start::Local,
            modules: None,
            end: End::Section(Section { name: "0ection.a" }),
        })
    );
    assert_eq!(
        section_parts(
            Path::new("#_ection.a"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        Some(LinkParts {
            start: Start::Local,
            modules: None,
            end: End::Section(Section { name: "_ection.a" }),
        })
    );

    assert_eq!(
        section_parts(
            Path::new("krate/#section"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        Some(LinkParts {
            start: Start::Crate,
            modules: None,
            end: End::Section(Section { name: "section" }),
        })
    );

    assert_eq!(
        section_parts(
            Path::new("../krate/#section"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        Some(LinkParts {
            start: Start::Crate,
            modules: None,
            end: End::Section(Section { name: "section" }),
        })
    );

    assert_eq!(
        section_parts(
            Path::new("mod1/#section"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        Some(LinkParts {
            start: Start::Mod("mod1"),
            modules: None,
            end: End::Section(Section { name: "section" }),
        })
    );

    assert_eq!(
        section_parts(
            Path::new("mod1/mod2/#section"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        Some(LinkParts {
            start: Start::Mod("mod1"),
            modules: Some(Path::new("mod2")),
            end: End::Section(Section { name: "section" }),
        })
    );

    assert_eq!(
        section_parts(
            Path::new("../../mod1/mod2/#section"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        Some(LinkParts {
            start: Start::Supers(2),
            modules: Some(Path::new("mod1/mod2")),
            end: End::Section(Section { name: "section" }),
        })
    );
}

#[test]
fn test_item_parts() {
    let mut rust_item = String::with_capacity(40);

    for &item in crate::ALL_ITEM_TYPES {
        rust_item.clear();
        rust_item.push_str(item);
        rust_item.push_str(".Type.html");
        assert_eq!(
            item_parts(
                Path::new(&rust_item),
                &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
            ),
            Some(LinkParts {
                start: Start::Empty,
                modules: None,
                end: End::Item {
                    dis: Disambiguator::from(item),
                    name: "Type",
                    added: None,
                }
            })
        );

        rust_item.clear();
        rust_item.push_str(item);
        rust_item.push_str(".Type.html#method.call");
        assert_eq!(
            item_parts(
                Path::new(&rust_item),
                &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
            ),
            Some(LinkParts {
                start: Start::Empty,
                modules: None,
                end: End::Item {
                    dis: Disambiguator::from(item),
                    name: "Type",
                    added: Some(AssocOrSection::Assoc(AssociatedItem {
                        dis: Disambiguator::Suffix("()"),
                        name: "call",
                    })),
                }
            })
        );

        rust_item.clear();
        rust_item.push_str(item);
        rust_item.push_str(".Type.html#section-name");
        assert_eq!(
            item_parts(
                Path::new(&rust_item),
                &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
            ),
            Some(LinkParts {
                start: Start::Empty,
                modules: None,
                end: End::Item {
                    dis: Disambiguator::from(item),
                    name: "Type",
                    added: Some(AssocOrSection::Section(Section {
                        name: "section-name",
                    })),
                }
            })
        );

        rust_item.clear();
        rust_item.push_str("./");
        rust_item.push_str(item);
        rust_item.push_str(".Type.html");
        assert_eq!(
            item_parts(
                Path::new(&rust_item),
                &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
            ),
            Some(LinkParts {
                start: Start::Local,
                modules: None,
                end: End::Item {
                    dis: Disambiguator::from(item),
                    name: "Type",
                    added: None,
                }
            })
        );

        rust_item.clear();
        rust_item.push_str("../");
        rust_item.push_str(item);
        rust_item.push_str(".Type.html");
        assert_eq!(
            item_parts(
                Path::new(&rust_item),
                &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
            ),
            Some(LinkParts {
                start: Start::Supers(1),
                modules: None,
                end: End::Item {
                    dis: Disambiguator::from(item),
                    name: "Type",
                    added: None,
                }
            })
        );

        rust_item.clear();
        rust_item.push_str("../mod1/mod2/");
        rust_item.push_str(item);
        rust_item.push_str(".Type.html");
        assert_eq!(
            item_parts(
                Path::new(&rust_item),
                &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
            ),
            Some(LinkParts {
                start: Start::Supers(1),
                modules: Some(Path::new("mod1/mod2")),
                end: End::Item {
                    dis: Disambiguator::from(item),
                    name: "Type",
                    added: None,
                }
            })
        );
    }

    assert_eq!(
        item_parts(
            Path::new("#section"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        None
    );
    assert_eq!(
        item_parts(
            Path::new("#fn.associated_item"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        None
    );
    assert_eq!(
        item_parts(
            Path::new("https://docs.rs/regex"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        None
    );
    assert_eq!(
        item_parts(
            Path::new("http://example.com"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        None
    );
    assert_eq!(
        item_parts(
            Path::new("mod1"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        None
    );
    assert_eq!(
        item_parts(
            Path::new("../mod1/mod2/index.html#section"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        None
    );
}

#[test]
fn test_module_parts() {
    let link = Path::new("regex");
    assert_eq!(
        module_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Module {
                name: "regex".into(),
                section: None
            },
        })
    );

    let link = Path::new("../../regex");
    assert_eq!(
        module_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Supers(2),
            modules: None,
            end: End::Module {
                name: "regex".into(),
                section: None
            },
        })
    );

    let link = Path::new("../../mod1/mod2/regex");
    assert_eq!(
        module_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Supers(2),
            modules: Some(Path::new("mod1/mod2")),
            end: End::Module {
                name: "regex".into(),
                section: None
            },
        })
    );

    let link = Path::new("../../krate/mod1/mod2/regex");
    assert_eq!(
        module_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Crate,
            modules: Some(Path::new("mod1/mod2")),
            end: End::Module {
                name: "regex".into(),
                section: None
            },
        })
    );

    let link = Path::new("regex/bytes/index.html");
    assert_eq!(
        module_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Mod("regex"),
            modules: None,
            end: End::Module {
                name: "bytes".into(),
                section: None,
            },
        })
    );

    let link = Path::new("regex/bytes/index.html#syntax");
    assert_eq!(
        module_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Mod("regex"),
            modules: Some(Path::new("bytes")),
            end: End::Section(Section { name: "syntax" }),
        })
    );

    assert_eq!(
        module_parts(
            Path::new("#section"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        None
    );
    assert_eq!(
        module_parts(
            Path::new("#fn.associated_item"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        None
    );
    assert_eq!(
        module_parts(
            Path::new("struct.Type.html#fn.associated_item"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        None
    );
    assert_eq!(
        module_parts(
            Path::new("struct.Type.html#section"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        None
    );
    assert_eq!(
        module_parts(
            Path::new("https://docs.rs/regex/latest/regex/index.html"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        None
    );
    assert_eq!(
        module_parts(
            Path::new("http://example.com"),
            &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
        ),
        None
    );
}

#[test]
fn disambiguate_from() {
    use Disambiguator::*;

    assert_eq!(Prefix("type@"), Disambiguator::from("struct"));
    assert_eq!(Prefix("type@"), Disambiguator::from("enum"));
    assert_eq!(Prefix("type@"), Disambiguator::from("trait"));
    assert_eq!(Prefix("type@"), Disambiguator::from("union"));
    assert_eq!(Prefix("type@"), Disambiguator::from("type"));

    assert_eq!(Prefix("value@"), Disambiguator::from("const"));
    assert_eq!(Prefix("value@"), Disambiguator::from("static"));
    assert_eq!(Prefix("value@"), Disambiguator::from("value"));

    assert_eq!(Prefix("macro@"), Disambiguator::from("derive"));
    assert_eq!(Prefix("macro@"), Disambiguator::from("attr"));

    assert_eq!(Prefix("prim@"), Disambiguator::from("primitive"));

    assert_eq!(Prefix("mod@"), Disambiguator::from("mod"));

    assert_eq!(Suffix("()"), Disambiguator::from("fn"));
    assert_eq!(Suffix("()"), Disambiguator::from("method"));

    assert_eq!(Suffix("!"), Disambiguator::from("macro"));

    assert_eq!(Empty, Disambiguator::from("other"));
    assert_eq!(Empty, Disambiguator::from("soomething else"));
}

#[test]
fn test_start_middle_end() {
    let link = Path::new("regex/bytes/index.html#examples");
    assert_eq!(
        start_middle_end(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Mod("regex"),
            modules: Some(Path::new("bytes")),
            end: End::Section(Section { name: "examples" }),
        })
    );

    let link = Path::new("std/string/struct.String.html#method.with_capacity");
    assert_eq!(
        start_middle_end(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Mod("std"),
            modules: Some(Path::new("string")),
            end: End::Item {
                dis: Disambiguator::Prefix("type@"),
                name: "String",
                added: Some(AssocOrSection::Assoc(AssociatedItem {
                    dis: Disambiguator::Suffix("()"),
                    name: "with_capacity"
                })),
            },
        })
    );

    let link = Path::new("#method.with_capacity");
    assert_eq!(
        start_middle_end(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Local,
            modules: None,
            end: End::Assoc(AssociatedItem {
                dis: Disambiguator::Suffix("()"),
                name: "with_capacity",
            }),
        })
    );

    let link = Path::new("bytes#examples");
    assert_eq!(
        start_middle_end(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Module {
                name: "bytes".into(),
                section: Some(Section { name: "examples" }),
            }
        })
    );
}

#[test]
fn test_start_and_middle() {
    let link = Path::new("regex/bytes");
    let end = End::Section(Section { name: "examples" });
    assert_eq!(
        start_and_middle(link, end, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Mod("regex"),
            modules: Some(Path::new("bytes")),
            end: End::Section(Section { name: "examples" }),
        })
    );

    let link = Path::new("std/string");
    let end = End::Item {
        dis: Disambiguator::Prefix("type@"),
        name: "String",
        added: Some(AssocOrSection::Assoc(AssociatedItem {
            dis: Disambiguator::Suffix("()"),
            name: "with_capacity",
        })),
    };
    assert_eq!(
        start_and_middle(link, end, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Mod("std"),
            modules: Some(Path::new("string")),
            end: End::Item {
                dis: Disambiguator::Prefix("type@"),
                name: "String",
                added: Some(AssocOrSection::Assoc(AssociatedItem {
                    dis: Disambiguator::Suffix("()"),
                    name: "with_capacity"
                })),
            },
        })
    );

    let link = Path::new("");
    let end = End::Assoc(AssociatedItem {
        dis: Disambiguator::Suffix("()"),
        name: "with_capacity",
    });
    assert_eq!(
        start_and_middle(link, end, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Assoc(AssociatedItem {
                dis: Disambiguator::Suffix("()"),
                name: "with_capacity",
            }),
        })
    );

    let link = Path::new("");
    let end = End::Module {
        name: "bytes".into(),
        section: Some(Section { name: "examples" }),
    };
    assert_eq!(
        start_and_middle(link, end, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Module {
                name: "bytes".into(),
                section: Some(Section { name: "examples" }),
            }
        })
    );
}

#[test]
fn link_parts_dis() {
    use Disambiguator::*;

    // Section.

    let mut lp = LinkParts {
        start: Start::Empty,
        modules: None,
        end: End::Section(Section { name: "examples" }),
    };
    assert_eq!(Empty, lp.dis());

    // Module, with section or not.

    lp.end = End::Module {
        name: "a".into(),
        section: None,
    };
    assert_eq!(Prefix("mod@"), lp.dis());

    lp.end = End::Module {
        name: "a".into(),
        section: Some(Section { name: "examples" }),
    };
    assert_eq!(Prefix("mod@"), lp.dis());

    // Associated item.

    lp.end = End::Assoc(AssociatedItem {
        dis: Empty,
        name: "item",
    });
    assert_eq!(Empty, lp.dis());

    lp.end = End::Assoc(AssociatedItem {
        dis: Prefix("type@"),
        name: "item",
    });
    assert_eq!(Prefix("type@"), lp.dis());

    lp.end = End::Assoc(AssociatedItem {
        dis: Suffix("()"),
        name: "item",
    });
    assert_eq!(Suffix("()"), lp.dis());

    // Item with an 'added' value.

    lp.end = End::Item {
        dis: Empty,
        name: "Item",
        added: Some(AssocOrSection::Assoc(AssociatedItem {
            dis: Empty,
            name: "item",
        })),
    };
    assert_eq!(Empty, lp.dis());

    lp.end = End::Item {
        dis: Empty,
        name: "Item",
        added: Some(AssocOrSection::Assoc(AssociatedItem {
            dis: Prefix("type@"),
            name: "item",
        })),
    };
    assert_eq!(Prefix("type@"), lp.dis());

    lp.end = End::Item {
        dis: Empty,
        name: "Item",
        added: Some(AssocOrSection::Assoc(AssociatedItem {
            dis: Suffix("()"),
            name: "item",
        })),
    };
    assert_eq!(Suffix("()"), lp.dis());

    // Item with no 'added' value.

    lp.end = End::Item {
        dis: Empty,
        name: "Item",
        added: None,
    };
    assert_eq!(Empty, lp.dis());

    lp.end = End::Item {
        dis: Prefix("type@"),
        name: "Item",
        added: None,
    };
    assert_eq!(Prefix("type@"), lp.dis());

    lp.end = End::Item {
        dis: Suffix("()"),
        name: "Item",
        added: None,
    };
    assert_eq!(Suffix("()"), lp.dis());
}

#[test]
fn test_link_parts() {
    const INVALID_FAVORED: &[&str] = &[
        "https://example.com",
        "https://docs.rs",
        "https://doc.rust-lang.org/",
        "https://doc.rust-lang.org/other",
        "https://doc.rust-lang.org/nightly/nightly-rustc",
        "https://docs.rs/krate-name/1.2.3/krate/struct.Type.html",
    ];

    for &invalid in INVALID_FAVORED {
        let link = Path::new(invalid);
        assert_eq!(
            link_parts(link, &crate::consts::OPTS_KRATE_DIS_NO_FAV).unwrap_err(),
            link.as_os_str()
        );
    }

    const VALID_FAVORED: &[&str] = &[
        // docs.rs - Krate
        "https://docs.rs/krate-name/1.2.3/krate/struct.Type.html",
        // docs.rs - Regex
        "https://docs.rs/regex/",
        "https://docs.rs/regex/1.4.2",
        "https://docs.rs/regex/1.4.2/regex",
        "https://docs.rs/regex/1.4.2/regex/struct.Regex.html",
        "https://docs.rs/regex/1.4.2/regex/struct.Regex.html#examples",
        "https://docs.rs/regex/1.4.2/regex/struct.Regex.html#method.is_match",
        "https://docs.rs/regex/1.4.2/regex/bytes/index.html",
        "https://docs.rs/regex/1.4.2/regex/bytes/index.html#syntax",
        "https://docs.rs/regex/1.4.2/regex/bytes/struct.Regex.html#examples",
        "https://docs.rs/regex/1.4.2/regex/bytes/struct.Regex.html#method.is_match",
        // doc.rust-lang.org - short
        "https://doc.rust-lang.org/std",
        "https://doc.rust-lang.org/alloc",
        "https://doc.rust-lang.org/core",
        "https://doc.rust-lang.org/test",
        "https://doc.rust-lang.org/proc_macro",
        "https://doc.rust-lang.org/std/string/index.html",
        "https://doc.rust-lang.org/std/string/struct.String.html",
        "https://doc.rust-lang.org/std/string/struct.String.html#examples",
        "https://doc.rust-lang.org/std/string/struct.String.html#method.drain",
        // doc.rust-lang.org - long
        "https://doc.rust-lang.org/nightly/std",
        "https://doc.rust-lang.org/nightly/alloc",
        "https://doc.rust-lang.org/nightly/core",
        "https://doc.rust-lang.org/nightly/test",
        "https://doc.rust-lang.org/nightly/proc_macro",
        "https://doc.rust-lang.org/nightly/std/string/index.html",
        "https://doc.rust-lang.org/nightly/std/string/struct.String.html",
        "https://doc.rust-lang.org/nightly/std/string/struct.String.html#examples",
        "https://doc.rust-lang.org/nightly/std/string/struct.String.html#method.drain",
    ];

    for &valid in VALID_FAVORED {
        let link = Path::new(valid);
        assert_eq!(
            link_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV).unwrap(),
            favored_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV).unwrap()
        )
    }

    const VALID_ASSOCIATED: &[&str] = &["#struct.Item", "./#struct.Item", "././#struct.Item"];

    for &valid in VALID_ASSOCIATED {
        let link = Path::new(valid);
        assert_eq!(
            link_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV).unwrap(),
            associated_item_parts(link).unwrap()
        )
    }

    const VALID_SECTION: &[&str] = &[
        "#section-a",
        "#section-1",
        "#section-A",
        "#section_a",
        "#section.a",
        "#Section.a",
        "#rection.a",
        "#0ection.a",
        "#_ection.a",
        "krate/#section",
        "../krate/#section",
        "mod1/#section",
        "mod1/mod2/#section",
        "../../mod1/mod2/#section",
    ];

    for &valid in VALID_SECTION {
        let link = Path::new(valid);
        assert_eq!(
            link_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV).unwrap(),
            section_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate).unwrap()
        )
    }

    let mut rust_item = String::with_capacity(40);

    for &item in crate::ALL_ITEM_TYPES {
        rust_item.clear();
        rust_item.push_str(item);
        rust_item.push_str(".Type.html");
        assert_eq!(
            link_parts(
                Path::new(&rust_item),
                &crate::consts::OPTS_KRATE_DIS_AND_FAV
            )
            .unwrap(),
            item_parts(
                Path::new(&rust_item),
                &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
            )
            .unwrap(),
        );

        rust_item.clear();
        rust_item.push_str(item);
        rust_item.push_str(".Type.html#method.call");
        assert_eq!(
            link_parts(
                Path::new(&rust_item),
                &crate::consts::OPTS_KRATE_DIS_AND_FAV
            )
            .unwrap(),
            item_parts(
                Path::new(&rust_item),
                &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
            )
            .unwrap(),
        );

        rust_item.clear();
        rust_item.push_str(item);
        rust_item.push_str(".Type.html#section-name");
        assert_eq!(
            link_parts(
                Path::new(&rust_item),
                &crate::consts::OPTS_KRATE_DIS_AND_FAV
            )
            .unwrap(),
            item_parts(
                Path::new(&rust_item),
                &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
            )
            .unwrap(),
        );

        rust_item.clear();
        rust_item.push_str("./");
        rust_item.push_str(item);
        rust_item.push_str(".Type.html");
        assert_eq!(
            link_parts(
                Path::new(&rust_item),
                &crate::consts::OPTS_KRATE_DIS_AND_FAV
            )
            .unwrap(),
            item_parts(
                Path::new(&rust_item),
                &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
            )
            .unwrap(),
        );

        rust_item.clear();
        rust_item.push_str("../");
        rust_item.push_str(item);
        rust_item.push_str(".Type.html");
        assert_eq!(
            link_parts(
                Path::new(&rust_item),
                &crate::consts::OPTS_KRATE_DIS_AND_FAV
            )
            .unwrap(),
            item_parts(
                Path::new(&rust_item),
                &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
            )
            .unwrap(),
        );

        rust_item.clear();
        rust_item.push_str("../mod1/mod2/");
        rust_item.push_str(item);
        rust_item.push_str(".Type.html");
        assert_eq!(
            link_parts(
                Path::new(&rust_item),
                &crate::consts::OPTS_KRATE_DIS_AND_FAV
            )
            .unwrap(),
            item_parts(
                Path::new(&rust_item),
                &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate
            )
            .unwrap(),
        );
    }

    const VALID_MODULES: &[&str] = &[
        "regex",
        "../../regex",
        "../../mod1/mod2/regex",
        "mod1/mod2",
        "../../krate/mod1/mod2/regex",
        "mod1/mod2",
        "regex/bytes/index.html",
        "regex/bytes/index.html#syntax",
    ];

    for &valid in VALID_MODULES {
        let link = Path::new(valid);
        assert_eq!(
            link_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV).unwrap(),
            module_parts(link, &crate::consts::OPTS_KRATE_DIS_AND_FAV.krate).unwrap()
        );
    }
}

#[test]
fn test_link_parts_transform() {
    use crate::ConversionContext;

    fn check_transform(value: &str, target: &str, ctx: &ConversionContext) {
        let parts = link_parts(Path::new(value), ctx.options()).unwrap();
        let transform = parts.clone().transform(&ctx);
        assert_eq!(
            target, transform,
            "\n--> Value: {:#?}, parts: {:#?}",
            value, parts
        );
    }

    // Both contexts can transform favored links, for a context that cannot
    // see `test_link_parts`.
    let mut ctx_dis =
        ConversionContext::with_options(crate::consts::OPTS_KRATE_DIS_AND_FAV.clone());
    let mut ctx_no_dis =
        ConversionContext::with_options(crate::consts::OPTS_KRATE_NO_DIS_BUT_FAV.clone());

    // Ensure sections and associated items are not transformed when the
    // current type block is empty.
    check_transform("#section", "#section", &ctx_dis);
    check_transform("#section", "#section", &ctx_no_dis);

    check_transform("#method.drain", "Self::drain()", &ctx_dis);
    check_transform("#method.drain", "Self::drain()", &ctx_no_dis);

    ctx_dis.set_current_type_block("Block".into());
    ctx_no_dis.set_current_type_block("Block".into());

    for &(value, with_dis, without_dis) in TEST_TRANSFORM_VALUES {
        check_transform(value, with_dis, &ctx_dis);
        check_transform(value, without_dis, &ctx_no_dis);
    }
}

const TEST_TRANSFORM_VALUES: &[(&str, &str, &str)] = &[
    (
        "https://docs.rs/krate-name/1.2.3/krate/struct.Type.html",
        "type@crate::Type",
        "crate::Type",
    ),
    ("https://docs.rs/regex/", "mod@regex", "regex"),
    ("https://docs.rs/regex/1.4.2", "mod@regex", "regex"),
    ("https://docs.rs/regex/1.4.2/regex", "mod@regex", "regex"),
    (
        "https://docs.rs/regex/1.4.2/regex/struct.Regex.html",
        "type@regex::Regex",
        "regex::Regex",
    ),
    (
        "https://docs.rs/regex/1.4.2/regex/struct.Regex.html#examples",
        "type@regex::Regex#examples",
        "regex::Regex#examples",
    ),
    (
        "https://docs.rs/regex/1.4.2/regex/struct.Regex.html#method.is_match",
        "regex::Regex::is_match()",
        "regex::Regex::is_match()",
    ),
    (
        "https://docs.rs/regex/1.4.2/regex/bytes/index.html",
        "mod@regex::bytes",
        "regex::bytes",
    ),
    (
        "https://docs.rs/regex/1.4.2/regex/bytes/index.html#syntax",
        "regex::bytes#syntax",
        "regex::bytes#syntax",
    ),
    (
        "https://docs.rs/regex/1.4.2/regex/bytes/struct.Regex.html#examples",
        "type@regex::bytes::Regex#examples",
        "regex::bytes::Regex#examples",
    ),
    (
        "https://docs.rs/regex/1.4.2/regex/bytes/struct.Regex.html#method.is_match",
        "regex::bytes::Regex::is_match()",
        "regex::bytes::Regex::is_match()",
    ),
    ("https://doc.rust-lang.org/std", "mod@std", "std"),
    ("https://doc.rust-lang.org/alloc", "mod@alloc", "alloc"),
    ("https://doc.rust-lang.org/core", "mod@core", "core"),
    ("https://doc.rust-lang.org/test", "mod@test", "test"),
    (
        "https://doc.rust-lang.org/proc_macro",
        "mod@proc_macro",
        "proc_macro",
    ),
    (
        "https://doc.rust-lang.org/std/string/index.html",
        "mod@std::string",
        "std::string",
    ),
    (
        "https://doc.rust-lang.org/std/string/struct.String.html",
        "type@std::string::String",
        "std::string::String",
    ),
    (
        "https://doc.rust-lang.org/std/string/struct.String.html#examples",
        "type@std::string::String#examples",
        "std::string::String#examples",
    ),
    (
        "https://doc.rust-lang.org/std/string/struct.String.html#method.drain",
        "std::string::String::drain()",
        "std::string::String::drain()",
    ),
    ("https://doc.rust-lang.org/nightly/std", "mod@std", "std"),
    (
        "https://doc.rust-lang.org/nightly/alloc",
        "mod@alloc",
        "alloc",
    ),
    ("https://doc.rust-lang.org/nightly/core", "mod@core", "core"),
    ("https://doc.rust-lang.org/nightly/test", "mod@test", "test"),
    (
        "https://doc.rust-lang.org/nightly/proc_macro",
        "mod@proc_macro",
        "proc_macro",
    ),
    (
        "https://doc.rust-lang.org/nightly/std/string/index.html",
        "mod@std::string",
        "std::string",
    ),
    (
        "https://doc.rust-lang.org/nightly/std/string/struct.String.html",
        "type@std::string::String",
        "std::string::String",
    ),
    (
        "https://doc.rust-lang.org/nightly/std/string/struct.String.html#examples",
        "type@std::string::String#examples",
        "std::string::String#examples",
    ),
    (
        "https://doc.rust-lang.org/nightly/std/string/struct.String.html#method.drain",
        "std::string::String::drain()",
        "std::string::String::drain()",
    ),
    ("#struct.Item", "type@Block::Item", "Block::Item"),
    ("./#struct.Item", "type@Block::Item", "Block::Item"),
    ("././#struct.Item", "type@Block::Item", "Block::Item"),
    ("#section-a", "#section-a", "#section-a"),
    ("#section-1", "#section-1", "#section-1"),
    ("#section-A", "#section-A", "#section-A"),
    ("#section_a", "#section_a", "#section_a"),
    ("#section.a", "#section.a", "#section.a"),
    ("#Section.a", "#Section.a", "#Section.a"),
    ("#rection.a", "#rection.a", "#rection.a"),
    ("#0ection.a", "#0ection.a", "#0ection.a"),
    ("#_ection.a", "#_ection.a", "#_ection.a"),
    ("krate/#section", "crate#section", "crate#section"),
    ("../krate/#section", "crate#section", "crate#section"),
    ("mod1/#section", "mod1#section", "mod1#section"),
    (
        "mod1/mod2/#section",
        "mod1::mod2#section",
        "mod1::mod2#section",
    ),
    (
        "../../mod1/mod2/#section",
        "super::super::mod1::mod2#section",
        "super::super::mod1::mod2#section",
    ),
    ("associatedconstant.Type.html", "Type", "Type"),
    (
        "associatedconstant.Type.html#method.call",
        "Type::call()",
        "Type::call()",
    ),
    (
        "associatedconstant.Type.html#section-name",
        "Type#section-name",
        "Type#section-name",
    ),
    ("./associatedconstant.Type.html", "Type", "Type"),
    (
        "../associatedconstant.Type.html",
        "super::Type",
        "super::Type",
    ),
    (
        "../mod1/mod2/associatedconstant.Type.html",
        "super::mod1::mod2::Type",
        "super::mod1::mod2::Type",
    ),
    ("associatedtype.Type.html", "Type", "Type"),
    (
        "associatedtype.Type.html#method.call",
        "Type::call()",
        "Type::call()",
    ),
    (
        "associatedtype.Type.html#section-name",
        "Type#section-name",
        "Type#section-name",
    ),
    ("./associatedtype.Type.html", "Type", "Type"),
    ("../associatedtype.Type.html", "super::Type", "super::Type"),
    (
        "../mod1/mod2/associatedtype.Type.html",
        "super::mod1::mod2::Type",
        "super::mod1::mod2::Type",
    ),
    ("attr.Type.html", "macro@Type", "Type"),
    ("attr.Type.html#method.call", "Type::call()", "Type::call()"),
    (
        "attr.Type.html#section-name",
        "macro@Type#section-name",
        "Type#section-name",
    ),
    ("./attr.Type.html", "macro@Type", "Type"),
    ("../attr.Type.html", "macro@super::Type", "super::Type"),
    (
        "../mod1/mod2/attr.Type.html",
        "macro@super::mod1::mod2::Type",
        "super::mod1::mod2::Type",
    ),
    ("constant.Type.html", "Type", "Type"),
    (
        "constant.Type.html#method.call",
        "Type::call()",
        "Type::call()",
    ),
    (
        "constant.Type.html#section-name",
        "Type#section-name",
        "Type#section-name",
    ),
    ("./constant.Type.html", "Type", "Type"),
    ("../constant.Type.html", "super::Type", "super::Type"),
    (
        "../mod1/mod2/constant.Type.html",
        "super::mod1::mod2::Type",
        "super::mod1::mod2::Type",
    ),
    ("derive.Type.html", "macro@Type", "Type"),
    (
        "derive.Type.html#method.call",
        "Type::call()",
        "Type::call()",
    ),
    (
        "derive.Type.html#section-name",
        "macro@Type#section-name",
        "Type#section-name",
    ),
    ("./derive.Type.html", "macro@Type", "Type"),
    ("../derive.Type.html", "macro@super::Type", "super::Type"),
    (
        "../mod1/mod2/derive.Type.html",
        "macro@super::mod1::mod2::Type",
        "super::mod1::mod2::Type",
    ),
    ("enum.Type.html", "type@Type", "Type"),
    ("enum.Type.html#method.call", "Type::call()", "Type::call()"),
    (
        "enum.Type.html#section-name",
        "type@Type#section-name",
        "Type#section-name",
    ),
    ("./enum.Type.html", "type@Type", "Type"),
    ("../enum.Type.html", "type@super::Type", "super::Type"),
    (
        "../mod1/mod2/enum.Type.html",
        "type@super::mod1::mod2::Type",
        "super::mod1::mod2::Type",
    ),
    ("externcrate.Type.html", "Type", "Type"),
    (
        "externcrate.Type.html#method.call",
        "Type::call()",
        "Type::call()",
    ),
    (
        "externcrate.Type.html#section-name",
        "Type#section-name",
        "Type#section-name",
    ),
    ("./externcrate.Type.html", "Type", "Type"),
    ("../externcrate.Type.html", "super::Type", "super::Type"),
    (
        "../mod1/mod2/externcrate.Type.html",
        "super::mod1::mod2::Type",
        "super::mod1::mod2::Type",
    ),
    ("fn.Type.html", "Type()", "Type()"),
    ("fn.Type.html#method.call", "Type::call()", "Type::call()"),
    (
        "fn.Type.html#section-name",
        "Type()#section-name",
        "Type()#section-name",
    ),
    ("./fn.Type.html", "Type()", "Type()"),
    ("../fn.Type.html", "super::Type()", "super::Type()"),
    (
        "../mod1/mod2/fn.Type.html",
        "super::mod1::mod2::Type()",
        "super::mod1::mod2::Type()",
    ),
    ("foreigntype.Type.html", "Type", "Type"),
    (
        "foreigntype.Type.html#method.call",
        "Type::call()",
        "Type::call()",
    ),
    (
        "foreigntype.Type.html#section-name",
        "Type#section-name",
        "Type#section-name",
    ),
    ("./foreigntype.Type.html", "Type", "Type"),
    ("../foreigntype.Type.html", "super::Type", "super::Type"),
    (
        "../mod1/mod2/foreigntype.Type.html",
        "super::mod1::mod2::Type",
        "super::mod1::mod2::Type",
    ),
    ("impl.Type.html", "Type", "Type"),
    ("impl.Type.html#method.call", "Type::call()", "Type::call()"),
    (
        "impl.Type.html#section-name",
        "Type#section-name",
        "Type#section-name",
    ),
    ("./impl.Type.html", "Type", "Type"),
    ("../impl.Type.html", "super::Type", "super::Type"),
    (
        "../mod1/mod2/impl.Type.html",
        "super::mod1::mod2::Type",
        "super::mod1::mod2::Type",
    ),
    ("import.Type.html", "Type", "Type"),
    (
        "import.Type.html#method.call",
        "Type::call()",
        "Type::call()",
    ),
    (
        "import.Type.html#section-name",
        "Type#section-name",
        "Type#section-name",
    ),
    ("./import.Type.html", "Type", "Type"),
    ("../import.Type.html", "super::Type", "super::Type"),
    (
        "../mod1/mod2/import.Type.html",
        "super::mod1::mod2::Type",
        "super::mod1::mod2::Type",
    ),
    ("keyword.Type.html", "Type", "Type"),
    (
        "keyword.Type.html#method.call",
        "Type::call()",
        "Type::call()",
    ),
    (
        "keyword.Type.html#section-name",
        "Type#section-name",
        "Type#section-name",
    ),
    ("./keyword.Type.html", "Type", "Type"),
    ("../keyword.Type.html", "super::Type", "super::Type"),
    (
        "../mod1/mod2/keyword.Type.html",
        "super::mod1::mod2::Type",
        "super::mod1::mod2::Type",
    ),
    ("macro.Type.html", "Type!", "Type!"),
    (
        "macro.Type.html#method.call",
        "Type::call()",
        "Type::call()",
    ),
    (
        "macro.Type.html#section-name",
        "Type!#section-name",
        "Type!#section-name",
    ),
    ("./macro.Type.html", "Type!", "Type!"),
    ("../macro.Type.html", "super::Type!", "super::Type!"),
    (
        "../mod1/mod2/macro.Type.html",
        "super::mod1::mod2::Type!",
        "super::mod1::mod2::Type!",
    ),
    ("method.Type.html", "Type()", "Type()"),
    (
        "method.Type.html#method.call",
        "Type::call()",
        "Type::call()",
    ),
    (
        "method.Type.html#section-name",
        "Type()#section-name",
        "Type()#section-name",
    ),
    ("./method.Type.html", "Type()", "Type()"),
    ("../method.Type.html", "super::Type()", "super::Type()"),
    (
        "../mod1/mod2/method.Type.html",
        "super::mod1::mod2::Type()",
        "super::mod1::mod2::Type()",
    ),
    ("mod.Type.html", "mod@Type", "Type"),
    ("mod.Type.html#method.call", "Type::call()", "Type::call()"),
    (
        "mod.Type.html#section-name",
        "mod@Type#section-name",
        "Type#section-name",
    ),
    ("./mod.Type.html", "mod@Type", "Type"),
    ("../mod.Type.html", "mod@super::Type", "super::Type"),
    (
        "../mod1/mod2/mod.Type.html",
        "mod@super::mod1::mod2::Type",
        "super::mod1::mod2::Type",
    ),
    ("opaque.Type.html", "Type", "Type"),
    (
        "opaque.Type.html#method.call",
        "Type::call()",
        "Type::call()",
    ),
    (
        "opaque.Type.html#section-name",
        "Type#section-name",
        "Type#section-name",
    ),
    ("./opaque.Type.html", "Type", "Type"),
    ("../opaque.Type.html", "super::Type", "super::Type"),
    (
        "../mod1/mod2/opaque.Type.html",
        "super::mod1::mod2::Type",
        "super::mod1::mod2::Type",
    ),
    ("primitive.Type.html", "prim@Type", "Type"),
    (
        "primitive.Type.html#method.call",
        "Type::call()",
        "Type::call()",
    ),
    (
        "primitive.Type.html#section-name",
        "prim@Type#section-name",
        "Type#section-name",
    ),
    ("./primitive.Type.html", "prim@Type", "Type"),
    ("../primitive.Type.html", "prim@super::Type", "super::Type"),
    (
        "../mod1/mod2/primitive.Type.html",
        "prim@super::mod1::mod2::Type",
        "super::mod1::mod2::Type",
    ),
    ("static.Type.html", "value@Type", "Type"),
    (
        "static.Type.html#method.call",
        "Type::call()",
        "Type::call()",
    ),
    (
        "static.Type.html#section-name",
        "value@Type#section-name",
        "Type#section-name",
    ),
    ("./static.Type.html", "value@Type", "Type"),
    ("../static.Type.html", "value@super::Type", "super::Type"),
    (
        "../mod1/mod2/static.Type.html",
        "value@super::mod1::mod2::Type",
        "super::mod1::mod2::Type",
    ),
    ("struct.Type.html", "type@Type", "Type"),
    (
        "struct.Type.html#method.call",
        "Type::call()",
        "Type::call()",
    ),
    (
        "struct.Type.html#section-name",
        "type@Type#section-name",
        "Type#section-name",
    ),
    ("./struct.Type.html", "type@Type", "Type"),
    ("../struct.Type.html", "type@super::Type", "super::Type"),
    (
        "../mod1/mod2/struct.Type.html",
        "type@super::mod1::mod2::Type",
        "super::mod1::mod2::Type",
    ),
    ("structfield.Type.html", "Type", "Type"),
    (
        "structfield.Type.html#method.call",
        "Type::call()",
        "Type::call()",
    ),
    (
        "structfield.Type.html#section-name",
        "Type#section-name",
        "Type#section-name",
    ),
    ("./structfield.Type.html", "Type", "Type"),
    ("../structfield.Type.html", "super::Type", "super::Type"),
    (
        "../mod1/mod2/structfield.Type.html",
        "super::mod1::mod2::Type",
        "super::mod1::mod2::Type",
    ),
    ("trait.Type.html", "type@Type", "Type"),
    (
        "trait.Type.html#method.call",
        "Type::call()",
        "Type::call()",
    ),
    (
        "trait.Type.html#section-name",
        "type@Type#section-name",
        "Type#section-name",
    ),
    ("./trait.Type.html", "type@Type", "Type"),
    ("../trait.Type.html", "type@super::Type", "super::Type"),
    (
        "../mod1/mod2/trait.Type.html",
        "type@super::mod1::mod2::Type",
        "super::mod1::mod2::Type",
    ),
    ("traitalias.Type.html", "Type", "Type"),
    (
        "traitalias.Type.html#method.call",
        "Type::call()",
        "Type::call()",
    ),
    (
        "traitalias.Type.html#section-name",
        "Type#section-name",
        "Type#section-name",
    ),
    ("./traitalias.Type.html", "Type", "Type"),
    ("../traitalias.Type.html", "super::Type", "super::Type"),
    (
        "../mod1/mod2/traitalias.Type.html",
        "super::mod1::mod2::Type",
        "super::mod1::mod2::Type",
    ),
    ("tymethod.Type.html", "Type", "Type"),
    (
        "tymethod.Type.html#method.call",
        "Type::call()",
        "Type::call()",
    ),
    (
        "tymethod.Type.html#section-name",
        "Type#section-name",
        "Type#section-name",
    ),
    ("./tymethod.Type.html", "Type", "Type"),
    ("../tymethod.Type.html", "super::Type", "super::Type"),
    (
        "../mod1/mod2/tymethod.Type.html",
        "super::mod1::mod2::Type",
        "super::mod1::mod2::Type",
    ),
    ("type.Type.html", "type@Type", "Type"),
    ("type.Type.html#method.call", "Type::call()", "Type::call()"),
    (
        "type.Type.html#section-name",
        "type@Type#section-name",
        "Type#section-name",
    ),
    ("./type.Type.html", "type@Type", "Type"),
    ("../type.Type.html", "type@super::Type", "super::Type"),
    (
        "../mod1/mod2/type.Type.html",
        "type@super::mod1::mod2::Type",
        "super::mod1::mod2::Type",
    ),
    ("union.Type.html", "type@Type", "Type"),
    (
        "union.Type.html#method.call",
        "Type::call()",
        "Type::call()",
    ),
    (
        "union.Type.html#section-name",
        "type@Type#section-name",
        "Type#section-name",
    ),
    ("./union.Type.html", "type@Type", "Type"),
    ("../union.Type.html", "type@super::Type", "super::Type"),
    (
        "../mod1/mod2/union.Type.html",
        "type@super::mod1::mod2::Type",
        "super::mod1::mod2::Type",
    ),
    ("variant.Type.html", "Type", "Type"),
    (
        "variant.Type.html#method.call",
        "Type::call()",
        "Type::call()",
    ),
    (
        "variant.Type.html#section-name",
        "Type#section-name",
        "Type#section-name",
    ),
    ("./variant.Type.html", "Type", "Type"),
    ("../variant.Type.html", "super::Type", "super::Type"),
    (
        "../mod1/mod2/variant.Type.html",
        "super::mod1::mod2::Type",
        "super::mod1::mod2::Type",
    ),
    ("regex", "mod@regex", "regex"),
    (
        "../../regex",
        "mod@super::super::regex",
        "super::super::regex",
    ),
    (
        "../../mod1/mod2/regex",
        "mod@super::super::mod1::mod2::regex",
        "super::super::mod1::mod2::regex",
    ),
    ("mod1/mod2", "mod@mod1::mod2", "mod1::mod2"),
    (
        "../../krate/mod1/mod2/regex",
        "mod@crate::mod1::mod2::regex",
        "crate::mod1::mod2::regex",
    ),
    ("mod1/mod2", "mod@mod1::mod2", "mod1::mod2"),
    ("regex/bytes/index.html", "mod@regex::bytes", "regex::bytes"),
    (
        "regex/bytes/index.html#syntax",
        "regex::bytes#syntax",
        "regex::bytes#syntax",
    ),
];
