use lazy_static;
use regex::Regex;

lazy_static::lazy_static! {
    /// An empty doc comment.
    pub static ref EMPTY_DOC_COMMENT: Regex = Regex::new(r"^\s*//[!/]$").unwrap();

    /// Used to detect doc comment lines, empty or not. This is the same regex
    /// as `EMPTY_DOC_COMMENT` without the ending `$`.
    pub static ref IS_DOC_COMMENT_LINE: Regex = Regex::new(r"^\s*//[!/]").unwrap();

    /// Will search for a doc comment link and be used to check if the two
    /// elements are the same, indicating a local path.
    pub static ref LOCAL_PATH: Regex = Regex::new(concat!(
        r"^\s*(?://[!/] )?",
        r"\[`?(?P<elem>.*?)`?\]: ",
        r"(?P<elem2>.*)$",
    )).unwrap();

    /// Start of a block that can be used to reference to `Self`.
    pub static ref IMPL_START: Regex = Regex::new(concat!(
        r"^(?P<spaces>\s*)",
        r"(?:pub(?:\(.+\))? )?",
        r"(?:impl|trait)(?:<.*?>)? ",
        r"(?:.* for )?",
        r"(?P<type>[\w]+)",
        r"(?:<.*>)?",
    ))
    .unwrap();

    /// Line that is a markdown link to a Rust item.
    pub static ref ITEM_LINK: Regex = Regex::new(concat!(
        r"^(?P<link_name>\s*(?://[!/] )?\[.*?\]: )",
        r"(?P<supers>(?:\.\./)*|\./)",
        r"(?:(?P<crate>[a-zA-Z_]+)/)?",
        r"(?P<intermediates>(?:.*/))?",
        r"(?:enum|struct|primitive|trait|constant|type|fn|macro)\.",
        r"(?P<elem2>.*)\.html",
        r"(?:#(?:method|variant|tymethod|associatedconstant)\.(?P<additional>\S*))?$",
    ))
    .unwrap();

    /// Line that is a markdown link to a Rust module.
    pub static ref MODULE_LINK: Regex = Regex::new(concat!(
        r"^(?P<link_name>\s*(?://[!/] )?\[.*?\]: )",
        r"(?P<supers>(?:\.\./)*)",
        r"(?:(?P<crate>[a-zA-Z_]+)/)?",
        r"(?P<mods>(?:.*?/)*)",
        r"index\.html$",
    ))
    .unwrap();

    /// Line that is a method of the `Self` type.
    ///
    /// This needs context for `Self` to be correctly inserted.
    pub static ref METHOD_ANCHOR: Regex = Regex::new(concat!(
        r"^(?P<link_name>\s*(?://[!/] )?\[.*?\]: )",
        r"#(?:method|variant|tymethod)\.(?P<additional>\S*)$",
    ))
    .unwrap();
}
