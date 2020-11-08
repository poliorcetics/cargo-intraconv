#![allow(unused)]
mod consts;

use std::path::Path;

/// Parts of a markdown link.
///
/// For now only long markdown links are handled correctly.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct Parts<'a> {
    /// Header of the link. This contains everything from the start of the line
    /// to the final `\s` character before the start of the link itself.
    header: &'a str,

    /// The backing link, seen as a path.
    ///
    /// Seeing it as a path (even when its a URL) will help with detection of
    /// false positives (a link to `https://example.com`) and of favored links
    /// (e.g a link to `https://docs.rs/regex`) as well as everything else
    /// (items, associated items, sections, modules, ...) since most
    /// information can be found either in the first or last component of the
    /// link once it has been separated by `/`.
    link: &'a Path,
}

impl<'a> Parts<'a> {
    /// Find the parts of a link in a line (ending with `\n`).
    ///
    /// When the line is not a matching link the passed line is returned in the
    /// `Result::Err` variant.
    fn from_line(line: &'a str) -> Result<Self, &str> {
        let captures = match consts::LINK_TO_TREAT_LONG.captures(line) {
            Some(c) => c,
            None => return Err(line),
        };

        let header = captures
            .name("header")
            .expect("'header' group missing")
            .as_str();
        let link = Path::new(
            captures
                .name("link")
                .expect("'link' group missing")
                .as_str(),
        );

        Ok(Parts { header, link })
    }
}

#[cfg(test)]
mod test {
    use super::Parts;
    use std::path::Path;

    #[test]
    fn parts_from_line_ok() {
        fn helper(line: &str, header: &str, link: &str) {
            assert_eq!(
                Parts::from_line(line).unwrap(),
                Parts {
                    header,
                    link: Path::new(link)
                }
            );
        }

        // Testing spacing.
        helper("[name]:mod1\n", "[name]:", "mod1");
        helper("[name]: mod1\n", "[name]: ", "mod1");
        helper("[name]:  mod1\n", "[name]:  ", "mod1");
        helper("[name]:\tmod1\n", "[name]:\t", "mod1");
        helper("  [name]: mod1\n", "  [name]: ", "mod1");
        helper("\t[name]: mod1\n", "\t[name]: ", "mod1");
        helper("/// [name]: mod1\n", "/// [name]: ", "mod1");
        helper("///  [name]: mod1\n", "///  [name]: ", "mod1");
        helper("///\t[name]: mod1\n", "///\t[name]: ", "mod1");
        helper(" /// [name]: mod1\n", " /// [name]: ", "mod1");
        helper("  /// [name]: mod1\n", "  /// [name]: ", "mod1");
        helper("\t/// [name]: mod1\n", "\t/// [name]: ", "mod1");

        // Testing bangs.
        helper("/// [name]: mod1\n", "/// [name]: ", "mod1");
        helper("//! [name]: mod1\n", "//! [name]: ", "mod1");

        // Testing code in links.
        helper("[`name`]: mod1\n", "[`name`]: ", "mod1");

        // Testing non-HTTP links
        helper("[name]: mod1/\n", "[name]: ", "mod1/");
        helper("[name]: mod1/mod2\n", "[name]: ", "mod1/mod2");
        helper(
            "[name]: mod1/mod2/struct.Type.html\n",
            "[name]: ",
            "mod1/mod2/struct.Type.html",
        );
        helper(
            "[name]: mod1/mod2/struct.Type.html#const.NAME\n",
            "[name]: ",
            "mod1/mod2/struct.Type.html#const.NAME",
        );
        helper(
            "[name]: mod1/mod2/#section\n",
            "[name]: ",
            "mod1/mod2/#section",
        );
        helper(
            "[name]: mod1/mod2#section\n",
            "[name]: ",
            "mod1/mod2#section",
        );
        helper(
            "[name]: mod1/mod2/index.html#section\n",
            "[name]: ",
            "mod1/mod2/index.html#section",
        );
        helper(
            "[name]: mod1/mod2/index.html/#section\n",
            "[name]: ",
            "mod1/mod2/index.html/#section",
        );
        helper("[name]: ../mod1/mod2\n", "[name]: ", "../mod1/mod2");
        helper("[name]: ./../mod1\n", "[name]: ", "./../mod1");

        // Testing HTTP links
        helper(
            "[name]: https://docs.rs/regex/\n",
            "[name]: ",
            "https://docs.rs/regex/",
        );
        helper(
            "[name]: https://docs.rs/regex/1.0.33/regex/mod1/mod2\n",
            "[name]: ",
            "https://docs.rs/regex/1.0.33/regex/mod1/mod2",
        );
        helper(
            "[name]: https://github.com/poliorcetics/cargo-intraconv/issues/21\n",
            "[name]: ",
            "https://github.com/poliorcetics/cargo-intraconv/issues/21",
        );

        // Testing all sorts of characters in the link name.
        helper("[azertyuiop]: mod1\n", "[azertyuiop]: ", "mod1");
        helper("[AZERTYUIOP]: mod1\n", "[AZERTYUIOP]: ", "mod1");
        helper("[@&é\"'(§è!çà)-]: mod1\n", "[@&é\"'(§è!çà)-]: ", "mod1");
        helper("[#1234567890°_]: mod1\n", "[#1234567890°_]: ", "mod1");
        helper("[•ë“‘{¶«¡Çø}—]: mod1\n", "[•ë“‘{¶«¡Çø}—]: ", "mod1");
        helper("[Ÿ´„”’[å»ÛÁØ]–]: mod1\n", "[Ÿ´„”’[å»ÛÁØ]–]: ", "mod1");
        helper("[æÂê®†Úºîœπ]: mod1\n", "[æÂê®†Úºîœπ]: ", "mod1");
        helper("[ÆÅÊ‚™ŸªïŒ∏]: mod1\n", "[ÆÅÊ‚™ŸªïŒ∏]: ", "mod1");
        helper("[‡Ò∂ƒﬁÌÏÈ¬µ]: mod1\n", "[‡Ò∂ƒﬁÌÏÈ¬µ]: ", "mod1");
        helper("[Ω∑∆·ﬂÎÍË|Ó]: mod1\n", "[Ω∑∆·ﬂÎÍË|Ó]: ", "mod1");
        helper("[‹≈©◊ß~]: mod1\n", "[‹≈©◊ß~]: ", "mod1");
        helper("[›⁄¢√∫ı]: mod1\n", "[›⁄¢√∫ı]: ", "mod1");
        helper("[^$ù`,;:=<]: mod1\n", "[^$ù`,;:=<]: ", "mod1");
        helper("[¨*%£?./+>]: mod1\n", "[¨*%£?./+>]: ", "mod1");
        helper("[ô€Ù@∞…÷≠≤]: mod1\n", "[ô€Ù@∞…÷≠≤]: ", "mod1");
        helper("[Ô¥‰#¿•\\±≥]: mod1\n", "[Ô¥‰#¿•\\±≥]: ", "mod1");
    }
}
