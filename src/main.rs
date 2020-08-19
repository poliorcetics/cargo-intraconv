use ansi_term::Color;
use lazy_static::lazy_static;
use regex::Regex;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;

fn main() {
    let args = env::args_os().skip(1);

    args.for_each(|arg| {
        let path = Path::new(&arg);

        let path_display = path.display().to_string();
        println!("{}", &path_display);
        // TODO: Not always perfect because of unicode, fix this
        println!("{}\n", "=".repeat(path_display.len()));

        let file = match File::open(path) {
            Ok(file) => BufReader::new(file),
            Err(err) => {
                eprintln!("Failed to open file '{}': {}", &path_display, err);
                return;
            }
        };

        match apply_regex(file, "std") {
            Ok(_) => (),
            Err(err) => {
                eprintln!("Failed to handle file '{}': {}", &path_display, err);
                return;
            }
        }
    });
}

lazy_static! {
    // An empty doc comment.
    static ref EMPTY_DOC_COMMENT: Regex = Regex::new(r"^\s*//(?:!|/)$").unwrap();

    // Used to detect doc comment lines, empty or not. This is the same regex
    // as `EMPTY_DOC_COMMENT` without the ending `$`.
    static ref IS_DOC_COMMENT_LINE: Regex = Regex::new(r"^\s*//(?:!|/)").unwrap();

    static ref IMPL_REGEX: Regex = Regex::new(concat!(
        r"^(?P<spaces>\s*)",
        r"(?:impl|(?:pub(?:\(.+\))? )?trait)",
        r"(?:<.*>)? ",
        r"(?:.* for )?",
        r"(?P<type>\S+)",
        r"(?:<.*>)?",
    )).unwrap();


    static ref LOCAL_PATH: Regex = Regex::new(concat!(
        r"^(?:\s*)//(?:!|/) ",
        r"\[`?(?P<elem>.*?)`?\]: ",
        r"(?P<elem2>.*)$",
    )).unwrap();

}

fn apply_regex<R: Read>(file: BufReader<R>, krate: &str) -> io::Result<Vec<String>> {
    let mut lines = Vec::new();
    let mut curr_impl = None;
    let mut end_impl = String::new();

    for (pos, line) in file.lines().enumerate() {
        let pos = pos + 1;
        let line = line?.trim_end().to_string();
        let prev_line = lines.last().map(|e: &String| e.as_str()).unwrap_or("");

        if EMPTY_DOC_COMMENT.is_match(&prev_line) {
            if EMPTY_DOC_COMMENT.is_match(&line) {
                print_del(&line, pos);
                print_del_reason("Consecutives empty comment lines");
                continue;
            } else if !IS_DOC_COMMENT_LINE.is_match(&line) {
                print_del(prev_line, pos - 1);
                print_del_reason("Empty comment line at the end of a comment");
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

        let capture = IMPL_REGEX.captures(&line);
        if let Some(capture) = capture {
            end_impl = capture.name("spaces").unwrap().as_str().into();
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
            r"^(?P<spaces>\s*)",
            r"//(?P<c>!|/) ",
            r"\[(?P<elem>.*?)\]: ",
            r"(?P<supers>\.\./)*",
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
    // TODO: move that to one captures group only since this is not used
    // anywhere else.
    new.push_str(captures.name("spaces").unwrap().as_str());
    new.push_str("//");
    new.push_str(captures.name("c").unwrap().as_str());
    new.push_str(" [");
    new.push_str(captures.name("elem").unwrap().as_str());
    new.push_str("]: ");

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
            r"^(?P<spaces>\s*)",
            r"//(?P<c>!|/) ",
            r"\[(?P<elem>.*?)\]: ",
            r"(?P<supers>\.\./)*",
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
    // TODO: move that to one captures group only since this is not used
    // anywhere else.
    new.push_str(captures.name("spaces").unwrap().as_str());
    new.push_str("//");
    new.push_str(captures.name("c").unwrap().as_str());
    new.push_str(" [");
    new.push_str(captures.name("elem").unwrap().as_str());
    new.push_str("]: ");

    // First elements like the crate or `super::`
    if let Some(root) = captures.name("root") {
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

    // TODO: fix

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
            r"^(?P<spaces>\s*)",
            r"//(?P<c>!|/) ",
            r"\[(?P<elem>.*)\]: ",
            r"#(?:method|variant|tymethod)\.(?P<additional>\S*)$",
        ))
        .unwrap();
    };

    let captures = METHOD_ANCHOR.captures(&line);

    if let Some(captures) = captures {
        let spaces = captures.name("spaces").unwrap().as_str();
        let c = captures.name("c").unwrap().as_str();
        let elem = captures.name("elem").unwrap().as_str();
        let additional = captures.name("additional").unwrap().as_str();

        print_del(&line, pos);
        line = format!(
            "{}//{} [{}]: {}::{}",
            spaces, c, elem, curr_impl, additional
        );
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
