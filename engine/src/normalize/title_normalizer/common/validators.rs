use regex::Regex;
use crate::title_normalizer::common::{SEPS, SEPS_NO_GROUPS, SEPS_NO_FS};

lazy_static::lazy_static! {
    static ref SEPS_RE: Regex = Regex::new(&format!("[{}]", regex::escape(SEPS))).unwrap();
    static ref SEPS_NO_GROUPS_RE: Regex = Regex::new(&format!("[{}]", regex::escape(SEPS_NO_GROUPS))).unwrap();
    static ref SEPS_NO_FS_RE: Regex = Regex::new(&format!("[{}]", regex::escape(SEPS_NO_FS))).unwrap();
    static ref INT_RE: Regex = Regex::new(r"^\d+$").unwrap();
}

/// Check if separators are present before the match
pub fn seps_before(string: &str, match_start: usize, _match_end: usize) -> bool {
    if match_start == 0 {
        return true; // Beginning of string is valid (no invalid chars)
    }
    
    let before = &string[..match_start];
    SEPS_RE.is_match(before)
}

/// Check if separators are present after the match
pub fn seps_after(string: &str, _match_start: usize, match_end: usize) -> bool {
    if match_end >= string.len() {
        return true; // End of string is considered valid
    }
    
    let after = &string[match_end..];
    SEPS_RE.is_match(after)
}

/// Check if separators are present both before and after the match
pub fn seps_surround(string: &str, match_start: usize, match_end: usize) -> bool {
    seps_before(string, match_start, match_end) && seps_after(string, match_start, match_end)
}

/// Check if separators (no groups) are present before the match
pub fn seps_before_no_groups(string: &str, match_start: usize, _match_end: usize) -> bool {
    if match_start == 0 {
        return true; // Beginning of string is valid (no invalid chars)
    }
    
    let before = &string[..match_start];
    SEPS_NO_GROUPS_RE.is_match(before)
}

/// Check if separators (no groups) are present after the match
pub fn seps_after_no_groups(string: &str, _match_start: usize, match_end: usize) -> bool {
    if match_end >= string.len() {
        return true;
    }
    
    let after = &string[match_end..];
    SEPS_NO_GROUPS_RE.is_match(after)
}

/// Check if separators (no groups) are present both before and after the match
pub fn seps_surround_no_groups(string: &str, match_start: usize, match_end: usize) -> bool {
    seps_before_no_groups(string, match_start, match_end) && seps_after_no_groups(string, match_start, match_end)
}

/// Check if separators (no filesystem) are present before the match
pub fn seps_before_no_fs(string: &str, match_start: usize, _match_end: usize) -> bool {
    if match_start == 0 {
        return true; // Beginning of string is valid (no invalid chars)
    }
    
    let before = &string[..match_start];
    SEPS_NO_FS_RE.is_match(before)
}

/// Check if separators (no filesystem) are present after the match
pub fn seps_after_no_fs(string: &str, _match_start: usize, match_end: usize) -> bool {
    if match_end >= string.len() {
        return true;
    }
    
    let after = &string[match_end..];
    SEPS_NO_FS_RE.is_match(after)
}

/// Check if separators (no filesystem) are present both before and after the match
pub fn seps_surround_no_fs(string: &str, match_start: usize, match_end: usize) -> bool {
    seps_before_no_fs(string, match_start, match_end) && seps_after_no_fs(string, match_start, match_end)
}

/// Check if the string can be coerced to an integer
pub fn int_coercable(string: &str, _match_start: usize, _match_end: usize) -> bool {
    INT_RE.is_match(string.trim())
}

/// Logical AND operation for validator functions
pub fn and(validators: &[fn(&str, usize, usize) -> bool]) -> impl Fn(&str, usize, usize) -> bool + '_ {
    move |string: &str, start: usize, end: usize| {
        validators.iter().all(|validator| validator(string, start, end))
    }
}

/// Logical OR operation for validator functions
pub fn or(validators: &[fn(&str, usize, usize) -> bool]) -> impl Fn(&str, usize, usize) -> bool + '_ {
    move |string: &str, start: usize, end: usize| {
        validators.iter().any(|validator| validator(string, start, end))
    }
}

/// Check if the match is at the beginning of the string
pub fn at_beginning(_string: &str, match_start: usize, _match_end: usize) -> bool {
    match_start == 0
}

/// Check if the match is at the end of the string
pub fn at_end(string: &str, _match_start: usize, match_end: usize) -> bool {
    match_end >= string.len()
}

/// Check if the match is surrounded by word boundaries
pub fn word_boundary(string: &str, match_start: usize, match_end: usize) -> bool {
    let before_valid = if match_start == 0 {
        true
    } else {
        let before_char = string.chars().nth(match_start - 1).unwrap_or(' ');
        !before_char.is_alphanumeric() && before_char != '_'
    };
    
    let after_valid = if match_end >= string.len() {
        true
    } else {
        let after_char = string.chars().nth(match_end).unwrap_or(' ');
        !after_char.is_alphanumeric() && after_char != '_'
    };
    
    before_valid && after_valid
}

/// Check if the match is not part of a larger word
pub fn standalone(string: &str, match_start: usize, match_end: usize) -> bool {
    word_boundary(string, match_start, match_end)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_seps_before() {
        assert!(seps_before(" test", 1, 5)); // Space before
        assert!(seps_before("test", 0, 4)); // Beginning of string is valid
        assert!(seps_before("-test", 1, 5)); // Dash before
        assert!(seps_before(".test", 1, 5)); // Dot before
    }
    
    #[test]
    fn test_seps_after() {
        assert!(seps_after("test ", 4, 5)); // Space after
        assert!(seps_after("test.", 4, 5)); // Dot after
        assert!(seps_after("test-", 4, 5)); // Dash after
        assert!(seps_after("test", 4, 4));  // End of string is valid
        assert!(seps_after("testa", 4, 5)); // No separator after but still valid
    }
    
    #[test]
    fn test_seps_surround() {
        assert!(seps_surround(" test ", 1, 5)); // Surrounded by spaces
        assert!(seps_surround("-test.", 1, 5)); // Surrounded by dash and dot
        assert!(seps_surround("test", 0, 4));   // Whole string
        assert!(!seps_surround("atest", 1, 5)); // No separator before
        assert!(!seps_surround("testa", 0, 4)); // No separator after
    }
    
    #[test]
    fn test_seps_before_no_groups() {
        assert!(seps_before_no_groups(" test", 1, 5)); // Space before
        assert!(seps_before_no_groups("[test", 1, 5)); // Bracket before is valid (not in SEPS_NO_GROUPS)
        assert!(seps_before_no_groups("test", 0, 4));  // Beginning of string is valid
    }
    
    #[test]
    fn test_seps_after_no_groups() {
        assert!(seps_after_no_groups("test ", 4, 5)); // Space after
        assert!(seps_after_no_groups("test]", 4, 5)); // Bracket after is valid (not in SEPS_NO_GROUPS)
        assert!(seps_after_no_groups("test", 4, 4));  // End of string
    }
    
    #[test]
    fn test_seps_before_no_fs() {
        assert!(seps_before_no_fs(" test", 1, 5)); // Space before
        assert!(!seps_before_no_fs("/test", 1, 5)); // Slash before (not allowed)
        assert!(seps_before_no_fs("test", 0, 4));  // Beginning of string is valid
    }
    
    #[test]
    fn test_seps_after_no_fs() {
        assert!(seps_after_no_fs("test ", 4, 5)); // Space after
        assert!(seps_after_no_fs("test\\", 4, 5)); // Backslash after is valid (not in SEPS_NO_FS)
        assert!(seps_after_no_fs("test", 4, 4));  // End of string
    }
    
    #[test]
    fn test_int_coercable() {
        assert!(int_coercable("123", 0, 3));
        assert!(int_coercable("456", 0, 3));
        assert!(int_coercable(" 789 ", 0, 5)); // With whitespace
        assert!(!int_coercable("abc", 0, 3));
        assert!(!int_coercable("12.34", 0, 5));
        assert!(!int_coercable("12a34", 0, 5));
    }
    
    #[test]
    fn test_and_validator() {
        let validators = vec![seps_before, seps_after];
        assert!(and(&validators[..])(" test ", 1, 5)); // Both conditions true
        assert!(and(&validators[..])("test ", 0, 4)); // Both conditions true (beginning is valid)
        assert!(!and(&validators[..])("atesta", 1, 5)); // Both conditions false (no separators)
    }
    
    #[test]
    fn test_or_validator() {
        let validators = vec![at_beginning, at_end];
        assert!(or(&validators[..])("test", 0, 4)); // Both conditions true
        assert!(!or(&validators[..])(" test ", 1, 5)); // Both conditions false
        assert!(or(&validators[..])(" test", 1, 5)); // Neither condition true
    }
    
    #[test]
    fn test_at_beginning() {
        assert!(at_beginning("test", 0, 4));
        assert!(!at_beginning(" test", 1, 5));
    }
    
    #[test]
    fn test_at_end() {
        assert!(at_end("test", 0, 4));
        assert!(!at_end("test ", 0, 4));
    }
    
    #[test]
    fn test_word_boundary() {
        assert!(word_boundary(" test ", 1, 5)); // Surrounded by spaces
        assert!(word_boundary("test", 0, 4));   // Whole string
        assert!(!word_boundary("atest", 1, 5)); // No word boundary before
        assert!(!word_boundary("testa", 0, 4)); // No word boundary after
    }
    
    #[test]
    fn test_standalone() {
        assert!(standalone(" test ", 1, 5)); // Standalone word
        assert!(standalone("test", 0, 4));   // Whole string
        assert!(!standalone("testing", 0, 4)); // Part of larger word
        assert!(!standalone("unittest", 4, 8)); // Part of larger word
    }
}
