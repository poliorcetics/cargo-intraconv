use crate::Action;
use lazy_static::lazy_static;
use regex::Regex;
use std::io::{self, BufRead};

/// All item types that can be produced by `rustdoc`.
const ITEM_TYPES: &[&str] = &[
    "associatedconstant",
    "associatedtype",
    "attr",
    "constant",
    "derive",
    "enum",
    "externcrate",
    "fn",
    "foreigntype",
    "impl",
    "import",
    "keyword",
    "macro",
    "method",
    "mod",
    "opaque",
    "primitive",
    "static",
    "struct",
    "structfield",
    "trait",
    "traitalias",
    "tymethod",
    "type",
    "union",
    "variant",
];

lazy_static! {
    /// Line that is a markdown link to a Rust item.
    ///
    /// `HTTP_LINK` is voluntarily very conservative in what is a link to
    /// avoid missing valid links. It is better not to break an existing
    /// and working link than to try and fail when replacing it or worse,
    /// transforming it but making it point to something else silently.
    static ref HTTP_LINK: Regex = Regex::new(r"^\s*(?://[!/] )?\[.*?\]:\s+https?://.*\n$").unwrap();

    /// Will search for a doc comment link and be used to check if the two
    /// elements are the same, indicating a local path.
    static ref LOCAL_PATH: Regex = Regex::new(&[
        r"^\s*(?://[!/] )?",
        r"\[`?(?P<elem>.*?)`?\]: ",
        &format!(r"(?:(?:{})@)?", ITEM_TYPES.join("|")),
        r"(?P<elem2>.*?)",
        r"(?:!|\(\))?\n$",
    ].join("")).unwrap();

    /// A partial link that needs a type to be complete.
    ///
    /// For more informations about how this is done, see the documentation
    /// and code for the `Context` type.
    static ref METHOD_ANCHOR: Regex = Regex::new(&[
        r"^(?P<link_name>\s*(?://[!/] )?\[.*?\]: )",
        &format!(
            r"#(?:{})\.(?P<additional>[\w_]+)\n$",
            ITEM_TYPES.join("|")
        ),
    ].join(""))
    .unwrap();

    /// Start of a block where `Self` has a sense.
    static ref TYPE_BLOCK_START: Regex = Regex::new(concat!(
        r"^(?P<spaces>\s*)",
        r"(?:pub(?:\(.+\))? )?",
        r"(?:struct|trait|impl(?:<.*?>)?(?: .*? for)?|enum|union) ",
        r"(?P<type>\w+)",
        r"(?:<.*?>)?",
        r"(?P<parenthese>\()?",
        r".*\n$",
    )).unwrap();
}

/// Context for the check. It notably contains informations about the crate and
/// the current type (e.g, for `#method.name` links).
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Context {
    /// Name of the crate on which the tool is run.
    /// It must have been checked to be a correct identifier for Rust.
    krate: String,
    /// Current line number.
    pos: usize,
    /// Name of the type that is `Self` for the current block.
    curr_type_block: Option<String>,
    /// End of the current block for `Self` (if any).
    end_type_block: String,
    // NOTE: at the moment nested type blocks are not handled.
    /// All types blocks known to the context.
    ///
    /// Calling `pop` on the `Vec` must give the next type block (if there is
    /// one).
    type_blocks: Vec<(String, String)>,
}

impl Context {
    /// Creates a new `Context` with the given crate.
    ///
    /// NOTE: the `krate` parameter must contain a valid Rust identifier for a
    /// crate name (basically the regex `[\w_]+`) else the links that use it
    /// will be broken by the conversion.
    pub fn new(krate: String) -> Self {
        Self {
            krate,
            pos: 0,
            curr_type_block: None,
            end_type_block: String::new(),
            type_blocks: Vec::new(),
        }
    }

    pub fn transform_file<R: BufRead>(&mut self, reader: R) -> io::Result<Vec<Action>> {
        // Reset the state before handling the file.
        self.pos = 0;
        self.curr_type_block = None;
        self.end_type_block = String::new();
        self.type_blocks.clear();

        let mut lines = Vec::new();
        for l in reader.lines() {
            let mut line = l?.trim_end().to_string();
            line.push('\n');
            lines.push(line);
        }

        self.find_type_blocks(lines.iter().map(|s| s.as_str()));

        let mut actions = Vec::with_capacity(lines.len());
        for line in lines.into_iter() {
            self.pos += 1;
            actions.push(self.transform_line(line));
        }

        Ok(actions)
    }

    /// Fills `self.type_block` with the types that are encountered, reversing the
    /// vector after they have all been added.
    /// This means that calling this function twice maybe have unintended consequences
    /// on the state `Context`.
    fn find_type_blocks<'a>(&mut self, lines: impl Iterator<Item = &'a str>) {
        for line in lines {
            // Early return on context change too, after updating the context.
            if let Some(captures) = TYPE_BLOCK_START.captures(&line) {
                let ty = captures.name("type").unwrap().as_str().into();
                let end = if line.ends_with(";\n") || line.ends_with("}\n") {
                    '\n'.into()
                } else {
                    // When the item is not simple we try to compute what will be
                    // the end of the block.
                    let mut s = captures.name("spaces").unwrap().as_str().to_string();
                    s.reserve(1);

                    if let Some(_) = captures.name("parenthese") {
                        s.push(')');
                    } else {
                        s.push('}');
                    }

                    s
                };

                self.type_blocks.push((ty, end));
            }
        }

        self.type_blocks.reverse();
    }

    /// Transform a single line, returning the action.
    fn transform_line(&mut self, line: String) -> Action {
        let copy = line.clone();

        // Updating the currently active `Self` type.
        if self.curr_type_block.is_none() {
            if let Some((curr_type, end)) = self.type_blocks.pop() {
                self.curr_type_block = Some(curr_type);
                self.end_type_block = end;
            }
        }

        if HTTP_LINK.is_match(&line) {
            return Action::Unchanged { line };
        }

        let line = self.transform_item(line);
        let line = self.transform_module(line);
        let line = self.transform_anchor(line);

        // When reaching the end of the current type block, update the context to
        // reflect it.
        if self.curr_type_block.is_some() && line.starts_with(&self.end_type_block) {
            self.curr_type_block = None;
            self.end_type_block.clear();
        }

        let line = self.transform_local(line);

        if line.is_empty() {
            Action::Deleted {
                line: copy,
                pos: self.pos,
            }
        } else if line == copy {
            Action::Unchanged { line: copy }
        } else {
            Action::Replaced {
                line: copy,
                new: line,
                pos: self.pos,
            }
        }
    }

    /// Try to transform a line as an item link. If it is not, the line
    /// is returned unmodified.
    fn transform_item(&mut self, line: String) -> String {
        let item_link = Regex::new(
            &[
                r"^(?P<link_name>\s*(?://[!/] )?\[.*?\]: )",
                r"(?:\./)?",
                r"(?P<supers>(?:\.\./)*)",
                &format!(r"(?:(?P<crate>{})/)?", self.krate),
                r"(?P<intermediates>(?:.*/))?",
                &format!(r"(?P<item_type>{})\.", ITEM_TYPES.join("|")),
                r"(?P<elem>.*)\.html",
                &format!(
                    r"(?:#(?P<add_item_type>{})\.(?P<additional>\S*))?\n$",
                    ITEM_TYPES.join("|")
                ),
            ]
            .join(""),
        )
        .unwrap();

        if let Some(captures) = item_link.captures(&line) {
            let link_name = captures.name("link_name").unwrap().as_str();
            let elem = captures.name("elem").unwrap().as_str();
            let item_type = captures.name("item_type").unwrap().as_str();
            let add_item_type = captures.name("add_item_type").map(|x| x.as_str());

            let mut new = String::with_capacity(64);
            new.push_str(link_name);

            let true_item_type = add_item_type.unwrap_or(item_type);
            let item = match true_item_type {
                "struct" | "enum" | "trait" | "union" | "type" => "type",
                "const" | "static" | "value" => "value",
                "derive" | "attr" => "macro",
                "primitive" => "primitive",
                "mod" => "mod",
                _ => "",
            };

            if !item.is_empty() {
                new.push_str(item);
                new.push('@');
            }

            // Handling the start of the path.
            if let Some(_) = captures.name("crate") {
                // This a path contained in the crate: the start of a full path is
                // 'crate', not the crate name in this case.
                new.push_str("crate::");
            } else if let Some(supers) = captures.name("supers").map(|x| x.as_str()) {
                // The path is not explicitely contained in the crate but has some
                // 'super' elements.
                let count = supers.matches('/').count();
                // This way we won't allocate a string only to immediately drop it.
                for _ in 0..count {
                    new.push_str("super::");
                }
            }

            // Intermediates element like a path through modules.
            // In the case of a path without 'super'
            if let Some(intermediates) = captures.name("intermediates").map(|x| x.as_str()) {
                if intermediates != "./" {
                    new.push_str(&intermediates.replace("/", "::"));
                }
            }

            // The main element of the link.
            new.push_str(elem);

            // Additional linked elements like a method or a variant.
            if let Some(additional) = captures.name("additional").map(|x| x.as_str()) {
                new.push_str("::");
                new.push_str(additional);
            }

            match true_item_type {
                "fn" | "method" => new.push_str("()"),
                "macro" => new.push('!'),
                _ => (),
            };

            // The regexes that will follow expect a `\n` at the end of the line.
            new.push('\n');

            new
        } else {
            line
        }
    }

    /// Try to transform a line as a module link. If it is not, the line is
    /// returned unmodified.
    fn transform_module(&mut self, line: String) -> String {
        let module_link = Regex::new(
            &[
                r"^(?P<link_name>\s*(?://[!/] )?\[.*?\]: )",
                r"(?:\./)?",
                r"(?P<supers>(?:\.\./)*)",
                &format!(r"(?:(?P<crate>{})/)?", self.krate),
                r"(?P<mods>(?:.*?/)*)",
                r"index\.html",
                r"(?P<section>#.+)?\n$",
            ]
            .join(""),
        )
        .unwrap();

        if let Some(captures) = module_link.captures(&line) {
            let link_name = captures.name("link_name").unwrap().as_str();
            let section = captures.name("section").map(|x| x.as_str()).unwrap_or("");

            let mut new = String::with_capacity(64);
            new.push_str(link_name);
            new.push_str("mod@");

            // Handling the start of the path.
            if let Some(_) = captures.name("crate") {
                // This a path contained in the crate: the start of a full path is
                // 'crate', not the crate name in this case.
                new.push_str("crate::");
            } else if let Some(supers) = captures.name("supers").map(|x| x.as_str()) {
                // The path is not explicitely contained in the crate but has some
                // 'super' elements.
                let count = supers.matches('/').count();
                // This way we won't allocate a string only to immediately drop it.
                for _ in 0..count {
                    new.push_str("super::");
                }
            }

            // Handling the modules names themselves.
            if let Some(mods) = captures.name("mods").map(|x| x.as_str()) {
                // If the link is simply `index.html` the line is removed.
                if mods.is_empty() && section.is_empty() {
                    return "".into();
                }

                new.push_str(mods.replace("/", "::").trim_end_matches("::"));
            }

            new.push_str(section);
            new.push('\n');

            // Some module links are only a link to a section, for those
            // don't insert the 'mod@' modifier.
            new.replace("]: mod@#", "]: #")
        } else {
            line
        }
    }

    /// Try to transform a line as an anchor link. If it is not, the line is
    /// returned unmodified. For the best results, ensure `find_type_blocks`
    /// has been called before.
    fn transform_anchor(&mut self, line: String) -> String {
        if let (Some(ref captures), Some(ref ty)) =
            (METHOD_ANCHOR.captures(&line), &self.curr_type_block)
        {
            let link_name = captures.name("link_name").unwrap().as_str();
            let additional = captures.name("additional").unwrap().as_str();

            format!(
                "{link}{ty}::{add}\n",
                link = link_name,
                ty = ty,
                add = additional
            )
        } else {
            line
        }
    }

    /// Try to transform a local link to an empty string. If it is not, the
    /// line is returned unmodified. Should be called after all the other
    /// transformations to ensure no local link is missed.
    fn transform_local(&mut self, line: String) -> String {
        if let Some(captures) = LOCAL_PATH.captures(&line) {
            let link = captures.name("elem").unwrap();
            let path = captures.name("elem2").unwrap();
            if path.as_str() == link.as_str() {
                return "".into();
            }
        }

        line
    }
}

#[cfg(test)]
mod tests;
