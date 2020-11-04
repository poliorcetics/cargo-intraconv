// TODO:
//
// - transform_file
use super::*;

impl PartialEq<str> for Action {
    fn eq(&self, other: &str) -> bool {
        match self {
            Action::Unchanged { line } => line == other,
            Action::Deleted { line, pos: _ } => line == other,
            Action::Replaced {
                line: _,
                new,
                pos: _,
            } => new == other,
        }
    }
}

impl PartialEq<Action> for str {
    fn eq(&self, other: &Action) -> bool {
        other == self
    }
}

impl PartialEq<Action> for &str {
    fn eq(&self, other: &Action) -> bool {
        other == *self
    }
}

impl PartialEq<Action> for String {
    fn eq(&self, other: &Action) -> bool {
        other == self.as_str()
    }
}

impl Action {
    fn is_deleted(&self) -> bool {
        match self {
            Self::Deleted { line: _, pos: _ } => true,
            _ => false,
        }
    }
}

mod regexes {
    use super::*;

    mod fav_links_tests {
        use super::*;

        #[test]
        fn docs_rs() {
            let string = "   //! [`name 1`]:  https://docs.rs/name-1/293-fdsf/name_1/index.html\n";
            assert!(HTTP_LINK.is_match(string));

            let string = "/// [name 1]: https://docs.rs/name_2-3/1.2.3/name_2_3/struct.Test.html\n";
            assert!(HTTP_LINK.is_match(string));

            let string =
                "/// [name 1]: https://docs.rs/name_2-3/1.2.3/name_2_3/module/struct.Test.html\n";
            assert!(HTTP_LINK.is_match(string));

            let string = "[`name 1`]:  https://docs.rs/name-1/293-fdsf/name_1/index.html\n";
            assert!(HTTP_LINK.is_match(string));

            let string = "[name 1]: https://docs.rs/name_2-3/1.2.3/name_2_3/struct.Test.html\n";
            assert!(HTTP_LINK.is_match(string));

            let string =
                "[name 1]: https://docs.rs/name_2-3/1.2.3/name_2_3/module/struct.Test.html\n";
            assert!(HTTP_LINK.is_match(string));
        }
    }

    #[test]
    fn http_link() {
        let string = "   //! [`name 1`]:  http://\n";
        assert!(HTTP_LINK.is_match(string));

        let string = "/// [name 1]: https://\n";
        assert!(HTTP_LINK.is_match(string));

        let string = "//! [name 1]:   http://actual-link.com\n";
        assert!(HTTP_LINK.is_match(string));

        let string = "/// [name 1]: https://actual-link.com\n";
        assert!(HTTP_LINK.is_match(string));

        let string = "[`name 1`]:  http://\n";
        assert!(HTTP_LINK.is_match(string));

        let string = "[name 1]: https://\n";
        assert!(HTTP_LINK.is_match(string));

        let string = "[name 1]:   http://actual-link.com\n";
        assert!(HTTP_LINK.is_match(string));

        let string = "[name 1]: https://actual-link.com\n";
        assert!(HTTP_LINK.is_match(string));

        // HTTP_LINK is voluntarily very conservative in what is a link to
        // avoid missing valid links. It is better not to break an existing
        // and working link than to try and fail when replacing it or worse,
        // transforming it but making it point to something else silently.
        let string = "//! [name 1]:   http://not-An€actual-link\n";
        assert!(HTTP_LINK.is_match(string));

        let string = "/// [name 1]: https://not-An€actual-link\n";
        assert!(HTTP_LINK.is_match(string));

        let string = "[name 1]:   http://not-An€actual-link\n";
        assert!(HTTP_LINK.is_match(string));

        let string = "[name 1]: https://not-An€actual-link\n";
        assert!(HTTP_LINK.is_match(string));
    }

    #[test]
    fn markdown_link_start() {
        let string = "   //! [`name 1`]:  http://\n";
        assert!(MARKDOWN_LINK_START.is_match(string));

        let string = "/// [name 1]: https://\n";
        assert!(MARKDOWN_LINK_START.is_match(string));

        let string = "//! [name 1]:   http://actual-link.com\n";
        assert!(MARKDOWN_LINK_START.is_match(string));

        let string = "/// [name 1]: https://actual-link.com\n";
        assert!(MARKDOWN_LINK_START.is_match(string));

        let string = "[`name 1`]:  http://\n";
        assert!(MARKDOWN_LINK_START.is_match(string));

        let string = "[name 1]: https://\n";
        assert!(MARKDOWN_LINK_START.is_match(string));

        let string = "[name 1]:   http://actual-link.com\n";
        assert!(MARKDOWN_LINK_START.is_match(string));

        let string = "[name 1]: https://actual-link.com\n";
        assert!(MARKDOWN_LINK_START.is_match(string));

        let string = "//! [name 1]:   http://not-An€actual-link\n";
        assert!(MARKDOWN_LINK_START.is_match(string));

        let string = "   //! [`name 1`]: struct.String.html\n";
        assert!(MARKDOWN_LINK_START.is_match(string));

        let string = "/// [name 1]: String\n";
        assert!(MARKDOWN_LINK_START.is_match(string));

        let string = "//! [name 1]:   ./struct.Type.html\n";
        assert!(MARKDOWN_LINK_START.is_match(string));

        let string = "/// [name 1]: ./../../mod1/mod2/a.b.c#section\n";
        assert!(MARKDOWN_LINK_START.is_match(string));
    }

    #[test]
    fn local_path() {
        fn check_captures(string: &str) {
            let captures = LOCAL_PATH.captures(string).unwrap();
            assert_eq!(
                "name 1",
                captures.name("elem").unwrap().as_str(),
                "{}",
                string
            );
            assert_eq!(
                "item",
                captures.name("elem2").unwrap().as_str(),
                "{}",
                string
            );
        }

        let string = "   //! [`name 1`]: item()\n";
        assert!(LOCAL_PATH.is_match(string));
        check_captures(string);

        let string = "   //! [`name 1`]: item!\n";
        assert!(LOCAL_PATH.is_match(string));
        check_captures(string);

        let string = "   /// [name 1]: item\n";
        assert!(LOCAL_PATH.is_match(string));
        check_captures(string);

        let string = "   [`name 1`]: item()\n";
        assert!(LOCAL_PATH.is_match(string));
        check_captures(string);

        let string = "[`name 1`]: item!\n";
        assert!(LOCAL_PATH.is_match(string));
        check_captures(string);

        let string = "[name 1]: item\n";
        assert!(LOCAL_PATH.is_match(string));
        check_captures(string);

        for item in ITEM_START_MARKERS {
            let string = &format!("//! [`name 1`]: {}@item()\n", item);
            assert!(LOCAL_PATH.is_match(string));
            check_captures(string);

            let string = &format!("/// [`name 1`]: {}@item!\n", item);
            assert!(LOCAL_PATH.is_match(string));
            check_captures(string);

            let string = &format!("/// [name 1]: {}@item\n", item);
            assert!(LOCAL_PATH.is_match(string));
            check_captures(string);

            let string = &format!("[`name 1`]: {}@item()\n", item);
            assert!(LOCAL_PATH.is_match(string));
            check_captures(string);

            let string = &format!("[`name 1`]: {}@item!\n", item);
            assert!(LOCAL_PATH.is_match(string));
            check_captures(string);

            let string = &format!("[name 1]: {}@item\n", item);
            assert!(LOCAL_PATH.is_match(string));
            check_captures(string);
        }
    }

    #[test]
    fn method_anchor() {
        for item in ITEM_TYPES {
            let string = &format!(" //! [`link`]: #{}.added\n", item);
            assert!(METHOD_ANCHOR.is_match(string));
            let captures = METHOD_ANCHOR.captures(string).unwrap();
            assert_eq!(
                " //! [`link`]: ",
                captures.name("link_name").unwrap().as_str()
            );
            assert_eq!("added", captures.name("additional").unwrap().as_str());

            let string = &format!("/// [link]: #{}.added\n", item);
            assert!(METHOD_ANCHOR.is_match(string));
            let captures = METHOD_ANCHOR.captures(string).unwrap();
            assert_eq!("/// [link]: ", captures.name("link_name").unwrap().as_str());
            assert_eq!("added", captures.name("additional").unwrap().as_str());

            let string = &format!(" [`link`]: #{}.added\n", item);
            assert!(METHOD_ANCHOR.is_match(string));
            let captures = METHOD_ANCHOR.captures(string).unwrap();
            assert_eq!(" [`link`]: ", captures.name("link_name").unwrap().as_str());
            assert_eq!("added", captures.name("additional").unwrap().as_str());

            let string = &format!("[link]: #{}.added\n", item);
            assert!(METHOD_ANCHOR.is_match(string));
            let captures = METHOD_ANCHOR.captures(string).unwrap();
            assert_eq!("[link]: ", captures.name("link_name").unwrap().as_str());
            assert_eq!("added", captures.name("additional").unwrap().as_str());
        }
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
                    assert!(TYPE_BLOCK_START.is_match(string), "{}", string);

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
                        assert!(TYPE_BLOCK_START.is_match(string), "{}", string);

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
                    assert!(TYPE_BLOCK_START.is_match(string), "{}", string);

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
}

#[test]
fn new() {
    let ctx = Context {
        krate: "name".into(),
        disambiguate: false,
        apply_favored: true,
        pos: 0,
        curr_type_block: None,
        end_type_block: String::new(),
        type_block_line: usize::MAX,
        type_blocks: Vec::new(),
    };

    assert_eq!(Context::new("name".into(), false, true), ctx);

    assert_ne!(Context::new("name".into(), true, true), ctx);
    assert_ne!(Context::new("name".into(), true, false), ctx);
    assert_ne!(Context::new("name".into(), false, false), ctx);

    assert_ne!(Context::new("not_name".into(), true, true), ctx);
    assert_ne!(Context::new("not_name".into(), true, false), ctx);
    assert_ne!(Context::new("not_name".into(), false, true), ctx);
    assert_ne!(Context::new("not_name".into(), false, false), ctx);
}

mod find_type_blocks {
    use super::*;

    #[test]
    fn empty_iter() {
        assert!(find_type_blocks(Vec::<String>::new().into_iter()).is_empty());
    }

    #[test]
    fn no_type_blocks() {
        let no_type_block_lines = vec![
            "let a = b;\n",
            "if a == b { let c = Type { toto: titi }; }\n",
            "/// struct X;\n",
            "//! struct X;\n",
            "// struct X;\n",
            "  // trait T {}\n",
            "\n",
            "'\n'.into()\n",
        ];

        assert!(find_type_blocks(no_type_block_lines.into_iter()).is_empty());
    }

    // This test is VERY long. It checks (I think) all possible combinations
    // and even some that aren't possible.
    //
    // Other tests will no be as thourough: they assume that if the combination
    // they test works, the other will, by virtue of this test.
    #[test]
    fn all_type_block_combinations() {
        use std::iter::once;

        let type_decls = ["struct", "trait", "enum", "union"];

        let visi_decls = [
            "pub",
            "pub(crate)",
            "pub(self)",
            "pub(super)",
            "pub(a)",
            "pub(b::a)",
        ];

        let generics = ["<A>", "<A, B>", "<A: Trait, const B: usize>"];

        let long_generics = [
            "where A: Trait",
            "where A: B + Sized",
            "where A: ?Sized",
            "where A: !Unpin",
        ];

        let with_ending = [("Type".into(), '\n'.into(), 1)];
        let with_bracket = [("Type".into(), '}'.into(), 1)];
        let with_parenthese = [("Type".into(), ')'.into(), 1)];

        let string = "impl Type {}\n";
        assert_eq!(find_type_blocks(once(string)), with_ending);

        let string = "impl Trait for Type {}\n";
        assert_eq!(find_type_blocks(once(string)), with_ending);

        for g in &generics {
            let string = format!("impl{gen} Type {{}}\n", gen = g);
            assert_eq!(find_type_blocks(once(string)), with_ending);

            let string = format!("impl{gen} Type {{\n", gen = g);
            assert_eq!(find_type_blocks(once(string)), with_bracket);

            let string = format!("impl{gen} Type{gen} {{}}\n", gen = g);
            assert_eq!(find_type_blocks(once(string)), with_ending);

            let string = format!("impl{gen} Type{gen} {{\n", gen = g);
            assert_eq!(find_type_blocks(once(string)), with_bracket);

            let string = format!("impl{gen} Trait for Type{gen} {{}}\n", gen = g);
            assert_eq!(find_type_blocks(once(string)), with_ending);

            let string = format!("impl{gen} Trait for Type{gen} {{\n", gen = g);
            assert_eq!(find_type_blocks(once(string)), with_bracket);

            let string = format!("impl{gen} Trait{gen} for Type{gen} {{}}\n", gen = g);
            assert_eq!(find_type_blocks(once(string)), with_ending);

            let string = format!("impl{gen} Trait{gen} for Type{gen} {{\n", gen = g);
            assert_eq!(find_type_blocks(once(string)), with_bracket);

            for lg in &long_generics {
                let string = format!("impl{gen} Type {long_gen} {{}}\n", gen = g, long_gen = lg);
                assert_eq!(find_type_blocks(once(string)), with_ending);

                let string = format!("impl{gen} Type {long_gen} {{\n", gen = g, long_gen = lg);
                assert_eq!(find_type_blocks(once(string)), with_bracket);

                let string = format!(
                    "impl{gen} Type{gen} {long_gen} {{}}\n",
                    gen = g,
                    long_gen = lg
                );
                assert_eq!(find_type_blocks(once(string)), with_ending);

                let string = format!(
                    "impl{gen} Type{gen} {long_gen} {{\n",
                    gen = g,
                    long_gen = lg
                );
                assert_eq!(find_type_blocks(once(string)), with_bracket);

                let string = format!(
                    "impl{gen} Trait for Type{gen} {long_gen} {{}}\n",
                    gen = g,
                    long_gen = lg
                );
                assert_eq!(find_type_blocks(once(string)), with_ending);

                let string = format!(
                    "impl{gen} Trait for Type{gen} {long_gen} {{\n",
                    gen = g,
                    long_gen = lg
                );
                assert_eq!(find_type_blocks(once(string)), with_bracket);

                let string = format!(
                    "impl{gen} Trait{gen} for Type{gen} {long_gen} {{}}\n",
                    gen = g,
                    long_gen = lg
                );
                assert_eq!(find_type_blocks(once(string)), with_ending);

                let string = format!(
                    "impl{gen} Trait{gen} for Type{gen} {long_gen} {{\n",
                    gen = g,
                    long_gen = lg
                );
                assert_eq!(find_type_blocks(once(string)), with_bracket);
            }
        }

        for t in &type_decls {
            // Testing with only the type declaration.
            let string = format!("{type_decl} Type;\n", type_decl = t);
            assert_eq!(find_type_blocks(once(string)), with_ending);

            let string = format!("{type_decl} Type();\n", type_decl = t);
            assert_eq!(find_type_blocks(once(string)), with_ending);

            let string = format!("{type_decl} Type{{}}\n", type_decl = t);
            assert_eq!(find_type_blocks(once(string)), with_ending);

            let string = format!("{type_decl} Type(\n", type_decl = t);
            assert_eq!(find_type_blocks(once(string)), with_parenthese);

            let string = format!("{type_decl} Type{{\n", type_decl = t);
            assert_eq!(find_type_blocks(once(string)), with_bracket);

            for v in &visi_decls {
                // Adding the visibility.
                let string = format!("{vis} {type_decl} Type;\n", type_decl = t, vis = v);
                assert_eq!(find_type_blocks(once(string)), with_ending);

                let string = format!("{vis} {type_decl} Type();\n", type_decl = t, vis = v);
                assert_eq!(find_type_blocks(once(string)), with_ending);

                let string = format!("{vis} {type_decl} Type{{}}\n", type_decl = t, vis = v);
                assert_eq!(find_type_blocks(once(string)), with_ending);

                let string = format!("{vis} {type_decl} Type(\n", type_decl = t, vis = v);
                assert_eq!(find_type_blocks(once(string)), with_parenthese);

                let string = format!("{vis} {type_decl} Type{{\n", type_decl = t, vis = v);
                assert_eq!(find_type_blocks(once(string)), with_bracket);

                for g in &generics {
                    // Adding the visibility.
                    let string = format!(
                        "{vis} {type_decl} Type{gen};\n",
                        type_decl = t,
                        vis = v,
                        gen = g
                    );
                    assert_eq!(find_type_blocks(once(string)), with_ending);

                    let string = format!(
                        "{vis} {type_decl} Type{gen}();\n",
                        type_decl = t,
                        vis = v,
                        gen = g
                    );
                    assert_eq!(find_type_blocks(once(string)), with_ending);

                    let string = format!(
                        "{vis} {type_decl} Type{gen}{{}}\n",
                        type_decl = t,
                        vis = v,
                        gen = g
                    );
                    assert_eq!(find_type_blocks(once(string)), with_ending);

                    let string = format!(
                        "{vis} {type_decl} Type{gen}{{\n",
                        type_decl = t,
                        vis = v,
                        gen = g
                    );
                    assert_eq!(find_type_blocks(once(string)), with_bracket);

                    let string = format!("{type_decl} Type{gen};\n", type_decl = t, gen = g);
                    assert_eq!(find_type_blocks(once(string)), with_ending);

                    let string = format!("{type_decl} Type{gen}();\n", type_decl = t, gen = g);
                    assert_eq!(find_type_blocks(once(string)), with_ending);

                    let string = format!("{type_decl} Type{gen}{{}}\n", type_decl = t, gen = g);
                    assert_eq!(find_type_blocks(once(string)), with_ending);

                    let string = format!("{type_decl} Type{gen}{{\n", type_decl = t, gen = g);
                    assert_eq!(find_type_blocks(once(string)), with_bracket);

                    for lg in &long_generics {
                        // Adding the possible endings.
                        let string = format!(
                            "{vis} {type_decl} Type{gen}() {long_gen};\n",
                            type_decl = t,
                            vis = v,
                            gen = g,
                            long_gen = lg
                        );
                        assert_eq!(find_type_blocks(once(string)), with_ending);

                        let string = format!(
                            "{vis} {type_decl} Type{gen} {long_gen} {{}}\n",
                            type_decl = t,
                            vis = v,
                            gen = g,
                            long_gen = lg
                        );
                        assert_eq!(find_type_blocks(once(string)), with_ending);

                        let string = format!(
                            "{vis} {type_decl} Type{gen} {long_gen} {{\n",
                            type_decl = t,
                            vis = v,
                            gen = g,
                            long_gen = lg
                        );
                        assert_eq!(find_type_blocks(once(string)), with_bracket);

                        let string = format!(
                            "{type_decl} Type{gen}() {long_gen};\n",
                            type_decl = t,
                            gen = g,
                            long_gen = lg
                        );
                        assert_eq!(find_type_blocks(once(string)), with_ending);

                        let string = format!(
                            "{type_decl} Type{gen} {long_gen} {{}}\n",
                            type_decl = t,
                            gen = g,
                            long_gen = lg
                        );
                        assert_eq!(find_type_blocks(once(string)), with_ending);

                        let string = format!(
                            "{type_decl} Type{gen} {long_gen} {{\n",
                            type_decl = t,
                            gen = g,
                            long_gen = lg
                        );
                        assert_eq!(find_type_blocks(once(string)), with_bracket);
                    }
                }
            }
        }
    }

    #[test]
    fn one_type_block_amongst_other_lines() {
        let lines = ["let a = b;\n", "struct A();\n", "// Comment\n"];

        assert_eq!(
            find_type_blocks(lines.iter()),
            [("A".into(), '\n'.into(), 2)]
        );
    }

    #[test]
    fn several_type_block_amongst_other_lines() {
        let lines = [
            "let a = b;\n",
            "struct A();\n",
            "// Comment\n",
            "struct B();\n",
        ];

        assert_eq!(
            find_type_blocks(lines.iter()),
            [("B".into(), '\n'.into(), 4), ("A".into(), '\n'.into(), 2)]
        );
    }
}

#[test]
fn item_type_markers() {
    let marked_items = [
        "struct",
        "enum",
        "trait",
        "union",
        "type",
        "const",
        "static",
        "value",
        "derive",
        "attr",
        "primitive",
        "mod",
        "fn",
        "method",
        "macro",
    ];

    assert_eq!(("type@", ""), super::item_type_markers("struct"));
    assert_eq!(("type@", ""), super::item_type_markers("enum"));
    assert_eq!(("type@", ""), super::item_type_markers("trait"));
    assert_eq!(("type@", ""), super::item_type_markers("union"));
    assert_eq!(("type@", ""), super::item_type_markers("type"));

    assert_eq!(("value@", ""), super::item_type_markers("const"));
    assert_eq!(("value@", ""), super::item_type_markers("static"));
    assert_eq!(("value@", ""), super::item_type_markers("value"));

    assert_eq!(("macro@", ""), super::item_type_markers("derive"));
    assert_eq!(("macro@", ""), super::item_type_markers("attr"));

    assert_eq!(("prim@", ""), super::item_type_markers("primitive"));

    assert_eq!(("mod@", ""), super::item_type_markers("mod"));

    assert_eq!(("", "()"), super::item_type_markers("fn"));
    assert_eq!(("", "()"), super::item_type_markers("method"));

    assert_eq!(("", "!"), super::item_type_markers("macro"));

    for item in ITEM_TYPES {
        if marked_items.contains(item) {
            continue;
        }

        assert_eq!(("", ""), super::item_type_markers(item));
    }
}

mod transform_item {
    use super::*;

    #[test]
    fn non_item() {
        let non_item_lines = [
            "let a = b;\n",
            "if a == b { let c = Type { toto: titi }; }\n",
            "/// struct X;\n",
            "//! struct X;\n",
            "// struct X;\n",
            "  // trait T {}\n",
            "\n",
            "'\n'.into()\n",
            "struct A(());\n",
            "/// [link]: https://toto.com\n",
        ];

        let ctx = Context::new("std".into(), true, false);
        for &line in &non_item_lines {
            assert_eq!(line, ctx.transform_item(line.into()));
        }

        let ctx = Context::new("std".into(), false, false);
        for &line in &non_item_lines {
            assert_eq!(line, ctx.transform_item(line.into()));
        }
    }

    #[test]
    fn matching_items() {
        let ctx = Context::new("my_crate".into(), true, false);

        let indentations = ["", "  ", "    "];
        let bangs = ["/", "!"];

        for it in ITEM_TYPES {
            let (start, end) = super::super::item_type_markers(it);
            for id in &indentations {
                for b in &bangs {
                    let link = format!(
                        "{ind}//{bang} [`Item`]: {item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}Item{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ./{item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}Item{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../{item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}super::Item{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ./../{item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}super::Item{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../{item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}super::super::Item{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../mod1/mod2/{item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}super::super::mod1::mod2::Item{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../my_crate/{item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}crate::Item{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../my_crate/mod1/mod2/{item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}crate::mod1::mod2::Item{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    // Testing links with a sub-item (e.g a method) at the end.

                    let link = format!(
                        "{ind}//{bang} [`Item`]: struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ./struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}super::Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ./../struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}super::Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}super::super::Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../mod1/mod2/struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}super::super::mod1::mod2::Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../my_crate/struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}crate::Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../my_crate/mod1/mod2/struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}crate::mod1::mod2::Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));
                }
            }
        }
    }

    #[test]
    fn matching_items_no_disambiguate() {
        let ctx = Context::new("my_crate".into(), false, false);

        let indentations = ["", "  ", "    "];
        let bangs = ["/", "!"];

        for it in ITEM_TYPES {
            let (_, end) = super::super::item_type_markers(it);
            for id in &indentations {
                for b in &bangs {
                    let link = format!(
                        "{ind}//{bang} [`Item`]: {item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: Item{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ./{item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: Item{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../{item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: super::Item{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ./../{item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: super::Item{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../{item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: super::super::Item{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../mod1/mod2/{item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: super::super::mod1::mod2::Item{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../my_crate/{item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: crate::Item{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../my_crate/mod1/mod2/{item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: crate::mod1::mod2::Item{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    // Testing links with a sub-item (e.g a method) at the end.

                    let link = format!(
                        "{ind}//{bang} [`Item`]: struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ./struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: super::Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ./../struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: super::Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: super::super::Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../mod1/mod2/struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: super::super::mod1::mod2::Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../my_crate/struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: crate::Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../my_crate/mod1/mod2/struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: crate::mod1::mod2::Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_item(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../my_crate/mod1/mod2/struct.Item.html#usage\n",
                        ind = id,
                        bang = b,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: crate::mod1::mod2::Item#usage\n",
                        ind = id,
                        bang = b,
                    );
                    assert_eq!(exp, ctx.transform_item(link));
                }
            }
        }
    }
}

mod transform_module {
    use super::*;

    #[test]
    fn non_module() {
        let non_module_lines = [
            "let a = b;\n",
            "if a == b { let c = Type { toto: titi }; }\n",
            "/// struct X;\n",
            "//! struct X;\n",
            "// struct X;\n",
            "  // trait T {}\n",
            "\n",
            "'\n'.into()\n",
            "struct A(());\n",
            "/// [link]: https://toto.com\n",
        ];

        let ctx = Context::new("std".into(), true, false);
        for &line in &non_module_lines {
            assert_eq!(line, ctx.transform_module(line.into()));
        }

        let ctx = Context::new("std".into(), false, false);
        for &line in &non_module_lines {
            assert_eq!(line, ctx.transform_module(line.into()));
        }
    }

    #[test]
    fn matching_modules() {
        let ctx = Context::new("my_crate".into(), true, false);

        let indentations = ["", "  ", "    "];
        let bangs = ["/", "!"];

        for i in &indentations {
            for b in &bangs {
                let line = format!("{ind}//{bang} [mod link]: index.html\n", ind = i, bang = b);
                let exp = format!("{ind}//{bang} [mod link]: mod@self\n", ind = i, bang = b);
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod@self#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ../index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!("{ind}//{bang} [mod link]: mod@super\n", ind = i, bang = b);
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ../index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod@super#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: my_crate/index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!("{ind}//{bang} [mod link]: mod@crate\n", ind = i, bang = b);
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: my_crate/index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod@crate#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ../my_crate/index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!("{ind}//{bang} [mod link]: mod@crate\n", ind = i, bang = b);
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ../my_crate/index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod@crate#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!("{ind}//{bang} [mod link]: mod@self\n", ind = i, bang = b);
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod@self#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./../index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod@super#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./my_crate/index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod@crate#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: mod1/mod2/index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod@mod1::mod2\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./mod1/mod2/index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod@mod1::mod2#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ../mod1/mod2/index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod@super::mod1::mod2\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./../mod1/mod2/index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod@super::mod1::mod2#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: my_crate/mod1/mod2/index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod@crate::mod1::mod2\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./my_crate/mod1/mod2/index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod@crate::mod1::mod2#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_module(line));
            }
        }
    }

    #[test]
    fn matching_modules_no_disambiguate() {
        let ctx = Context::new("my_crate".into(), false, false);

        let indentations = ["", "  ", "    "];
        let bangs = ["/", "!"];

        for i in &indentations {
            for b in &bangs {
                let line = format!("{ind}//{bang} [mod link]: index.html\n", ind = i, bang = b);
                let exp = format!("{ind}//{bang} [mod link]: self\n", ind = i, bang = b);
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: self#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ../index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!("{ind}//{bang} [mod link]: super\n", ind = i, bang = b);
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ../index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: super#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: my_crate/index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!("{ind}//{bang} [mod link]: crate\n", ind = i, bang = b);
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: my_crate/index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: crate#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ../my_crate/index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!("{ind}//{bang} [mod link]: crate\n", ind = i, bang = b);
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ../my_crate/index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: crate#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!("{ind}//{bang} [mod link]: self\n", ind = i, bang = b);
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: self#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./../index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: super#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./my_crate/index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: crate#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: mod1/mod2/index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!("{ind}//{bang} [mod link]: mod1::mod2\n", ind = i, bang = b);
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./mod1/mod2/index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod1::mod2#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ../mod1/mod2/index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: super::mod1::mod2\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./../mod1/mod2/index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: super::mod1::mod2#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: my_crate/mod1/mod2/index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: crate::mod1::mod2\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_module(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./my_crate/mod1/mod2/index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: crate::mod1::mod2#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_module(line));
            }
        }
    }
}

mod transform_local {
    use super::*;

    #[test]
    fn non_local() {
        let non_local_lines = [
            "let a = b;\n",
            "if a == b { let c = Type { toto: titi }; }\n",
            "/// struct X;\n",
            "//! struct X;\n",
            "// struct X;\n",
            "  // trait T {}\n",
            "\n",
            "'\n'.into()\n",
            "struct A(());\n",
            "/// [link]: https://toto.com\n",
            "/// [non local link]: Link\n",
            "/// [Link]: super::Link\n",
        ];

        for &line in &non_local_lines {
            assert_eq!(line, transform_local(line.into()));
        }
    }

    #[test]
    fn matching_local_links() {
        let indentations = ["", "  ", "    "];
        let bangs = ["/", "!"];

        for item in ITEM_TYPES {
            let (start, end) = item_type_markers(item);

            for i in &indentations {
                for b in &bangs {
                    let line = format!("{ind}//{bang} [link]: link\n", ind = i, bang = b);
                    assert_eq!("", transform_local(line));

                    let line = format!(
                        "{ind}//{bang} [super::Type]: super::Type\n",
                        ind = i,
                        bang = b
                    );
                    assert_eq!("", transform_local(line));

                    let line = format!(
                        "{ind}//{bang} [link]: {s}link{e}\n",
                        ind = i,
                        bang = b,
                        s = start,
                        e = end
                    );
                    assert_eq!(
                        if start != "" || end != "" { &line } else { "" },
                        transform_local(line.clone())
                    );

                    let line = format!(
                        "{ind}//{bang} [super::Type]: {s}super::Type{e}\n",
                        ind = i,
                        bang = b,
                        s = start,
                        e = end
                    );
                    assert_eq!(
                        if start != "" || end != "" { &line } else { "" },
                        transform_local(line.clone())
                    );

                    let line = format!("{ind}//{bang} [`link`]: link\n", ind = i, bang = b);
                    assert_eq!("", transform_local(line));

                    let line = format!(
                        "{ind}//{bang} [`super::Type`]: super::Type\n",
                        ind = i,
                        bang = b
                    );
                    assert_eq!("", transform_local(line));

                    let line = format!(
                        "{ind}//{bang} [`link`]: {s}link{e}\n",
                        ind = i,
                        bang = b,
                        s = start,
                        e = end
                    );
                    assert_eq!(
                        if start != "" || end != "" { &line } else { "" },
                        transform_local(line.clone())
                    );

                    let line = format!(
                        "{ind}//{bang} [`super::Type`]: {s}super::Type{e}\n",
                        ind = i,
                        bang = b,
                        s = start,
                        e = end
                    );
                    assert_eq!(
                        if start != "" || end != "" { &line } else { "" },
                        transform_local(line.clone())
                    );
                }
            }
        }
    }
}

mod transform_anchor {
    use super::*;

    #[test]
    fn non_anchor() {
        let non_anchor_lines = [
            "let a = b;\n",
            "if a == b { let c = Type { toto: titi }; }\n",
            "/// struct X;\n",
            "//! struct X;\n",
            "// struct X;\n",
            "  // trait T {}\n",
            "\n",
            "'\n'.into()\n",
            "struct A(());\n",
            "/// [link]: https://toto.com\n",
            "/// [non local link]: Link\n",
            "/// [Link]: super::Link\n",
        ];

        let ctx = Context::new("my_crate".into(), true, false);
        for &line in &non_anchor_lines {
            assert_eq!(line, ctx.transform_anchor(line.into()));
        }

        let ctx = Context::new("my_crate".into(), false, false);
        for &line in &non_anchor_lines {
            assert_eq!(line, ctx.transform_anchor(line.into()));
        }
    }

    #[test]
    fn matching_anchors() {
        let none_ctx = Context::new("my_crate_none".into(), true, false);
        let mut some_ctx = Context::new("my_crate_some".into(), true, false);

        some_ctx.curr_type_block = Some("Type".into());
        some_ctx.end_type_block = "}".into();
        some_ctx.type_block_line = 1;

        let indentations = ["", "  ", "    "];
        let bangs = ["/", "!"];

        for item in ITEM_TYPES {
            let (start, end) = item_type_markers(item);

            for i in &indentations {
                for b in &bangs {
                    let line = format!(
                        "{id}//{bg} [method]: #{it}.name\n",
                        id = i,
                        bg = b,
                        it = item
                    );
                    let exp = format!(
                        "{id}//{bg} [method]: {s}Type::name{e}\n",
                        id = i,
                        bg = b,
                        s = start,
                        e = end
                    );
                    assert_eq!(line.clone(), none_ctx.transform_anchor(line.clone()));
                    assert_eq!(exp, some_ctx.transform_anchor(line));

                    let line = format!(
                        "{id}//{bg} [`method`]: #{it}.name\n",
                        id = i,
                        bg = b,
                        it = item
                    );
                    let exp = format!(
                        "{id}//{bg} [`method`]: {s}Type::name{e}\n",
                        id = i,
                        bg = b,
                        s = start,
                        e = end
                    );
                    assert_eq!(line.clone(), none_ctx.transform_anchor(line.clone()));
                    assert_eq!(exp, some_ctx.transform_anchor(line));
                }
            }
        }
    }

    #[test]
    fn matching_anchors_no_disambiguate() {
        let none_ctx = Context::new("my_crate_none".into(), false, false);
        let mut some_ctx = Context::new("my_crate_some".into(), false, false);

        some_ctx.curr_type_block = Some("Type".into());
        some_ctx.end_type_block = "}".into();
        some_ctx.type_block_line = 1;

        let indentations = ["", "  ", "    "];
        let bangs = ["/", "!"];

        for item in ITEM_TYPES {
            let (_, end) = item_type_markers(item);

            for i in &indentations {
                for b in &bangs {
                    let line = format!(
                        "{id}//{bg} [method]: #{it}.name\n",
                        id = i,
                        bg = b,
                        it = item
                    );
                    let exp = format!(
                        "{id}//{bg} [method]: Type::name{e}\n",
                        id = i,
                        bg = b,
                        e = end
                    );
                    assert_eq!(line.clone(), none_ctx.transform_anchor(line.clone()));
                    assert_eq!(exp, some_ctx.transform_anchor(line));

                    let line = format!(
                        "{id}//{bg} [`method`]: #{it}.name\n",
                        id = i,
                        bg = b,
                        it = item
                    );
                    let exp = format!(
                        "{id}//{bg} [`method`]: Type::name{e}\n",
                        id = i,
                        bg = b,
                        e = end
                    );
                    assert_eq!(line.clone(), none_ctx.transform_anchor(line.clone()));
                    assert_eq!(exp, some_ctx.transform_anchor(line));
                }
            }
        }
    }
}

mod transform_line {
    use super::*;

    #[test]
    fn non_line() {
        let non_line_lines = [
            "let a = b;\n",
            "if a == b { let c = Type { toto: titi }; }\n",
            "/// struct X;\n",
            "//! struct X;\n",
            "// struct X;\n",
            "  // trait T {}\n",
            "\n",
            "'\n'.into()\n",
            "struct A(());\n",
            "/// [link]: https://toto.com\n",
            "/// [non local link]: Link\n",
            "/// [Link]: super::Link\n",
        ];

        let mut ctx = Context::new("my_crate".into(), true, true);
        for &line in &non_line_lines {
            assert_eq!(line, ctx.transform_line(line.into()));
        }

        let mut ctx = Context::new("my_crate".into(), false, true);
        for &line in &non_line_lines {
            assert_eq!(line, ctx.transform_line(line.into()));
        }
    }

    #[test]
    fn type_block_none_to_some() {
        let mut ctx = Context::new("my_crate".into(), true, true);

        ctx.type_blocks = vec![("Type".into(), "}".into(), 1)];

        ctx.transform_line("".into());

        assert_eq!(Some("Type".into()), ctx.curr_type_block);
        assert_eq!("}", ctx.end_type_block);
        assert_eq!(1, ctx.type_block_line);
        assert!(ctx.type_blocks.is_empty());

        // No disambiguate
        let mut ctx = Context::new("my_crate".into(), false, true);

        ctx.type_blocks = vec![("Type".into(), "}".into(), 1)];

        ctx.transform_line("".into());

        assert_eq!(Some("Type".into()), ctx.curr_type_block);
        assert_eq!("}", ctx.end_type_block);
        assert_eq!(1, ctx.type_block_line);
        assert!(ctx.type_blocks.is_empty());
    }

    #[test]
    fn type_block_some_to_none() {
        let mut ctx = Context::new("my_crate".into(), true, true);

        ctx.pos = 2;

        ctx.curr_type_block = Some("Type".into());
        ctx.end_type_block = "}".into();
        ctx.type_block_line = 1;

        ctx.transform_line("}".into());

        assert_eq!(None, ctx.curr_type_block);
        assert_eq!("", ctx.end_type_block);
        assert_eq!(usize::MAX, ctx.type_block_line);

        // No disambiguate
        let mut ctx = Context::new("my_crate".into(), true, true);

        ctx.pos = 2;

        ctx.curr_type_block = Some("Type".into());
        ctx.end_type_block = "}".into();
        ctx.type_block_line = 1;

        ctx.transform_line("}".into());

        assert_eq!(None, ctx.curr_type_block);
        assert_eq!("", ctx.end_type_block);
        assert_eq!(usize::MAX, ctx.type_block_line);
    }

    #[test]
    fn matching_lines_anchor() {
        let mut none_ctx = Context::new("my_crate_none".into(), true, false);
        let mut some_ctx = Context::new("my_crate_some".into(), true, false);

        some_ctx.curr_type_block = Some("Type".into());
        some_ctx.end_type_block = "}".into();
        some_ctx.type_block_line = 1;

        let indentations = ["", "  ", "    "];
        let bangs = ["/", "!"];

        for item in ITEM_TYPES {
            let (start, end) = item_type_markers(item);

            for i in &indentations {
                for b in &bangs {
                    let line = format!(
                        "{id}//{bg} [method]: #{it}.name\n",
                        id = i,
                        bg = b,
                        it = item
                    );
                    let exp = format!(
                        "{id}//{bg} [method]: {s}Type::name{e}\n",
                        id = i,
                        bg = b,
                        s = start,
                        e = end
                    );
                    assert_eq!(line.clone(), none_ctx.transform_line(line.clone()));
                    assert_eq!(exp, some_ctx.transform_line(line));

                    let line = format!(
                        "{id}//{bg} [`method`]: #{it}.name\n",
                        id = i,
                        bg = b,
                        it = item
                    );
                    let exp = format!(
                        "{id}//{bg} [`method`]: {s}Type::name{e}\n",
                        id = i,
                        bg = b,
                        s = start,
                        e = end
                    );
                    assert_eq!(line.clone(), none_ctx.transform_line(line.clone()));
                    assert_eq!(exp, some_ctx.transform_line(line));
                }
            }
        }
    }

    #[test]
    fn matching_lines_anchor_no_disambiguate() {
        let mut none_ctx = Context::new("my_crate_none".into(), false, true);
        let mut some_ctx = Context::new("my_crate_some".into(), false, true);

        some_ctx.curr_type_block = Some("Type".into());
        some_ctx.end_type_block = "}".into();
        some_ctx.type_block_line = 1;

        let indentations = ["", "  ", "    "];
        let bangs = ["/", "!"];

        for item in ITEM_TYPES {
            let (_, end) = item_type_markers(item);

            for i in &indentations {
                for b in &bangs {
                    let line = format!(
                        "{id}//{bg} [method]: #{it}.name\n",
                        id = i,
                        bg = b,
                        it = item
                    );
                    let exp = format!(
                        "{id}//{bg} [method]: Type::name{e}\n",
                        id = i,
                        bg = b,
                        e = end
                    );
                    assert_eq!(line.clone(), none_ctx.transform_line(line.clone()));
                    assert_eq!(exp, some_ctx.transform_line(line));

                    let line = format!(
                        "{id}//{bg} [`method`]: #{it}.name\n",
                        id = i,
                        bg = b,
                        it = item
                    );
                    let exp = format!(
                        "{id}//{bg} [`method`]: Type::name{e}\n",
                        id = i,
                        bg = b,
                        e = end
                    );
                    assert_eq!(line.clone(), none_ctx.transform_line(line.clone()));
                    assert_eq!(exp, some_ctx.transform_line(line));
                }
            }
        }
    }

    #[test]
    fn matching_lines_local_links() {
        let mut ctx = Context::new("my_crate".into(), true, true);

        let indentations = ["", "  ", "    "];
        let bangs = ["/", "!"];

        for item in ITEM_TYPES {
            let (start, end) = item_type_markers(item);

            for i in &indentations {
                for b in &bangs {
                    let line = format!("{ind}//{bang} [link]: link\n", ind = i, bang = b);
                    assert!(ctx.transform_line(line).is_deleted());

                    let line = format!(
                        "{ind}//{bang} [super::Type]: super::Type\n",
                        ind = i,
                        bang = b
                    );
                    assert!(ctx.transform_line(line).is_deleted());

                    let line = format!(
                        "{ind}//{bang} [link]: {s}link{e}\n",
                        ind = i,
                        bang = b,
                        s = start,
                        e = end
                    );
                    assert_eq!(line, ctx.transform_line(line.clone()));

                    let line = format!(
                        "{ind}//{bang} [super::Type]: {s}super::Type{e}\n",
                        ind = i,
                        bang = b,
                        s = start,
                        e = end
                    );
                    assert_eq!(line, ctx.transform_line(line.clone()));

                    let line = format!("{ind}//{bang} [`link`]: link\n", ind = i, bang = b);
                    assert!(ctx.transform_line(line).is_deleted());

                    let line = format!(
                        "{ind}//{bang} [`super::Type`]: super::Type\n",
                        ind = i,
                        bang = b
                    );
                    assert!(ctx.transform_line(line).is_deleted());

                    let line = format!(
                        "{ind}//{bang} [`link`]: {s}link{e}\n",
                        ind = i,
                        bang = b,
                        s = start,
                        e = end
                    );
                    assert_eq!(line, ctx.transform_line(line.clone()));

                    let line = format!(
                        "{ind}//{bang} [`super::Type`]: {s}super::Type{e}\n",
                        ind = i,
                        bang = b,
                        s = start,
                        e = end
                    );
                    assert_eq!(line, ctx.transform_line(line.clone()));
                }
            }
        }
    }

    #[test]
    fn matching_lines_local_links_no_disambiguate() {
        let mut ctx = Context::new("my_crate".into(), false, true);

        let indentations = ["", "  ", "    "];
        let bangs = ["/", "!"];

        for item in ITEM_TYPES {
            let (_, end) = item_type_markers(item);

            for i in &indentations {
                for b in &bangs {
                    let line = format!("{ind}//{bang} [link]: link\n", ind = i, bang = b);
                    assert!(ctx.transform_line(line).is_deleted());

                    let line = format!(
                        "{ind}//{bang} [super::Type]: super::Type\n",
                        ind = i,
                        bang = b
                    );
                    assert!(ctx.transform_line(line).is_deleted());

                    let line = format!(
                        "{ind}//{bang} [link]: link{e}\n",
                        ind = i,
                        bang = b,
                        e = end
                    );
                    assert_eq!(line, ctx.transform_line(line.clone()));

                    let line = format!(
                        "{ind}//{bang} [super::Type]: super::Type{e}\n",
                        ind = i,
                        bang = b,
                        e = end
                    );
                    assert_eq!(line, ctx.transform_line(line.clone()));

                    let line = format!("{ind}//{bang} [`link`]: link\n", ind = i, bang = b);
                    assert!(ctx.transform_line(line).is_deleted());

                    let line = format!(
                        "{ind}//{bang} [`super::Type`]: super::Type\n",
                        ind = i,
                        bang = b
                    );
                    assert!(ctx.transform_line(line).is_deleted());

                    let line = format!(
                        "{ind}//{bang} [`link`]: link{e}\n",
                        ind = i,
                        bang = b,
                        e = end
                    );
                    assert_eq!(line, ctx.transform_line(line.clone()));

                    let line = format!(
                        "{ind}//{bang} [`super::Type`]: super::Type{e}\n",
                        ind = i,
                        bang = b,
                        e = end
                    );
                    assert_eq!(line, ctx.transform_line(line.clone()));
                }
            }
        }
    }

    #[test]
    fn matching_lines_modules() {
        let mut ctx = Context::new("my_crate".into(), true, true);

        let indentations = ["", "  ", "    "];
        let bangs = ["/", "!"];

        for i in &indentations {
            for b in &bangs {
                let line = format!("{ind}//{bang} [mod link]: index.html\n", ind = i, bang = b);
                let exp = format!("{ind}//{bang} [mod link]: mod@self\n", ind = i, bang = b);
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod@self#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ../index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!("{ind}//{bang} [mod link]: mod@super\n", ind = i, bang = b);
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ../index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod@super#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: my_crate/index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!("{ind}//{bang} [mod link]: mod@crate\n", ind = i, bang = b);
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: my_crate/index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod@crate#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ../my_crate/index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!("{ind}//{bang} [mod link]: mod@crate\n", ind = i, bang = b);
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ../my_crate/index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod@crate#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!("{ind}//{bang} [mod link]: mod@self\n", ind = i, bang = b);
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod@self#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./../index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod@super#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./my_crate/index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod@crate#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: mod1/mod2/index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod@mod1::mod2\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./mod1/mod2/index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod@mod1::mod2#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ../mod1/mod2/index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod@super::mod1::mod2\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./../mod1/mod2/index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod@super::mod1::mod2#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: my_crate/mod1/mod2/index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod@crate::mod1::mod2\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./my_crate/mod1/mod2/index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod@crate::mod1::mod2#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_line(line));
            }
        }
    }

    #[test]
    fn matching_lines_modules_no_disambiguate() {
        let mut ctx = Context::new("my_crate".into(), false, true);

        let indentations = ["", "  ", "    "];
        let bangs = ["/", "!"];

        for i in &indentations {
            for b in &bangs {
                let line = format!("{ind}//{bang} [mod link]: index.html\n", ind = i, bang = b);
                let exp = format!("{ind}//{bang} [mod link]: self\n", ind = i, bang = b);
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: self#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ../index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!("{ind}//{bang} [mod link]: super\n", ind = i, bang = b);
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ../index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: super#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: my_crate/index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!("{ind}//{bang} [mod link]: crate\n", ind = i, bang = b);
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: my_crate/index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: crate#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ../my_crate/index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!("{ind}//{bang} [mod link]: crate\n", ind = i, bang = b);
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ../my_crate/index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: crate#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!("{ind}//{bang} [mod link]: self\n", ind = i, bang = b);
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: self#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./../index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: super#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./my_crate/index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: crate#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: mod1/mod2/index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!("{ind}//{bang} [mod link]: mod1::mod2\n", ind = i, bang = b);
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./mod1/mod2/index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: mod1::mod2#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ../mod1/mod2/index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: super::mod1::mod2\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./../mod1/mod2/index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: super::mod1::mod2#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: my_crate/mod1/mod2/index.html\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: crate::mod1::mod2\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_line(line));

                let line = format!(
                    "{ind}//{bang} [mod link]: ./my_crate/mod1/mod2/index.html#section\n",
                    ind = i,
                    bang = b
                );
                let exp = format!(
                    "{ind}//{bang} [mod link]: crate::mod1::mod2#section\n",
                    ind = i,
                    bang = b
                );
                assert_eq!(exp, ctx.transform_line(line));
            }
        }
    }

    #[test]
    fn matching_lines_items() {
        let mut ctx = Context::new("my_crate".into(), true, true);

        let indentations = ["", "  ", "    "];
        let bangs = ["/", "!"];

        for it in ITEM_TYPES {
            let (start, end) = super::super::item_type_markers(it);

            for id in &indentations {
                for b in &bangs {
                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../{item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}super::Item{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ./../{item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}super::Item{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../{item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}super::super::Item{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../mod1/mod2/{item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}super::super::mod1::mod2::Item{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../my_crate/{item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}crate::Item{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../my_crate/mod1/mod2/{item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}crate::mod1::mod2::Item{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));

                    // Testing links with a sub-item (e.g a method) at the end.

                    let link = format!(
                        "{ind}//{bang} [`Item`]: struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ./struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}super::Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ./../struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}super::Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}super::super::Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../mod1/mod2/struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}super::super::mod1::mod2::Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../my_crate/struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}crate::Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../my_crate/mod1/mod2/struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: {start}crate::mod1::mod2::Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        start = start,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));
                }
            }
        }
    }

    #[test]
    fn matching_lines_items_no_disambiguate() {
        let mut ctx = Context::new("my_crate".into(), false, true);

        let indentations = ["", "  ", "    "];
        let bangs = ["/", "!"];

        for it in ITEM_TYPES {
            let (_, end) = super::super::item_type_markers(it);

            for id in &indentations {
                for b in &bangs {
                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../{item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: super::Item{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ./../{item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: super::Item{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../{item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: super::super::Item{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../mod1/mod2/{item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: super::super::mod1::mod2::Item{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../my_crate/{item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: crate::Item{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../my_crate/mod1/mod2/{item}.Item.html\n",
                        ind = id,
                        bang = b,
                        item = it
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: crate::mod1::mod2::Item{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));

                    // Testing links with a sub-item (e.g a method) at the end.

                    let link = format!(
                        "{ind}//{bang} [`Item`]: struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ./struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: super::Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ./../struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: super::Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: super::super::Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../mod1/mod2/struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: super::super::mod1::mod2::Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../my_crate/struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: crate::Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));

                    let link = format!(
                        "{ind}//{bang} [`Item`]: ../../my_crate/mod1/mod2/struct.Item.html#{add}.subitem\n",
                        ind = id,
                        bang = b,
                        add = it,
                    );
                    let exp = format!(
                        "{ind}//{bang} [`Item`]: crate::mod1::mod2::Item::subitem{end}\n",
                        ind = id,
                        bang = b,
                        end = end
                    );
                    assert_eq!(exp, ctx.transform_line(link));
                }
            }
        }
    }

    #[test]
    fn matching_favored_links_no_favored() {
        let mut ctx = Context::new("my_krate".into(), false, false);

        let line = "/// [link]: https://docs.rs/name/latest/name/index.html\n";
        assert_eq!(line, ctx.transform_line(line.into()));

        let line = "//! [link]: https://docs.rs/name/latest/name/mod/index.html\n";
        assert_eq!(line, ctx.transform_line(line.into()));

        let line = "//! [link]: https://docs.rs/name/latest/name/mod/index.html#section\n";
        assert_eq!(line, ctx.transform_line(line.into()));

        let line = "    /// [`link`]: https://docs.rs/name/latest/name/mod/struct.Type.html\n";
        assert_eq!(line, ctx.transform_line(line.into()));

        let line = "[link]: https://docs.rs/name/latest/name/index.html\n";
        assert_eq!(line, ctx.transform_line(line.into()));

        let line = "[link]: https://docs.rs/name/latest/name/mod/index.html\n";
        assert_eq!(line, ctx.transform_line(line.into()));

        let line = "[link]: https://docs.rs/name/latest/name/mod/index.html#section\n";
        assert_eq!(line, ctx.transform_line(line.into()));

        let line = "[`link`]: https://docs.rs/name/latest/name/mod/struct.Type.html\n";
        assert_eq!(line, ctx.transform_line(line.into()));
    }

    #[test]
    fn matching_favored_links_favored() {
        let mut ctx = Context::new("my_krate".into(), false, true);

        let line = "/// [link]: https://docs.rs/name/latest/name/index.html\n";
        let exp = "/// [link]: name\n";
        assert_eq!(exp, ctx.transform_line(line.into()));

        let line = "//! [link]: https://docs.rs/name/latest/name/mod/index.html\n";
        let exp = "//! [link]: name::mod\n";
        assert_eq!(exp, ctx.transform_line(line.into()));

        let line = "//! [link]: https://docs.rs/name/latest/name/mod/index.html#section\n";
        let exp = "//! [link]: name::mod#section\n";
        assert_eq!(exp, ctx.transform_line(line.into()));

        let line = "    /// [`link`]: https://docs.rs/name/latest/name/mod/struct.Type.html\n";
        let exp = "    /// [`link`]: name::mod::Type\n";
        assert_eq!(exp, ctx.transform_line(line.into()));

        let line = "[link]: https://docs.rs/name/latest/name/index.html\n";
        let exp = "[link]: name\n";
        assert_eq!(exp, ctx.transform_line(line.into()));

        let line = "[link]: https://docs.rs/name/latest/name/mod/index.html\n";
        let exp = "[link]: name::mod\n";
        assert_eq!(exp, ctx.transform_line(line.into()));

        let line = "[link]: https://docs.rs/name/latest/name/mod/index.html#section\n";
        let exp = "[link]: name::mod#section\n";
        assert_eq!(exp, ctx.transform_line(line.into()));

        let line = "[`link`]: https://docs.rs/name/latest/name/mod/struct.Type.html\n";
        let exp = "[`link`]: name::mod::Type\n";
        assert_eq!(exp, ctx.transform_line(line.into()));

        let line = "//! [link]: https://docs.rs/name/latest/name/mod/#section\n";
        let exp = "//! [link]: name::mod#section\n";
        assert_eq!(exp, ctx.transform_line(line.into()));

        let line = "//! [link]: https://docs.rs/name/latest/name/mod#section\n";
        let exp = "//! [link]: name::mod#section\n";
        assert_eq!(exp, ctx.transform_line(line.into()));

        let line = "/// [link]: https://doc.rust-lang.org/nightly/std/primitive.str.html\n";
        let exp = "/// [link]: std::str\n";
        assert_eq!(exp, ctx.transform_line(line.into()));

        let line = "/// [link]: https://doc.rust-lang.org/beta/std/primitive.str.html\n";
        let exp = "/// [link]: std::str\n";
        assert_eq!(exp, ctx.transform_line(line.into()));

        let line = "/// [link]: https://doc.rust-lang.org/stable/std/primitive.str.html\n";
        let exp = "/// [link]: std::str\n";
        assert_eq!(exp, ctx.transform_line(line.into()));

        let line = "/// [link]: https://doc.rust-lang.org/1.42.1/std/primitive.str.html\n";
        let exp = "/// [link]: std::str\n";
        assert_eq!(exp, ctx.transform_line(line.into()));

        let line = "/// [link]: https://doc.rust-lang.org/nightly/std/primitive.str.html#usage\n";
        let exp = "/// [link]: std::str#usage\n";
        assert_eq!(exp, ctx.transform_line(line.into()));

        let line = "/// [link]: https://doc.rust-lang.org/beta/std/primitive.str.html#usage\n";
        let exp = "/// [link]: std::str#usage\n";
        assert_eq!(exp, ctx.transform_line(line.into()));

        let line = "/// [link]: https://doc.rust-lang.org/stable/std/primitive.str.html#usage\n";
        let exp = "/// [link]: std::str#usage\n";
        assert_eq!(exp, ctx.transform_line(line.into()));

        let line = "/// [link]: https://doc.rust-lang.org/1.42.1/std/primitive.str.html#usage\n";
        let exp = "/// [link]: std::str#usage\n";
        assert_eq!(exp, ctx.transform_line(line.into()));

        let line = "/// [link]: https://doc.rust-lang.org/nightly/std/\n";
        let exp = "/// [link]: std\n";
        assert_eq!(exp, ctx.transform_line(line.into()));

        let line = "/// [link]: https://doc.rust-lang.org/beta/std/string/struct.String.html\n";
        let exp = "/// [link]: std::string::String\n";
        assert_eq!(exp, ctx.transform_line(line.into()));

        let line =
            "/// [link]: https://doc.rust-lang.org/beta/std/string/struct.String.html#section\n";
        let exp = "/// [link]: std::string::String#section\n";
        assert_eq!(exp, ctx.transform_line(line.into()));

        let line = "/// [link]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/enum.TyKind.html\n";
        let exp = "/// [link]: rustc_hir::TyKind\n";
        assert_eq!(exp, ctx.transform_line(line.into()));
    }
}

mod transform_favored_links {
    use super::*;

    #[test]
    fn non_favored() {
        let non_favored_links = [
            "let a = b;\n",
            "if a == b { let c = Type { toto: titi }; }\n",
            "/// struct X;\n",
            "//! struct X;\n",
            "// struct X;\n",
            "  // trait T {}\n",
            "\n",
            "'\n'.into()\n",
            "struct A(());\n",
            "/// [link]: https://toto.com\n",
            "/// [non local link]: Link\n",
            "/// [Link]: super::Link\n",
        ];

        for &line in &non_favored_links {
            assert_eq!(line, transform_favored_link(line.into()));
        }
    }

    #[test]
    fn matching_favored_links_docs_rs_long() {
        let line = "/// [link]: https://docs.rs/name/latest/name/index.html\n";
        let exp = "/// [link]: name/index.html\n";
        assert_eq!(exp, transform_favored_link(line.into()));

        let line = "//! [link]: https://docs.rs/name/latest/name/mod/index.html\n";
        let exp = "//! [link]: name/mod/index.html\n";
        assert_eq!(exp, transform_favored_link(line.into()));

        let line = "//! [link]: https://docs.rs/name/latest/name/mod/index.html#section\n";
        let exp = "//! [link]: name/mod/index.html#section\n";
        assert_eq!(exp, transform_favored_link(line.into()));

        let line = "//! [link]: https://docs.rs/name/latest/name/mod/#section\n";
        let exp = "//! [link]: name/mod/index.html#section\n";
        assert_eq!(exp, transform_favored_link(line.into()));

        let line = "//! [link]: https://docs.rs/name/latest/name/mod#section\n";
        let exp = "//! [link]: name/mod/index.html#section\n";
        assert_eq!(exp, transform_favored_link(line.into()));

        let line = "    /// [`link`]: https://docs.rs/name/latest/name/mod/struct.Type.html\n";
        let exp = "    /// [`link`]: name/mod/struct.Type.html\n";
        assert_eq!(exp, transform_favored_link(line.into()));

        let line = "[link]: https://docs.rs/name_A/latest/name_A/index.html\n";
        let exp = "[link]: name_A/index.html\n";
        assert_eq!(exp, transform_favored_link(line.into()));

        let line = "[link]: https://docs.rs/name-A/latest/name_A/mod/index.html\n";
        let exp = "[link]: name_A/mod/index.html\n";
        assert_eq!(exp, transform_favored_link(line.into()));

        let line = "[link]: https://docs.rs/name_1/latest/name_1/mod/index.html#section\n";
        let exp = "[link]: name_1/mod/index.html#section\n";
        assert_eq!(exp, transform_favored_link(line.into()));

        let line = "[`link`]: https://docs.rs/name-1/latest/name_1/mod/struct.Type.html\n";
        let exp = "[`link`]: name_1/mod/struct.Type.html\n";
        assert_eq!(exp, transform_favored_link(line.into()));
    }

    #[test]
    fn matching_favored_links_docs_rs_short() {
        let line = "/// [link]: https://docs.rs/name_A\n";
        let exp = "/// [link]: name_A\n";
        assert_eq!(exp, transform_favored_link(line.into()));

        let line = "    //! [link]: https://docs.rs/name-A/\n";
        let exp = "    //! [link]: name_A\n";
        assert_eq!(exp, transform_favored_link(line.into()));

        let line = "    [link]: https://docs.rs/name-1\n";
        let exp = "    [link]: name_1\n";
        assert_eq!(exp, transform_favored_link(line.into()));

        let line = "[link]: https://docs.rs/name_1\n";
        let exp = "[link]: name_1\n";
        assert_eq!(exp, transform_favored_link(line.into()));
    }

    #[test]
    fn matching_favored_links_doc_rust_lang_long() {
        let line = "/// [link]: https://doc.rust-lang.org/nightly/std/primitive.str.html\n";
        let exp = "/// [link]: std/primitive.str.html\n";
        assert_eq!(exp, transform_favored_link(line.into()));

        let line = "/// [link]: https://doc.rust-lang.org/beta/std/primitive.str.html\n";
        let exp = "/// [link]: std/primitive.str.html\n";
        assert_eq!(exp, transform_favored_link(line.into()));

        let line = "/// [link]: https://doc.rust-lang.org/stable/std/primitive.str.html\n";
        let exp = "/// [link]: std/primitive.str.html\n";
        assert_eq!(exp, transform_favored_link(line.into()));

        let line = "/// [link]: https://doc.rust-lang.org/1.42.1/std/primitive.str.html\n";
        let exp = "/// [link]: std/primitive.str.html\n";
        assert_eq!(exp, transform_favored_link(line.into()));

        let line = "/// [link]: https://doc.rust-lang.org/nightly/std/primitive.str.html#usage\n";
        let exp = "/// [link]: std/primitive.str.html#usage\n";
        assert_eq!(exp, transform_favored_link(line.into()));

        let line = "/// [link]: https://doc.rust-lang.org/beta/std/primitive.str.html#usage\n";
        let exp = "/// [link]: std/primitive.str.html#usage\n";
        assert_eq!(exp, transform_favored_link(line.into()));

        let line = "/// [link]: https://doc.rust-lang.org/stable/std/primitive.str.html#usage\n";
        let exp = "/// [link]: std/primitive.str.html#usage\n";
        assert_eq!(exp, transform_favored_link(line.into()));

        let line = "/// [link]: https://doc.rust-lang.org/1.42.1/std/primitive.str.html#usage\n";
        let exp = "/// [link]: std/primitive.str.html#usage\n";
        assert_eq!(exp, transform_favored_link(line.into()));

        let line = "/// [link]: https://doc.rust-lang.org/nightly/std/\n";
        let exp = "/// [link]: std/index.html\n";
        assert_eq!(exp, transform_favored_link(line.into()));

        let line = "/// [link]: https://doc.rust-lang.org/beta/std/string/struct.String.html\n";
        let exp = "/// [link]: std/string/struct.String.html\n";
        assert_eq!(exp, transform_favored_link(line.into()));

        let line =
            "/// [link]: https://doc.rust-lang.org/beta/std/string/struct.String.html#section\n";
        let exp = "/// [link]: std/string/struct.String.html#section\n";
        assert_eq!(exp, transform_favored_link(line.into()));

        let line = "/// [link]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/enum.TyKind.html\n";
        let exp = "/// [link]: rustc_hir/enum.TyKind.html\n";
        assert_eq!(exp, transform_favored_link(line.into()));
    }
}
