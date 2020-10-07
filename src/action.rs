use ansi_term::Color;
use std::fmt;

/// The action taken for a particular line of text.
///
/// This action can then be displayed to show diffs with the `Display` impl or
/// saved somewhere else through the `as_new_line` method.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Action {
    /// `line` was unchanged and no specific action was taken.
    Unchanged { line: String },

    /// `line` was deleted. The position of the line is saved for display.
    Deleted { line: String, pos: usize },

    /// `line` was replaced by a `new` one. As with `Deleted`, the position is
    /// given.
    Replaced {
        line: String,
        new: String,
        pos: usize,
    },
}

impl Action {
    /// Check if the current action is `Action::Unchanged`.
    pub fn is_unchanged(&self) -> bool {
        match self {
            Action::Unchanged { line: _ } => true,
            _ => false,
        }
    }

    /// Returns the new line to add.
    ///
    /// - `Action::Unchanged` returns its line unchanged.
    /// - `Action::Deleted` returns an empty line.
    /// - `Action::Replaced` returns its `new` line.
    pub fn as_new_line(&self) -> &str {
        match self {
            Action::Unchanged { line } => line,
            Action::Deleted { line: _, pos: _ } => "",
            Action::Replaced {
                line: _,
                new,
                pos: _,
            } => new,
        }
    }
}

impl fmt::Display for Action {
    /// Special display that will only write `Deleted` and `Replaced` variants,
    /// unchanged lines are simply ignored.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Unchanged { line: _ } => Ok(()),
            Action::Deleted { line, pos } => {
                write!(f, "{:5}:  \"{}\"", pos, Color::Red.paint(line),)
            }
            Action::Replaced { line, new, pos } => write!(
                f,
                "{:5}:  \"{}\"\n        \"{}\"",
                pos,
                Color::Red.paint(line),
                Color::Green.paint(new)
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_unchanged() {
        assert!(Action::Unchanged {
            line: "line".into()
        }
        .is_unchanged());

        assert!(!Action::Deleted {
            line: "line".into(),
            pos: 3
        }
        .is_unchanged());

        assert!(!Action::Replaced {
            line: "line".into(),
            new: "new".into(),
            pos: 3
        }
        .is_unchanged());
    }

    #[test]
    fn as_new_line() {
        assert_eq!(
            Action::Unchanged {
                line: "line".into()
            }
            .as_new_line(),
            "line"
        );

        assert_eq!(
            Action::Deleted {
                line: "line".into(),
                pos: 3
            }
            .as_new_line(),
            ""
        );

        assert_eq!(
            Action::Replaced {
                line: "line".into(),
                new: "new".into(),
                pos: 3
            }
            .as_new_line(),
            "new"
        );
    }
}
