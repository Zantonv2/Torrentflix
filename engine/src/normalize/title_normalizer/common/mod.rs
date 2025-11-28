pub mod pattern;
pub mod numeral;
pub mod date;
pub mod validators;
pub mod formatters;
pub mod quantity;
pub mod words;
pub mod comparators;
pub mod expected;

// Separator constants from Rust metadata parser common module
pub const SEPS: &str = r" [](){}+*|=-_~#/\\.,;:";  // list of tags/words separators
pub const SEPS_NO_GROUPS: &str = r" []+*|=-_~#/\\.,;:";  // separators without groups
pub const SEPS_NO_FS: &str = r" [](){}+*|=-_~#.,;:";  // separators without filesystem

pub const TITLE_SEPS: &str = r"-+/|";  // separators for title

// Pattern abbreviations used by many rebulk objects
pub const DASH: (&str, &str) = (r"-", SEPS_NO_FS);
pub const ALT_DASH: (&str, &str) = (r"@", SEPS_NO_FS);

/// Make a regex pattern optional
pub fn optional(pattern: &str) -> String {
    format!("(?:{})?", pattern)
}

/// Check if character is a separator
pub fn is_separator(c: char) -> bool {
    SEPS.contains(c)
}

/// Check if character is a title separator
pub fn is_title_separator(c: char) -> bool {
    TITLE_SEPS.contains(c)
}
