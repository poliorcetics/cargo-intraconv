use super::*;

lazy_static::lazy_static! {
    static ref OPTS_KRATE_DIS_AND_FAV: ConversionOptions = ConversionOptions {
        krate: Krate::new("krate").unwrap(),
        disambiguate: true,
        favored_links: true,
    };

    static ref OPTS_KRATE_NO_DIS_NO_FAV: ConversionOptions = ConversionOptions {
        krate: Krate::new("krate").unwrap(),
        disambiguate: false,
        favored_links: false,
    };

    static ref OPTS_KRATE_NO_DIS_BUT_FAV: ConversionOptions = ConversionOptions {
        krate: Krate::new("krate").unwrap(),
        disambiguate: false,
        favored_links: true,
    };

    static ref OPTS_KRATE_DIS_NO_FAV: ConversionOptions = ConversionOptions {
        krate: Krate::new("krate").unwrap(),
        disambiguate: true,
        favored_links: false,
    };
}

#[test]
fn test_favored_parts() {
    // FAVORED INACTIVE
    let link = Path::new("https://docs.rs/regex/1.4.2/regex");
    assert_eq!(favored_parts(link, &OPTS_KRATE_DIS_NO_FAV), None);

    // SAME CRATE
    let link = Path::new("https://docs.rs/krate-name/1.2.3/krate/struct.Type.html");
    assert_eq!(
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
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

    // FAVORED ACTIVE

    // None.
    let link = Path::new("https://example.com");
    assert_eq!(favored_parts(link, &OPTS_KRATE_DIS_AND_FAV), None);

    // doc.rs
    let link = Path::new("https://docs.rs");
    assert_eq!(favored_parts(link, &OPTS_KRATE_DIS_AND_FAV), None);

    let link = Path::new("https://docs.rs/regex/");
    assert_eq!(
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
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
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
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
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
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
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
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
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
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
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
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
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
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
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
        Some(LinkParts {
            start: Start::Mod("regex"),
            modules: Some(Path::new("bytes")),
            end: End::Section(Section { name: "syntax" }),
        })
    );

    let link = Path::new("https://docs.rs/regex/1.4.2/regex/bytes/struct.Regex.html#examples");
    assert_eq!(
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
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
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
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

    // doc.rust-lang.org
    // Missing/Invalid crate
    let link = Path::new("https://doc.rust-lang.org/");
    assert_eq!(
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/other");
    assert_eq!(
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/nightly");
    assert_eq!(
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/beta");
    assert_eq!(
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/stable");
    assert_eq!(
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/1.42.0");
    assert_eq!(
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/nightly/nightly-rustc");
    assert_eq!(
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate)
    );

    // Valid crate, short form
    let link = Path::new("https://doc.rust-lang.org/std");
    assert_eq!(
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/alloc");
    assert_eq!(
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/core");
    assert_eq!(
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/test");
    assert_eq!(
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/proc_macro");
    assert_eq!(
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate)
    );

    // Valid crate, long form
    let link = Path::new("https://doc.rust-lang.org/nightly/std");
    assert_eq!(
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/nightly/alloc");
    assert_eq!(
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/nightly/core");
    assert_eq!(
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/nightly/test");
    assert_eq!(
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/nightly/proc_macro");
    assert_eq!(
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/nightly/std/string/index.html");
    assert_eq!(
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link = Path::new("https://doc.rust-lang.org/nightly/std/string/struct.String.html");
    assert_eq!(
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link =
        Path::new("https://doc.rust-lang.org/nightly/std/string/struct.String.html#examples");
    assert_eq!(
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate)
    );

    let link =
        Path::new("https://doc.rust-lang.org/nightly/std/string/struct.String.html#method.drain");
    assert_eq!(
        favored_parts(link, &OPTS_KRATE_DIS_AND_FAV),
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate)
    );
}

#[test]
fn test_favored_doc_rust_lang_org() {
    // Missing/Invalid crate
    let link = Path::new("https://doc.rust-lang.org/");
    assert_eq!(
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate),
        None
    );

    let link = Path::new("https://doc.rust-lang.org/other");
    assert_eq!(
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate),
        None
    );

    let link = Path::new("https://doc.rust-lang.org/nightly");
    assert_eq!(
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate),
        None
    );

    let link = Path::new("https://doc.rust-lang.org/beta");
    assert_eq!(
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate),
        None
    );

    let link = Path::new("https://doc.rust-lang.org/stable");
    assert_eq!(
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate),
        None
    );

    let link = Path::new("https://doc.rust-lang.org/1.42.0");
    assert_eq!(
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate),
        None
    );

    let link = Path::new("https://doc.rust-lang.org/nightly/nightly-rustc");
    assert_eq!(
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate),
        None
    );

    // Valid crate, short form
    let link = Path::new("https://doc.rust-lang.org/std");
    assert_eq!(
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate),
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
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate),
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
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate),
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
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate),
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
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate),
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
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate),
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
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate),
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
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate),
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
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate),
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
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate),
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
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate),
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
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate),
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
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate),
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
        favored_doc_rust_lang_org(link, &OPTS_KRATE_DIS_AND_FAV.krate),
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
        section_parts(Path::new("#struct.Item"), &OPTS_KRATE_DIS_AND_FAV.krate),
        None
    );
    assert_eq!(
        section_parts(Path::new("./#struct.Item"), &OPTS_KRATE_DIS_AND_FAV.krate),
        None
    );
    assert_eq!(
        section_parts(Path::new("././#struct.Item"), &OPTS_KRATE_DIS_AND_FAV.krate),
        None
    );

    assert_eq!(
        section_parts(Path::new("#section/rest"), &OPTS_KRATE_DIS_AND_FAV.krate),
        None
    );

    // Short sections

    assert_eq!(
        section_parts(Path::new("#section-a"), &OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Section(Section { name: "section-a" }),
        })
    );
    assert_eq!(
        section_parts(Path::new("#section-1"), &OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Section(Section { name: "section-1" }),
        })
    );
    assert_eq!(
        section_parts(Path::new("#section-A"), &OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Section(Section { name: "section-A" }),
        })
    );
    assert_eq!(
        section_parts(Path::new("#section_a"), &OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Section(Section { name: "section_a" }),
        })
    );
    assert_eq!(
        section_parts(Path::new("#section.a"), &OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Section(Section { name: "section.a" }),
        })
    );
    assert_eq!(
        section_parts(Path::new("#Section.a"), &OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Section(Section { name: "Section.a" }),
        })
    );
    assert_eq!(
        section_parts(Path::new("#rection.a"), &OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Section(Section { name: "rection.a" }),
        })
    );
    assert_eq!(
        section_parts(Path::new("#0ection.a"), &OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Section(Section { name: "0ection.a" }),
        })
    );
    assert_eq!(
        section_parts(Path::new("#_ection.a"), &OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Empty,
            modules: None,
            end: End::Section(Section { name: "_ection.a" }),
        })
    );

    assert_eq!(
        section_parts(Path::new("krate/#section"), &OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Crate,
            modules: None,
            end: End::Section(Section { name: "section" }),
        })
    );

    assert_eq!(
        section_parts(
            Path::new("../krate/#section"),
            &OPTS_KRATE_DIS_AND_FAV.krate
        ),
        Some(LinkParts {
            start: Start::Crate,
            modules: None,
            end: End::Section(Section { name: "section" }),
        })
    );

    assert_eq!(
        section_parts(Path::new("mod1/#section"), &OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Mod("mod1"),
            modules: None,
            end: End::Section(Section { name: "section" }),
        })
    );

    assert_eq!(
        section_parts(
            Path::new("mod1/mod2/#section"),
            &OPTS_KRATE_DIS_AND_FAV.krate
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
            &OPTS_KRATE_DIS_AND_FAV.krate
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
            item_parts(Path::new(&rust_item), &OPTS_KRATE_DIS_AND_FAV.krate),
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
            item_parts(Path::new(&rust_item), &OPTS_KRATE_DIS_AND_FAV.krate),
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
            item_parts(Path::new(&rust_item), &OPTS_KRATE_DIS_AND_FAV.krate),
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
            item_parts(Path::new(&rust_item), &OPTS_KRATE_DIS_AND_FAV.krate),
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
            item_parts(Path::new(&rust_item), &OPTS_KRATE_DIS_AND_FAV.krate),
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
            item_parts(Path::new(&rust_item), &OPTS_KRATE_DIS_AND_FAV.krate),
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
        item_parts(Path::new("#section"), &OPTS_KRATE_DIS_AND_FAV.krate),
        None
    );
    assert_eq!(
        item_parts(
            Path::new("#fn.associated_item"),
            &OPTS_KRATE_DIS_AND_FAV.krate
        ),
        None
    );
    assert_eq!(
        item_parts(
            Path::new("https://docs.rs/regex"),
            &OPTS_KRATE_DIS_AND_FAV.krate
        ),
        None
    );
    assert_eq!(
        item_parts(
            Path::new("http://example.com"),
            &OPTS_KRATE_DIS_AND_FAV.krate
        ),
        None
    );
    assert_eq!(
        item_parts(Path::new("mod1"), &OPTS_KRATE_DIS_AND_FAV.krate),
        None
    );
    assert_eq!(
        item_parts(
            Path::new("../mod1/mod2/index.html#section"),
            &OPTS_KRATE_DIS_AND_FAV.krate
        ),
        None
    );
}

#[test]
fn test_module_parts() {
    let link = Path::new("regex");
    assert_eq!(
        module_parts(link, &OPTS_KRATE_DIS_AND_FAV.krate),
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
        module_parts(link, &OPTS_KRATE_DIS_AND_FAV.krate),
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
        module_parts(link, &OPTS_KRATE_DIS_AND_FAV.krate),
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
        module_parts(link, &OPTS_KRATE_DIS_AND_FAV.krate),
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
        module_parts(link, &OPTS_KRATE_DIS_AND_FAV.krate),
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
        module_parts(link, &OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Mod("regex"),
            modules: Some(Path::new("bytes")),
            end: End::Section(Section { name: "syntax" }),
        })
    );

    assert_eq!(
        module_parts(Path::new("#section"), &OPTS_KRATE_DIS_AND_FAV.krate),
        None
    );
    assert_eq!(
        module_parts(
            Path::new("#fn.associated_item"),
            &OPTS_KRATE_DIS_AND_FAV.krate
        ),
        None
    );
    assert_eq!(
        module_parts(
            Path::new("struct.Type.html#fn.associated_item"),
            &OPTS_KRATE_DIS_AND_FAV.krate
        ),
        None
    );
    assert_eq!(
        module_parts(
            Path::new("struct.Type.html#section"),
            &OPTS_KRATE_DIS_AND_FAV.krate
        ),
        None
    );
    assert_eq!(
        module_parts(
            Path::new("https://docs.rs/regex/latest/regex/index.html"),
            &OPTS_KRATE_DIS_AND_FAV.krate
        ),
        None
    );
    assert_eq!(
        module_parts(
            Path::new("http://example.com"),
            &OPTS_KRATE_DIS_AND_FAV.krate
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
        start_middle_end(link, &OPTS_KRATE_DIS_AND_FAV.krate),
        Some(LinkParts {
            start: Start::Mod("regex"),
            modules: Some(Path::new("bytes")),
            end: End::Section(Section { name: "examples" }),
        })
    );

    let link = Path::new("std/string/struct.String.html#method.with_capacity");
    assert_eq!(
        start_middle_end(link, &OPTS_KRATE_DIS_AND_FAV.krate),
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
        start_middle_end(link, &OPTS_KRATE_DIS_AND_FAV.krate),
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
        start_middle_end(link, &OPTS_KRATE_DIS_AND_FAV.krate),
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
        start_and_middle(link, end, &OPTS_KRATE_DIS_AND_FAV.krate),
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
        start_and_middle(link, end, &OPTS_KRATE_DIS_AND_FAV.krate),
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
        start_and_middle(link, end, &OPTS_KRATE_DIS_AND_FAV.krate),
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
        start_and_middle(link, end, &OPTS_KRATE_DIS_AND_FAV.krate),
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
fn assoc_or_section_dis() {
    let elem = AssocOrSection::Assoc(AssociatedItem {
        dis: Disambiguator::Prefix("type@"),
        name: "Type",
    });
    assert_eq!(elem.dis(), Disambiguator::Prefix("type@"));

    let elem = AssocOrSection::Assoc(AssociatedItem {
        dis: Disambiguator::Empty,
        name: "Item",
    });
    assert_eq!(elem.dis(), Disambiguator::Empty);

    let elem = AssocOrSection::Assoc(AssociatedItem {
        dis: Disambiguator::Suffix("()"),
        name: "method_call",
    });
    assert_eq!(elem.dis(), Disambiguator::Suffix("()"));

    let elem = AssocOrSection::Section(Section { name: "examples" });
    assert_eq!(elem.dis(), Disambiguator::Prefix("mod@"));
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
        added: None
    };
    assert_eq!(Empty, lp.dis());

    lp.end = End::Item {
        dis: Prefix("type@"),
        name: "Item",
        added: None
    };
    assert_eq!(Prefix("type@"), lp.dis());

    lp.end = End::Item {
        dis: Suffix("()"),
        name: "Item",
        added: None
    };
    assert_eq!(Suffix("()"), lp.dis());
}
