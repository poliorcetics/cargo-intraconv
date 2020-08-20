use ansi_term::Color;
use argh::FromArgs;
use lazy_static::lazy_static;
use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::PathBuf;

/// Converter from path-based links to intra-doc links for the `rust-lang/rust`
/// project.
///
/// This is not perfect and the modified files should still be reviewed after
/// running it.
///
/// By default it will only print the changes and not apply them, use `-a`
/// (`--apply`) to write them.
///
/// If you are modifying `core` or `alloc` instead of `std`, you can pass the
/// `-c core` (`--crate core`) flag to mark the change in the root crate.
#[derive(FromArgs, Debug)]
struct Args {
    /// root crate (one of `std`, `core` or `alloc`)
    #[argh(
        option,
        long = "crate",
        short = 'c',
        from_str_fn(check_krate),
        default = "\"std\""
    )]
    krate: &'static str,

    /// apply the proposed changes
    #[argh(switch, short = 'a')]
    apply: bool,

    /// files to search links in
    #[argh(positional)]
    paths: Vec<PathBuf>,
}

fn check_krate(krate: &str) -> Result<&'static str, String> {
    match krate {
        "std" => Ok("std"),
        "core" => Ok("core"),
        "alloc" => Ok("alloc"),
        _ => Err("Valid crate options are `std`, `core` and `alloc`.".into()),
    }
}

fn main() {
    let args: Args = argh::from_env();

    args.paths.iter().for_each(|path| handle_path(path, args.krate));
}

fn handle_path(path: &PathBuf, krate: &str) {
    // First display the path of the file that is about to be opened and tested.
    let path_display = path.display().to_string();
    println!("{}", &path_display);
    // TODO: Not always perfect because of unicode, fix this.
    println!("{}\n", "=".repeat(path_display.len()));

    // Then open the file, reporting if it fails.
    let file = match File::open(path) {
        Ok(file) => BufReader::new(file),
        Err(err) => {
            eprintln!("Failed to open file '{}': {}", &path_display, err);
            return;
        }
    };

    // Then apply the regexes to search for links.
    match search_links(file, krate) {
        Ok(_) => (),
        Err(err) => {
            eprintln!("Failed to handle file '{}': {}", &path_display, err);
            return;
        }
    }

    // TODO: use the `apply` flag to modify the file if need be.
}

lazy_static! {
    // An empty doc comment.
    static ref EMPTY_DOC_COMMENT: Regex = Regex::new(r"^\s*//[!/]$").unwrap();

    // Used to detect doc comment lines, empty or not. This is the same regex
    // as `EMPTY_DOC_COMMENT` without the ending `$`.
    static ref IS_DOC_COMMENT_LINE: Regex = Regex::new(r"^\s*//[!/]").unwrap();

    // Will search for a doc comment link and be used to check if the two
    // elements are the same, indicating a local path.
    static ref LOCAL_PATH: Regex = Regex::new(concat!(
        r"^\s*//[!/] ",
        r"\[`?(?P<elem>.*?)`?\]: ",
        r"(?P<elem2>.*)$",
    )).unwrap();

}

fn search_links<R: Read>(file: BufReader<R>, krate: &str) -> io::Result<Vec<String>> {
    lazy_static! {
        static ref IMPL_START: Regex = Regex::new(concat!(
            r"^(?P<spaces>\s*)",
            r"(?:impl|(?:pub(?:\(.+\))? )?trait)",
            r"(?:<.*>)? ",
            r"(?:.* for )?",
            r"(?P<type>\S+)",
            r"(?:<.*>)?",
        )).unwrap();
    }


    let mut lines = Vec::<String>::new();
    let mut curr_impl = None;
    let mut end_impl = String::new();

    for (pos, line) in file.lines().enumerate() {
        let pos = pos + 1;
        let line = line?.trim_end().to_string();

        if let Some(prev_line) = lines.last() {
            if EMPTY_DOC_COMMENT.is_match(prev_line) {
                if EMPTY_DOC_COMMENT.is_match(&line) {
                    print_del(&line, pos);
                    print_del_reason("Consecutives empty comment lines");
                    continue;
                } else if !IS_DOC_COMMENT_LINE.is_match(&line) {
                    print_del(prev_line, pos - 1);
                    print_del_reason("Empty comment line at the end of a comment");
                }
            }
        }

        let line = match handle_comment_link(line, pos, krate) {
            Some(line) => line,
            None => continue,
        };

        let line = match handle_module_link(line, pos, krate) {
            Some(line) => line,
            None => continue,
        };

        if let Some(capture) = IMPL_START.captures(&line) {
            end_impl.clear();
            end_impl.push_str(capture.name("spaces").unwrap().as_str());
            end_impl.push('}');
            curr_impl = Some(capture.name("type").unwrap().as_str().to_string());
        }

        if line == end_impl {
            curr_impl = None;
            end_impl.clear();
        }

        let line = if let Some(ref curr_impl) = curr_impl {
            handle_method_anchor(line, pos, curr_impl)
        } else {
            line
        };

        lines.push(line);
    }

    Ok(lines)
}

fn handle_comment_link(line: String, pos: usize, krate: &str) -> Option<String> {
    lazy_static! {
        static ref COMMENT_LINK: Regex = Regex::new(concat!(
            r"^(?P<link_name>\s*//[!/] \[.*?\]: )",
            r"(?P<supers>(?:\.\./)*)",
            r"(?:(?P<crate>std|core|alloc)/)?",
            r"(?P<intermediates>(?:.*/))?",
            r"(?:enum|struct|primitive|trait|constant|type|fn|macro)\.",
            r"(?P<elem2>.*)\.html",
            r"(?:#(?:method|variant|tymethod)\.(?P<additional>\S*))?$",
        ))
        .unwrap();
    }

    let captures = match COMMENT_LINK.captures(&line) {
        Some(c) => c,
        None => return Some(line),
    };

    // Preparing the new line
    let mut new = String::with_capacity(line.len());

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
            return Some(line);
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

    print_del(&line, pos);

    // Check if the link has become a local path
    if let Some(local) = LOCAL_PATH.captures(&new) {
        let elem = local.name("elem").unwrap().as_str();
        let elem2 = local.name("elem2").unwrap().as_str();
        if elem == elem2 {
            print_del_reason("Local path");
            return None;
        }
    }

    print_add(&new);

    Some(new)
}

fn handle_module_link(line: String, pos: usize, krate: &str) -> Option<String> {
    lazy_static! {
        static ref COMMENT_MODULE: Regex = Regex::new(concat!(
            r"^(?P<link_name>\s*//[!/] \[.*?\]: )",
            r"(?P<supers>(?:\.\./)*)",
            r"(?:(?P<crate>std|core|alloc)/)?",
            r"(?P<mods>(?:.*?/)*)",
            r"index\.html$",
        ))
        .unwrap();
    }

    let captures = match COMMENT_MODULE.captures(&line) {
        Some(c) => c,
        None => return Some(line),
    };

    // Preparing the new line
    let mut new = String::with_capacity(line.len());

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
        new.push_str(mods.as_str().replace("/", "::").trim_end_matches("::"));
    }

    print_del(&line, pos);

    // Check if the link has become a local path
    if let Some(local) = LOCAL_PATH.captures(&new) {
        let elem = local.name("elem").unwrap().as_str();
        let elem2 = local.name("elem2").unwrap().as_str();
        if elem == elem2 {
            print_del_reason("Local path");
            return None;
        }
    }

    print_add(&new);

    Some(new)
}

fn handle_method_anchor(mut line: String, pos: usize, curr_impl: &str) -> String {
    lazy_static! {
        static ref METHOD_ANCHOR: Regex = Regex::new(concat!(
            r"^(?P<link_name>\s*//[!/] \[.*?\]: )",
            r"#(?:method|variant|tymethod)\.(?P<additional>\S*)$",
        ))
        .unwrap();
    };

    if let Some(captures) = METHOD_ANCHOR.captures(&line) {
        let spaces = captures.name("link_name").unwrap().as_str();
        let additional = captures.name("additional").unwrap().as_str();

        print_del(&line, pos);
        line = format!("{}{}::{}", spaces, curr_impl, additional);
        print_add(&line);
    }

    line
}

fn print_del(line: &str, pos: usize) {
    println!("{:5}:  \"{}\"", pos, Color::Red.paint(line))
}

fn print_add(line: &str) {
    println!("        \"{}\"\n", Color::Green.paint(line))
}

fn print_del_reason(reason: &str) {
    println!("        {}\n", Color::Yellow.paint(reason))
}
