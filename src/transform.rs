use crate::Action;
use lazy_static::lazy_static;
use regex::Regex;
use std::io::{self, BufRead};

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
    /// It must have been checked to be a correct identifier for Rust.
    krate: String,
    /// Current line number.
    pos: usize,
    /// Name of the type that is `Self` for the current block.
    curr_type_block: Option<String>,
    /// End of the current block for `Self` (if any).
    end_type_block: String,
    // NOTE: at the moment nested type blocks are not handled.
    /// All types blocks known to the context.
    ///
    /// Calling `pop` on the `Vec` must give the next type block (if there is
    /// one).
    type_blocks: Vec<(String, String)>,
}

impl Context {
    pub fn new(krate: String) -> Self {
        Self {
            krate,
            pos: 0,
            curr_type_block: None,
            end_type_block: String::new(),
            type_blocks: Vec::new(),
        }
    }

    pub fn transform_file<R: BufRead>(&mut self, reader: R) -> io::Result<Vec<Action>> {
        // Reset the state before handling the file.
        self.pos = 0;
        self.curr_type_block = None;
        self.end_type_block = String::new();
        self.type_blocks.clear();

        let mut lines = Vec::new();
        for l in reader.lines() {
            let mut line = l?.trim_end().to_string();
            line.push('\n');
            lines.push(line);
        }

        self.find_type_blocks(lines.iter().map(|s| s.as_str()));

        let mut actions = Vec::with_capacity(lines.len());
        for line in lines.into_iter() {
            self.pos += 1;
            actions.push(self.transform_line(line));
        }

        Ok(actions)
    }

    /// Fills `self.type_block` with the types that are encountered, reversing the
    /// vector after they have all been added.
    /// This means that calling this function twice maybe have unintended consequences
    /// on the state `Context`.
    fn find_type_blocks<'a>(&mut self, lines: impl Iterator<Item = &'a str>) {
        for line in lines {
            // Early return on context change too, after updating the context.
            if let Some(captures) = TYPE_BLOCK_START.captures(&line) {
                let ty = captures.name("type").unwrap().as_str().into();
                let end = if line.ends_with(";\n") || line.ends_with("}\n") {
                    '\n'.into()
                } else {
                    // When the item is not simple we try to compute what will be
                    // the end of the block.
                    let mut s = captures.name("spaces").unwrap().as_str().to_string();
                    s.reserve(1);

                    if let Some(_) = captures.name("parenthese") {
                        s.push(')');
                    } else {
                        s.push('}');
                    }

                    s
                };

                self.type_blocks.push((ty, end));
            }
        }

        self.type_blocks.reverse();
    }

    /// Transform a single line, returning the action.
    fn transform_line(&mut self, line: String) -> Action {
        let copy = line.clone();

        // Updating the currently active `Self` type.
        if self.curr_type_block.is_none() {
            if let Some((curr_type, end)) = self.type_blocks.pop() {
                self.curr_type_block = Some(curr_type);
                self.end_type_block = end;
            }
        }

        if HTTP_LINK.is_match(&line) {
            return Action::Unchanged { line };
        }

        let line = self.transform_item(line);
        let line = self.transform_module(line);
        let line = self.transform_anchor(line);

        // When reaching the end of the current type block, update the context to
        // reflect it.
        if self.curr_type_block.is_some() && line.starts_with(&self.end_type_block) {
            self.curr_type_block = None;
            self.end_type_block.clear();
        }

        let line = self.transform_local(line);

        if line.is_empty() {
            Action::Deleted {
                line: copy,
                pos: self.pos,
            }
        } else if line == copy {
            Action::Unchanged { line: copy }
        } else {
            Action::Replaced {
                line: copy,
                new: line,
                pos: self.pos,
            }
        }
    }

    /// Try to transform a line as an item link. If it is not, the line
    /// is returned unmodified.
    fn transform_item(&mut self, line: String) -> String {
        let item_link = Regex::new(
            &[
                r"^(?P<link_name>\s*(?://[!/] )?\[.*?\]: )",
                r"(?:\./)?",
                r"(?P<supers>(?:\.\./)*)",
                &format!(r"(?:(?P<crate>{})/)?", self.krate),
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

        if let Some(captures) = item_link.captures(&line) {
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
        }
    }

    /// Try to transform a line as a module link. If it is not, the line is
    /// returned unmodified.
    fn transform_module(&mut self, line: String) -> String {
        let module_link = Regex::new(
            &[
                r"^(?P<link_name>\s*(?://[!/] )?\[.*?\]: )",
                r"(?:\./)?",
                r"(?P<supers>(?:\.\./)*)",
                &format!(r"(?:(?P<crate>{})/)?", self.krate),
                r"(?P<mods>(?:.*?/)*)",
                r"index\.html",
                r"(?P<section>#.+)?\n$",
            ]
            .join(""),
        )
        .unwrap();

        if let Some(captures) = module_link.captures(&line) {
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
            line
        }
    }

    /// Try to transform a line as an anchor link. If it is not, the line is
    /// returned unmodified. For the best results, ensure `find_type_blocks`
    /// has been called before.
    fn transform_anchor(&mut self, line: String) -> String {
        if let (Some(ref captures), Some(ref ty)) =
            (METHOD_ANCHOR.captures(&line), &self.curr_type_block)
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
            line
        }
    }

    /// Try to transform a local link to an empty string. If it is not, the
    /// line is returned unmodified. Should be called after all the other
    /// transformations to ensure no local link is missed.
    fn transform_local(&mut self, line: String) -> String {
        if let Some(captures) = LOCAL_PATH.captures(&line) {
            let link = captures.name("elem").unwrap();
            let path = captures.name("elem2").unwrap();
            if path.as_str() == link.as_str() {
                return "".into();
            }
        }

        line
    }
}

#[cfg(test)]
mod tests {
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

    lazy_static! {
        static ref STD_CTX: Context = Context {
            krate: "std".into(),
            pos: 0,
            curr_type_block: None,
            end_type_block: "".into(),
            type_blocks: Vec::new(),
        };
        static ref CORE_CTX: Context = Context {
            krate: "core".into(),
            pos: 0,
            curr_type_block: None,
            end_type_block: "".into(),
            type_blocks: Vec::new(),
        };
    }

    mod context {
        use super::*;

        #[test]
        fn find_type_blocks() {
            let mut ctx = STD_CTX.clone();

            let lines = vec![
                "struct User {\n",
                "    username: String,\n",
                "}\n",
                "let user1 = User {\n",
                "    email: String::from(\"someone@example.com\"),\n",
                "};\n",
                "struct A(usize);\n",
                "    struct Struct {\n",
                "        username: String,\n",
                "    }\n",
            ];

            ctx.find_type_blocks(lines.into_iter());

            assert_eq!(
                ctx.type_blocks,
                &[
                    ("Struct".into(), "    }".into()),
                    ("A".into(), "\n".into()),
                    ("User".into(), "}".into()),
                ]
            );
        }

        #[test]
        fn struct_blocks() {
            let mut ctx = STD_CTX.clone();

            let line = "struct A {}\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "\n".into())]);
            ctx.type_blocks.clear();

            let line = "struct A();\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "\n".into())]);
            ctx.type_blocks.clear();

            let line = "struct A { inner: String, }\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "\n".into())]);
            ctx.type_blocks.clear();

            let line = "struct A(usize);\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "\n".into())]);
            ctx.type_blocks.clear();

            let line = "struct A<'a, B=u8> where B: Trait + 'a {\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "}".into())]);
            ctx.type_blocks.clear();

            let line = "struct C<T=u8>(usize, (isize, T));\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("C".into(), "\n".into())]);
            ctx.type_blocks.clear();
        }

        #[test]
        fn trait_blocks() {
            let mut ctx = STD_CTX.clone();

            let line = "trait A {}\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "\n".into())]);
            ctx.type_blocks.clear();

            let line = "trait A {\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "}".into())]);
            ctx.type_blocks.clear();

            let line = "trait A { type T: Into<String>, }\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "\n".into())]);
            ctx.type_blocks.clear();

            let line = "trait A<'a, B=u8> where B: Trait + 'a {\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "}".into())]);
            ctx.type_blocks.clear();
        }

        #[test]
        fn enum_blocks() {
            let mut ctx = STD_CTX.clone();

            let line = "enum A {}\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "\n".into())]);
            ctx.type_blocks.clear();

            let line = "enum A {\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "}".into())]);
            ctx.type_blocks.clear();

            let line = "enum A { Variant1, Variant2 }\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "\n".into())]);
            ctx.type_blocks.clear();

            let line = "enum A<'a, B=u8> where B: Trait + 'a {\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "}".into())]);
            ctx.type_blocks.clear();
        }

        #[test]
        fn union_blocks() {
            let mut ctx = STD_CTX.clone();

            let line = "union A {}\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "\n".into())]);
            ctx.type_blocks.clear();

            let line = "union A {\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "}".into())]);
            ctx.type_blocks.clear();

            let line = "union A { f: f64, u: u64 }\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "\n".into())]);
            ctx.type_blocks.clear();

            let line = "union A<'a, B=u8> where B: Trait + 'a {\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "}".into())]);
            ctx.type_blocks.clear();
        }

        #[test]
        fn impl_blocks() {
            let mut ctx = STD_CTX.clone();

            let line = "impl Trait for A {}\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "\n".into())]);
            ctx.type_blocks.clear();

            let line = "impl A {\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "}".into())]);
            ctx.type_blocks.clear();

            let line = "impl <T> Toto for A<T> {\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "}".into())]);
            ctx.type_blocks.clear();

            let line = "impl Trait for A { type B = String }\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "\n".into())]);
            ctx.type_blocks.clear();

            let line = "impl<'a: 'static, B> Trait for A where B: Toto + 'a {\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "}".into())]);
            ctx.type_blocks.clear();

            let line = "impl<'a, 'b, B: Trait<IntoIterator<Item=String>>> Toto for A where B: Toto + 'a, 'b: 'a, I: A<I> {\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "}".into())]);
            ctx.type_blocks.clear();
        }

        #[test]
        fn visibility_modifiers_are_handled() {
            let mut ctx = STD_CTX.clone();

            let line = "pub struct A {}\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "\n".into())]);
            ctx.type_blocks.clear();

            let line = "pub(crate) struct A();\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "\n".into())]);
            ctx.type_blocks.clear();

            let line = "pub(super) struct A { inner: String, }\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "\n".into())]);
            ctx.type_blocks.clear();

            let line = "pub(self) struct A(usize);\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "\n".into())]);
            ctx.type_blocks.clear();

            let line = "pub(crate::module) struct A<'a, B=u8> where B: Trait + 'a {\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "}".into())]);
            ctx.type_blocks.clear();

            let line = "pub(mod1::mod2) struct C<T=u8>(usize, (isize, T));\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("C".into(), "\n".into())]);
            ctx.type_blocks.clear();
        }

        #[test]
        fn indentation_is_remembered() {
            let mut ctx = STD_CTX.clone();

            let line = "    struct A {}\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "\n".into())]);
            ctx.type_blocks.clear();

            let line = "  struct A();\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "\n".into())]);
            ctx.type_blocks.clear();

            let line = "   struct A { inner: String, }\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "\n".into())]);
            ctx.type_blocks.clear();

            let line = " struct A(usize);\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "\n".into())]);
            ctx.type_blocks.clear();

            let line = "  struct A<'a, B=u8> where B: Trait + 'a {\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("A".into(), "  }".into())]);
            ctx.type_blocks.clear();

            let line = "    struct C<'a, B=u8> where B: Trait + 'a {\n";
            ctx.find_type_blocks(std::iter::once(line));
            assert_eq!(ctx.type_blocks, [("C".into(), "    }".into())]);
            ctx.type_blocks.clear();
        }

        #[test]
        fn transform_item_unchanged() {
            let mut ctx = STD_CTX.clone();

            let lines = [
                "let a = b;\n",
                "// let a = b;\n",
                "    /// let a = b;\n",
                "//! let a = b;\n",
                "    struct A<B: Trait> {\n",
                "/// [module]: ./module/index.html",
            ];

            for &line in &lines {
                assert_eq!(line, ctx.transform_item(line.into()));
            }
        }

        #[test]
        fn transform_item_changed() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`String`]: struct.String.html\n";
            assert_eq!("/// [`String`]: String\n", ctx.transform_item(line.into()));

            let line = "    /// [String]: struct.String.html\n";
            assert_eq!(
                "    /// [String]: String\n",
                ctx.transform_item(line.into())
            );

            let line = "[`String`]: struct.String.html\n";
            assert_eq!("[`String`]: String\n", ctx.transform_item(line.into()));

            let line = "    [String]: struct.String.html\n";
            assert_eq!("    [String]: String\n", ctx.transform_item(line.into()));

            let line = "/// [`String`]: ./struct.String.html\n";
            assert_eq!("/// [`String`]: String\n", ctx.transform_item(line.into()));

            let line = "    /// [String]: ./struct.String.html\n";
            assert_eq!(
                "    /// [String]: String\n",
                ctx.transform_item(line.into())
            );

            let line = "[`String`]: ./struct.String.html\n";
            assert_eq!("[`String`]: String\n", ctx.transform_item(line.into()));

            let line = "    [String]: ./struct.String.html\n";
            assert_eq!("    [String]: String\n", ctx.transform_item(line.into()));

            let line = "/// [`String`]: ./string/struct.String.html\n";
            assert_eq!(
                "/// [`String`]: string::String\n",
                ctx.transform_item(line.into())
            );

            let line = "    /// [String]: ./string/struct.String.html\n";
            assert_eq!(
                "    /// [String]: string::String\n",
                ctx.transform_item(line.into())
            );

            let line = "[`String`]: ./string/struct.String.html\n";
            assert_eq!(
                "[`String`]: string::String\n",
                ctx.transform_item(line.into())
            );

            let line = "    [String]: ./string/struct.String.html\n";
            assert_eq!(
                "    [String]: string::String\n",
                ctx.transform_item(line.into())
            );

            let line = "/// [`String`]: string/struct.String.html\n";
            assert_eq!(
                "/// [`String`]: string::String\n",
                ctx.transform_item(line.into())
            );

            let line = "    /// [String]: string/struct.String.html\n";
            assert_eq!(
                "    /// [String]: string::String\n",
                ctx.transform_item(line.into())
            );

            let line = "[`String`]: string/struct.String.html\n";
            assert_eq!(
                "[`String`]: string::String\n",
                ctx.transform_item(line.into())
            );

            let line = "    [String]: string/struct.String.html\n";
            assert_eq!(
                "    [String]: string::String\n",
                ctx.transform_item(line.into())
            );
        }

        #[test]
        fn transform_module_unchanged() {
            let mut ctx = STD_CTX.clone();

            let lines = [
                "let a = b;\n",
                "// let a = b;\n",
                "    /// let a = b;\n",
                "//! let a = b;\n",
                "    struct A<B: Trait> {\n",
                "/// [item]: ./module/fn.name.html",
            ];

            for &line in &lines {
                assert_eq!(line, ctx.transform_module(line.into()));
            }
        }
    }

    mod unchanged_lines {
        use super::*;

        #[test]
        fn code_line_is_unchanged() {
            let mut ctx = STD_CTX.clone();

            let line = "let res = a + b;\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn normal_comment_line_is_unchanged() {
            let mut ctx = STD_CTX.clone();

            let line = "// let res = a + b;\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn normal_doc_comment_line_is_unchanged() {
            let mut ctx = STD_CTX.clone();

            let line = "/// let res = a + b;\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn normal_header_doc_comment_line_is_unchanged() {
            let mut ctx = STD_CTX.clone();

            let line = "//! let res = a + b;\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn indentation_is_unchanged() {
            let mut ctx = STD_CTX.clone();

            let line = "  //! let res = a + b;\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            let line = "    //! let res = a + b;\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn http_link_is_ignored() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`String`]: http://www.example.com/index.html#section\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            let line = "    /// [`String`]: https://www.example.com/index.html#section\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            assert_eq!(*STD_CTX, ctx);
        }
    }

    mod paths {
        use super::*;

        #[test]
        fn local_path_is_deleted() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`String`]: String\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            let line = "    /// [String]: String\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            let line = "[`String`]: String\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            let line = "    [String]: String\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn long_path_is_unchanged() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`String`]: string::String\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            let line = "    /// [String]: string::String\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            let line = "[`String`]: string::String\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            let line = "    [String]: string::String\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn full_path_is_unchanged() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`String`]: ::std::string::String\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            let line = "    /// [String]: ::std::string::String\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            let line = "[`String`]: ::std::string::String\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            let line = "    [String]: ::std::string::String\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            assert_eq!(*STD_CTX, ctx);
        }
    }

    mod item_tests {
        use super::*;

        #[test]
        fn local_link_is_deleted() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`String`]: struct.String.html\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            let line = "    /// [String]: struct.String.html\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            let line = "[`String`]: struct.String.html\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            let line = "    [String]: struct.String.html\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            let line = "/// [`String`]: ./struct.String.html\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            let line = "    /// [String]: ./struct.String.html\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            let line = "[`String`]: ./struct.String.html\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            let line = "    [String]: ./struct.String.html\n";
            assert_eq!(line, ctx.transform_line(line.into()));

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn long_link_is_transformed() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`String`]: ./string/struct.String.html\n";
            assert_eq!(
                "/// [`String`]: string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "    /// [String]: ./string/struct.String.html\n";
            assert_eq!(
                "    /// [String]: string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "[`String`]: ./string/struct.String.html\n";
            assert_eq!(
                "[`String`]: string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "    [String]: ./string/struct.String.html\n";
            assert_eq!(
                "    [String]: string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "/// [`String`]: string/struct.String.html\n";
            assert_eq!(
                "/// [`String`]: string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "    /// [String]: string/struct.String.html\n";
            assert_eq!(
                "    /// [String]: string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "[`String`]: string/struct.String.html\n";
            assert_eq!(
                "[`String`]: string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "    [String]: string/struct.String.html\n";
            assert_eq!(
                "    [String]: string::String\n",
                ctx.transform_line(line.into())
            );

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn full_link_is_transformed_crate() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`String`]: std/string/struct.String.html\n";
            assert_eq!(
                "/// [`String`]: crate::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "    /// [String]: std/string/struct.String.html\n";
            assert_eq!(
                "    /// [String]: crate::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "[`String`]: std/string/struct.String.html\n";
            assert_eq!(
                "[`String`]: crate::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "    [String]: std/string/struct.String.html\n";
            assert_eq!(
                "    [String]: crate::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "/// [`String`]: ./std/string/struct.String.html\n";
            assert_eq!(
                "/// [`String`]: crate::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "    /// [String]: ./std/string/struct.String.html\n";
            assert_eq!(
                "    /// [String]: crate::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "[`String`]: ./std/string/struct.String.html\n";
            assert_eq!(
                "[`String`]: crate::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "    [String]: ./std/string/struct.String.html\n";
            assert_eq!(
                "    [String]: crate::string::String\n",
                ctx.transform_line(line.into())
            );

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn full_link_is_transformed_crate_over_super() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`String`]: ../../std/string/struct.String.html\n";
            assert_eq!(
                "/// [`String`]: crate::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "    /// [String]: ../../std/string/struct.String.html\n";
            assert_eq!(
                "    /// [String]: crate::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "[`String`]: ../../std/string/struct.String.html\n";
            assert_eq!(
                "[`String`]: crate::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "    [String]: ../../std/string/struct.String.html\n";
            assert_eq!(
                "    [String]: crate::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "/// [`String`]: ./../../std/string/struct.String.html\n";
            assert_eq!(
                "/// [`String`]: crate::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "    /// [String]: ./../../std/string/struct.String.html\n";
            assert_eq!(
                "    /// [String]: crate::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "[`String`]: ./../../std/string/struct.String.html\n";
            assert_eq!(
                "[`String`]: crate::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "    [String]: ./../../std/string/struct.String.html\n";
            assert_eq!(
                "    [String]: crate::string::String\n",
                ctx.transform_line(line.into())
            );

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn full_link_is_transformed_not_crate() {
            let mut ctx = CORE_CTX.clone();

            let line = "/// [`String`]: std/string/struct.String.html\n";
            assert_eq!(
                "/// [`String`]: std::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "    /// [String]: std/string/struct.String.html\n";
            assert_eq!(
                "    /// [String]: std::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "[`String`]: std/string/struct.String.html\n";
            assert_eq!(
                "[`String`]: std::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "    [String]: std/string/struct.String.html\n";
            assert_eq!(
                "    [String]: std::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "/// [`String`]: ./std/string/struct.String.html\n";
            assert_eq!(
                "/// [`String`]: std::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "    /// [String]: ./std/string/struct.String.html\n";
            assert_eq!(
                "    /// [String]: std::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "[`String`]: ./std/string/struct.String.html\n";
            assert_eq!(
                "[`String`]: std::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "    [String]: ./std/string/struct.String.html\n";
            assert_eq!(
                "    [String]: std::string::String\n",
                ctx.transform_line(line.into())
            );

            assert_eq!(*CORE_CTX, ctx);
        }

        #[test]
        fn full_link_is_transformed_super() {
            let mut ctx = CORE_CTX.clone();

            let line = "/// [`SpanTrace`]: ../struct.SpanTrace.html\n";
            assert_eq!(
                "/// [`SpanTrace`]: super::SpanTrace\n",
                ctx.transform_line(line.into())
            );

            let line = "/// [`String`]: ../../string/struct.String.html\n";
            assert_eq!(
                "/// [`String`]: super::super::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "    /// [String]: ../../string/struct.String.html\n";
            assert_eq!(
                "    /// [String]: super::super::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "[`String`]: ../../string/struct.String.html\n";
            assert_eq!(
                "[`String`]: super::super::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "    [String]: ../../string/struct.String.html\n";
            assert_eq!(
                "    [String]: super::super::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "/// [`String`]: ./../../string/struct.String.html\n";
            assert_eq!(
                "/// [`String`]: super::super::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "    /// [String]: ./../../string/struct.String.html\n";
            assert_eq!(
                "    /// [String]: super::super::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "[`String`]: ./../../string/struct.String.html\n";
            assert_eq!(
                "[`String`]: super::super::string::String\n",
                ctx.transform_line(line.into())
            );

            let line = "    [String]: ./../../string/struct.String.html\n";
            assert_eq!(
                "    [String]: super::super::string::String\n",
                ctx.transform_line(line.into())
            );

            assert_eq!(*CORE_CTX, ctx);
        }

        #[test]
        fn additional_is_kept() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`String`]: struct.String.html#method.as_ref\n";
            assert_eq!(
                "/// [`String`]: String::as_ref\n",
                ctx.transform_line(line.into())
            );

            let line = "    /// [String]: struct.String.html#method.as_ref\n";
            assert_eq!(
                "    /// [String]: String::as_ref\n",
                ctx.transform_line(line.into())
            );

            let line = "[`String`]: struct.String.html#method.as_ref\n";
            assert_eq!(
                "[`String`]: String::as_ref\n",
                ctx.transform_line(line.into())
            );

            let line = "    [String]: struct.String.html#method.as_ref\n";
            assert_eq!(
                "    [String]: String::as_ref\n",
                ctx.transform_line(line.into())
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
                        ctx.transform_line(line)
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

            fn assert_deleted(a: Action) {
                match a {
                    Action::Deleted { line: _, pos: _ } => (),
                    _ => assert!(false, "{} is not a Deleted action", a),
                }
            }

            let line = "/// [`string`]: string/index.html\n";
            let res = ctx.transform_line(line.into());
            assert_eq!(line, res);
            assert_deleted(res);

            let line = "    //! [string]: string/index.html\n";
            let res = ctx.transform_line(line.into());
            assert_eq!(line, res);
            assert_deleted(res);

            let line = "[`string`]: string/index.html\n";
            let res = ctx.transform_line(line.into());
            assert_eq!(line, res);
            assert_deleted(res);

            let line = "    [string]: string/index.html\n";
            let res = ctx.transform_line(line.into());
            assert_eq!(line, res);
            assert_deleted(res);

            let line = "/// [`string`]: index.html\n";
            let res = ctx.transform_line(line.into());
            assert_eq!(line, res);
            assert_deleted(res);

            let line = "    /// [string]: index.html\n";
            let res = ctx.transform_line(line.into());
            assert_eq!(line, res);
            assert_deleted(res);

            let line = "[`string`]: index.html\n";
            let res = ctx.transform_line(line.into());
            assert_eq!(line, res);
            assert_deleted(res);

            let line = "    [string]: index.html\n";
            let res = ctx.transform_line(line.into());
            assert_eq!(line, res);
            assert_deleted(res);

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn long_link_is_transformed() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`string`]: module/string/index.html\n";
            assert_eq!(
                "/// [`string`]: module::string\n",
                ctx.transform_line(line.into())
            );

            let line = "    /// [string]: module/string/index.html\n";
            assert_eq!(
                "    /// [string]: module::string\n",
                ctx.transform_line(line.into())
            );

            let line = "[`string`]: module/string/index.html\n";
            assert_eq!(
                "[`string`]: module::string\n",
                ctx.transform_line(line.into())
            );

            let line = "    [string]: module/string/index.html\n";
            assert_eq!(
                "    [string]: module::string\n",
                ctx.transform_line(line.into())
            );

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn full_link_is_transformed_crate() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`string`]: std/string/index.html\n";
            assert_eq!(
                "/// [`string`]: crate::string\n",
                ctx.transform_line(line.into())
            );

            let line = "    /// [string]: std/string/index.html\n";
            assert_eq!(
                "    /// [string]: crate::string\n",
                ctx.transform_line(line.into())
            );

            let line = "[`string`]: std/string/index.html\n";
            assert_eq!(
                "[`string`]: crate::string\n",
                ctx.transform_line(line.into())
            );

            let line = "    [string]: std/string/index.html\n";
            assert_eq!(
                "    [string]: crate::string\n",
                ctx.transform_line(line.into())
            );

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn full_link_is_transformed_crate_over_super() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`string`]: ../../std/string/index.html\n";
            assert_eq!(
                "/// [`string`]: crate::string\n",
                ctx.transform_line(line.into())
            );

            let line = "    /// [string]: ../../std/string/index.html\n";
            assert_eq!(
                "    /// [string]: crate::string\n",
                ctx.transform_line(line.into())
            );

            let line = "[`string`]: ../../std/string/index.html\n";
            assert_eq!(
                "[`string`]: crate::string\n",
                ctx.transform_line(line.into())
            );

            let line = "    [string]: ../../std/string/index.html\n";
            assert_eq!(
                "    [string]: crate::string\n",
                ctx.transform_line(line.into())
            );

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn full_link_is_transformed_not_crate() {
            let mut ctx = CORE_CTX.clone();

            let line = "/// [`string`]: std/string/index.html\n";
            assert_eq!(
                "/// [`string`]: std::string\n",
                ctx.transform_line(line.into())
            );

            let line = "    /// [string]: std/string/index.html\n";
            assert_eq!(
                "    /// [string]: std::string\n",
                ctx.transform_line(line.into())
            );

            let line = "[`string`]: std/string/index.html\n";
            assert_eq!("[`string`]: std::string\n", ctx.transform_line(line.into()));

            let line = "    [string]: std/string/index.html\n";
            assert_eq!(
                "    [string]: std::string\n",
                ctx.transform_line(line.into())
            );

            assert_eq!(*CORE_CTX, ctx);
        }

        #[test]
        fn full_link_is_transformed_super() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`string`]: ../../string/index.html\n";
            assert_eq!(
                "/// [`string`]: super::super::string\n",
                ctx.transform_line(line.into())
            );

            let line = "    /// [string]: ../../string/index.html\n";
            assert_eq!(
                "    /// [string]: super::super::string\n",
                ctx.transform_line(line.into())
            );

            let line = "[`string`]: ../../string/index.html\n";
            assert_eq!(
                "[`string`]: super::super::string\n",
                ctx.transform_line(line.into())
            );

            let line = "    [string]: ../../string/index.html\n";
            assert_eq!(
                "    [string]: super::super::string\n",
                ctx.transform_line(line.into())
            );

            assert_eq!(*STD_CTX, ctx);
        }

        #[test]
        fn section_is_kept() {
            let mut ctx = STD_CTX.clone();

            let line = "/// [`string`]: string/index.html#my-section\n";
            assert_eq!(
                "/// [`string`]: string#my-section\n",
                ctx.transform_line(line.into())
            );

            let line = "    /// [string]: string/index.html#my-section\n";
            assert_eq!(
                "    /// [string]: string#my-section\n",
                ctx.transform_line(line.into())
            );

            let line = "[`string`]: string/index.html#my-section\n";
            assert_eq!(
                "[`string`]: string#my-section\n",
                ctx.transform_line(line.into())
            );

            let line = "    [string]: string/index.html#my-section\n";
            assert_eq!(
                "    [string]: string#my-section\n",
                ctx.transform_line(line.into())
            );

            let line = "/// [`see my section`]: index.html#my-section\n";
            assert_eq!(
                "/// [`see my section`]: #my-section\n",
                ctx.transform_line(line.into())
            );

            let line = "    /// [see my section]: index.html#my-section\n";
            assert_eq!(
                "    /// [see my section]: #my-section\n",
                ctx.transform_line(line.into())
            );

            let line = "[`see my section`]: index.html#my-section\n";
            assert_eq!(
                "[`see my section`]: #my-section\n",
                ctx.transform_line(line.into())
            );

            let line = "    [see my section]: index.html#my-section\n";
            assert_eq!(
                "    [see my section]: #my-section\n",
                ctx.transform_line(line.into())
            );

            assert_eq!(*STD_CTX, ctx);
        }
    }

    mod type_block {
        use super::*;

        #[test]
        fn end_set_block_to_none() {
            let mut ctx = STD_CTX.clone();
            ctx.curr_type_block = Some("String".into());
            ctx.end_type_block = '\n'.into();

            let line = "\n";
            assert_eq!(line, ctx.transform_line(line.into()));
            assert_eq!(ctx.curr_type_block, None);
            assert_eq!(ctx.end_type_block, "");

            ctx.curr_type_block = Some("String".into());
            ctx.end_type_block = "    }".into();
            let line = "\n";
            assert_eq!(line, ctx.transform_line(line.into()));
            assert_eq!(ctx.curr_type_block, Some("String".into()));
            assert_eq!(ctx.end_type_block, "    }");

            let line = "    }";
            assert_eq!(line, ctx.transform_line(line.into()));
            assert_eq!(ctx.curr_type_block, None);
            assert_eq!(ctx.end_type_block, "");

            ctx.curr_type_block = Some("String".into());
            ctx.end_type_block = "  )".into();

            let line = "  )";
            assert_eq!(line, ctx.transform_line(line.into()));
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
                assert_eq!(line.clone(), ctx.transform_line(line));

                let line = format!("    //! [link name]: #{}.as_ref", item);
                assert_eq!(line.clone(), ctx.transform_line(line));

                let line = format!("[`link name`]: #{}.as_ref", item);
                assert_eq!(line.clone(), ctx.transform_line(line));

                let line = format!("    [link name]: #{}.as_ref", item);
                assert_eq!(line.clone(), ctx.transform_line(line));
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
                assert_eq!(expected, ctx.transform_line(line));

                let line = format!("    //! [link name]: #{}.as_ref\n", item);
                let expected = format!("    //! [link name]: {}::as_ref\n", ty);
                assert_eq!(expected, ctx.transform_line(line));

                let line = format!("[`link name`]: #{}.as_ref\n", item);
                let expected = format!("[`link name`]: {}::as_ref\n", ty);
                assert_eq!(expected, ctx.transform_line(line));

                let line = format!("    [link name]: #{}.as_ref\n", item);
                let expected = format!("    [link name]: {}::as_ref\n", ty);
                assert_eq!(expected, ctx.transform_line(line));
            }
        }
    }

    mod complete_texts {
        use super::*;

        lazy_static! {
            static ref NO_LINKS_CODE: [&'static str; 4] = [
                "fn main() {\n",
                "    println!(\"{:b}\", !1_usize);\n",
                "    println!(\"{:b}\", !usize::MAX);\n",
                "}\n",
            ];
            static ref WITH_SELF_LINKS: [&'static str; 7] = [
                "/// [`b`]: #method.b\n",
                "/// [b]: #method.b\n",
                "struct A;\n",
                "\n",
                "impl A {\n",
                "    fn b(self) {}\n",
                "}\n",
            ];
        }

        #[test]
        fn no_links_code_empty_context() {
            let mut ctx = STD_CTX.clone();
            let unchanged = ctx.clone();

            for &line in NO_LINKS_CODE.into_iter() {
                assert_eq!(line, ctx.transform_line(line.into()));
                assert_eq!(ctx, unchanged);
            }
        }

        #[test]
        fn with_self_links() {
            let mut ctx = STD_CTX.clone();

            ctx.find_type_blocks(WITH_SELF_LINKS.to_vec().into_iter());
            assert_eq!(
                ctx.type_blocks,
                [("A".into(), "}".into()), ("A".into(), "\n".into())]
            );

            let mut iter = WITH_SELF_LINKS.into_iter();

            let line = *iter.next().unwrap();
            assert_eq!("/// [`b`]: A::b\n", ctx.transform_line(line.into()));

            let line = *iter.next().unwrap();
            assert_eq!("/// [b]: A::b\n", ctx.transform_line(line.into()));

            for &line in iter {
                assert_eq!(line, ctx.transform_line(line.into()));
            }
        }
    }
}
