use crate::consts;
use crate::Action;
use regex::Captures;
use std::iter::Enumerate;

#[derive(Debug, Eq, PartialEq)]
struct Context {
    krate: String,
    prev_line: Option<String>,
    curr_impl: Option<String>,
    end_impl: String,
}

impl Context {
    fn new(krate: String) -> Self {
        Self {
            krate,
            prev_line: None,
            curr_impl: None,
            end_impl: String::new(),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Line {
    action: Action,
    treatment_is_pending: bool,
}

impl Line {
    fn treated(action: Action) -> Self {
        Self {
            action,
            treatment_is_pending: false,
        }
    }

    fn pending(action: Action) -> Self {
        Self {
            action,
            treatment_is_pending: true,
        }
    }
}

#[derive(Debug)]
pub struct Process<I> {
    treated: Vec<Line>,
    pending: Vec<usize>,
    incoming: Enumerate<I>,
    ctx: Context,
}

impl<I: Iterator<Item = String>> Process<I> {
    pub fn new(lines: I, krate: String) -> Self {
        Self {
            treated: Vec::new(),
            pending: Vec::new(),
            incoming: lines.enumerate(),
            ctx: Context::new(krate),
        }
    }

    pub fn process_next_line(&mut self) -> Option<()> {
        let (pos, curr_line) = self.incoming.next()?;
        let pos = pos + 1;

        // Start by taking the previous line (when there is one) out of the
        // context to check some deletion case.
        if let Some(prev_line) = self.ctx.prev_line.take() {
            let prev_is_empty = consts::EMPTY_DOC_COMMENT.is_match(&prev_line);

            if prev_is_empty && consts::EMPTY_DOC_COMMENT.is_match(&curr_line) {
                self.treated.push(Line::treated(Action::Deleted {
                    line: curr_line,
                    reason: "Consecutives empty lines",
                    pos: pos,
                }));

                return Some(());
            } else if prev_is_empty && !consts::IS_DOC_COMMENT_LINE.is_match(&curr_line) {
                // Coming here means there was at least one element in the treated lines.
                let last = self.treated.len() - 1;
                self.treated[last] = Line::treated(Action::Deleted {
                    line: prev_line,
                    reason: "Empty comment line at the end of a comment",
                    pos: pos - 1,
                });

                return Some(());
            }

            // If no deletion took place, put back the previous line in the
            // context.
            self.ctx.prev_line = Some(prev_line)
        }

        self.treated.push(Line::treated(Action::Unchanged {
            line: curr_line.clone(),
        }));
        self.ctx.prev_line = Some(curr_line);

        Some(())
    }
}

/// Handles items matching the `ITEM_LINK` regex.
fn comment_link(captures: Captures, pos: usize, krate: &str) -> Line {
    // Preparing the new line, most intra-doc comments will fit in 64 char.
    let mut new = String::with_capacity(64);

    // Building the base of the link, which is always the same.
    new.push_str(captures.name("link_name").unwrap().as_str());

    // First elements like the crate or `super::`
    if let Some(root) = captures.name("crate") {
        let root = root.as_str();
        new.push_str(if root == krate { "crate" } else { root });
        new.push_str("::");
    } else if let Some(supers) = captures.name("supers") {
        let supers = supers.as_str();
        let count = supers.matches("/").count();
        // This way we won't allocate a string only to immediately drop it
        for _ in 0..count {
            new.push_str("super::");
        }
    }

    // Intermediates element like a path through modules.
    if let Some(intermediates) = captures.name("intermediates") {
        let intermediates: &str = intermediates.as_str();
        if intermediates.starts_with("http") {
            return Line::treated(Action::Unchanged {
                line: captures.get(0).unwrap().as_str().into(),
            });
        }
        if intermediates != "./" {
            new.push_str(&intermediates.replace("/", "::"));
        }
    }

    new.push_str(captures.name("elem2").unwrap().as_str());

    // Additional linked elements like a method or a variant
    if let Some(additional) = captures.name("additional") {
        new.push_str("::");
        new.push_str(additional.as_str());
    }

    // Check if the link has become a local path
    if let Some(local) = consts::LOCAL_PATH.captures(&new) {
        if local.name("elem").unwrap().as_str() == local.name("elem2").unwrap().as_str() {
            return Line::treated(Action::Deleted {
                line: captures.get(0).unwrap().as_str().into(),
                reason: "Local path",
                pos,
            });
        }
    }

    Line::treated(Action::Replaced {
        line: captures.get(0).unwrap().as_str().into(),
        new,
        pos,
    })
}

#[cfg(test)]
mod context {
    use super::*;

    #[test]
    fn new() {
        let ctx = Context::new("core".into());
        assert_eq!(ctx.krate, "core");
        assert_eq!(ctx.prev_line, None);
        assert_eq!(ctx.curr_impl, None);
        assert_eq!(ctx.end_impl, "");
    }
}

#[cfg(test)]
mod line {
    use super::*;

    #[test]
    fn treated() {
        let action = Action::Deleted {
            line: "Line".into(),
            reason: "remove things",
            pos: 1,
        };

        assert_eq!(
            Line::treated(action.clone()),
            Line {
                action,
                treatment_is_pending: false
            }
        )
    }

    #[test]
    fn pending() {
        let action = Action::Deleted {
            line: "Line".into(),
            reason: "remove things",
            pos: 1,
        };

        assert_eq!(
            Line::pending(action.clone()),
            Line {
                action,
                treatment_is_pending: true
            }
        )
    }
}

#[cfg(test)]
mod process {
    use super::*;

    #[test]
    fn new() {
        let ps = Process::new(Some("line".into()).into_iter(), "core".into());
        assert!(ps.treated.is_empty());
        assert!(ps.pending.is_empty());
        assert_eq!(
            ps.incoming.map(|e| e.1).collect::<Vec<_>>(),
            ["line".to_string()]
        );
        assert_eq!(ps.ctx, Context::new("core".into()));

        let ps = Process::new(None.into_iter(), "core".into());
        assert!(ps.treated.is_empty());
        assert!(ps.pending.is_empty());
        assert!(ps.incoming.collect::<Vec<_>>().is_empty());
        assert_eq!(ps.ctx, Context::new("core".into()));
    }

    #[test]
    fn process_empty_lines() {
        let mut ps = Process::new(None.into_iter(), "core".into());

        assert_eq!(ps.process_next_line(), None);

        assert!(ps.treated.is_empty());
        assert!(ps.pending.is_empty());
        assert!(ps.incoming.collect::<Vec<_>>().is_empty());
        assert_eq!(ps.ctx, Context::new("core".into()));
    }

    #[test]
    fn process_empty_doc_comment() {
        let mut ps = Process::new(Some("///\n".into()).into_iter(), "core".into());

        ps.treated = vec![Line::treated(Action::Unchanged {
            line: "///\n".into(),
        })];
        ps.ctx.prev_line = Some("///\n".into());

        assert_eq!(ps.process_next_line(), Some(()));

        assert_eq!(
            ps.treated,
            [
                Line::treated(Action::Unchanged {
                    line: "///\n".into()
                }),
                Line::treated(Action::Deleted {
                    line: "///\n".into(),
                    reason: "Consecutives empty lines",
                    pos: 1,
                })
            ]
        );
        assert!(ps.pending.is_empty());
        assert!(ps.incoming.collect::<Vec<_>>().is_empty());
        assert_eq!(ps.ctx, Context::new("core".into()));
    }

    #[test]
    fn process_empty_doc_comment_at_end_of_block() {
        let mut ps = Process::new(
            vec!["///\n".into(), "some text\n".into()].into_iter(),
            "core".into(),
        );

        assert_eq!(ps.process_next_line(), Some(()));
        assert_eq!(ps.process_next_line(), Some(()));

        assert_eq!(
            ps.treated,
            [Line::treated(Action::Deleted {
                line: "///\n".into(),
                reason: "Empty comment line at the end of a comment",
                pos: 1,
            })]
        );
        assert!(ps.pending.is_empty());
        assert!(ps.incoming.collect::<Vec<_>>().is_empty());
        assert_eq!(ps.ctx, Context::new("core".into()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_link_different_crate() {
        let krate = "other";

        let lines = [
            // Replaced links
            "/// [`Write`]: ../../std/io/trait.Write.html\n",
            "/// [`TcpStream`]: ../../std/net/struct.TcpStream.html\n",
            "/// [`TcpStream::write`]: ../../std/net/struct.TcpStream.html#method.write\n",
            "/// [`SocketAddr`]: ../../std/net/enum.SocketAddr.html\n",
            "[`Write`]: ../../std/io/trait.Write.html\n",
            "[`TcpStream`]: ../../std/net/struct.TcpStream.html\n",
            "[`TcpStream::write`]: ../../std/net/struct.TcpStream.html#method.write\n",
            "[`SocketAddr`]: ../../std/net/enum.SocketAddr.html\n",
            // Deleted links
            "/// [`IpAddr`]: enum.IpAddr.html\n",
            "/// [`IpAddr::method_1`]: enum.IpAddr.html#method.method_1\n",
            "[`IpAddr`]: enum.IpAddr.html\n",
            "[`IpAddr::method_1`]: enum.IpAddr.html#method.method_1\n",
            // Unchanged links
            "/// [Arc direct link]: https://doc.rust-lang.org/stable/std/sync/struct.Arc.html\n",
            "[Arc direct link]: https://doc.rust-lang.org/stable/std/sync/struct.Arc.html\n",
        ];

        let res = [
            // Replaced links
            Line::treated(Action::Replaced {
                line: "/// [`Write`]: ../../std/io/trait.Write.html\n".into(),
                new: "/// [`Write`]: std::io::Write".into(),
                pos: 1,
            }),
            Line::treated(Action::Replaced {
                line: "/// [`TcpStream`]: ../../std/net/struct.TcpStream.html\n".into(),
                new: "/// [`TcpStream`]: std::net::TcpStream".into(),
                pos: 2,
            }),
            Line::treated(Action::Replaced {
                line: "/// [`TcpStream::write`]: ../../std/net/struct.TcpStream.html#method.write\n".into(),
                new: "/// [`TcpStream::write`]: std::net::TcpStream::write".into(),
                pos: 3,
            }),
            Line::treated(Action::Replaced {
                line: "/// [`SocketAddr`]: ../../std/net/enum.SocketAddr.html\n".into(),
                new: "/// [`SocketAddr`]: std::net::SocketAddr".into(),
                pos: 4,
            }),
            Line::treated(Action::Replaced {
                line: "[`Write`]: ../../std/io/trait.Write.html\n".into(),
                new: "[`Write`]: std::io::Write".into(),
                pos: 5,
            }),
            Line::treated(Action::Replaced {
                line: "[`TcpStream`]: ../../std/net/struct.TcpStream.html\n".into(),
                new: "[`TcpStream`]: std::net::TcpStream".into(),
                pos: 6,
            }),
            Line::treated(Action::Replaced {
                line: "[`TcpStream::write`]: ../../std/net/struct.TcpStream.html#method.write\n".into(),
                new: "[`TcpStream::write`]: std::net::TcpStream::write".into(),
                pos: 7,
            }),
            Line::treated(Action::Replaced {
                line: "[`SocketAddr`]: ../../std/net/enum.SocketAddr.html\n".into(),
                new: "[`SocketAddr`]: std::net::SocketAddr".into(),
                pos: 8,
            }),
            // Deleted links
            Line::treated(Action::Deleted {
                line: "/// [`IpAddr`]: enum.IpAddr.html\n".into(),
                reason: "Local path",
                pos: 9,
            }),
            Line::treated(Action::Deleted {
                line: "/// [`IpAddr::method_1`]: enum.IpAddr.html#method.method_1\n".into(),
                reason: "Local path",
                pos: 10,
            }),
            Line::treated(Action::Deleted {
                line: "[`IpAddr`]: enum.IpAddr.html\n".into(),
                reason: "Local path",
                pos: 11,
            }),
            Line::treated(Action::Deleted {
                line: "[`IpAddr::method_1`]: enum.IpAddr.html#method.method_1\n".into(),
                reason: "Local path",
                pos: 12,
            }),
            // Unchanged links
            Line::treated(Action::Unchanged {
                line: "/// [Arc direct link]: https://doc.rust-lang.org/stable/std/sync/struct.Arc.html\n".into(),
            }),
            Line::treated(Action::Unchanged {
                line: "[Arc direct link]: https://doc.rust-lang.org/stable/std/sync/struct.Arc.html\n".into(),
            }),
        ];

        for (raw_pos, &line) in lines.iter().enumerate() {
            let captures = consts::ITEM_LINK.captures(line).unwrap();
            assert_eq!(comment_link(captures, raw_pos + 1, krate), res[raw_pos]);
        }
    }

    #[test]
    fn test_comment_link_same_crate() {
        let krate = "std";

        let lines = [
            // Replaced links
            "/// [`Write`]: ../../std/io/trait.Write.html\n",
            "/// [`TcpStream`]: ../../std/net/struct.TcpStream.html\n",
            "/// [`TcpStream::write`]: ../../std/net/struct.TcpStream.html#method.write\n",
            "/// [`SocketAddr`]: ../../std/net/enum.SocketAddr.html\n",
            "[`Write`]: ../../std/io/trait.Write.html\n",
            "[`TcpStream`]: ../../std/net/struct.TcpStream.html\n",
            "[`TcpStream::write`]: ../../std/net/struct.TcpStream.html#method.write\n",
            "[`SocketAddr`]: ../../std/net/enum.SocketAddr.html\n",
            // Deleted links
            "/// [`IpAddr`]: enum.IpAddr.html\n",
            "/// [`IpAddr::method_1`]: enum.IpAddr.html#method.method_1\n",
            "[`IpAddr`]: enum.IpAddr.html\n",
            "[`IpAddr::method_1`]: enum.IpAddr.html#method.method_1\n",
            // Unchanged links
            "/// [Arc direct link]: https://doc.rust-lang.org/stable/std/sync/struct.Arc.html\n",
            "[Arc direct link]: https://doc.rust-lang.org/stable/std/sync/struct.Arc.html\n",
        ];

        let res = [
            // Replaced links
            Line::treated(Action::Replaced {
                line: "/// [`Write`]: ../../std/io/trait.Write.html\n".into(),
                new: "/// [`Write`]: crate::io::Write".into(),
                pos: 1,
            }),
            Line::treated(Action::Replaced {
                line: "/// [`TcpStream`]: ../../std/net/struct.TcpStream.html\n".into(),
                new: "/// [`TcpStream`]: crate::net::TcpStream".into(),
                pos: 2,
            }),
            Line::treated(Action::Replaced {
                line: "/// [`TcpStream::write`]: ../../std/net/struct.TcpStream.html#method.write\n".into(),
                new: "/// [`TcpStream::write`]: crate::net::TcpStream::write".into(),
                pos: 3,
            }),
            Line::treated(Action::Replaced {
                line: "/// [`SocketAddr`]: ../../std/net/enum.SocketAddr.html\n".into(),
                new: "/// [`SocketAddr`]: crate::net::SocketAddr".into(),
                pos: 4,
            }),
            Line::treated(Action::Replaced {
                line: "[`Write`]: ../../std/io/trait.Write.html\n".into(),
                new: "[`Write`]: crate::io::Write".into(),
                pos: 5,
            }),
            Line::treated(Action::Replaced {
                line: "[`TcpStream`]: ../../std/net/struct.TcpStream.html\n".into(),
                new: "[`TcpStream`]: crate::net::TcpStream".into(),
                pos: 6,
            }),
            Line::treated(Action::Replaced {
                line: "[`TcpStream::write`]: ../../std/net/struct.TcpStream.html#method.write\n".into(),
                new: "[`TcpStream::write`]: crate::net::TcpStream::write".into(),
                pos: 7,
            }),
            Line::treated(Action::Replaced {
                line: "[`SocketAddr`]: ../../std/net/enum.SocketAddr.html\n".into(),
                new: "[`SocketAddr`]: crate::net::SocketAddr".into(),
                pos: 8,
            }),
            // Deleted links
            Line::treated(Action::Deleted {
                line: "/// [`IpAddr`]: enum.IpAddr.html\n".into(),
                reason: "Local path",
                pos: 9,
            }),
            Line::treated(Action::Deleted {
                line: "/// [`IpAddr::method_1`]: enum.IpAddr.html#method.method_1\n".into(),
                reason: "Local path",
                pos: 10,
            }),
            Line::treated(Action::Deleted {
                line: "[`IpAddr`]: enum.IpAddr.html\n".into(),
                reason: "Local path",
                pos: 11,
            }),
            Line::treated(Action::Deleted {
                line: "[`IpAddr::method_1`]: enum.IpAddr.html#method.method_1\n".into(),
                reason: "Local path",
                pos: 12,
            }),
            // Unchanged links
            Line::treated(Action::Unchanged {
                line: "/// [Arc direct link]: https://doc.rust-lang.org/stable/std/sync/struct.Arc.html\n".into(),
            }),
            Line::treated(Action::Unchanged {
                line: "[Arc direct link]: https://doc.rust-lang.org/stable/std/sync/struct.Arc.html\n".into(),
            }),
        ];

        for (raw_pos, &line) in lines.iter().enumerate() {
            let captures = consts::ITEM_LINK.captures(line).unwrap();
            assert_eq!(comment_link(captures, raw_pos + 1, krate), res[raw_pos]);
        }
    }
}
