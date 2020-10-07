use crate::Action;
use crate::consts;
use regex::Captures;
use std::fmt::Write;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead as _, BufReader, Read, Write as _};
use std::num::NonZeroUsize;
use std::path::PathBuf;

/// Find the elements to transform inside the file described by `path`. The root
/// crate is considered to be `krate`. If `apply` is `true`, the changes will be
/// collected and the whole file at `path` will be rewritten to include them.
pub fn handle_path(path: &PathBuf, krate: &str, apply: bool) {
    // First display the path of the file that is about to be opened and tested.
    let path_display = path.display().to_string();
    println!("{}", &path_display);
    // TODO: Not always perfect because of unicode, fix this.
    println!("{}\n", "=".repeat(path_display.len()));

    // Then open the file, reporting if it fails.
    let file = match File::open(path) {
        Ok(file) => BufReader::new(file),
        Err(err) => {
            eprintln!("Failed to open file '{}' for read: {}", &path_display, err);
            return;
        }
    };

    // Then apply the regexes to search for links.
    let lines = match search_links(file, krate) {
        Ok(lines) => lines,
        Err(err) => {
            eprintln!("Failed to handle file '{}': {}", &path_display, err);
            return;
        }
    };

    // Do not allocate when unecessary.
    let mut string = if apply {
        String::with_capacity(64 * lines.len())
    } else {
        String::new()
    };

    // Display the changes that can be made.
    for l in lines {
        if !l.is_unchanged() {
            println!("{}\n", l);
        }
        if apply {
            write!(string, "{}\n", l.as_new_line()).unwrap();
        }
    }

    // If the changes are just displayed, not applied, stop execution here.
    if !apply {
        return;
    }

    let mut file = match OpenOptions::new().write(true).truncate(true).open(path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Failed to open file '{}' for write: {}", &path_display, err);
            return;
        }
    };

    match write!(file, "{}", string) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Failed to write to '{}': {}", &path_display, err);
            return;
        }
    };
}

fn search_links<R: Read>(file: BufReader<R>, krate: &str) -> io::Result<Vec<Action>> {
    let mut lines = Vec::new();
    let mut curr_impl = None;
    let mut end_impl = String::new();

    for (raw_pos, curr_line) in file.lines().enumerate() {
        // SAFETY: `raw_pos >= 0` so `raw_pos + 1 >= 1`.
        let pos = unsafe { NonZeroUsize::new_unchecked(raw_pos + 1) };
        let curr_line = curr_line?.trim_end().to_string();

        if let Some(Action::Unchanged { line: prev_line }) = lines.last() {
            if consts::EMPTY_DOC_COMMENT.is_match(prev_line) {
                if consts::EMPTY_DOC_COMMENT.is_match(&curr_line) {
                    lines.push(Action::Deleted {
                        line: curr_line,
                        reason: "Consecutives empty comment lines",
                        pos,
                    });
                    continue;
                } else if !consts::IS_DOC_COMMENT_LINE.is_match(&curr_line) {
                    let i = lines.len() - 1;
                    lines[i] = Action::Deleted {
                        line: prev_line.clone(),
                        reason: "Empty comment line at the end of a comment",
                        // SAFETY: for this to happen there must be a previous
                        // line so `raw_pos` is at least 1.
                        pos: unsafe { NonZeroUsize::new_unchecked(raw_pos) },
                    };
                    continue;
                }
            }
        }

        if let Some(captures) = consts::ITEM_LINK.captures(&curr_line) {
            lines.push(comment_link(captures, curr_line.clone(), pos, krate));
            continue;
        }

        if let Some(captures) = consts::MODULE_LINK.captures(&curr_line) {
            lines.push(module_link(captures, curr_line.clone(), pos, krate));
            continue;
        }

        if let Some(captures) = consts::IMPL_START.captures(&curr_line) {
            end_impl.clear();
            end_impl.push_str(captures.name("spaces").unwrap().as_str());
            end_impl.push('}');
            curr_impl = Some(captures.name("type").unwrap().as_str().to_string());
        }

        if curr_line == end_impl {
            curr_impl = None;
            end_impl.clear();
        }

        if let Some(ref curr_impl) = curr_impl {
            if let Some(captures) = consts::METHOD_ANCHOR.captures(&curr_line) {
                lines.push(method_anchor(captures, curr_line.clone(), pos, curr_impl));
                continue;
            }
        }

        // TODO: in `rust/library/core/src/num/mod.rs`, lines 23{29, 30} there
        // are links that are not caught. They should be for now I don't catch
        // links when the relevant block comes later. This is a bug and will be
        // fixed in the future.

        lines.push(Action::Unchanged { line: curr_line });
    }

    Ok(lines)
}

fn comment_link(captures: Captures, line: String, pos: NonZeroUsize, krate: &str) -> Action {
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
        let mut intermediates: &str = intermediates.as_str();
        if intermediates.starts_with("http") {
            return Action::Unchanged { line };
        }
        if intermediates.starts_with("./") {
            intermediates = &intermediates[2..];
        }
        new.push_str(&intermediates.replace("/", "::"));
    }

    new.push_str(captures.name("elem2").unwrap().as_str());

    // Additional linked elements like a method or a variant
    if let Some(additional) = captures.name("additional") {
        new.push_str("::");
        new.push_str(additional.as_str());
    }

    // Check if the link has become a local path
    if let Some(local) = consts::LOCAL_PATH.captures(&new) {
        if local.name("elem") == local.name("elem2") {
            return Action::Deleted {
                line,
                reason: "Local path",
                pos,
            };
        }
    }

    Action::Replaced { line, new, pos }
}

fn module_link(captures: Captures, line: String, pos: NonZeroUsize, krate: &str) -> Action {
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

    if let Some(mods) = captures.name("mods") {
        let mods = mods.as_str();
        let mods = if mods.starts_with("./") {
            &mods[2..]
        } else {
            &mods
        };
        new.push_str(mods.replace("/", "::").trim_end_matches("::"));
    }

    // Check if the link has become a local path
    if let Some(local) = consts::LOCAL_PATH.captures(&new) {
        if local.name("elem") == local.name("elem2") {
            return Action::Deleted {
                line,
                reason: "Local path",
                pos,
            };
        }
    }

    Action::Replaced { line, new, pos }
}

fn method_anchor(captures: Captures, line: String, pos: NonZeroUsize, curr_impl: &str) -> Action {
    let spaces = captures.name("link_name").unwrap().as_str();
    let additional = captures.name("additional").unwrap().as_str();

    Action::Replaced {
        line,
        new: format!("{}{}::{}", spaces, curr_impl, additional),
        pos,
    }
}
