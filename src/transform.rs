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

/// Markers that can be found once a link has been transformed at the start
/// of the element, to disambiguate it, e.g. for the `usize` module and the
/// `usize` primitive.
const ITEM_START_MARKERS: &[&str] = &["value", "type", "macro", "prim", "mod"];

/// Regexes that detect *favored* links: they are links which should be
/// transformed into intra-doc links when they match and the relevant
/// option is `true`.
mod fav_links {
    use super::{lazy_static, Regex, ITEM_TYPES};

    lazy_static! {
        /// Line that is a long markdown link to a https://docs.rs item.
        pub static ref DOCS_RS_LONG: Regex = Regex::new(&[
            r"^(?P<link_name>\s*(?://[!/] )?\[.+?\]:\s+)",
            r"https?://docs.rs/(?:[A-Za-z_][A-Za-z0-9_-]*/)(?:.+?/)",
            r"(?P<rest>",
            // Detect crate name + modules
            r"(?:(?:[A-Za-z0-9_]+/)+)?",
            r"(?:",
            // Detect item
            &format!(r"(?:{})\.", ITEM_TYPES.join("|")),
            r"(?:.*)\.html",
            &format!(
                r"(?:#(?:{})\.(?:\S*))?",
                ITEM_TYPES.join("|"),
            ),
            r"|",
            // Detect module
            r"(?:index\.html|/|(?P<final_mod>[A-Za-z0-9_]+/?)?)",
            r")",
            r"(?:#[a-zA-Z0-9_\-\.]+)?",
            "\n)$",
        ].join("")).unwrap();

        /// Line that is a short markdown link to a https://docs.rs crate.
        pub static ref DOCS_RS_SHORT: Regex = Regex::new(concat!(
            r"^(?P<link_name>\s*(?://[!/] )?\[.+?\]:\s+)",
            r"https?://docs.rs/",
            r"(?P<krate>[A-Za-z_][A-Za-z0-9_-]*)",
            r"/?\n$",
        )).unwrap();

        /// Line that is a markdown link to a https://doc.rust-lang.org item.
        pub static ref DOC_RUST_LANG_LONG: Regex =  Regex::new(&[
            r"^(?P<link_name>\s*(?://[!/] )?\[.+?\]:\s+)",
            r"https?://doc.rust-lang.org/",
            r"(?:nightly|stable|beta|1\.\d+\.\d+)/",
            // Detect crate name while ignoring nightly-rustc
            r"(?:nightly-rustc/|(?P<crate>(?:std|alloc|core|test|proc_macro)/))",
            r"(?P<rest>",
            // Detect (when nightly-rustc) crate name + (always) modules
            r"(?:(?:[A-Za-z0-9_]+/)+)?",
            r"(?:",
            // Detect item
            &format!(r"(?:{})\.", ITEM_TYPES.join("|")),
            r"(?:.*)\.html",
            &format!(
                r"(?:#(?:{})\.(?:\S*))?",
                ITEM_TYPES.join("|"),
            ),
            r"|",
            // Detect module
            r"(?:index\.html|/|(?P<final_mod>[A-Za-z0-9_]+/?)?)",
            r")",
            r"(?:#[a-zA-Z0-9_\-\.]+)?",
            "\n)$",
        ].join("")).unwrap();
    }
}

lazy_static! {
    /// Line that is a markdown link to a Rust item.
    ///
    /// `HTTP_LINK` is voluntarily very conservative in what is a link to
    /// avoid missing valid links. It is better not to break an existing
    /// and working link than to try and fail when replacing it or worse,
    /// transforming it but making it point to something else silently.
    static ref HTTP_LINK: Regex = Regex::new(r"^\s*(?://[!/] )?\[.+?\]:\s+https?://.*\n$").unwrap();

    /// If this regex matches, the tested line is probably a markdown link.
    static ref MARKDOWN_LINK_START: Regex = Regex::new(r"^\s*(?://[!/] )?\[.+?\]:\s+").unwrap();

    /// Will search for a doc comment link and be used to check if the two
    /// elements are the same, indicating a local path.
    static ref LOCAL_PATH: Regex = Regex::new(&[
        r"^\s*(?://[!/] )?",
        r"\[`?(?P<elem>.*?)`?\]: ",
        &format!(r"(?:(?P<dis_start>{})@)?", ITEM_START_MARKERS.join("|")),
        r"(?P<elem2>.*?)",
        r"(?P<dis_end>!|\(\))?\n$",
    ].join("")).unwrap();

    /// A partial link that needs a type to be complete.
    ///
    /// For more informations about how this is done, see the documentation
    /// and code for the `Context` type.
    static ref METHOD_ANCHOR: Regex = Regex::new(&[
        r"^(?P<link_name>\s*(?://[!/] )?\[.*?\]: )",
        &format!(
            r"#(?P<item_type>{})",
            ITEM_TYPES.join("|")
        ),
        r"\.(?P<additional>[\w_]+)\n$",
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
    /// Use rustdoc disambiguators in front of the transformed links
    /// ('type@', ...). Ending disambiguators like '()' and '!' are always
    /// added, regardless of this option.
    disambiguate: bool,
    /// If `true` the regexes in the `fav_links` module will be checked for
    /// and their transformation applied before HTTP link are ignored.
    ///
    /// See also the `transform_favored_link` function.
    apply_favored: bool,
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

impl Context {
    /// Creates a new `Context` with the given crate.
    ///
    /// NOTE: the `krate` parameter must contain a valid Rust identifier for a
    /// crate name (basically the regex `[\w_]+`) else the links that use it
    /// will be broken by the conversion.
    pub fn new(krate: String, disambiguate: bool, apply_favored: bool) -> Self {
        Self {
            krate,
            disambiguate,
            apply_favored,
            pos: 0,
            curr_type_block: None,
            end_type_block: String::new(),
            type_block_line: usize::MAX,
            type_blocks: Vec::new(),
        }
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
            let mut line = l?.trim_end().to_string();
            line.push('\n');
            lines.push(line);
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

        let line = if self.apply_favored {
            transform_favored_link(line)
        } else {
            line
        };

        // Detect as soon as possible when a line is not a link and so should
        // not be transformed.
        if self.end_type_block.is_empty()
            && (HTTP_LINK.is_match(&line) || !MARKDOWN_LINK_START.is_match(&line))
        {
            return Action::Unchanged { line };
        }

        // Clone here and not before to avoid doing it when the line is an
        // HTTP link.
        let copy = line.clone();

        let line = self.transform_item(line);
        let line = self.transform_module(line);
        let line = self.transform_anchor(line);

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

        let line = transform_local(line);

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
    fn transform_item(&self, line: String) -> String {
        let item_link = Regex::new(
            &[
                r"^(?P<link_name>\s*(?://[!/] )?\[.*?\]: )",
                r"(?:\./)?",
                r"(?P<supers>(?:\.\./)+)?",
                &format!(r"(?:(?P<crate>{})/)?", self.krate),
                r"(?P<intermediates>(?:[A-Za-z0-9_]+/)+)?",
                &format!(r"(?P<item_type>{})\.", ITEM_TYPES.join("|")),
                r"(?P<elem>[a-zA-Z0-9_]+)\.html",
                r"(?:",
                &format!(
                    r"#(?P<add_item_type>{})\.(?P<additional>\S*)",
                    ITEM_TYPES.join("|")
                ),
                r"|",
                r"#(?P<section>[a-zA-Z0-9\-_]+)",
                r")?\n$",
            ]
            .join(""),
        )
        .unwrap();

        // Early return if the line is not an item link.
        let captures = match item_link.captures(&line) {
            Some(captures) => captures,
            None => return line,
        };

        // Getting a maximum of values as early as possible to keep them close
        // to their related regex.
        let link_name = captures.name("link_name").unwrap().as_str();
        let elem = captures.name("elem").unwrap().as_str();
        let item_type = captures.name("item_type").unwrap().as_str();
        let add_item_type = captures.name("add_item_type").map(|x| x.as_str());
        let section = captures.name("section").map(|x| x.as_str()).unwrap_or("");

        let mut new = String::with_capacity(64);
        new.push_str(link_name);

        let true_item_type = add_item_type.unwrap_or(item_type);
        let (item_marker_start, item_marker_end) = item_type_markers(true_item_type);

        if self.disambiguate {
            new.push_str(item_marker_start);
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

        if !section.is_empty() {
            new.push('#');
            new.push_str(section);
        }
        new.push_str(item_marker_end);
        // The regexes that will follow expect a `\n` at the end of the line.
        new.push('\n');

        new
    }

    /// Try to transform a line as a module link. If it is not, the line is
    /// returned unmodified.
    fn transform_module(&self, line: String) -> String {
        let module_link = Regex::new(
            &[
                r"^(?P<link_name>\s*(?://[!/] )?\[.*?\]: )",
                r"(?:\./)?",
                r"(?P<supers>(?:\.\./)+)?",
                &format!(r"(?:(?P<crate>{})/)?", self.krate),
                r"(?P<mods>(?:[a-zA-Z0-9_]+?/)+)?",
                r"index\.html",
                r"(?P<section>#[a-zA-Z0-9_\-\.]+)?\n$",
            ]
            .join(""),
        )
        .unwrap();

        let captures = match module_link.captures(&line) {
            Some(captures) => captures,
            None => return line,
        };

        let link_name = captures.name("link_name").unwrap().as_str();
        let section = captures.name("section").map(|x| x.as_str()).unwrap_or("");

        let mut new = String::with_capacity(64);
        new.push_str(link_name);
        if self.disambiguate {
            new.push_str("mod@");
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

        // Handling the modules names themselves.
        if let Some(mods) = captures.name("mods").map(|x| x.as_str()) {
            // If the link is simply `index.html` the line is removed.
            if mods.is_empty() && section.is_empty() {
                return "".into();
            }

            new.push_str(mods.replace("/", "::").as_ref());
        }

        new = new.trim_end_matches("::").into();

        // Ensuring `self` is present when no other module name is.
        if captures.name("crate").is_none()
            && captures.name("supers").is_none()
            && captures.name("mods").is_none()
        {
            new.push_str("self");
        }

        new.push_str(section);
        new.push('\n');

        // Some module links are only a link to a section, for those
        // don't insert the 'mod@' modifier.
        new.replace("]: mod@#", "]: #")
    }

    /// Try to transform a line as an anchor link. If it is not, the line is
    /// returned unmodified. For the best results, ensure `find_type_blocks`
    /// has been called before.
    fn transform_anchor(&self, line: String) -> String {
        if let (Some(ref captures), Some(ref ty)) =
            (METHOD_ANCHOR.captures(&line), &self.curr_type_block)
        {
            let link_name = captures.name("link_name").unwrap().as_str();
            let item_type = captures.name("item_type").unwrap().as_str();
            let additional = captures.name("additional").unwrap().as_str();

            let (start, end) = item_type_markers(item_type);

            let start = if self.disambiguate { start } else { "" };

            format!(
                "{link}{s}{ty}::{add}{e}\n",
                link = link_name,
                ty = ty,
                add = additional,
                s = start,
                e = end,
            )
        } else {
            line
        }
    }
}

/// Try to transform a local link to an empty string. If it is not, the
/// line is returned unmodified. Should be called after all the other
/// transformations to ensure no local link is missed.
///
/// Links with disambiguators in them will be returned without any changes
/// because the disambiguator could be the only thing helping rustdoc correctly
/// determine the item true link.
fn transform_local(line: String) -> String {
    if let Some(captures) = LOCAL_PATH.captures(&line) {
        // Don't remove links that have disambiguators.
        if captures.name("dis_start").is_some() || captures.name("dis_end").is_some() {
            return line;
        }

        let link = captures.name("elem").unwrap();
        let path = captures.name("elem2").unwrap();
        if path.as_str() == link.as_str() {
            return "".into();
        }
    }

    line
}

/// Try to transform an http(s) link to a form suitable for intra-doc link
/// processing.
///
/// Should be called before http(s) links are ignored else it will never do
/// anything.
fn transform_favored_link(line: String) -> String {
    if let Some(captures) = fav_links::DOCS_RS_LONG.captures(&line) {
        let link_name = captures.name("link_name").unwrap().as_str();
        let rest = captures.name("rest").unwrap().as_str();
        // `link_name` and `rest` respectively contain the necessary spacing
        // and line ending characters.
        let res = format!("{ln}{r}", ln = link_name, r = rest).replace("/\n", "/index.html\n");

        return if let Some(final_mod) = captures.name("final_mod").map(|x| x.as_str()) {
            res.replace(final_mod, &format!("{}/index.html", final_mod))
        } else {
            res.replace("/#", "/index.html#")
        };
    } else if let Some(captures) = fav_links::DOCS_RS_SHORT.captures(&line) {
        let link_name = captures.name("link_name").unwrap().as_str();
        let krate = captures.name("krate").unwrap().as_str().replace("-", "_");
        // `link_name` contains the necessary spacing.
        return format!("{ln}{k}\n", ln = link_name, k = krate);
    } else if let Some(captures) = fav_links::DOC_RUST_LANG_LONG.captures(&line) {
        let link_name = captures.name("link_name").unwrap().as_str();
        let krate = captures.name("crate").map(|x| x.as_str()).unwrap_or("");
        let rest = captures.name("rest").unwrap().as_str();
        // `link_name` and `rest` respectively contain the necessary spacing
        // and line ending characters.
        let res = format!("{ln}{c}{r}", ln = link_name, c = krate, r = rest)
            .replace("/\n", "/index.html\n");

        return if let Some(final_mod) = captures.name("final_mod").map(|x| x.as_str()) {
            res.replace(final_mod, &format!("{}/index.html", final_mod))
        } else {
            res.replace("/#", "/index.html#")
        };
    }

    line
}

/// Returns the reversed list of type blocks found in the given iterator.
///
/// The returned values are `(type name, end marker, starting line of the type block)`.
/// The `starting line` value is found by enumerating over the iterator and
/// adding `1` to the index.
fn find_type_blocks<'a, S, I>(lines: I) -> Vec<(String, String, usize)>
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

                if let Some(_) = captures.name("parenthese") {
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

/// Return the markers for the given item type, the one before and the one
/// after. If necessary the '@' is present in the returned value.
fn item_type_markers(item_type: &str) -> (&'static str, &'static str) {
    match item_type {
        "struct" | "enum" | "trait" | "union" | "type" => ("type@", ""),
        "const" | "static" | "value" => ("value@", ""),
        "derive" | "attr" => ("macro@", ""),
        "primitive" => ("prim@", ""),
        "mod" => ("mod@", ""),
        "fn" | "method" => ("", "()"),
        "macro" => ("", "!"),
        _ => ("", ""),
    }
}

#[cfg(test)]
mod tests;
