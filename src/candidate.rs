use std::ffi::OsStr;
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
    inner: CandidateInner<'a>,
}

impl<'a> Candidate<'a> {
    /// Find the parts of a link in a line (ending with `\n` or not).
    ///
    /// When the line cannot be an intra-doc link candidate the passed line is
    /// returned in the `Result::Err` variant.
    pub fn from_line<S>(line: &'a S) -> Option<Self>
    where
        S: AsRef<OsStr> + ?Sized + 'a,
    {
        let inner = CandidateInner::from_line(line)?;
        Some(Self { inner })
    }

    /// Apply the transformation based on the given context.
    pub fn transform(self, ctx: &crate::ConversionContext) -> Option<String> {
        self.inner.transform(ctx)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum CandidateInner<'a> {
    Long {
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
    },
    Short {
        orig: &'a str,
    },
}

impl<'a> CandidateInner<'a> {
    fn from_line<S>(line: &'a S) -> Option<Self>
    where
        S: AsRef<OsStr> + ?Sized + 'a,
    {
        let string = line.as_ref().to_str()?;
        if let Some(captures) = crate::LINK_TO_TREAT_LONG.captures(string) {
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
                None
            } else {
                Some(Self::Long { header, link })
            }
        } else if crate::LINK_TO_TREAT_SHORT.is_match(string) {
            Some(Self::Short { orig: string })
        } else {
            None
        }
    }

    fn transform(self, ctx: &crate::ConversionContext) -> Option<String> {
        match self {
            Self::Long { header, link } => {
                if ctx.options().is_ignored(header, link) {
                    return None;
                }

                let parts = crate::link_parts::link_parts(link, ctx.options()).ok()?;
                let link = parts.transform(ctx);
                Some(format!("{h}{l}", h = header, l = link))
            }
            Self::Short { orig } => {
                let replaced =
                    crate::LINK_TO_TREAT_SHORT.replace_all(orig, |cap: &regex::Captures| {
                        let header = cap.name("header").expect("'header' group missing").as_str();
                        let link =
                            Path::new(cap.name("link").expect("'link' group missing").as_str());

                        if ctx.options().is_ignored(header, link) {
                            return cap.get(0).unwrap().as_str().to_string();
                        }

                        let name = cap.name("name").expect("'name' group missing").as_str();
                        let c1 = cap.name("c1").map(|x| x.as_str()).unwrap_or("");
                        let c2 = cap.name("c2").map(|x| x.as_str()).unwrap_or("");

                        if link.is_absolute() {
                            // UNWRAP: full match is always successul if a Captures
                            // was constructed.
                            return cap.get(0).unwrap().as_str().to_string();
                        }

                        let parts = match crate::link_parts::link_parts(link, ctx.options()) {
                            // UNWRAP: see above.
                            Err(_) => return cap.get(0).unwrap().as_str().to_string(),
                            Ok(p) => p,
                        };

                        let link = parts.transform(ctx);
                        let mut res = format!("[{c1}{n}{c2}]", c1 = c1, c2 = c2, n = name);
                        if link != name {
                            res.push('(');
                            res.push_str(&link);
                            res.push(')');
                        }
                        res
                    });

                Some(replaced.into_owned())
            }
        }
    }
}

#[cfg(test)]
mod tests;
