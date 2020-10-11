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

mod regexes {
    use super::*;

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

        for item in ITEM_TYPES {
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
        pos: 0,
        curr_type_block: None,
        end_type_block: String::new(),
        type_block_line: usize::MAX,
        type_blocks: Vec::new(),
    };

    assert_eq!(Context::new("name".into()), ctx);
    assert_ne!(Context::new("not_name".into()), ctx);
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
