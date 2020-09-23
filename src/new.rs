use lazy_static::lazy_static;
use regex::Regex;

/// All item types that can be produced by `rustdoc`.
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

    /// Start of a block where `Self` has a sense.
    static ref TYPE_BLOCK_START: Regex = Regex::new(concat!(
        r"^(?P<spaces>\s*)",
        r"(?:pub(?:\(.+\))? )?",
        r"(?:struct|trait|impl(?:<.*?>)?(?: .*? for)?|enum|union) ",
        r"(?P<type>\w+)",
        r"(?:<.*?>)?",
        r"(?P<parenthese>\()?",
        r".*\n$",
    )).unwrap();

    static ref METHOD_ANCHOR: Regex = Regex::new(&[
        r"^(?P<link_name>\s*(?://[!/] )?\[.*?\]: )",
        &format!(
            r"#(?:{})\.(?P<additional>[\w_]+)\n$",
            ITEM_TYPES.join("|")
        ),
    ].join(""))
    .unwrap();
}

/// Context for the check. It notably contains informations about the crate and
/// the current type (e.g, for `#method.name` links).
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Context {
    /// Name of the crate on which the tool is run.
    krate: String,
    /// Name of the type that is `Self` for the current block.
    curr_type_block: Option<String>,
    /// End of the current block for `Self` (if any).
    end_type_block: String,
    // NOTE: at the moment nested type blocks are not handled.
}

pub fn check(line: String, ctx: &mut Context) -> String {
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

    // Early return on http links.
    if HTTP_LINK.is_match(&line) {
        return line;
    }

    // Early return on context change too, after updating the context.
    if let Some(captures) = TYPE_BLOCK_START.captures(&line) {
        ctx.curr_type_block = captures.name("type").map(|x| x.as_str().to_string());
        ctx.end_type_block = {
            // Tuple struct or very simple item (empty enum for example).
            // We hope the next empty line will come before any link.
            if line.ends_with(";\n") || line.ends_with("}\n") {
                '\n'.into()
            } else {
                // When the item is not simple we try to compute what will be
                // the end of the block.
                let mut s = captures.name("spaces").unwrap().as_str().to_string();

                if let Some(_) = captures.name("parenthese") {
                    s.push(')');
                } else {
                    s.push('}');
                }

                s
            }
        };

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

    let new = if let (Some(ref captures), Some(ref ty)) =
        (METHOD_ANCHOR.captures(&new), &ctx.curr_type_block)
    {
        let link_name = captures.name("link_name").unwrap().as_str();
        let additional = captures.name("additional").unwrap().as_str();

        format!(
            "{link}{ty}::{add}\n",
            link = link_name,
            ty = ty,
            add = additional
        )
    } else {
        new
    };

    // When reaching the end of the current type block, update the context to
    // reflect it.
    if ctx.curr_type_block.is_some() && new.starts_with(&ctx.end_type_block) {
        ctx.curr_type_block = None;
        ctx.end_type_block.clear();
    }

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
            krate: "std".into(),
            curr_type_block: None,
            end_type_block: "".into(),
        };
        static ref CORE_CTX: Context = Context {
            krate: "core".into(),
            curr_type_block: None,
            end_type_block: "".into(),
        };
    }

    mod unchanged_lines {
        use super::*;

        #[test]
        fn code_line_is_unchanged() {
            let mut ctx = STD_CTX.clone();

            let line = "let res = a + b;\n";
            assert_eq!(line, check(line.into(), &mut ctx));

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn normal_comment_line_is_unchanged() {
            let mut ctx = STD_CTX.clone();

            let line = "// let res = a + b;\n";
            assert_eq!(line, check(line.into(), &mut ctx));

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn normal_doc_comment_line_is_unchanged() {
            let mut ctx = STD_CTX.clone();

            let line = "/// let res = a + b;\n";
            assert_eq!(line, check(line.into(), &mut ctx));

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn normal_header_doc_comment_line_is_unchanged() {
            let mut ctx = STD_CTX.clone();

            let line = "//! let res = a + b;\n";
            assert_eq!(line, check(line.into(), &mut ctx));

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn indentation_is_unchanged() {
            let mut ctx = STD_CTX.clone();

            let line = "  //! let res = a + b;\n";
            assert_eq!(line, check(line.into(), &mut ctx));

            let line = "    //! let res = a + b;\n";
            assert_eq!(line, check(line.into(), &mut ctx));

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn http_link_is_ignored() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`String`]: http://www.example.com/index.html#section\n";
            assert_eq!(line, check(line.into(), &mut ctx));

            let line = "    /// [`String`]: https://www.example.com/index.html#section\n";
            assert_eq!(line, check(line.into(), &mut ctx));

            assert_eq!(*STD_CTX, ctx);
        }
    }

    mod paths {
        use super::*;

        #[test]
        fn local_path_is_deleted() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`String`]: String\n";
            assert_eq!("", check(line.into(), &mut ctx));

            let line = "    /// [String]: String\n";
            assert_eq!("", check(line.into(), &mut ctx));

            let line = "[`String`]: String\n";
            assert_eq!("", check(line.into(), &mut ctx));

            let line = "    [String]: String\n";
            assert_eq!("", check(line.into(), &mut ctx));

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn long_path_is_unchanged() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`String`]: string::String\n";
            assert_eq!(line, check(line.into(), &mut ctx));

            let line = "    /// [String]: string::String\n";
            assert_eq!(line, check(line.into(), &mut ctx));

            let line = "[`String`]: string::String\n";
            assert_eq!(line, check(line.into(), &mut ctx));

            let line = "    [String]: string::String\n";
            assert_eq!(line, check(line.into(), &mut ctx));

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn full_path_is_unchanged() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`String`]: ::std::string::String\n";
            assert_eq!(line, check(line.into(), &mut ctx));

            let line = "    /// [String]: ::std::string::String\n";
            assert_eq!(line, check(line.into(), &mut ctx));

            let line = "[`String`]: ::std::string::String\n";
            assert_eq!(line, check(line.into(), &mut ctx));

            let line = "    [String]: ::std::string::String\n";
            assert_eq!(line, check(line.into(), &mut ctx));

            assert_eq!(*STD_CTX, ctx);
        }
    }

    mod item_tests {
        use super::*;

        #[test]
        fn local_link_is_deleted() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`String`]: struct.String.html\n";
            assert_eq!("", check(line.into(), &mut ctx));

            let line = "    /// [String]: struct.String.html\n";
            assert_eq!("", check(line.into(), &mut ctx));

            let line = "[`String`]: struct.String.html\n";
            assert_eq!("", check(line.into(), &mut ctx));

            let line = "    [String]: struct.String.html\n";
            assert_eq!("", check(line.into(), &mut ctx));

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn long_link_is_transformed() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`String`]: string/struct.String.html\n";
            assert_eq!(
                "/// [`String`]: string::String\n",
                check(line.into(), &mut ctx)
            );

            let line = "    /// [String]: string/struct.String.html\n";
            assert_eq!(
                "    /// [String]: string::String\n",
                check(line.into(), &mut ctx)
            );

            let line = "[`String`]: string/struct.String.html\n";
            assert_eq!("[`String`]: string::String\n", check(line.into(), &mut ctx));

            let line = "    [String]: string/struct.String.html\n";
            assert_eq!(
                "    [String]: string::String\n",
                check(line.into(), &mut ctx)
            );

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn full_link_is_transformed_crate() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`String`]: std/string/struct.String.html\n";
            assert_eq!(
                "/// [`String`]: crate::string::String\n",
                check(line.into(), &mut ctx)
            );

            let line = "    /// [String]: std/string/struct.String.html\n";
            assert_eq!(
                "    /// [String]: crate::string::String\n",
                check(line.into(), &mut ctx)
            );

            let line = "[`String`]: std/string/struct.String.html\n";
            assert_eq!(
                "[`String`]: crate::string::String\n",
                check(line.into(), &mut ctx)
            );

            let line = "    [String]: std/string/struct.String.html\n";
            assert_eq!(
                "    [String]: crate::string::String\n",
                check(line.into(), &mut ctx)
            );

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]

        fn full_link_is_transformed_crate_over_super() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`String`]: ../../std/string/struct.String.html\n";
            assert_eq!(
                "/// [`String`]: crate::string::String\n",
                check(line.into(), &mut ctx)
            );

            let line = "    /// [String]: ../../std/string/struct.String.html\n";
            assert_eq!(
                "    /// [String]: crate::string::String\n",
                check(line.into(), &mut ctx)
            );

            let line = "[`String`]: ../../std/string/struct.String.html\n";
            assert_eq!(
                "[`String`]: crate::string::String\n",
                check(line.into(), &mut ctx)
            );

            let line = "    [String]: ../../std/string/struct.String.html\n";
            assert_eq!(
                "    [String]: crate::string::String\n",
                check(line.into(), &mut ctx)
            );

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn full_link_is_transformed_not_crate() {
            let mut ctx = CORE_CTX.clone();

            let line = "/// [`String`]: std/string/struct.String.html\n";
            assert_eq!(
                "/// [`String`]: std::string::String\n",
                check(line.into(), &mut ctx)
            );

            let line = "    /// [String]: std/string/struct.String.html\n";
            assert_eq!(
                "    /// [String]: std::string::String\n",
                check(line.into(), &mut ctx)
            );

            let line = "[`String`]: std/string/struct.String.html\n";
            assert_eq!(
                "[`String`]: std::string::String\n",
                check(line.into(), &mut ctx)
            );

            let line = "    [String]: std/string/struct.String.html\n";
            assert_eq!(
                "    [String]: std::string::String\n",
                check(line.into(), &mut ctx)
            );

            assert_eq!(*CORE_CTX, ctx);
        }

        #[test]
        fn full_link_is_transformed_super() {
            let mut ctx = CORE_CTX.clone();

            let line = "/// [`String`]: ../../string/struct.String.html\n";
            assert_eq!(
                "/// [`String`]: super::super::string::String\n",
                check(line.into(), &mut ctx)
            );

            let line = "    /// [String]: ../../string/struct.String.html\n";
            assert_eq!(
                "    /// [String]: super::super::string::String\n",
                check(line.into(), &mut ctx)
            );

            let line = "[`String`]: ../../string/struct.String.html\n";
            assert_eq!(
                "[`String`]: super::super::string::String\n",
                check(line.into(), &mut ctx)
            );

            let line = "    [String]: ../../string/struct.String.html\n";
            assert_eq!(
                "    [String]: super::super::string::String\n",
                check(line.into(), &mut ctx)
            );

            assert_eq!(*CORE_CTX, ctx);
        }

        #[test]
        fn additional_is_kept() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`String`]: struct.String.html#method.as_ref\n";
            assert_eq!(
                "/// [`String`]: String::as_ref\n",
                check(line.into(), &mut ctx)
            );

            let line = "    /// [String]: struct.String.html#method.as_ref\n";
            assert_eq!(
                "    /// [String]: String::as_ref\n",
                check(line.into(), &mut ctx)
            );

            let line = "[`String`]: struct.String.html#method.as_ref\n";
            assert_eq!("[`String`]: String::as_ref\n", check(line.into(), &mut ctx));

            let line = "    [String]: struct.String.html#method.as_ref\n";
            assert_eq!(
                "    [String]: String::as_ref\n",
                check(line.into(), &mut ctx)
            );

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn all_items_combination() {
            let mut ctx = STD_CTX.clone();

            for item in ITEM_TYPES {
                for added in ITEM_TYPES {
                    let line = format!(
                        "//! [`String`]: ../../std/string/{item}.String.html#{added}.as_ref\n",
                        item = item,
                        added = added
                    );
                    assert_eq!(
                        "//! [`String`]: crate::string::String::as_ref\n",
                        check(line, &mut ctx)
                    );

                    assert_eq!(*STD_CTX, ctx);
                }
            }
        }
    }

    mod module_tests {
        use super::*;

        #[test]
        fn local_link_is_deleted() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`string`]: string/index.html\n";
            assert_eq!("", check(line.into(), &mut ctx));

            let line = "    /// [string]: string/index.html\n";
            assert_eq!("", check(line.into(), &mut ctx));

            let line = "[`string`]: string/index.html\n";
            assert_eq!("", check(line.into(), &mut ctx));

            let line = "    [string]: string/index.html\n";
            assert_eq!("", check(line.into(), &mut ctx));

            let line = "/// [`string`]: index.html\n";
            assert_eq!("", check(line.into(), &mut ctx));

            let line = "    /// [string]: index.html\n";
            assert_eq!("", check(line.into(), &mut ctx));

            let line = "[`string`]: index.html\n";
            assert_eq!("", check(line.into(), &mut ctx));

            let line = "    [string]: index.html\n";
            assert_eq!("", check(line.into(), &mut ctx));

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn long_link_is_transformed() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`string`]: module/string/index.html\n";
            assert_eq!(
                "/// [`string`]: module::string\n",
                check(line.into(), &mut ctx)
            );

            let line = "    /// [string]: module/string/index.html\n";
            assert_eq!(
                "    /// [string]: module::string\n",
                check(line.into(), &mut ctx)
            );

            let line = "[`string`]: module/string/index.html\n";
            assert_eq!("[`string`]: module::string\n", check(line.into(), &mut ctx));

            let line = "    [string]: module/string/index.html\n";
            assert_eq!(
                "    [string]: module::string\n",
                check(line.into(), &mut ctx)
            );

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn full_link_is_transformed_crate() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`string`]: std/string/index.html\n";
            assert_eq!(
                "/// [`string`]: crate::string\n",
                check(line.into(), &mut ctx)
            );

            let line = "    /// [string]: std/string/index.html\n";
            assert_eq!(
                "    /// [string]: crate::string\n",
                check(line.into(), &mut ctx)
            );

            let line = "[`string`]: std/string/index.html\n";
            assert_eq!("[`string`]: crate::string\n", check(line.into(), &mut ctx));

            let line = "    [string]: std/string/index.html\n";
            assert_eq!(
                "    [string]: crate::string\n",
                check(line.into(), &mut ctx)
            );

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn full_link_is_transformed_crate_over_super() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`string`]: ../../std/string/index.html\n";
            assert_eq!(
                "/// [`string`]: crate::string\n",
                check(line.into(), &mut ctx)
            );

            let line = "    /// [string]: ../../std/string/index.html\n";
            assert_eq!(
                "    /// [string]: crate::string\n",
                check(line.into(), &mut ctx)
            );

            let line = "[`string`]: ../../std/string/index.html\n";
            assert_eq!("[`string`]: crate::string\n", check(line.into(), &mut ctx));

            let line = "    [string]: ../../std/string/index.html\n";
            assert_eq!(
                "    [string]: crate::string\n",
                check(line.into(), &mut ctx)
            );

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn full_link_is_transformed_not_crate() {
            let mut ctx = CORE_CTX.clone();

            let line = "/// [`string`]: std/string/index.html\n";
            assert_eq!(
                "/// [`string`]: std::string\n",
                check(line.into(), &mut ctx)
            );

            let line = "    /// [string]: std/string/index.html\n";
            assert_eq!(
                "    /// [string]: std::string\n",
                check(line.into(), &mut ctx)
            );

            let line = "[`string`]: std/string/index.html\n";
            assert_eq!("[`string`]: std::string\n", check(line.into(), &mut ctx));

            let line = "    [string]: std/string/index.html\n";
            assert_eq!("    [string]: std::string\n", check(line.into(), &mut ctx));

            assert_eq!(*CORE_CTX, ctx);
        }

        #[test]
        fn full_link_is_transformed_super() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`string`]: ../../string/index.html\n";
            assert_eq!(
                "/// [`string`]: super::super::string\n",
                check(line.into(), &mut ctx)
            );

            let line = "    /// [string]: ../../string/index.html\n";
            assert_eq!(
                "    /// [string]: super::super::string\n",
                check(line.into(), &mut ctx)
            );

            let line = "[`string`]: ../../string/index.html\n";
            assert_eq!(
                "[`string`]: super::super::string\n",
                check(line.into(), &mut ctx)
            );

            let line = "    [string]: ../../string/index.html\n";
            assert_eq!(
                "    [string]: super::super::string\n",
                check(line.into(), &mut ctx)
            );

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn section_is_kept() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`string`]: string/index.html#my-section\n";
            assert_eq!(
                "/// [`string`]: string#my-section\n",
                check(line.into(), &mut ctx)
            );

            let line = "    /// [string]: string/index.html#my-section\n";
            assert_eq!(
                "    /// [string]: string#my-section\n",
                check(line.into(), &mut ctx)
            );

            let line = "[`string`]: string/index.html#my-section\n";
            assert_eq!(
                "[`string`]: string#my-section\n",
                check(line.into(), &mut ctx)
            );

            let line = "    [string]: string/index.html#my-section\n";
            assert_eq!(
                "    [string]: string#my-section\n",
                check(line.into(), &mut ctx)
            );

            let line = "/// [`see my section`]: index.html#my-section\n";
            assert_eq!(
                "/// [`see my section`]: #my-section\n",
                check(line.into(), &mut ctx)
            );

            let line = "    /// [see my section]: index.html#my-section\n";
            assert_eq!(
                "    /// [see my section]: #my-section\n",
                check(line.into(), &mut ctx)
            );

            let line = "[`see my section`]: index.html#my-section\n";
            assert_eq!(
                "[`see my section`]: #my-section\n",
                check(line.into(), &mut ctx)
            );

            let line = "    [see my section]: index.html#my-section\n";
            assert_eq!(
                "    [see my section]: #my-section\n",
                check(line.into(), &mut ctx)
            );

            assert_eq!(*STD_CTX, ctx);
        }
    }

    mod type_block {
        use super::*;

        #[test]
        fn struct_blocks() {
            let mut ctx = STD_CTX.clone();

            let line = "struct A {}\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "\n");

            let line = "struct A();\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "\n");

            let line = "struct A { inner: String, }\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "\n");

            let line = "struct A(usize);\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "\n");

            let line = "struct A<'a, B=u8> where B: Trait + 'a {\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "}");

            let line = "struct C<T=u8>(usize, (isize, T));\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("C".into()));
            assert_eq!(ctx.end_type_block, "\n");
        }

        #[test]
        fn trait_blocks() {
            let mut ctx = STD_CTX.clone();

            let line = "trait A {}\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "\n");

            let line = "trait A {\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "}");

            let line = "trait A { type T: Into<String>, }\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "\n");

            let line = "trait A<'a, B=u8> where B: Trait + 'a {\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "}");
        }

        #[test]
        fn enum_blocks() {
            let mut ctx = STD_CTX.clone();

            let line = "enum A {}\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "\n");

            let line = "enum A {\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "}");

            let line = "enum A { Variant1, Variant2 }\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "\n");

            let line = "enum A<'a, B=u8> where B: Trait + 'a {\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "}");
        }

        #[test]
        fn union_blocks() {
            let mut ctx = STD_CTX.clone();

            let line = "union A {}\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "\n");

            let line = "union A {\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "}");

            let line = "union A { f: f64, u: u64 }\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "\n");

            let line = "union A<'a, B=u8> where B: Trait + 'a {\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "}");
        }

        #[test]
        fn impl_blocks() {
            let mut ctx = STD_CTX.clone();

            let line = "impl Trait for A {}\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "\n");

            let line = "impl A {\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "}");

            let line = "impl <T> Toto for A<T> {\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "}");

            let line = "impl Trait for A { type B = String }\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "\n");

            let line = "impl<'a: 'static, B> Trait for A where B: Toto + 'a {\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "}");

            let line = "impl<'a, 'b, B: Trait<IntoIterator<Item=String>>> Toto for A where B: Toto + 'a, 'b: 'a, I: A<I> {\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "}");
        }

        #[test]
        fn visibility_modifiers_are_handled() {
            let mut ctx = STD_CTX.clone();

            let line = "pub struct A {}\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "\n");

            let line = "pub(crate) struct A();\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "\n");

            let line = "pub(super) struct A { inner: String, }\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "\n");

            let line = "pub(self) struct A(usize);\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "\n");

            let line = "pub(crate::module) struct A<'a, B=u8> where B: Trait + 'a {\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "}");

            let line = "pub(mod1::mod2) struct C<T=u8>(usize, (isize, T));\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("C".into()));
            assert_eq!(ctx.end_type_block, "\n");
        }

        #[test]
        fn indentation_is_remembered() {
            let mut ctx = STD_CTX.clone();

            let line = "    struct A {}\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "\n");

            let line = "  struct A();\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "\n");

            let line = "   struct A { inner: String, }\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "\n");

            let line = " struct A(usize);\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "\n");

            let line = "  struct A<'a, B=u8> where B: Trait + 'a {\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "  }");

            let line = "  struct A<'a, B=u8> where B: Trait + 'a {\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "  }");

            let line = "    struct A<'a, B=u8> where B: Trait + 'a {\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("A".into()));
            assert_eq!(ctx.end_type_block, "    }");
        }

        #[test]
        fn end_set_block_to_none() {
            let mut ctx = STD_CTX.clone();
            ctx.curr_type_block = Some("String".into());
            ctx.end_type_block = '\n'.into();

            let line = "\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, None);
            assert_eq!(ctx.end_type_block, "");

            ctx.curr_type_block = Some("String".into());
            ctx.end_type_block = "    }".into();
            let line = "\n";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, Some("String".into()));
            assert_eq!(ctx.end_type_block, "    }");

            let line = "    }";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, None);
            assert_eq!(ctx.end_type_block, "");

            ctx.curr_type_block = Some("String".into());
            ctx.end_type_block = "  )".into();

            let line = "  )";
            assert_eq!(line, check(line.into(), &mut ctx));
            assert_eq!(ctx.curr_type_block, None);
            assert_eq!(ctx.end_type_block, "");
        }

        #[test]
        fn method_anchor_when_type_is_none() {
            let mut ctx = STD_CTX.clone();
            ctx.curr_type_block = None;
            ctx.end_type_block = "".into();

            for &item in ITEM_TYPES {
                let line = format!("/// [`link name`]: #{}.as_ref", item);
                assert_eq!(line.clone(), check(line, &mut ctx));

                let line = format!("    //! [link name]: #{}.as_ref", item);
                assert_eq!(line.clone(), check(line, &mut ctx));

                let line = format!("[`link name`]: #{}.as_ref", item);
                assert_eq!(line.clone(), check(line, &mut ctx));

                let line = format!("    [link name]: #{}.as_ref", item);
                assert_eq!(line.clone(), check(line, &mut ctx));
            }
        }

        #[test]
        fn method_anchor_when_type_is_some() {
            let mut ctx = STD_CTX.clone();
            let ty = "String";
            ctx.curr_type_block = Some(ty.into());
            ctx.end_type_block = '\n'.into();

            for &item in ITEM_TYPES {
                let line = format!("/// [`link name`]: #{}.as_ref\n", item);
                let expected = format!("/// [`link name`]: {}::as_ref\n", ty);
                assert_eq!(expected, check(line, &mut ctx));

                let line = format!("    //! [link name]: #{}.as_ref\n", item);
                let expected = format!("    //! [link name]: {}::as_ref\n", ty);
                assert_eq!(expected, check(line, &mut ctx));

                let line = format!("[`link name`]: #{}.as_ref\n", item);
                let expected = format!("[`link name`]: {}::as_ref\n", ty);
                assert_eq!(expected, check(line, &mut ctx));

                let line = format!("    [link name]: #{}.as_ref\n", item);
                let expected = format!("    [link name]: {}::as_ref\n", ty);
                assert_eq!(expected, check(line, &mut ctx));
            }
        }
    }
}
