use crate::Action;

use std::borrow::Cow;
use std::path::Path;

/// A markdown link that has the right format to be transformed to an intra-doc
/// link.
///
/// Since Rust only allows identifiers in the ASCII range, non ASCII link will
/// automatically fail their conversion, either in `Candidate::from_line` or
/// `Candidate::transform`.
///
/// For now only long markdown links are handled correctly.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Candidate<'a> {
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

impl<'a> Candidate<'a> {
    /// Find the parts of a link in a line (ending with `\n`).
    ///
    /// When the line cannot be an intra-doc link candidate the passed line is
    /// returned in the `Result::Err` variant.
    fn from_line<S>(line: &'a S) -> Result<Self, &'a S>
    where
        S: AsRef<std::ffi::OsStr> + ?Sized + 'a,
    {
        let string = line.as_ref().to_str().ok_or(line)?;
        let captures = crate::LINK_TO_TREAT_LONG.captures(string).ok_or(line)?;

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

        // Absolute paths cannot be intra-doc links.
        if link.is_absolute() {
            Err(line)
        } else {
            Ok(Candidate { header, link })
        }
    }

    /// Apply the transformation based on the given context.
    ///
    /// If the transformation does not modify the link (e.g.: it is an HTTP
    /// one that is not favored), the operation will make zero allocations.
    ///
    /// When the link is transformed, the function will try to do as little
    /// allocations as possible.
    pub fn transform(self, ctx: &crate::ConversionContext) -> Action {
        todo!("Apply transformation");
    }
}

#[test]
fn candidate_from_line_ok() {
    fn helper(line: &str, header: &str, link: &str) {
        assert_eq!(
            Candidate::from_line(line).unwrap(),
            Candidate {
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

#[test]
fn candidate_from_line_err() {
    fn helper(line: &str) {
        assert_eq!(line, Candidate::from_line(line).unwrap_err());
    }

    helper("not a link");
    helper("[name]: /absolute/path");
    helper("[name]: intra::doc::link");
}
