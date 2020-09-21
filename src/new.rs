use lazy_static::lazy_static;
use regex::Regex;

const ITEM_TYPES: &[&str] = &[
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
    /// Will search for a doc comment link and be used to check if the two
    /// elements are the same, indicating a local path.
    static ref LOCAL_PATH: Regex = Regex::new(concat!(
        r"^\s*(?://[!/] )?",
        r"\[`?(?P<elem>.*?)`?\]: ",
        r"(?P<elem2>.*)\n$",
    )).unwrap();

    /// Line that is a markdown link to a Rust item.
    static ref HTTP_LINK: Regex = Regex::new(r"^\s*(?://[!/] )?\[.*?\]:\s+https?://.*\n$").unwrap();
}

pub struct Context {
    krate: String,
}

pub fn check(line: String, ctx: &Context) -> String {
    let item_link = Regex::new(
        &[
            r"^(?P<link_name>\s*(?://[!/] )?\[.*?\]: )",
            r"(?P<supers>(?:\.\./)*)",
            &format!(r"(?:(?P<crate>{})/)?", ctx.krate),
            r"(?P<intermediates>(?:.*/))?",
            &format!(r"(?:{})\.", ITEM_TYPES.join("|")),
            r"(?P<elem>.*)\.html",
            &format!(
                r"(?:#(?:{})\.(?P<additional>\S*))?\n$",
                ITEM_TYPES.join("|")
            ),
        ]
        .join(""),
    )
    .unwrap();

    let module_link = Regex::new(
        &[
            r"^(?P<link_name>\s*(?://[!/] )?\[.*?\]: )",
            r"(?P<supers>(?:\.\./)*)",
            &format!(r"(?:(?P<crate>{})/)?", ctx.krate),
            r"(?P<mods>(?:.*?/)*)",
            r"index\.html",
            r"(?P<section>#.+)?\n$",
        ]
        .join(""),
    )
    .unwrap();

    if HTTP_LINK.is_match(&line) {
        return line;
    }

    // Handling (possibly complex) regular links.
    let new = if let Some(captures) = item_link.captures(&line) {
        let link_name = captures.name("link_name").unwrap().as_str();
        let elem = captures.name("elem").unwrap().as_str();

        let mut new = String::with_capacity(64);
        new.push_str(link_name);

        // Handling the start of the path.
        if let Some(_) = captures.name("crate") {
            // This a path contained in the crate: the start of a full path is
            // 'crate', not the crate name in this case.
            new.push_str("crate::");
        } else if let Some(supers) = captures.name("supers").map(|x| x.as_str()) {
            // The path is not explicitely contained in the crate but has some
            // 'super' elements.
            let count = supers.matches('/').count();
            // This way we won't allocate a string only to immediately drop it.
            for _ in 0..count {
                new.push_str("super::");
            }
        }

        // Intermediates element like a path through modules.
        // In the case of a path without 'super'
        if let Some(intermediates) = captures.name("intermediates").map(|x| x.as_str()) {
            if intermediates != "./" {
                new.push_str(&intermediates.replace("/", "::"));
            }
        }

        // The main element of the link.
        new.push_str(elem);

        // Additional linked elements like a method or a variant.
        if let Some(additional) = captures.name("additional").map(|x| x.as_str()) {
            new.push_str("::");
            new.push_str(additional);
        }

        // The regexes that will follow expect a `\n` at the end of the line.
        new.push('\n');

        new
    } else {
        line
    };

    let new = if let Some(captures) = module_link.captures(&new) {
        let link_name = captures.name("link_name").unwrap().as_str();
        let section = captures.name("section").map(|x| x.as_str()).unwrap_or("");

        let mut new = String::with_capacity(64);
        new.push_str(link_name);

        // Handling the start of the path.
        if let Some(_) = captures.name("crate") {
            // This a path contained in the crate: the start of a full path is
            // 'crate', not the crate name in this case.
            new.push_str("crate::");
        } else if let Some(supers) = captures.name("supers").map(|x| x.as_str()) {
            // The path is not explicitely contained in the crate but has some
            // 'super' elements.
            let count = supers.matches('/').count();
            // This way we won't allocate a string only to immediately drop it.
            for _ in 0..count {
                new.push_str("super::");
            }
        }

        // Handling the modules names themselves.
        if let Some(mods) = captures.name("mods").map(|x| x.as_str()) {
            // If the link is simply `index.html` the line is removed.
            if mods.is_empty() && section.is_empty() {
                return "".into();
            }

            new.push_str(mods.replace("/", "::").trim_end_matches("::"));
        }

        new.push_str(section);
        new.push('\n');

        new
    } else {
        new
    };

    // Handling local paths.
    if let Some(captures) = LOCAL_PATH.captures(&new) {
        let link = captures.name("elem").unwrap();
        let path = captures.name("elem2").unwrap();
        if path.as_str() == link.as_str() {
            return "".into();
        }
    }

    new
}

#[cfg(test)]
mod tests {
    use super::*;

    lazy_static! {
        static ref STD_CTX: Context = Context {
            krate: String::from("std"),
        };
        static ref CORE_CTX: Context = Context {
            krate: String::from("core"),
        };
    }

    mod unchanged_lines {
        use super::*;

        #[test]
        fn code_line_is_unchanged() {
            let line = "let res = a + b;\n";
            assert_eq!(line, check(line.into(), &STD_CTX));
        }

        #[test]
        fn normal_comment_line_is_unchanged() {
            let line = "// let res = a + b;\n";
            assert_eq!(line, check(line.into(), &STD_CTX));
        }

        #[test]
        fn normal_doc_comment_line_is_unchanged() {
            let line = "/// let res = a + b;\n";
            assert_eq!(line, check(line.into(), &STD_CTX));
        }

        #[test]
        fn normal_header_doc_comment_line_is_unchanged() {
            let line = "//! let res = a + b;\n";
            assert_eq!(line, check(line.into(), &STD_CTX));
        }

        #[test]
        fn indentation_is_unchanged() {
            let line = "  //! let res = a + b;\n";
            assert_eq!(line, check(line.into(), &STD_CTX));

            let line = "    //! let res = a + b;\n";
            assert_eq!(line, check(line.into(), &STD_CTX));
        }

        #[test]
        fn http_link_is_ignored() {
            let line = "/// [`String`]: http://www.example.com/index.html#section\n";
            assert_eq!(line, check(line.into(), &STD_CTX));

            let line = "    /// [`String`]: https://www.example.com/index.html#section\n";
            assert_eq!(line, check(line.into(), &STD_CTX));
        }
    }

    mod paths {
        use super::*;

        #[test]
        fn local_path_is_deleted() {
            let line = "/// [`String`]: String\n";
            assert_eq!("", check(line.into(), &STD_CTX));

            let line = "    /// [String]: String\n";
            assert_eq!("", check(line.into(), &STD_CTX));

            let line = "[`String`]: String\n";
            assert_eq!("", check(line.into(), &STD_CTX));

            let line = "    [String]: String\n";
            assert_eq!("", check(line.into(), &STD_CTX));
        }

        #[test]
        fn long_path_is_unchanged() {
            let line = "/// [`String`]: string::String\n";
            assert_eq!(line, check(line.into(), &STD_CTX));

            let line = "    /// [String]: string::String\n";
            assert_eq!(line, check(line.into(), &STD_CTX));

            let line = "[`String`]: string::String\n";
            assert_eq!(line, check(line.into(), &STD_CTX));

            let line = "    [String]: string::String\n";
            assert_eq!(line, check(line.into(), &STD_CTX));
        }

        #[test]
        fn full_path_is_unchanged() {
            let line = "/// [`String`]: ::std::string::String\n";
            assert_eq!(line, check(line.into(), &STD_CTX));

            let line = "    /// [String]: ::std::string::String\n";
            assert_eq!(line, check(line.into(), &STD_CTX));

            let line = "[`String`]: ::std::string::String\n";
            assert_eq!(line, check(line.into(), &STD_CTX));

            let line = "    [String]: ::std::string::String\n";
            assert_eq!(line, check(line.into(), &STD_CTX));
        }
    }

    mod item_tests {
        use super::*;

        #[test]
        fn local_link_is_deleted() {
            let line = "/// [`String`]: struct.String.html\n";
            assert_eq!("", check(line.into(), &STD_CTX));

            let line = "    /// [String]: struct.String.html\n";
            assert_eq!("", check(line.into(), &STD_CTX));

            let line = "[`String`]: struct.String.html\n";
            assert_eq!("", check(line.into(), &STD_CTX));

            let line = "    [String]: struct.String.html\n";
            assert_eq!("", check(line.into(), &STD_CTX));
        }

        #[test]
        fn long_link_is_transformed() {
            let line = "/// [`String`]: string/struct.String.html\n";
            assert_eq!(
                "/// [`String`]: string::String\n",
                check(line.into(), &STD_CTX)
            );

            let line = "    /// [String]: string/struct.String.html\n";
            assert_eq!(
                "    /// [String]: string::String\n",
                check(line.into(), &STD_CTX)
            );

            let line = "[`String`]: string/struct.String.html\n";
            assert_eq!("[`String`]: string::String\n", check(line.into(), &STD_CTX));

            let line = "    [String]: string/struct.String.html\n";
            assert_eq!(
                "    [String]: string::String\n",
                check(line.into(), &STD_CTX)
            );
        }

        #[test]
        fn full_link_is_transformed_crate() {
            let line = "/// [`String`]: std/string/struct.String.html\n";
            assert_eq!(
                "/// [`String`]: crate::string::String\n",
                check(line.into(), &STD_CTX)
            );

            let line = "    /// [String]: std/string/struct.String.html\n";
            assert_eq!(
                "    /// [String]: crate::string::String\n",
                check(line.into(), &STD_CTX)
            );

            let line = "[`String`]: std/string/struct.String.html\n";
            assert_eq!(
                "[`String`]: crate::string::String\n",
                check(line.into(), &STD_CTX)
            );

            let line = "    [String]: std/string/struct.String.html\n";
            assert_eq!(
                "    [String]: crate::string::String\n",
                check(line.into(), &STD_CTX)
            );
        }

        #[test]
        fn full_link_is_transformed_crate_over_super() {
            let line = "/// [`String`]: ../../std/string/struct.String.html\n";
            assert_eq!(
                "/// [`String`]: crate::string::String\n",
                check(line.into(), &STD_CTX)
            );

            let line = "    /// [String]: ../../std/string/struct.String.html\n";
            assert_eq!(
                "    /// [String]: crate::string::String\n",
                check(line.into(), &STD_CTX)
            );

            let line = "[`String`]: ../../std/string/struct.String.html\n";
            assert_eq!(
                "[`String`]: crate::string::String\n",
                check(line.into(), &STD_CTX)
            );

            let line = "    [String]: ../../std/string/struct.String.html\n";
            assert_eq!(
                "    [String]: crate::string::String\n",
                check(line.into(), &STD_CTX)
            );
        }

        #[test]
        fn full_link_is_transformed_not_crate() {
            let line = "/// [`String`]: std/string/struct.String.html\n";
            assert_eq!(
                "/// [`String`]: std::string::String\n",
                check(line.into(), &CORE_CTX)
            );

            let line = "    /// [String]: std/string/struct.String.html\n";
            assert_eq!(
                "    /// [String]: std::string::String\n",
                check(line.into(), &CORE_CTX)
            );

            let line = "[`String`]: std/string/struct.String.html\n";
            assert_eq!(
                "[`String`]: std::string::String\n",
                check(line.into(), &CORE_CTX)
            );

            let line = "    [String]: std/string/struct.String.html\n";
            assert_eq!(
                "    [String]: std::string::String\n",
                check(line.into(), &CORE_CTX)
            );
        }

        #[test]
        fn full_link_is_transformed_super() {
            let line = "/// [`String`]: ../../string/struct.String.html\n";
            assert_eq!(
                "/// [`String`]: super::super::string::String\n",
                check(line.into(), &CORE_CTX)
            );

            let line = "    /// [String]: ../../string/struct.String.html\n";
            assert_eq!(
                "    /// [String]: super::super::string::String\n",
                check(line.into(), &CORE_CTX)
            );

            let line = "[`String`]: ../../string/struct.String.html\n";
            assert_eq!(
                "[`String`]: super::super::string::String\n",
                check(line.into(), &CORE_CTX)
            );

            let line = "    [String]: ../../string/struct.String.html\n";
            assert_eq!(
                "    [String]: super::super::string::String\n",
                check(line.into(), &CORE_CTX)
            );
        }

        #[test]
        fn additional_is_kept() {
            let line = "/// [`String`]: struct.String.html#method.as_ref\n";
            assert_eq!(
                "/// [`String`]: String::as_ref\n",
                check(line.into(), &STD_CTX)
            );

            let line = "    /// [String]: struct.String.html#method.as_ref\n";
            assert_eq!(
                "    /// [String]: String::as_ref\n",
                check(line.into(), &STD_CTX)
            );

            let line = "[`String`]: struct.String.html#method.as_ref\n";
            assert_eq!("[`String`]: String::as_ref\n", check(line.into(), &STD_CTX));

            let line = "    [String]: struct.String.html#method.as_ref\n";
            assert_eq!(
                "    [String]: String::as_ref\n",
                check(line.into(), &STD_CTX)
            );
        }
    }

    mod module_tests {
        use super::*;

        #[test]
        fn local_link_is_deleted() {
            let line = "/// [`string`]: string/index.html\n";
            assert_eq!("", check(line.into(), &STD_CTX));

            let line = "    /// [string]: string/index.html\n";
            assert_eq!("", check(line.into(), &STD_CTX));

            let line = "[`string`]: string/index.html\n";
            assert_eq!("", check(line.into(), &STD_CTX));

            let line = "    [string]: string/index.html\n";
            assert_eq!("", check(line.into(), &STD_CTX));

            let line = "/// [`string`]: index.html\n";
            assert_eq!("", check(line.into(), &STD_CTX));

            let line = "    /// [string]: index.html\n";
            assert_eq!("", check(line.into(), &STD_CTX));

            let line = "[`string`]: index.html\n";
            assert_eq!("", check(line.into(), &STD_CTX));

            let line = "    [string]: index.html\n";
            assert_eq!("", check(line.into(), &STD_CTX));
        }

        #[test]
        fn long_link_is_transformed() {
            let line = "/// [`string`]: module/string/index.html\n";
            assert_eq!(
                "/// [`string`]: module::string\n",
                check(line.into(), &STD_CTX)
            );

            let line = "    /// [string]: module/string/index.html\n";
            assert_eq!(
                "    /// [string]: module::string\n",
                check(line.into(), &STD_CTX)
            );

            let line = "[`string`]: module/string/index.html\n";
            assert_eq!("[`string`]: module::string\n", check(line.into(), &STD_CTX));

            let line = "    [string]: module/string/index.html\n";
            assert_eq!(
                "    [string]: module::string\n",
                check(line.into(), &STD_CTX)
            );
        }

        #[test]
        fn full_link_is_transformed_crate() {
            let line = "/// [`string`]: std/string/index.html\n";
            assert_eq!(
                "/// [`string`]: crate::string\n",
                check(line.into(), &STD_CTX)
            );

            let line = "    /// [string]: std/string/index.html\n";
            assert_eq!(
                "    /// [string]: crate::string\n",
                check(line.into(), &STD_CTX)
            );

            let line = "[`string`]: std/string/index.html\n";
            assert_eq!("[`string`]: crate::string\n", check(line.into(), &STD_CTX));

            let line = "    [string]: std/string/index.html\n";
            assert_eq!(
                "    [string]: crate::string\n",
                check(line.into(), &STD_CTX)
            );
        }

        #[test]
        fn full_link_is_transformed_crate_over_super() {
            let line = "/// [`string`]: ../../std/string/index.html\n";
            assert_eq!(
                "/// [`string`]: crate::string\n",
                check(line.into(), &STD_CTX)
            );

            let line = "    /// [string]: ../../std/string/index.html\n";
            assert_eq!(
                "    /// [string]: crate::string\n",
                check(line.into(), &STD_CTX)
            );

            let line = "[`string`]: ../../std/string/index.html\n";
            assert_eq!("[`string`]: crate::string\n", check(line.into(), &STD_CTX));

            let line = "    [string]: ../../std/string/index.html\n";
            assert_eq!(
                "    [string]: crate::string\n",
                check(line.into(), &STD_CTX)
            );
        }

        #[test]
        fn full_link_is_transformed_not_crate() {
            let line = "/// [`string`]: std/string/index.html\n";
            assert_eq!(
                "/// [`string`]: std::string\n",
                check(line.into(), &CORE_CTX)
            );

            let line = "    /// [string]: std/string/index.html\n";
            assert_eq!(
                "    /// [string]: std::string\n",
                check(line.into(), &CORE_CTX)
            );

            let line = "[`string`]: std/string/index.html\n";
            assert_eq!("[`string`]: std::string\n", check(line.into(), &CORE_CTX));

            let line = "    [string]: std/string/index.html\n";
            assert_eq!("    [string]: std::string\n", check(line.into(), &CORE_CTX));
        }

        #[test]
        fn full_link_is_transformed_super() {
            let line = "/// [`string`]: ../../string/index.html\n";
            assert_eq!(
                "/// [`string`]: super::super::string\n",
                check(line.into(), &STD_CTX)
            );

            let line = "    /// [string]: ../../string/index.html\n";
            assert_eq!(
                "    /// [string]: super::super::string\n",
                check(line.into(), &STD_CTX)
            );

            let line = "[`string`]: ../../string/index.html\n";
            assert_eq!(
                "[`string`]: super::super::string\n",
                check(line.into(), &STD_CTX)
            );

            let line = "    [string]: ../../string/index.html\n";
            assert_eq!(
                "    [string]: super::super::string\n",
                check(line.into(), &STD_CTX)
            );
        }

        #[test]
        fn section_is_kept() {
            let line = "/// [`string`]: string/index.html#my-section\n";
            assert_eq!(
                "/// [`string`]: string#my-section\n",
                check(line.into(), &STD_CTX)
            );

            let line = "    /// [string]: string/index.html#my-section\n";
            assert_eq!(
                "    /// [string]: string#my-section\n",
                check(line.into(), &STD_CTX)
            );

            let line = "[`string`]: string/index.html#my-section\n";
            assert_eq!(
                "[`string`]: string#my-section\n",
                check(line.into(), &STD_CTX)
            );

            let line = "    [string]: string/index.html#my-section\n";
            assert_eq!(
                "    [string]: string#my-section\n",
                check(line.into(), &STD_CTX)
            );

            let line = "/// [`see my section`]: index.html#my-section\n";
            assert_eq!(
                "/// [`see my section`]: #my-section\n",
                check(line.into(), &STD_CTX)
            );

            let line = "    /// [see my section]: index.html#my-section\n";
            assert_eq!(
                "    /// [see my section]: #my-section\n",
                check(line.into(), &STD_CTX)
            );

            let line = "[`see my section`]: index.html#my-section\n";
            assert_eq!(
                "[`see my section`]: #my-section\n",
                check(line.into(), &STD_CTX)
            );

            let line = "    [see my section]: index.html#my-section\n";
            assert_eq!(
                "    [see my section]: #my-section\n",
                check(line.into(), &STD_CTX)
            );
        }
    }
}
