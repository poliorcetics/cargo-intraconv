use crate::{Action, TYPE_BLOCK_START};
use std::io::{self, BufRead};

/// Context for the check. It notably contains informations about the crate and
/// the current type (e.g, for `#method.name` links).
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ConversionContext {
    /// Options to apply when transforming a single file.
    options: crate::ConversionOptions,

    /// Current line number.
    pos: usize,

    /// Name of the type that is `Self` for the current block.
    curr_type_block: Option<String>,

    /// End of the current block for `Self` (if any).
    end_type_block: String,

    /// Line at which the current type block was declared.
    ///
    /// A type block cannot end before this line (obviously).
    /// The line numbers start at one, to match those of the file being
    /// transformed.
    type_block_line: usize,

    // NOTE: at the moment nested type blocks are not handled.
    /// All types blocks known to the context.
    ///
    /// Calling `pop` on the `Vec` must give the next type block (if there is
    /// one).
    ///
    /// The tuple is (type block, end of type block, line of type block)
    type_blocks: Vec<(String, String, usize)>,
}

impl ConversionContext {
    /// Creates a new `ConversionContext` with the given options.
    pub fn with_options(options: crate::ConversionOptions) -> Self {
        Self {
            options,
            pos: 0,
            curr_type_block: None,
            end_type_block: String::new(),
            type_block_line: usize::MAX,
            type_blocks: Vec::new(),
        }
    }

    /// The type currently active or an empty string.
    pub fn current_type_block(&self) -> Option<&str> {
        self.curr_type_block.as_deref()
    }

    /// Reference to the options for the context.
    pub fn options(&self) -> &crate::ConversionOptions {
        &self.options
    }

    #[cfg(test)]
    pub(crate) fn set_current_type_block(&mut self, ctb: String) {
        self.curr_type_block = Some(ctb);
    }

    /// Iterates over a `BufRead` reader to find the links and transform them.
    ///
    /// This function will make only one pass over the entire buffer,
    /// erroring if it fails to read a line.
    pub fn transform_file<R: BufRead>(&mut self, reader: R) -> io::Result<Vec<Action>> {
        // Reset the state before handling the file.
        self.pos = 0;
        self.curr_type_block = None;
        self.end_type_block = String::new();
        self.type_blocks.clear();

        let mut lines = Vec::new();
        for l in reader.lines() {
            lines.push(l?);
        }

        self.type_blocks = find_type_blocks(lines.iter());

        let mut actions = Vec::with_capacity(lines.len());
        for line in lines.into_iter() {
            self.pos += 1;
            actions.push(self.transform_line(line));
        }

        Ok(actions)
    }

    /// Transform a single line, returning the action.
    fn transform_line(&mut self, line: String) -> Action {
        // Updating the currently active `Self` type.
        if self.curr_type_block.is_none() {
            if let Some((curr_type, end, ln)) = self.type_blocks.pop() {
                self.curr_type_block = Some(curr_type);
                self.end_type_block = end;
                self.type_block_line = ln;
            }
        }

        // When reaching the end of the current type block, update the context to
        // reflect it. Updating the `self.type_block_line` value shouldn't
        // be necessary and it is done for clarity and consistency, just like
        // `self.end_type_block`.
        if self.curr_type_block.is_some()
            && line.starts_with(&self.end_type_block)
            && self.pos >= self.type_block_line
        {
            self.curr_type_block = None;
            self.end_type_block.clear();
            self.type_block_line = usize::MAX;
        }

        let candidate = match crate::Candidate::from_line(&line) {
            Ok(c) => c,
            Err(_) => {
                let mut line = line;
                line.push('\n');
                return Action::Unchanged { line };
            }
        };

        let transformed = match candidate.transform(self) {
            Ok(t) => t,
            Err(_) => {
                let mut line = line;
                line.push('\n');
                return Action::Unchanged { line };
            }
        };

        if let Some(captures) = crate::LOCAL_PATH_LONG.captures(&transformed) {
            let header = captures
                .name("header")
                .expect("Must be present to match")
                .as_str();
            let link = captures
                .name("link")
                .expect("Must be present to match")
                .as_str();

            if header == link {
                return Action::Deleted {
                    line,
                    pos: self.pos,
                };
            }
        }

        if line == transformed {
            let mut line = line;
            line.push('\n');
            Action::Unchanged { line }
        } else {
            let mut transformed = transformed;
            transformed.push('\n');
            Action::Replaced {
                line,
                new: transformed,
                pos: self.pos,
            }
        }
    }
}

/// Returns the reversed list of type blocks found in the given iterator.
///
/// The returned values are `(type name, end marker, starting line of the type block)`.
/// The `starting line` value is found by enumerating over the iterator and
/// adding `1` to the index.
fn find_type_blocks<S, I>(lines: I) -> Vec<(String, String, usize)>
where
    S: AsRef<str>,
    I: Iterator<Item = S>,
{
    let mut type_blocks = Vec::new();

    for (ln, line) in lines.enumerate() {
        let line = line.as_ref();
        // Early return on context change too, after updating the context.
        if let Some(captures) = TYPE_BLOCK_START.captures(line) {
            let ty = captures.name("type").unwrap().as_str().into();
            let end = if line.ends_with(";\n") || line.ends_with("}\n") {
                '\n'.into()
            } else {
                // When the item is not simple we try to compute what will be
                // the end of the block.
                let mut s = captures.name("spaces").unwrap().as_str().to_string();
                s.reserve(1);

                if captures.name("parenthese").is_some() {
                    s.push(')');
                } else {
                    s.push('}');
                }

                s
            };

            type_blocks.push((ty, end, ln + 1));
        }
    }

    type_blocks.reverse();
    type_blocks
}

#[cfg(test)]
mod tests;
