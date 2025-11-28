use std::cmp::Ordering;

/// Compare two strings case-insensitively
pub fn case_insensitive_cmp(a: &str, b: &str) -> Ordering {
    a.to_lowercase().cmp(&b.to_lowercase())
}

/// Compare two strings by length
pub fn length_cmp(a: &str, b: &str) -> Ordering {
    a.len().cmp(&b.len())
}

/// Compare two strings by length in reverse order (longer first)
pub fn length_reverse_cmp(a: &str, b: &str) -> Ordering {
    b.len().cmp(&a.len())
}

/// Compare two strings by their numeric value if they're numbers, otherwise lexicographically
pub fn numeric_or_lexical_cmp(a: &str, b: &str) -> Ordering {
    if let (Ok(a_num), Ok(b_num)) = (a.parse::<i64>(), b.parse::<i64>()) {
        a_num.cmp(&b_num)
    } else if a.parse::<i64>().is_ok() && b.parse::<i64>().is_err() {
        // Numeric strings are considered greater than non-numeric strings
        Ordering::Greater
    } else if a.parse::<i64>().is_err() && b.parse::<i64>().is_ok() {
        // Non-numeric strings are considered less than numeric strings
        Ordering::Less
    } else {
        a.cmp(b)
    }
}

/// Compare two strings by their numeric value if they're floats, otherwise lexicographically
pub fn float_or_lexical_cmp(a: &str, b: &str) -> Ordering {
    if let (Ok(a_num), Ok(b_num)) = (a.parse::<f64>(), b.parse::<f64>()) {
        a_num.partial_cmp(&b_num).unwrap_or(Ordering::Equal)
    } else {
        a.cmp(b)
    }
}

/// Compare two strings by their position in a predefined list
pub fn list_position_cmp<T: PartialEq>(list: &[T], a: &T, b: &T) -> Ordering {
    let a_pos = list.iter().position(|item| item == a);
    let b_pos = list.iter().position(|item| item == b);
    
    match (a_pos, b_pos) {
        (Some(a_idx), Some(b_idx)) => a_idx.cmp(&b_idx),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

/// Compare two markers by their properties (simplified version of Rust metadata parser marker_sorted)
pub fn marker_sorted_cmp(a: &str, b: &str) -> Ordering {
    // This is a simplified version - in Rust metadata parser it's much more complex
    // For now, just use case-insensitive comparison
    case_insensitive_cmp(a, b)
}

/// Compare two strings by the number of words they contain
pub fn word_count_cmp(a: &str, b: &str) -> Ordering {
    let a_words = a.split_whitespace().count();
    let b_words = b.split_whitespace().count();
    a_words.cmp(&b_words)
}

/// Compare two strings by their alphabetical order, but with numbers sorted numerically
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    let a_parts = split_numeric_parts(a);
    let b_parts = split_numeric_parts(b);
    
    for (a_part, b_part) in a_parts.iter().zip(b_parts.iter()) {
        match (a_part, b_part) {
            (NumericPart::Num(a_num), NumericPart::Num(b_num)) => {
                match a_num.cmp(b_num) {
                    Ordering::Equal => continue,
                    other => return other,
                }
            }
            (NumericPart::Text(a_text), NumericPart::Text(b_text)) => {
                match a_text.cmp(b_text) {
                    Ordering::Equal => continue,
                    other => return other,
                }
            }
            (NumericPart::Num(_), NumericPart::Text(_)) => return Ordering::Less,
            (NumericPart::Text(_), NumericPart::Num(_)) => return Ordering::Greater,
        }
    }
    
    a_parts.len().cmp(&b_parts.len())
}

/// Enum representing parts of a string for natural sorting
#[derive(Debug, PartialEq)]
enum NumericPart {
    Num(i64),
    Text(String),
}

/// Split a string into numeric and text parts for natural sorting
fn split_numeric_parts(s: &str) -> Vec<NumericPart> {
    let mut parts = Vec::new();
    let mut current_num = String::new();
    let mut current_text = String::new();
    let mut in_number = false;
    
    for ch in s.chars() {
        if ch.is_numeric() {
            if !in_number {
                if !current_text.is_empty() {
                    parts.push(NumericPart::Text(current_text.clone()));
                    current_text.clear();
                }
                in_number = true;
            }
            current_num.push(ch);
        } else {
            if in_number {
                if !current_num.is_empty() {
                    if let Ok(num) = current_num.parse::<i64>() {
                        parts.push(NumericPart::Num(num));
                    }
                    current_num.clear();
                }
                in_number = false;
            }
            current_text.push(ch);
        }
    }
    
    // Add remaining parts
    if !current_num.is_empty() {
        if let Ok(num) = current_num.parse::<i64>() {
            parts.push(NumericPart::Num(num));
        }
    }
    if !current_text.is_empty() {
        parts.push(NumericPart::Text(current_text));
    }
    
    parts
}

/// Compare two strings by their similarity to a target string
pub fn similarity_cmp(target: &str, a: &str, b: &str) -> Ordering {
    let a_similarity = calculate_similarity(target, a);
    let b_similarity = calculate_similarity(target, b);
    b_similarity.partial_cmp(&a_similarity).unwrap_or(Ordering::Equal)
}

/// Calculate similarity between two strings (simple Levenshtein distance based)
fn calculate_similarity(a: &str, b: &str) -> f64 {
    let distance = levenshtein_distance(a, b);
    let max_len = a.len().max(b.len());
    
    if max_len == 0 {
        1.0
    } else {
        1.0 - (distance as f64 / max_len as f64)
    }
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();
    
    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }
    
    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];
    
    // Initialize first row and column
    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }
    
    // Fill the matrix
    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = [
                matrix[i - 1][j] + 1,      // deletion
                matrix[i][j - 1] + 1,      // insertion
                matrix[i - 1][j - 1] + cost, // substitution
            ].iter().min().copied().unwrap();
        }
    }
    
    matrix[a_len][b_len]
}

/// Compare two strings by how well they match a pattern (regex)
pub fn pattern_match_cmp(pattern: &regex::Regex, a: &str, b: &str) -> Ordering {
    let a_match = pattern.is_match(a);
    let b_match = pattern.is_match(b);
    
    match (a_match, b_match) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        (true, true) | (false, false) => Ordering::Equal,
    }
}

/// Compare two strings by their position in alphabetical order, but with custom priority
pub fn priority_cmp(priority_list: &[&str], a: &str, b: &str) -> Ordering {
    let a_priority = priority_list.iter().position(|&item| item == a);
    let b_priority = priority_list.iter().position(|&item| item == b);
    
    match (a_priority, b_priority) {
        (Some(a_idx), Some(b_idx)) => a_idx.cmp(&b_idx),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => case_insensitive_cmp(a, b),
    }
}

/// Sort a slice of strings using a custom comparator
pub fn sort_with<T, F>(slice: &mut [T], mut cmp: F)
where
    F: FnMut(&T, &T) -> Ordering,
{
    slice.sort_by(&mut cmp);
}

/// Sort a slice of strings case-insensitively
pub fn sort_case_insensitive<T: AsRef<str>>(slice: &mut [T]) {
    slice.sort_by(|a, b| case_insensitive_cmp(a.as_ref(), b.as_ref()));
}

/// Sort a slice of strings by length
pub fn sort_by_length<T: AsRef<str>>(slice: &mut [T]) {
    slice.sort_by(|a, b| length_cmp(a.as_ref(), b.as_ref()));
}

/// Sort a slice of strings naturally
pub fn sort_naturally<T: AsRef<str>>(slice: &mut [T]) {
    slice.sort_by(|a, b| natural_cmp(a.as_ref(), b.as_ref()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    
    #[test]
    fn test_case_insensitive_cmp() {
        assert_eq!(case_insensitive_cmp("apple", "Banana"), Ordering::Less);
        assert_eq!(case_insensitive_cmp("Apple", "apple"), Ordering::Equal);
        assert_eq!(case_insensitive_cmp("zebra", "Apple"), Ordering::Greater);
    }
    
    #[test]
    fn test_length_cmp() {
        assert_eq!(length_cmp("short", "longer"), Ordering::Less);
        assert_eq!(length_cmp("equal", "equal"), Ordering::Equal);
        assert_eq!(length_cmp("longer", "short"), Ordering::Greater);
    }
    
    #[test]
    fn test_length_reverse_cmp() {
        assert_eq!(length_reverse_cmp("short", "longer"), Ordering::Greater);
        assert_eq!(length_reverse_cmp("longer", "short"), Ordering::Less);
    }
    
    #[test]
    fn test_numeric_or_lexical_cmp() {
        assert_eq!(numeric_or_lexical_cmp("10", "2"), Ordering::Greater);
        assert_eq!(numeric_or_lexical_cmp("apple", "banana"), Ordering::Less);
        assert_eq!(numeric_or_lexical_cmp("123", "abc"), Ordering::Greater);
    }
    
    #[test]
    fn test_float_or_lexical_cmp() {
        assert_eq!(float_or_lexical_cmp("3.14", "2.71"), Ordering::Greater);
        assert_eq!(float_or_lexical_cmp("apple", "banana"), Ordering::Less);
    }
    
    #[test]
    fn test_list_position_cmp() {
        let list = vec!["apple", "banana", "cherry"];
        assert_eq!(list_position_cmp(&list, &"apple", &"cherry"), Ordering::Less);
        assert_eq!(list_position_cmp(&list, &"banana", &"apple"), Ordering::Greater);
        assert_eq!(list_position_cmp(&list, &"apple", &"orange"), Ordering::Less);
        assert_eq!(list_position_cmp(&list, &"orange", &"pear"), Ordering::Equal);
    }
    
    #[test]
    fn test_word_count_cmp() {
        assert_eq!(word_count_cmp("one two", "one"), Ordering::Greater);
        assert_eq!(word_count_cmp("single", "multiple words here"), Ordering::Less);
        assert_eq!(word_count_cmp("equal words", "same count"), Ordering::Equal);
    }
    
    #[test]
    fn test_natural_cmp() {
        assert_eq!(natural_cmp("file1.txt", "file10.txt"), Ordering::Less);
        assert_eq!(natural_cmp("file10.txt", "file2.txt"), Ordering::Greater);
        assert_eq!(natural_cmp("apple", "banana"), Ordering::Less);
        assert_eq!(natural_cmp("version1.2", "version1.10"), Ordering::Less);
    }
    
    #[test]
    fn test_split_numeric_parts() {
        let parts = split_numeric_parts("file123text456");
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0], NumericPart::Text("file".to_string()));
        assert_eq!(parts[1], NumericPart::Num(123));
        assert_eq!(parts[2], NumericPart::Text("text".to_string()));
        assert_eq!(parts[3], NumericPart::Num(456));
    }
    
    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("a", ""), 1);
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("flaw", "lawn"), 2);
    }
    
    #[test]
    fn test_calculate_similarity() {
        assert_eq!(calculate_similarity("", ""), 1.0);
        assert_eq!(calculate_similarity("same", "same"), 1.0);
        assert!(calculate_similarity("kitten", "sitting") > 0.5);
        assert!(calculate_similarity("completely", "different") < 0.5);
    }
    
    #[test]
    fn test_similarity_cmp() {
        let target = "hello";
        assert_eq!(similarity_cmp(target, "hello", "world"), Ordering::Less);
        assert_eq!(similarity_cmp(target, "world", "hello"), Ordering::Greater);
    }
    
    #[test]
    fn test_pattern_match_cmp() {
        let pattern = Regex::new(r"^\d+$").unwrap();
        assert_eq!(pattern_match_cmp(&pattern, "123", "abc"), Ordering::Less);
        assert_eq!(pattern_match_cmp(&pattern, "abc", "123"), Ordering::Greater);
        assert_eq!(pattern_match_cmp(&pattern, "123", "456"), Ordering::Equal);
        assert_eq!(pattern_match_cmp(&pattern, "abc", "def"), Ordering::Equal);
    }
    
    #[test]
    fn test_priority_cmp() {
        let priority = vec!["high", "medium", "low"];
        assert_eq!(priority_cmp(&priority, "high", "low"), Ordering::Less);
        assert_eq!(priority_cmp(&priority, "medium", "high"), Ordering::Greater);
        assert_eq!(priority_cmp(&priority, "unknown", "high"), Ordering::Greater);
        assert_eq!(priority_cmp(&priority, "unknown", "unknown"), Ordering::Equal);
    }
    
    #[test]
    fn test_sort_case_insensitive() {
        let mut items = vec!["Banana", "apple", "Cherry"];
        sort_case_insensitive(&mut items);
        assert_eq!(items, vec!["apple", "Banana", "Cherry"]);
    }
    
    #[test]
    fn test_sort_by_length() {
        let mut items = vec!["short", "very long", "medium"];
        sort_by_length(&mut items);
        assert_eq!(items, vec!["short", "medium", "very long"]);
    }
    
    #[test]
    fn test_sort_naturally() {
        let mut items = vec!["file10.txt", "file2.txt", "file1.txt"];
        sort_naturally(&mut items);
        assert_eq!(items, vec!["file1.txt", "file2.txt", "file10.txt"]);
    }
}
