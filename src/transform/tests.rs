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
        assert_eq!(
            "/// [`String`]: type@String\n",
            ctx.transform_item(line.into())
        );

        let line = "    /// [String]: struct.String.html\n";
        assert_eq!(
            "    /// [String]: type@String\n",
            ctx.transform_item(line.into())
        );

        let line = "[`String`]: struct.String.html\n";
        assert_eq!("[`String`]: type@String\n", ctx.transform_item(line.into()));

        let line = "    [String]: struct.String.html\n";
        assert_eq!(
            "    [String]: type@String\n",
            ctx.transform_item(line.into())
        );

        let line = "/// [`String`]: ./struct.String.html\n";
        assert_eq!(
            "/// [`String`]: type@String\n",
            ctx.transform_item(line.into())
        );

        let line = "    /// [String]: ./struct.String.html\n";
        assert_eq!(
            "    /// [String]: type@String\n",
            ctx.transform_item(line.into())
        );

        let line = "[`String`]: ./struct.String.html\n";
        assert_eq!("[`String`]: type@String\n", ctx.transform_item(line.into()));

        let line = "    [String]: ./struct.String.html\n";
        assert_eq!(
            "    [String]: type@String\n",
            ctx.transform_item(line.into())
        );

        let line = "/// [`String`]: ./string/struct.String.html\n";
        assert_eq!(
            "/// [`String`]: type@string::String\n",
            ctx.transform_item(line.into())
        );

        let line = "    /// [String]: ./string/struct.String.html\n";
        assert_eq!(
            "    /// [String]: type@string::String\n",
            ctx.transform_item(line.into())
        );

        let line = "[`String`]: ./string/struct.String.html\n";
        assert_eq!(
            "[`String`]: type@string::String\n",
            ctx.transform_item(line.into())
        );

        let line = "    [String]: ./string/struct.String.html\n";
        assert_eq!(
            "    [String]: type@string::String\n",
            ctx.transform_item(line.into())
        );

        let line = "/// [`String`]: string/struct.String.html\n";
        assert_eq!(
            "/// [`String`]: type@string::String\n",
            ctx.transform_item(line.into())
        );

        let line = "    /// [String]: string/struct.String.html\n";
        assert_eq!(
            "    /// [String]: type@string::String\n",
            ctx.transform_item(line.into())
        );

        let line = "[`String`]: string/struct.String.html\n";
        assert_eq!(
            "[`String`]: type@string::String\n",
            ctx.transform_item(line.into())
        );

        let line = "    [String]: string/struct.String.html\n";
        assert_eq!(
            "    [String]: type@string::String\n",
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
            "/// [`String`]: type@string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "    /// [String]: ./string/struct.String.html\n";
        assert_eq!(
            "    /// [String]: type@string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "[`String`]: ./string/struct.String.html\n";
        assert_eq!(
            "[`String`]: type@string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "    [String]: ./string/struct.String.html\n";
        assert_eq!(
            "    [String]: type@string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "/// [`String`]: string/struct.String.html\n";
        assert_eq!(
            "/// [`String`]: type@string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "    /// [String]: string/struct.String.html\n";
        assert_eq!(
            "    /// [String]: type@string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "[`String`]: string/struct.String.html\n";
        assert_eq!(
            "[`String`]: type@string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "    [String]: string/struct.String.html\n";
        assert_eq!(
            "    [String]: type@string::String\n",
            ctx.transform_line(line.into())
        );

        assert_eq!(*STD_CTX, ctx);
    }

    #[test]
    fn full_link_is_transformed_crate() {
        let mut ctx = STD_CTX.clone();

        let line = "/// [`String`]: std/string/struct.String.html\n";
        assert_eq!(
            "/// [`String`]: type@crate::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "    /// [String]: std/string/struct.String.html\n";
        assert_eq!(
            "    /// [String]: type@crate::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "[`String`]: std/string/struct.String.html\n";
        assert_eq!(
            "[`String`]: type@crate::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "    [String]: std/string/struct.String.html\n";
        assert_eq!(
            "    [String]: type@crate::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "/// [`String`]: ./std/string/struct.String.html\n";
        assert_eq!(
            "/// [`String`]: type@crate::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "    /// [String]: ./std/string/struct.String.html\n";
        assert_eq!(
            "    /// [String]: type@crate::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "[`String`]: ./std/string/struct.String.html\n";
        assert_eq!(
            "[`String`]: type@crate::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "    [String]: ./std/string/struct.String.html\n";
        assert_eq!(
            "    [String]: type@crate::string::String\n",
            ctx.transform_line(line.into())
        );

        assert_eq!(*STD_CTX, ctx);
    }

    #[test]
    fn full_link_is_transformed_crate_over_super() {
        let mut ctx = STD_CTX.clone();

        let line = "/// [`String`]: ../../std/string/struct.String.html\n";
        assert_eq!(
            "/// [`String`]: type@crate::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "    /// [String]: ../../std/string/struct.String.html\n";
        assert_eq!(
            "    /// [String]: type@crate::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "[`String`]: ../../std/string/struct.String.html\n";
        assert_eq!(
            "[`String`]: type@crate::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "    [String]: ../../std/string/struct.String.html\n";
        assert_eq!(
            "    [String]: type@crate::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "/// [`String`]: ./../../std/string/struct.String.html\n";
        assert_eq!(
            "/// [`String`]: type@crate::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "    /// [String]: ./../../std/string/struct.String.html\n";
        assert_eq!(
            "    /// [String]: type@crate::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "[`String`]: ./../../std/string/struct.String.html\n";
        assert_eq!(
            "[`String`]: type@crate::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "    [String]: ./../../std/string/struct.String.html\n";
        assert_eq!(
            "    [String]: type@crate::string::String\n",
            ctx.transform_line(line.into())
        );

        assert_eq!(*STD_CTX, ctx);
    }

    #[test]
    fn full_link_is_transformed_not_crate() {
        let mut ctx = CORE_CTX.clone();

        let line = "/// [`String`]: std/string/struct.String.html\n";
        assert_eq!(
            "/// [`String`]: type@std::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "    /// [String]: std/string/struct.String.html\n";
        assert_eq!(
            "    /// [String]: type@std::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "[`String`]: std/string/struct.String.html\n";
        assert_eq!(
            "[`String`]: type@std::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "    [String]: std/string/struct.String.html\n";
        assert_eq!(
            "    [String]: type@std::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "/// [`String`]: ./std/string/struct.String.html\n";
        assert_eq!(
            "/// [`String`]: type@std::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "    /// [String]: ./std/string/struct.String.html\n";
        assert_eq!(
            "    /// [String]: type@std::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "[`String`]: ./std/string/struct.String.html\n";
        assert_eq!(
            "[`String`]: type@std::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "    [String]: ./std/string/struct.String.html\n";
        assert_eq!(
            "    [String]: type@std::string::String\n",
            ctx.transform_line(line.into())
        );

        assert_eq!(*CORE_CTX, ctx);
    }

    #[test]
    fn full_link_is_transformed_super() {
        let mut ctx = CORE_CTX.clone();

        let line = "/// [`SpanTrace`]: ../struct.SpanTrace.html\n";
        assert_eq!(
            "/// [`SpanTrace`]: type@super::SpanTrace\n",
            ctx.transform_line(line.into())
        );

        let line = "/// [`String`]: ../../string/struct.String.html\n";
        assert_eq!(
            "/// [`String`]: type@super::super::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "    /// [String]: ../../string/struct.String.html\n";
        assert_eq!(
            "    /// [String]: type@super::super::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "[`String`]: ../../string/struct.String.html\n";
        assert_eq!(
            "[`String`]: type@super::super::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "    [String]: ../../string/struct.String.html\n";
        assert_eq!(
            "    [String]: type@super::super::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "/// [`String`]: ./../../string/struct.String.html\n";
        assert_eq!(
            "/// [`String`]: type@super::super::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "    /// [String]: ./../../string/struct.String.html\n";
        assert_eq!(
            "    /// [String]: type@super::super::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "[`String`]: ./../../string/struct.String.html\n";
        assert_eq!(
            "[`String`]: type@super::super::string::String\n",
            ctx.transform_line(line.into())
        );

        let line = "    [String]: ./../../string/struct.String.html\n";
        assert_eq!(
            "    [String]: type@super::super::string::String\n",
            ctx.transform_line(line.into())
        );

        assert_eq!(*CORE_CTX, ctx);
    }

    #[test]
    fn additional_is_kept() {
        let mut ctx = STD_CTX.clone();

        let line = "/// [`String`]: struct.String.html#method.as_ref\n";
        assert_eq!(
            "/// [`String`]: String::as_ref()\n",
            ctx.transform_line(line.into())
        );

        let line = "    /// [String]: struct.String.html#method.as_ref\n";
        assert_eq!(
            "    /// [String]: String::as_ref()\n",
            ctx.transform_line(line.into())
        );

        let line = "[`String`]: struct.String.html#method.as_ref\n";
        assert_eq!(
            "[`String`]: String::as_ref()\n",
            ctx.transform_line(line.into())
        );

        let line = "    [String]: struct.String.html#method.as_ref\n";
        assert_eq!(
            "    [String]: String::as_ref()\n",
            ctx.transform_line(line.into())
        );

        assert_eq!(*STD_CTX, ctx);
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

        let line = "/// [`string`]: ./string/index.html\n";
        let res = ctx.transform_line(line.into());
        assert_eq!(line, res);
        assert_deleted(res);

        let line = "    //! [string]: ./string/index.html\n";
        let res = ctx.transform_line(line.into());
        assert_eq!(line, res);
        assert_deleted(res);

        let line = "[`string`]: ./string/index.html\n";
        let res = ctx.transform_line(line.into());
        assert_eq!(line, res);
        assert_deleted(res);

        let line = "    [string]: ./string/index.html\n";
        let res = ctx.transform_line(line.into());
        assert_eq!(line, res);
        assert_deleted(res);

        let line = "/// [`string`]: ./index.html\n";
        let res = ctx.transform_line(line.into());
        assert_eq!(line, res);
        assert_deleted(res);

        let line = "    /// [string]: ./index.html\n";
        let res = ctx.transform_line(line.into());
        assert_eq!(line, res);
        assert_deleted(res);

        let line = "[`string`]: ./index.html\n";
        let res = ctx.transform_line(line.into());
        assert_eq!(line, res);
        assert_deleted(res);

        let line = "    [string]: ./index.html\n";
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
            "/// [`string`]: mod@module::string\n",
            ctx.transform_line(line.into())
        );

        let line = "    /// [string]: module/string/index.html\n";
        assert_eq!(
            "    /// [string]: mod@module::string\n",
            ctx.transform_line(line.into())
        );

        let line = "[`string`]: module/string/index.html\n";
        assert_eq!(
            "[`string`]: mod@module::string\n",
            ctx.transform_line(line.into())
        );

        let line = "    [string]: module/string/index.html\n";
        assert_eq!(
            "    [string]: mod@module::string\n",
            ctx.transform_line(line.into())
        );

        assert_eq!(*STD_CTX, ctx);
    }

    #[test]
    fn full_link_is_transformed_crate() {
        let mut ctx = STD_CTX.clone();

        let line = "/// [`string`]: std/string/index.html\n";
        assert_eq!(
            "/// [`string`]: mod@crate::string\n",
            ctx.transform_line(line.into())
        );

        let line = "    /// [string]: std/string/index.html\n";
        assert_eq!(
            "    /// [string]: mod@crate::string\n",
            ctx.transform_line(line.into())
        );

        let line = "[`string`]: std/string/index.html\n";
        assert_eq!(
            "[`string`]: mod@crate::string\n",
            ctx.transform_line(line.into())
        );

        let line = "    [string]: std/string/index.html\n";
        assert_eq!(
            "    [string]: mod@crate::string\n",
            ctx.transform_line(line.into())
        );

        assert_eq!(*STD_CTX, ctx);
    }

    #[test]
    fn full_link_is_transformed_crate_over_super() {
        let mut ctx = STD_CTX.clone();

        let line = "/// [`string`]: ../../std/string/index.html\n";
        assert_eq!(
            "/// [`string`]: mod@crate::string\n",
            ctx.transform_line(line.into())
        );

        let line = "    /// [string]: ../../std/string/index.html\n";
        assert_eq!(
            "    /// [string]: mod@crate::string\n",
            ctx.transform_line(line.into())
        );

        let line = "[`string`]: ../../std/string/index.html\n";
        assert_eq!(
            "[`string`]: mod@crate::string\n",
            ctx.transform_line(line.into())
        );

        let line = "    [string]: ../../std/string/index.html\n";
        assert_eq!(
            "    [string]: mod@crate::string\n",
            ctx.transform_line(line.into())
        );

        assert_eq!(*STD_CTX, ctx);
    }

    #[test]
    fn full_link_is_transformed_not_crate() {
        let mut ctx = CORE_CTX.clone();

        let line = "/// [`string`]: std/string/index.html\n";
        assert_eq!(
            "/// [`string`]: mod@std::string\n",
            ctx.transform_line(line.into())
        );

        let line = "    /// [string]: std/string/index.html\n";
        assert_eq!(
            "    /// [string]: mod@std::string\n",
            ctx.transform_line(line.into())
        );

        let line = "[`string`]: std/string/index.html\n";
        assert_eq!(
            "[`string`]: mod@std::string\n",
            ctx.transform_line(line.into())
        );

        let line = "    [string]: std/string/index.html\n";
        assert_eq!(
            "    [string]: mod@std::string\n",
            ctx.transform_line(line.into())
        );

        assert_eq!(*CORE_CTX, ctx);
    }

    #[test]
    fn full_link_is_transformed_super() {
        let mut ctx = STD_CTX.clone();

        let line = "/// [`string`]: ../../string/index.html\n";
        assert_eq!(
            "/// [`string`]: mod@super::super::string\n",
            ctx.transform_line(line.into())
        );

        let line = "    /// [string]: ../../string/index.html\n";
        assert_eq!(
            "    /// [string]: mod@super::super::string\n",
            ctx.transform_line(line.into())
        );

        let line = "[`string`]: ../../string/index.html\n";
        assert_eq!(
            "[`string`]: mod@super::super::string\n",
            ctx.transform_line(line.into())
        );

        let line = "    [string]: ../../string/index.html\n";
        assert_eq!(
            "    [string]: mod@super::super::string\n",
            ctx.transform_line(line.into())
        );

        assert_eq!(*STD_CTX, ctx);
    }

    #[test]
    fn section_is_kept() {
        let mut ctx = STD_CTX.clone();

        let line = "/// [`string`]: string/index.html#my-section\n";
        assert_eq!(
            "/// [`string`]: mod@string#my-section\n",
            ctx.transform_line(line.into())
        );

        let line = "    /// [string]: string/index.html#my-section\n";
        assert_eq!(
            "    /// [string]: mod@string#my-section\n",
            ctx.transform_line(line.into())
        );

        let line = "[`string`]: string/index.html#my-section\n";
        assert_eq!(
            "[`string`]: mod@string#my-section\n",
            ctx.transform_line(line.into())
        );

        let line = "    [string]: string/index.html#my-section\n";
        assert_eq!(
            "    [string]: mod@string#my-section\n",
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
