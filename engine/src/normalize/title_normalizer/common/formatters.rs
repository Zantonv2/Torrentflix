use crate::normalize::title_normalizer::common::{SEPS, TITLE_SEPS};
use regex::Regex;

/// Separators for cleanup (excluding dots and dashes to preserve them after punctuation reduction)
pub const CLEANUP_SEPS: &str = " ,_;:|()[]{}'\"";

lazy_static::lazy_static! {
    static ref MULTISPACE_RE: Regex = Regex::new(r"\s+").unwrap();
    static ref CLEANUP_RE: Regex = Regex::new(r"[^\w\s\-\.]").unwrap();
    static ref REPEATED_PUNCTUATION_RE: Regex = Regex::new(r"([\-_.:,]){2,}").unwrap();
    static ref TITLE_CLEANUP_RE: Regex = Regex::new(&format!("[{}]", regex::escape(TITLE_SEPS))).unwrap();
    static ref SEPS_RE: Regex = Regex::new(&format!("[{}]", regex::escape(SEPS))).unwrap();
    static ref CLEANUP_SEPS_RE: Regex = Regex::new(&format!("[{}]", regex::escape(CLEANUP_SEPS))).unwrap();
}

/// Clean up text by removing unwanted characters and normalizing whitespace
pub fn cleanup(text: &str) -> String {
    let mut result = text.to_string();

    // Clean up repeated punctuation first
    result = REPEATED_PUNCTUATION_RE
        .replace_all(&result, "$1")
        .to_string();

    // Replace separators with spaces (excluding dashes to preserve them after punctuation reduction)
    result = CLEANUP_SEPS_RE.replace_all(&result, " ").to_string();

    // Remove unwanted characters
    result = CLEANUP_RE.replace_all(&result, " ").to_string();

    // Normalize whitespace
    result = MULTISPACE_RE.replace_all(&result, " ").to_string();

    result.trim().to_string()
}

/// Raw cleanup that preserves more characters
pub fn raw_cleanup(text: &str) -> String {
    let mut result = text.to_string();

    // Replace separators with spaces
    result = SEPS_RE.replace_all(&result, " ").to_string();

    // Normalize whitespace
    result = MULTISPACE_RE.replace_all(&result, " ").to_string();

    result.trim().to_string()
}

/// Strip whitespace from both ends
pub fn strip(text: &str) -> String {
    text.trim().to_string()
}

/// Clean up title-specific separators
pub fn cleanup_title(text: &str) -> String {
    let mut result = text.to_string();

    // Replace title separators with spaces (use TITLE_CLEANUP_RE to preserve dots for year extraction)
    result = TITLE_CLEANUP_RE.replace_all(&result, " ").to_string();

    // Also replace backslashes which aren't in TITLE_SEPS
    result = result.replace('\\', " ");

    // Normalize whitespace
    result = MULTISPACE_RE.replace_all(&result, " ").to_string();

    result.trim().to_string()
}

/// Reorder title components (simplified version)
pub fn reorder_title(title: &str) -> String {
    // This is a simplified version - in Rust metadata parser it's more complex
    // Use cleanup() then manually replace dots with spaces for title reordering
    let mut result = cleanup(title);
    result = result.replace('.', " ");
    result.trim().to_string()
}

/// Convert to title case (first letter of each word capitalized)
pub fn title_case(text: &str) -> String {
    text.split_whitespace()
        .map(|word| {
            let mut chars: Vec<char> = word.chars().collect();
            if !chars.is_empty() {
                chars[0] = chars[0].to_uppercase().next().unwrap_or(chars[0]);
                for c in chars.iter_mut().skip(1) {
                    *c = c.to_lowercase().next().unwrap_or(*c);
                }
            }
            chars.into_iter().collect()
        })
        .collect::<Vec<String>>()
        .join(" ")
}

/// Convert to uppercase
pub fn uppercase(text: &str) -> String {
    text.to_uppercase()
}

/// Convert to lowercase
pub fn lowercase(text: &str) -> String {
    text.to_lowercase()
}

/// Remove all non-alphanumeric characters
pub fn alphanumeric_only(text: &str) -> String {
    text.chars().filter(|c| c.is_alphanumeric()).collect()
}

/// Remove all non-alphabetic characters
pub fn alphabetic_only(text: &str) -> String {
    text.chars().filter(|c| c.is_alphabetic()).collect()
}

/// Remove all non-numeric characters
pub fn numeric_only(text: &str) -> String {
    text.chars().filter(|c| c.is_numeric()).collect()
}

/// Normalize dashes (replace various dash types with standard dash)
pub fn normalize_dashes(text: &str) -> String {
    text.replace(['–', '—', '―'], "-")
}

/// Remove duplicate words
pub fn remove_duplicate_words(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();

    for word in words {
        if !seen.contains(word) {
            seen.insert(word);
            result.push(word);
        }
    }

    result.join(" ")
}

/// Remove extra spaces (multiple spaces -> single space)
pub fn remove_extra_spaces(text: &str) -> String {
    MULTISPACE_RE.replace_all(text, " ").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleanup() {
        assert_eq!(cleanup("Hello, World!"), "Hello World");
        assert_eq!(cleanup("  Multiple   spaces  "), "Multiple spaces");
        assert_eq!(cleanup("Repeated---punctuation"), "Repeated-punctuation");
        assert_eq!(cleanup("Mixed..up__chars"), "Mixed.up chars");
    }

    #[test]
    fn test_raw_cleanup() {
        assert_eq!(raw_cleanup("Hello.World"), "Hello World");
        assert_eq!(raw_cleanup("Test-File_Name"), "Test File Name");
        assert_eq!(raw_cleanup("Multiple   spaces"), "Multiple spaces");
    }

    #[test]
    fn test_strip() {
        assert_eq!(strip("  Hello World  "), "Hello World");
        assert_eq!(strip("\tTest\n"), "Test");
        assert_eq!(strip("NoSpaces"), "NoSpaces");
    }

    #[test]
    fn test_cleanup_title() {
        assert_eq!(cleanup_title("Movie-2023/1080p"), "Movie 2023 1080p");
        assert_eq!(cleanup_title("Show|Season+1"), "Show Season 1");
        assert_eq!(
            cleanup_title("Title/With\\Separators"),
            "Title With Separators"
        );
    }

    #[test]
    fn test_reorder_title() {
        assert_eq!(reorder_title("  Messy   Title  "), "Messy Title");
        assert_eq!(reorder_title("Clean.Title"), "Clean Title");
    }

    #[test]
    fn test_title_case() {
        assert_eq!(title_case("hello world"), "Hello World");
        assert_eq!(title_case("THIS IS A TEST"), "This Is A Test");
        assert_eq!(title_case("mixed CASE Title"), "Mixed Case Title");
        assert_eq!(title_case(""), "");
    }

    #[test]
    fn test_uppercase() {
        assert_eq!(uppercase("hello"), "HELLO");
        assert_eq!(uppercase("Mixed Case"), "MIXED CASE");
    }

    #[test]
    fn test_lowercase() {
        assert_eq!(lowercase("HELLO"), "hello");
        assert_eq!(lowercase("Mixed Case"), "mixed case");
    }

    #[test]
    fn test_alphanumeric_only() {
        assert_eq!(alphanumeric_only("Hello, World! 123"), "HelloWorld123");
        assert_eq!(alphanumeric_only("Test@#$%^File"), "TestFile");
    }

    #[test]
    fn test_alphabetic_only() {
        assert_eq!(alphabetic_only("Hello123 World!"), "HelloWorld");
        assert_eq!(alphabetic_only("Test@#$%^File"), "TestFile");
    }

    #[test]
    fn test_numeric_only() {
        assert_eq!(numeric_only("Hello123 World!"), "123");
        assert_eq!(numeric_only("Test456File"), "456");
    }

    #[test]
    fn test_normalize_dashes() {
        assert_eq!(normalize_dashes("Hello–World—Test"), "Hello-World-Test");
        assert_eq!(normalize_dashes("No dashes"), "No dashes");
    }

    #[test]
    fn test_remove_duplicate_words() {
        assert_eq!(remove_duplicate_words("hello world hello"), "hello world");
        assert_eq!(remove_duplicate_words("Test Test Test"), "Test");
        assert_eq!(
            remove_duplicate_words("Mixed CASE mixed case"),
            "Mixed CASE mixed case"
        );
    }

    #[test]
    fn test_remove_extra_spaces() {
        assert_eq!(remove_extra_spaces("Multiple   spaces"), "Multiple spaces");
        assert_eq!(
            remove_extra_spaces("  Leading and trailing  "),
            " Leading and trailing "
        );
        assert_eq!(remove_extra_spaces("Single"), "Single");
    }
}
