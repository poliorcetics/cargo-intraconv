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

        let file = match File::open(path) {
            Ok(file) => BufReader::new(file),
            Err(err) => {
                eprintln!("Failed to open file '{}': {}", path.display(), err);
                return;
            }
        };

        match apply_regex(file) {
            Ok(_) => (),
            Err(err) => {
                eprintln!("Failed to handle file '{}': {}", path.display(), err);
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
}

fn apply_regex<R: Read>(file: BufReader<R>) -> io::Result<()> {
    let mut lines = Vec::new();

    for (pos, line) in file.lines().enumerate() {
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

        lines.push(line);
    }

    Ok(())
}

fn print_del(line: &str, pos: usize) {
    println!("{:5}:  \"{}\"", pos, Color::Red.paint(line))
}

fn print_add(line: &str, pos: usize) {
    println!("{:5}:  \"{}\"", pos, Color::Green.paint(line))
}

fn print_del_reason(reason: &str) {
    println!("        \"{}\"", Color::Yellow.paint(reason))
}
