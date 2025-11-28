use regex::Regex;
use std::collections::HashMap;

pub const DIGITAL_NUMERAL: &str = r"\d{1,4}";

pub const ROMAN_NUMERAL: &str = r"M{0,4}(?:CM|CD|D?C{0,3})(?:XC|XL|L?X{0,3})(?:IX|IV|V?I{0,3})";

pub const ENGLISH_WORD_NUMERAL_LIST: &[&str] = &[
    "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
    "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen", "seventeen", "eighteen", "nineteen", "twenty"
];

pub const FRENCH_WORD_NUMERAL_LIST: &[&str] = &[
    "zéro", "un", "deux", "trois", "quatre", "cinq", "six", "sept", "huit", "neuf", "dix",
    "onze", "douze", "treize", "quatorze", "quinze", "seize", "dix-sept", "dix-huit", "dix-neuf", "vingt"
];

pub const FRENCH_ALT_WORD_NUMERAL_LIST: &[&str] = &[
    "zero", "une", "deux", "trois", "quatre", "cinq", "six", "sept", "huit", "neuf", "dix",
    "onze", "douze", "treize", "quatorze", "quinze", "seize", "dixsept", "dixhuit", "dixneuf", "vingt"
];

lazy_static::lazy_static! {
    static ref WORD_NUMERAL_MAP: HashMap<String, i32> = {
        let mut map = HashMap::new();
        
        // English words
        for (i, word) in ENGLISH_WORD_NUMERAL_LIST.iter().enumerate() {
            map.insert(word.to_string(), i as i32);
        }
        
        // French words
        for (i, word) in FRENCH_WORD_NUMERAL_LIST.iter().enumerate() {
            map.insert(word.to_string(), i as i32);
        }
        
        // French alternative words
        for (i, word) in FRENCH_ALT_WORD_NUMERAL_LIST.iter().enumerate() {
            map.insert(word.to_string(), i as i32);
        }
        
        map
    };
    
    static ref WORD_NUMERAL_REGEX: Regex = {
        let all_words: Vec<String> = WORD_NUMERAL_MAP.keys().cloned().collect();
        let pattern = format!(r"(?:(?=\w+)(?:{}))", all_words.join("|"));
        Regex::new(&pattern).unwrap()
    };
    
    static ref ROMAN_NUMERAL_MAP: Vec<(&'static str, i32)> = vec![
        ("M", 1000), ("CM", 900), ("D", 500), ("CD", 400), ("C", 100),
        ("XC", 90), ("L", 50), ("XL", 40), ("X", 10), ("IX", 9),
        ("V", 5), ("IV", 4), ("I", 1)
    ];
    
    static ref ROMAN_NUMERAL_REGEX: Regex = Regex::new(&format!("^{}$", ROMAN_NUMERAL)).unwrap();
    
    static ref CLEAN_RE: Regex = Regex::new(r"[^\d]*(\d+)[^\d]*").unwrap();
}

pub fn build_word_numeral() -> String {
    let all_words: Vec<String> = WORD_NUMERAL_MAP.keys().cloned().collect();
    format!(r"(?:(?=\w+)(?:{}))", all_words.join("|"))
}

pub fn get_word_numeral() -> &'static str {
    lazy_static::initialize(&WORD_NUMERAL_REGEX);
    WORD_NUMERAL_REGEX.as_str()
}

pub fn get_numeral() -> String {
    format!("(?:{}|{}|{})", DIGITAL_NUMERAL, ROMAN_NUMERAL, get_word_numeral())
}

fn parse_roman(value: &str) -> Result<i32, String> {
    if !ROMAN_NUMERAL_REGEX.is_match(value) {
        return Err(format!("Invalid Roman numeral: {}", value));
    }
    
    let mut result = 0;
    let mut index = 0;
    
    for (roman, integer) in ROMAN_NUMERAL_MAP.iter() {
        while value.get(index..index + roman.len()) == Some(roman) {
            result += integer;
            index += roman.len();
        }
    }
    
    Ok(result)
}

fn parse_word(value: &str) -> Result<i32, String> {
    let lowercase = value.to_lowercase();
    WORD_NUMERAL_MAP.get(&lowercase)
        .copied()
        .ok_or_else(|| format!("Invalid word numeral: {}", value))
}

pub fn parse_numeral(
    value: &str,
    int_enabled: bool,
    roman_enabled: bool,
    word_enabled: bool,
    clean: bool
) -> Result<i32, String> {
    // Try integer parsing
    if int_enabled {
        if clean {
            if let Some(captures) = CLEAN_RE.captures(value) {
                if let Some(num_match) = captures.get(1) {
                    if let Ok(num) = num_match.as_str().parse::<i32>() {
                        return Ok(num);
                    }
                }
            }
        } else if let Ok(num) = value.parse::<i32>() {
            return Ok(num);
        }
    }
    
    // Try Roman numeral parsing
    if roman_enabled {
        if clean {
            for word in value.split_whitespace() {
                if let Ok(num) = parse_roman(word) {
                    return Ok(num);
                }
            }
        } else if let Ok(num) = parse_roman(value) {
            return Ok(num);
        }
    }
    
    // Try word numeral parsing
    if word_enabled {
        if clean {
            for word in value.split_whitespace() {
                if let Ok(num) = parse_word(word) {
                    return Ok(num);
                }
            }
        } else if let Ok(num) = parse_word(value) {
            return Ok(num);
        }
    }
    
    Err(format!("Invalid numeral: {}", value))
}

/// Convenience function with all parsing enabled
pub fn parse_numeral_all(value: &str) -> Result<i32, String> {
    parse_numeral(value, true, true, true, true)
}

/// Convenience function for clean parsing with all formats enabled
pub fn parse_numeral_clean(value: &str) -> Result<i32, String> {
    parse_numeral(value, true, true, true, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_digital_numeral() {
        assert_eq!(parse_numeral_all("123").unwrap(), 123);
        assert_eq!(parse_numeral_all("45").unwrap(), 45);
    }
    
    #[test]
    fn test_parse_roman_numeral() {
        assert_eq!(parse_numeral_all("IV").unwrap(), 4);
        assert_eq!(parse_numeral_all("IX").unwrap(), 9);
        assert_eq!(parse_numeral_all("X").unwrap(), 10);
        assert_eq!(parse_numeral_all("XL").unwrap(), 40);
        assert_eq!(parse_numeral_all("XC").unwrap(), 90);
        assert_eq!(parse_numeral_all("CD").unwrap(), 400);
        assert_eq!(parse_numeral_all("CM").unwrap(), 900);
        assert_eq!(parse_numeral_all("MCMXCIV").unwrap(), 1994);
    }
    
    #[test]
    fn test_parse_english_word_numeral() {
        assert_eq!(parse_numeral_all("one").unwrap(), 1);
        assert_eq!(parse_numeral_all("five").unwrap(), 5);
        assert_eq!(parse_numeral_all("ten").unwrap(), 10);
        assert_eq!(parse_numeral_all("twenty").unwrap(), 20);
    }
    
    #[test]
    fn test_parse_french_word_numeral() {
        assert_eq!(parse_numeral_all("un").unwrap(), 1);
        assert_eq!(parse_numeral_all("deux").unwrap(), 2);
        assert_eq!(parse_numeral_all("dix").unwrap(), 10);
        assert_eq!(parse_numeral_all("vingt").unwrap(), 20);
    }
    
    #[test]
    fn test_parse_clean_numeral() {
        assert_eq!(parse_numeral_clean("Episode 123").unwrap(), 123);
        assert_eq!(parse_numeral_clean("Season IV").unwrap(), 4);
        assert_eq!(parse_numeral_clean("Part two").unwrap(), 2);
    }
    
    #[test]
    fn test_parse_invalid_numeral() {
        assert!(parse_numeral_all("invalid").is_err());
        assert!(parse_numeral_all("XYZ").is_err());
    }
    
    #[test]
    fn test_parse_numeral_with_options() {
        // Only integer enabled
        assert_eq!(parse_numeral("123", true, false, false, true).unwrap(), 123);
        assert!(parse_numeral("IV", true, false, false, true).is_err());
        
        // Only Roman enabled
        assert!(parse_numeral("123", false, true, false, true).is_err());
        assert_eq!(parse_numeral("IV", false, true, false, true).unwrap(), 4);
        
        // Only word enabled
        assert!(parse_numeral("123", false, false, true, true).is_err());
        assert_eq!(parse_numeral("one", false, false, true, true).unwrap(), 1);
    }
    
    #[test]
    fn test_roman_numeral_validation() {
        assert!(parse_roman("IV").is_ok());
        assert!(parse_roman("MCMXCIV").is_ok());
        assert!(parse_roman("IC").is_err()); // Invalid Roman numeral
        assert!(parse_roman("VV").is_err()); // Invalid Roman numeral
    }
    
    #[test]
    fn test_word_numeral_mapping() {
        assert_eq!(WORD_NUMERAL_MAP.get("zero"), Some(&0));
        assert_eq!(WORD_NUMERAL_MAP.get("twenty"), Some(&20));
        assert_eq!(WORD_NUMERAL_MAP.get("vingt"), Some(&20));
        assert_eq!(WORD_NUMERAL_MAP.get("unknown"), None);
    }
}
