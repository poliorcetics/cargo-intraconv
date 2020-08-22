use ansi_term::Color;
use std::fmt;
use std::num::NonZeroUsize;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
/// The action taken for a particular line of text.
pub enum Action {
    /// `line` was unchanged and no specific action was taken.
    Unchanged { line: String },

    /// `line` was deleted for `reason`. The position of the line is saved too
    /// for improved display.
    Deleted {
        line: String,
        reason: &'static str,
        pos: NonZeroUsize,
    },

    /// `line` was replaced by a `new` one. As with `Deleted`, the position is
    /// given.
    Replaced {
        line: String,
        new: String,
        pos: NonZeroUsize,
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

    pub fn new_line(self) -> String {
        match self {
            Action::Unchanged { line } => line,
            Action::Deleted {
                line,
                reason: _,
                pos: _,
            } => line,
            Action::Replaced {
                line: _,
                new,
                pos: _,
            } => new,
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Unchanged { line } => write!(f, "{}", line),
            Action::Deleted { line, reason, pos } => write!(
                f,
                "{:5}:  \"{}\"\n        {}",
                pos,
                Color::Red.paint(line),
                Color::Yellow.paint(*reason)
            ),
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
            reason: "reason",
            pos: NonZeroUsize::new(3).unwrap()
        }
        .is_unchanged());

        assert!(!Action::Replaced {
            line: "line".into(),
            new: "new".into(),
            pos: NonZeroUsize::new(3).unwrap()
        }
        .is_unchanged());
    }
}
