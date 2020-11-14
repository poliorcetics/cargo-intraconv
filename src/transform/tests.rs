use super::*;
use crate::{ConversionOptions, Krate};

lazy_static::lazy_static! {
    static ref CTX_KRATE_DIS_AND_FAV: ConversionContext = ConversionContext::with_options(ConversionOptions {
        krate: Krate::new("krate").unwrap(),
        disambiguate: true,
        favored_links: true,
    });

    static ref CTX_KRATE_NO_DIS_NO_FAV: ConversionContext = ConversionContext::with_options(ConversionOptions {
        krate: Krate::new("krate").unwrap(),
        disambiguate: false,
        favored_links: false,
    });

    static ref CTX_KRATE_NO_DIS_BUT_FAV: ConversionContext = ConversionContext::with_options(ConversionOptions {
        krate: Krate::new("krate").unwrap(),
        disambiguate: false,
        favored_links: true,
    });

    static ref CTX_KRATE_DIS_NO_FAV: ConversionContext = ConversionContext::with_options(ConversionOptions {
        krate: Krate::new("krate").unwrap(),
        disambiguate: true,
        favored_links: false,
    });
}

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

#[test]
fn new() {
    let krate = Krate::new("name").unwrap();
    let not_krate = Krate::new("not_name").unwrap();

    let ctx = ConversionContext {
        options: ConversionOptions {
            krate: krate.clone(),
            disambiguate: false,
            favored_links: true,
        },
        pos: 0,
        curr_type_block: None,
        end_type_block: String::new(),
        type_block_line: usize::MAX,
        type_blocks: Vec::new(),
    };

    assert_eq!(
        ConversionContext::with_options(ConversionOptions {
            krate: krate.clone(),
            disambiguate: false,
            favored_links: true
        }),
        ctx
    );

    assert_ne!(
        ConversionContext::with_options(ConversionOptions {
            krate: krate.clone(),
            disambiguate: true,
            favored_links: true
        }),
        ctx
    );
    assert_ne!(
        ConversionContext::with_options(ConversionOptions {
            krate: krate.clone(),
            disambiguate: true,
            favored_links: false
        }),
        ctx
    );
    assert_ne!(
        ConversionContext::with_options(ConversionOptions {
            krate: krate.clone(),
            disambiguate: false,
            favored_links: false
        }),
        ctx
    );

    assert_ne!(
        ConversionContext::with_options(ConversionOptions {
            krate: not_krate.clone(),
            disambiguate: true,
            favored_links: true
        }),
        ctx
    );
    assert_ne!(
        ConversionContext::with_options(ConversionOptions {
            krate: not_krate.clone(),
            disambiguate: true,
            favored_links: false
        }),
        ctx
    );
    assert_ne!(
        ConversionContext::with_options(ConversionOptions {
            krate: not_krate.clone(),
            disambiguate: false,
            favored_links: true
        }),
        ctx
    );
    assert_ne!(
        ConversionContext::with_options(ConversionOptions {
            krate: not_krate.clone(),
            disambiguate: false,
            favored_links: false
        }),
        ctx
    );
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
fn delete_local_links() {
    let mut ctx = CTX_KRATE_NO_DIS_BUT_FAV.clone();

    assert!(ctx.transform_line("[name]: name".into()).is_deleted());
    assert!(ctx
        .transform_line("[`name::Item`]: name/Item".into())
        .is_deleted());

    let mut ctx = CTX_KRATE_DIS_AND_FAV.clone();
    assert!(!ctx
        .transform_line("[`name::Item`]: name/struct.Item.html".into())
        .is_deleted());
}

#[test]
fn section_add_nothing() {
    let mut ctx = CTX_KRATE_DIS_AND_FAV.clone();

    assert_eq!(
        "[name]: #section\n",
        ctx.transform_line("[name]: #section".into())
    );
}

#[test]
fn assoc_add_self_when_type_block_is_empty() {
    let mut ctx = CTX_KRATE_DIS_AND_FAV.clone();

    assert_eq!(
        "[name]: Self::drain()\n",
        ctx.transform_line("[name]: #method.drain".into())
    );
}

#[test]
fn non_line() {
    let non_line_lines = [
        "let a = b;",
        "if a == b { let c = Type { toto: titi }; }",
        "/// struct X;",
        "//! struct X;",
        "// struct X;",
        "  // trait T {}",
        "",
        "'\n'.into()",
        "struct A(());",
        "/// [link]: https://toto.com",
    ];

    let mut ctx = CTX_KRATE_DIS_AND_FAV.clone();
    for &line in &non_line_lines {
        assert_eq!(
            line,
            ctx.transform_line(line.into()).as_new_line().trim_end()
        );
    }

    let mut ctx = CTX_KRATE_NO_DIS_BUT_FAV.clone();
    for &line in &non_line_lines {
        assert_eq!(
            line,
            ctx.transform_line(line.into()).as_new_line().trim_end()
        );
    }
}

#[test]
fn type_block_none_to_some() {
    let mut ctx = CTX_KRATE_DIS_AND_FAV.clone();

    ctx.type_blocks = vec![("Type".into(), "}".into(), 1)];

    ctx.transform_line("".into());

    assert_eq!(Some("Type".into()), ctx.curr_type_block);
    assert_eq!("}", ctx.end_type_block);
    assert_eq!(1, ctx.type_block_line);
    assert!(ctx.type_blocks.is_empty());

    // No disambiguate
    let mut ctx = CTX_KRATE_NO_DIS_BUT_FAV.clone();

    ctx.type_blocks = vec![("Type".into(), "}".into(), 1)];

    ctx.transform_line("".into());

    assert_eq!(Some("Type".into()), ctx.curr_type_block);
    assert_eq!("}", ctx.end_type_block);
    assert_eq!(1, ctx.type_block_line);
    assert!(ctx.type_blocks.is_empty());
}

#[test]
fn type_block_some_to_none() {
    let mut ctx = CTX_KRATE_DIS_AND_FAV.clone();

    ctx.pos = 2;

    ctx.curr_type_block = Some("Type".into());
    ctx.end_type_block = "}".into();
    ctx.type_block_line = 1;

    ctx.transform_line("}".into());

    assert_eq!(None, ctx.curr_type_block);
    assert_eq!("", ctx.end_type_block);
    assert_eq!(usize::MAX, ctx.type_block_line);

    // No disambiguate
    let mut ctx = CTX_KRATE_DIS_AND_FAV.clone();

    ctx.pos = 2;

    ctx.curr_type_block = Some("Type".into());
    ctx.end_type_block = "}".into();
    ctx.type_block_line = 1;

    ctx.transform_line("}".into());

    assert_eq!(None, ctx.curr_type_block);
    assert_eq!("", ctx.end_type_block);
    assert_eq!(usize::MAX, ctx.type_block_line);
}
