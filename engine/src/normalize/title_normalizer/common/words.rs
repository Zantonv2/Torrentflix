use crate::normalize::title_normalizer::common::SEPS;
use std::collections::HashSet;

/// Iterator over words in a string, handling separators properly
pub struct WordIterator<'a> {
    text: &'a str,
    position: usize,
    separators: &'a str,
}

impl<'a> WordIterator<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            position: 0,
            separators: SEPS,
        }
    }

    pub fn new_with_separators(text: &'a str, separators: &'a str) -> Self {
        Self {
            text,
            position: 0,
            separators,
        }
    }
}

impl<'a> Iterator for WordIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        // Skip leading separators
        while self.position < self.text.len()
            && self
                .separators
                .contains(self.text.chars().nth(self.position)?)
        {
            self.position += 1;
        }

        if self.position >= self.text.len() {
            return None;
        }

        // Find the next separator
        let start = self.position;
        while self.position < self.text.len()
            && !self
                .separators
                .contains(self.text.chars().nth(self.position)?)
        {
            self.position += 1;
        }

        let word = &self.text[start..self.position];
        if word.is_empty() {
            None
        } else {
            Some(word)
        }
    }
}

/// Split text into words using separators
pub fn split_words(text: &str) -> Vec<&str> {
    WordIterator::new(text).collect()
}

/// Split text into words using custom separators
pub fn split_words_with_separators<'a>(text: &'a str, separators: &'a str) -> Vec<&'a str> {
    WordIterator::new_with_separators(text, separators).collect()
}

/// Check if a string contains only word characters (letters, numbers, underscore)
pub fn is_word(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// Check if a string contains only alphabetic characters
pub fn is_alpha_word(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|c| c.is_alphabetic())
}

/// Check if a string contains only numeric characters
pub fn is_numeric_word(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|c| c.is_numeric())
}

/// Check if a string is a mixed alphanumeric word
pub fn is_alphanumeric_word(text: &str) -> bool {
    !text.is_empty()
        && text.chars().all(|c| c.is_alphanumeric())
        && text.chars().any(|c| c.is_alphabetic())
        && text.chars().any(|c| c.is_numeric())
}

/// Get unique words from a text, preserving order
pub fn get_unique_words(text: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut unique_words = Vec::new();

    for word in WordIterator::new(text) {
        let word_lower = word.to_lowercase();
        if !seen.contains(&word_lower) {
            seen.insert(word_lower);
            unique_words.push(word.to_string());
        }
    }

    unique_words
}

/// Count words in text
pub fn count_words(text: &str) -> usize {
    WordIterator::new(text).count()
}

/// Find the longest word in text
pub fn find_longest_word(text: &str) -> Option<&str> {
    WordIterator::new(text).max_by_key(|word| word.len())
}

/// Find all words of a specific length
pub fn find_words_of_length(text: &str, length: usize) -> Vec<&str> {
    WordIterator::new(text)
        .filter(|word| word.len() == length)
        .collect()
}

/// Check if text contains a specific word (case-insensitive)
pub fn contains_word(text: &str, target_word: &str) -> bool {
    let target_lower = target_word.to_lowercase();
    WordIterator::new(text).any(|word| word.to_lowercase() == target_lower)
}

/// Find all occurrences of a specific word (case-insensitive)
pub fn find_word_occurrences(text: &str, target_word: &str) -> Vec<usize> {
    let target_lower = target_word.to_lowercase();
    let mut positions = Vec::new();
    let mut current_pos = 0;

    for word in WordIterator::new(text) {
        if word.to_lowercase() == target_lower {
            positions.push(current_pos);
        }
        current_pos += word.len() + 1; // +1 for separator
    }

    positions
}

/// Replace all occurrences of a word with another word
pub fn replace_word(text: &str, old_word: &str, new_word: &str) -> String {
    let words: Vec<String> = WordIterator::new(text)
        .map(|word| {
            if word.to_lowercase() == old_word.to_lowercase() {
                new_word.to_string()
            } else {
                word.to_string()
            }
        })
        .collect();

    words.join(" ")
}

/// Extract words that match a pattern
pub fn extract_words_matching<'a>(text: &'a str, pattern: &regex::Regex) -> Vec<&'a str> {
    WordIterator::new(text)
        .filter(|word| pattern.is_match(word))
        .collect()
}

/// Filter words by a predicate
pub fn filter_words<F>(text: &str, predicate: F) -> Vec<&str>
where
    F: Fn(&str) -> bool,
{
    WordIterator::new(text)
        .filter(|word| predicate(word))
        .collect()
}

/// Get word statistics for a text
#[derive(Debug, Clone)]
pub struct WordStats {
    pub total_words: usize,
    pub unique_words: usize,
    pub average_word_length: f64,
    pub longest_word: Option<String>,
    pub shortest_word: Option<String>,
}

pub fn get_word_stats(text: &str) -> WordStats {
    let words: Vec<&str> = WordIterator::new(text).collect();

    if words.is_empty() {
        return WordStats {
            total_words: 0,
            unique_words: 0,
            average_word_length: 0.0,
            longest_word: None,
            shortest_word: None,
        };
    }

    let unique_count: HashSet<_> = words.iter().map(|w| w.to_lowercase()).collect();
    let total_length: usize = words.iter().map(|w| w.len()).sum();
    let average_length = total_length as f64 / words.len() as f64;

    let longest = words.iter().max_by_key(|w| w.len()).map(|w| w.to_string());
    let shortest = words.iter().min_by_key(|w| w.len()).map(|w| w.to_string());

    WordStats {
        total_words: words.len(),
        unique_words: unique_count.len(),
        average_word_length: average_length,
        longest_word: longest,
        shortest_word: shortest,
    }
}

/// Check if a word looks like a number (contains digits)
pub fn is_numeric_like(word: &str) -> bool {
    word.chars().any(|c| c.is_numeric())
}

/// Check if a word looks like an abbreviation (all uppercase and short)
pub fn is_abbreviation(word: &str) -> bool {
    word.len() <= 6 && word.chars().all(|c| c.is_uppercase())
}

/// Check if a word is in title case (first letter uppercase, rest lowercase)
pub fn is_title_case(word: &str) -> bool {
    if word.is_empty() {
        return false;
    }

    let chars: Vec<char> = word.chars().collect();
    chars[0].is_uppercase() && chars.iter().skip(1).all(|c| c.is_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_word_iterator() {
        let text = "Hello World Test";
        let words: Vec<&str> = WordIterator::new(text).collect();
        assert_eq!(words, vec!["Hello", "World", "Test"]);

        let text = "Multiple   spaces   here";
        let words: Vec<&str> = WordIterator::new(text).collect();
        assert_eq!(words, vec!["Multiple", "spaces", "here"]);

        let text = "  Leading and trailing  ";
        let words: Vec<&str> = WordIterator::new(text).collect();
        assert_eq!(words, vec!["Leading", "and", "trailing"]);
    }

    #[test]
    fn test_split_words() {
        assert_eq!(
            split_words("Hello.World-Test"),
            vec!["Hello", "World", "Test"]
        );
        assert_eq!(split_words("Multiple   spaces"), vec!["Multiple", "spaces"]);
        assert_eq!(split_words(""), Vec::<&str>::new());
    }

    #[test]
    fn test_is_word() {
        assert!(is_word("hello"));
        assert!(is_word("test123"));
        assert!(is_word("_underscore"));
        assert!(!is_word("hello-world"));
        assert!(!is_word(""));
    }

    #[test]
    fn test_is_alpha_word() {
        assert!(is_alpha_word("hello"));
        assert!(is_alpha_word("World"));
        assert!(!is_alpha_word("test123"));
        assert!(!is_alpha_word("test_123"));
    }

    #[test]
    fn test_is_numeric_word() {
        assert!(is_numeric_word("123"));
        assert!(is_numeric_word("456789"));
        assert!(!is_numeric_word("test123"));
        assert!(!is_numeric_word("12a34"));
    }

    #[test]
    fn test_is_alphanumeric_word() {
        assert!(is_alphanumeric_word("test123"));
        assert!(is_alphanumeric_word("ABC123"));
        assert!(!is_alphanumeric_word("test"));
        assert!(!is_alphanumeric_word("123"));
    }

    #[test]
    fn test_get_unique_words() {
        let text = "hello world hello test world";
        let unique = get_unique_words(text);
        assert_eq!(unique, vec!["hello", "world", "test"]);
    }

    #[test]
    fn test_count_words() {
        assert_eq!(count_words("Hello World Test"), 3);
        assert_eq!(count_words("Multiple   spaces   here"), 3);
        assert_eq!(count_words(""), 0);
    }

    #[test]
    fn test_find_longest_word() {
        assert_eq!(find_longest_word("short longest word"), Some("longest"));
        assert_eq!(find_longest_word(""), None);
    }

    #[test]
    fn test_find_words_of_length() {
        let words = find_words_of_length("cat dog fish bird", 3);
        assert_eq!(words, vec!["cat", "dog"]);
    }

    #[test]
    fn test_contains_word() {
        assert!(contains_word("Hello World Test", "world"));
        assert!(contains_word("Hello WORLD Test", "world"));
        assert!(!contains_word("Hello Test", "world"));
    }

    #[test]
    fn test_replace_word() {
        assert_eq!(
            replace_word("hello world hello", "world", "earth"),
            "hello earth hello"
        );
        assert_eq!(replace_word("HELLO world", "hello", "hi"), "hi world");
    }

    #[test]
    fn test_extract_words_matching() {
        let pattern = Regex::new(r"^[A-Z][a-z]+$").unwrap();
        let words = extract_words_matching("Hello world Test Pattern", &pattern);
        assert_eq!(words, vec!["Hello", "Test", "Pattern"]);
    }

    #[test]
    fn test_filter_words() {
        let words = filter_words("hello world test123 456", |word| word.len() > 4);
        assert_eq!(words, vec!["hello", "world", "test123"]);
    }

    #[test]
    fn test_get_word_stats() {
        let text = "hello world hello test";
        let stats = get_word_stats(text);

        assert_eq!(stats.total_words, 4);
        assert_eq!(stats.unique_words, 3);
        assert!(stats.average_word_length > 0.0);
        assert_eq!(stats.longest_word, Some("hello".to_string()));
        assert_eq!(stats.shortest_word, Some("test".to_string()));

        let empty_stats = get_word_stats("");
        assert_eq!(empty_stats.total_words, 0);
    }

    #[test]
    fn test_is_numeric_like() {
        assert!(is_numeric_like("test123"));
        assert!(is_numeric_like("123"));
        assert!(is_numeric_like("version1.2"));
        assert!(!is_numeric_like("hello"));
    }

    #[test]
    fn test_is_abbreviation() {
        assert!(is_abbreviation("USA"));
        assert!(is_abbreviation("BBC"));
        assert!(!is_abbreviation("TOOLONG"));
        assert!(!is_abbreviation("Hello"));
    }

    #[test]
    fn test_is_title_case() {
        assert!(is_title_case("Hello"));
        assert!(is_title_case("World"));
        assert!(!is_title_case("HELLO"));
        assert!(!is_title_case("hello"));
        assert!(!is_title_case(""));
    }
}
