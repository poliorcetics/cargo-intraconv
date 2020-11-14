use lazy_static::lazy_static;
use regex::Regex;

/// All item types that can be produced by `rustdoc`.
pub const ALL_ITEM_TYPES: &[&str] = &[
    "associatedconstant",
    "associatedtype",
    "attr",
    "constant",
    "derive",
    "enum",
    "externcrate",
    "fn",
    "foreigntype",
    "impl",
    "import",
    "keyword",
    "macro",
    "method",
    "mod",
    "opaque",
    "primitive",
    "static",
    "struct",
    "structfield",
    "trait",
    "traitalias",
    "tymethod",
    "type",
    "union",
    "variant",
];

lazy_static! {
    /// This regex will match the long form of markdown link.
    ///
    /// It tries not to catch links that are already intra-doc links but its
    /// genericity means it will catch their shortest form
    /// `/// [name]: module_or_type`.
    pub static ref LINK_TO_TREAT_LONG: Regex = Regex::new(concat!(
        r"^",
        r"(?P<header>\s*(?://[!/]\s*)?\[.+?\]:\s*)",
        // The special case for 'http(s):' is to avoid catching links with a
        // '::' by putting ':' in the regex: they are already intra-doc links.
        r"(?P<link>(?:https?:)?[a-zA-Z0-9_#/\-\.]+)",
        r"\n?$",
    ))
    .unwrap();

    /// Will search for a doc comment link and be used to check if the two
    /// elements are the same, indicating a local path.
    pub static ref LOCAL_PATH_LONG: Regex = Regex::new(concat!(
        r"^",
        r"\s*(?://[!/]\s*)?\[`?(?P<header>.+?)`?\]:\s*",
        r"(?P<link>.+)",
        r"\n?$"
    )).unwrap();

    /// Non-capturing regex to check if something is exactly an item type as
    /// seen by rustdoc.
    ///
    /// This is not directly a `regex::Regex` but a `String` because it used as
    /// additional precision when building more focused regexes.
    pub static ref ITEM_TYPES: String = format!(r"(?:{})", ALL_ITEM_TYPES.join("|"));

    pub static ref RUST_IDENTIFIER_RE: Regex = Regex::new(
        &format!(r"^{}$", RUST_IDENTIFIER)
    ).unwrap();

    /// Start of a block where `Self` has a sense.
    pub static ref TYPE_BLOCK_START: Regex = Regex::new(concat!(
        r"^(?P<spaces>\s*)",
        r"(?:pub(?:\(.+\))? )?",
        r"(?:struct|trait|impl(?:<.*?>)?(?: .*? for)?|enum|union) ",
        r"(?P<type>\w+)",
        r"(?:<.*?>)?",
        r"(?P<parenthese>\()?",
        r".*\n?$",
    )).unwrap();
}

pub const RUST_IDENTIFIER: &'static str = r"(?:[a-zA-Z_][a-zA-Z0-9_]*)";

pub const HTML_SECTION: &'static str = r"(?:#[a-zA-Z0-9_\-\.]+)";

#[cfg(test)]
lazy_static::lazy_static! {
    pub static ref OPTS_KRATE_DIS_AND_FAV: crate::ConversionOptions = crate::ConversionOptions {
        krate: crate::Krate::new("krate").unwrap(),
        disambiguate: true,
        favored_links: true,
    };

    pub static ref OPTS_KRATE_NO_DIS_NO_FAV: crate::ConversionOptions = crate::ConversionOptions {
        krate: crate::Krate::new("krate").unwrap(),
        disambiguate: false,
        favored_links: false,
    };

    pub static ref OPTS_KRATE_NO_DIS_BUT_FAV: crate::ConversionOptions = crate::ConversionOptions {
        krate: crate::Krate::new("krate").unwrap(),
        disambiguate: false,
        favored_links: true,
    };

    pub static ref OPTS_KRATE_DIS_NO_FAV: crate::ConversionOptions = crate::ConversionOptions {
        krate: crate::Krate::new("krate").unwrap(),
        disambiguate: true,
        favored_links: false,
    };
}

#[test]
fn link_to_treat_long_matching() {
    // Testing spacing.
    assert!(LINK_TO_TREAT_LONG.is_match("[name]:mod1\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[name]: mod1\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[name]:  mod1\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[name]:\tmod1\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("  [name]: mod1\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("\t[name]: mod1\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("/// [name]: mod1\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("///  [name]: mod1\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("///\t[name]: mod1\n"));
    assert!(LINK_TO_TREAT_LONG.is_match(" /// [name]: mod1\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("  /// [name]: mod1\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("\t/// [name]: mod1\n"));

    // Testing bangs.
    assert!(LINK_TO_TREAT_LONG.is_match("/// [name]: mod1\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("//! [name]: mod1\n"));

    // Testing code in links.
    assert!(LINK_TO_TREAT_LONG.is_match("[`name`]: mod1\n"));

    // Testing non-HTTP links
    assert!(LINK_TO_TREAT_LONG.is_match("[name]: mod1/\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[name]: mod1/mod2\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[name]: mod1/mod2/struct.Type.html\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[name]: mod1/mod2/struct.Type.html#const.NAME\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[name]: mod1/mod2/#section\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[name]: mod1/mod2#section\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[name]: mod1/mod2/index.html#section\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[name]: mod1/mod2/index.html/#section\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[name]: ../mod1/mod2\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[name]: ./../mod1\n"));

    // Testing HTTP links
    assert!(LINK_TO_TREAT_LONG.is_match("[name]: https://docs.rs/regex/\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[name]: https://docs.rs/regex/1.0.33/regex/mod1/mod2\n"));
    assert!(LINK_TO_TREAT_LONG
        .is_match("[name]: https://github.com/poliorcetics/cargo-intraconv/issues/21\n"));

    // Testing all sorts of characters in the link name.
    assert!(LINK_TO_TREAT_LONG.is_match("[azertyuiop]: mod1\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[AZERTYUIOP]: mod1\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[@&é\"'(§è!çà)-]: mod1\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[#1234567890°_]: mod1\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[•ë“‘{¶«¡Çø}—]: mod1\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[Ÿ´„”’[å»ÛÁØ]–]: mod1\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[æÂê®†Úºîœπ]: mod1\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[ÆÅÊ‚™ŸªïŒ∏]: mod1\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[‡Ò∂ƒﬁÌÏÈ¬µ]: mod1\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[Ω∑∆·ﬂÎÍË|Ó]: mod1\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[‹≈©◊ß~]: mod1\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[›⁄¢√∫ı]: mod1\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[^$ù`,;:=<]: mod1\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[¨*%£?./+>]: mod1\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[ô€Ù@∞…÷≠≤]: mod1\n"));
    assert!(LINK_TO_TREAT_LONG.is_match("[Ô¥‰#¿•\\±≥]: mod1\n"));
}

#[test]
fn item_types() {
    let reg = Regex::new(&ITEM_TYPES).unwrap();
    for item in ALL_ITEM_TYPES {
        assert!(reg.is_match(item));
    }

    assert!(!reg.is_match("text"));
    assert!(!reg.is_match("0BDS"));
    assert!(!reg.is_match("sdfd"));
    assert!(!reg.is_match("STRUCT"));
}

#[test]
fn rust_identifier() {
    let reg = Regex::new(&format!("^{}$", RUST_IDENTIFIER)).unwrap();

    assert!(reg.is_match("a"));
    assert!(reg.is_match("A"));
    assert!(reg.is_match("_"));

    assert!(reg.is_match("aa"));
    assert!(reg.is_match("aA"));
    assert!(reg.is_match("a0"));
    assert!(reg.is_match("a_"));

    assert!(reg.is_match("Aa"));
    assert!(reg.is_match("AA"));
    assert!(reg.is_match("A0"));
    assert!(reg.is_match("A_"));

    assert!(reg.is_match("_a"));
    assert!(reg.is_match("_A"));
    assert!(reg.is_match("_0"));
    assert!(reg.is_match("__"));

    assert!(!reg.is_match("0"));
    assert!(!reg.is_match("."));
    assert!(!reg.is_match("#"));
    assert!(!reg.is_match("abc()"));
}

#[test]
fn rust_identifier_re() {
    let reg = RUST_IDENTIFIER_RE.clone();

    assert!(reg.is_match("a"));
    assert!(reg.is_match("A"));
    assert!(reg.is_match("_"));

    assert!(reg.is_match("aa"));
    assert!(reg.is_match("aA"));
    assert!(reg.is_match("a0"));
    assert!(reg.is_match("a_"));

    assert!(reg.is_match("Aa"));
    assert!(reg.is_match("AA"));
    assert!(reg.is_match("A0"));
    assert!(reg.is_match("A_"));

    assert!(reg.is_match("_a"));
    assert!(reg.is_match("_A"));
    assert!(reg.is_match("_0"));
    assert!(reg.is_match("__"));

    assert!(!reg.is_match("0"));
    assert!(!reg.is_match("."));
    assert!(!reg.is_match("#"));
    assert!(!reg.is_match("abc()"));
}

#[test]
fn html_section() {
    let reg = Regex::new(&format!("^{}$", HTML_SECTION)).unwrap();

    assert!(reg.is_match("#a"));
    assert!(reg.is_match("#A"));
    assert!(reg.is_match("#0"));
    assert!(reg.is_match("#_"));
    assert!(reg.is_match("#-"));
    assert!(reg.is_match("#."));
    assert!(reg.is_match("#test"));
    // While this is really an associated item, this regex will catch it all
    // the same. It is up to functions with more information to make the
    // difference.
    assert!(reg.is_match("#fn.item"));

    assert!(!reg.is_match("#"));
    assert!(!reg.is_match("abc"));
}

#[test]
fn type_block_start() {
    let type_decls = ["struct", "trait", "enum", "union"];

    let visi_decls = [
        "",
        "pub",
        "pub(crate)",
        "pub(self)",
        "pub(super)",
        "pub(a)",
        "pub(b::a)",
    ];

    let generics = ["", "<A>", "<A, B>", "<A: Trait, const B: usize>"];

    let parentheses = ["", "(", "{", "where A: Trait", "where B: C {"];

    for v in &visi_decls {
        for t in &type_decls {
            for g in &generics {
                let string = &format!("{} {} Type{}\n", v, t, g);
                let captures = TYPE_BLOCK_START.captures(string).unwrap();
                assert_eq!(
                    "Type",
                    captures.name("type").unwrap().as_str(),
                    "{}",
                    string
                );
            }
        }
    }

    for v in &visi_decls {
        for t in &type_decls {
            for g in &generics {
                for p in &parentheses {
                    let string = &format!("{} {} Type{} {}\n", v, t, g, p);
                    let captures = TYPE_BLOCK_START.captures(string).unwrap();
                    assert_eq!(
                        "Type",
                        captures.name("type").unwrap().as_str(),
                        "{}",
                        string
                    );
                }
            }
        }
    }

    for g1 in &generics {
        for g2 in &generics {
            for p in &parentheses {
                let string = &format!("impl{} Type{} {}\n", g1, g2, p);
                let captures = TYPE_BLOCK_START.captures(string).unwrap();
                assert_eq!(
                    "Type",
                    captures.name("type").unwrap().as_str(),
                    "{}",
                    string
                );
            }
        }
    }
}

#[test]
fn local_path_long() {
    fn check_captures(string: &str) {
        let captures = LOCAL_PATH_LONG.captures(string).unwrap();
        assert_eq!(
            "name 1",
            captures.name("header").unwrap().as_str(),
            "{}",
            string
        );
        assert_eq!(
            "item",
            captures.name("link").unwrap().as_str(),
            "{}",
            string
        );
    }

    check_captures("[name 1]: item");
    check_captures("/// [name 1]: item");
    check_captures("//! [name 1]: item");
    check_captures("///\t[name 1]: item");
    check_captures("\t[name 1]:\titem");
    check_captures("[name 1]:\titem");
    check_captures("[name 1]:item");
}
