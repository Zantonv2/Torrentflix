#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::title_normalizer::common::*;
    use regex::Regex;
    
    #[test]
    fn test_complete_common_module_integration() {
        // Test pattern validation
        let mut context = std::collections::HashMap::new();
        context.insert("excludes".to_string(), vec!["test_pattern".to_string()]);
        
        assert!(is_disabled(&Some(context), "test_pattern"));
        assert!(!is_disabled(&None, "any_pattern"));
        
        // Test numeral parsing with all formats
        assert_eq!(parse_numeral_all("123"), Ok(123));
        assert_eq!(parse_numeral_all("IV"), Ok(4));
        assert_eq!(parse_numeral_all("one"), Ok(1));
        assert_eq!(parse_numeral_all("un"), Ok(1));
        
        // Test date parsing
        let date_result = search_date("Movie.2023.04.22.1080p", None, None);
        assert!(date_result.is_some());
        let (_, _, date) = date_result.unwrap();
        assert_eq!(date.year(), 2023);
        assert_eq!(date.month(), 4);
        assert_eq!(date.day(), 22);
        
        // Test validators
        assert!(seps_surround(" test ", 1, 5));
        assert!(int_coercable("123"));
        assert!(word_boundary(" test ", 1, 5));
        
        // Test formatters
        assert_eq!(cleanup("Hello, World!"), "Hello World");
        assert_eq!(title_case("hello world"), "Hello World");
        assert_eq!(remove_duplicate_words("hello world hello"), "hello world");
        
        // Test quantity parsing
        assert!(parse_quantity("1.5GB").is_some());
        assert!(parse_quantity("30 fps").is_some());
        assert!(parse_quantity("320kbps").is_some());
        assert!(parse_quantity("5 min").is_some());
        
        // Test words utilities
        assert_eq!(count_words("Hello World Test"), 3);
        assert!(contains_word("Hello World", "world"));
        assert_eq!(get_unique_words("hello world hello"), vec!["hello", "world"]);
        
        // Test comparators
        assert_eq!(case_insensitive_cmp("apple", "Banana"), std::cmp::Ordering::Less);
        assert_eq!(natural_cmp("file1.txt", "file10.txt"), std::cmp::Ordering::Less);
        assert_eq!(length_cmp("short", "longer"), std::cmp::Ordering::Less);
        
        // Test expected values
        let expected_fn = exact_match("test");
        assert!(expected_fn("test"));
        assert!(!expected_fn("other"));
        
        let valid_year_fn = valid_year();
        assert!(valid_year_fn("2023"));
        assert!(!valid_year_fn("1800"));
    }
    
    #[test]
    fn test_separator_constants() {
        assert!(SEPS.contains(' '));
        assert!(SEPS.contains('.'));
        assert!(SEPS.contains('-'));
        assert!(SEPS.contains('_'));
        
        assert!(!SEPS_NO_GROUPS.contains('['));
        assert!(!SEPS_NO_GROUPS.contains(']'));
        assert!(!SEPS_NO_GROUPS.contains('{'));
        assert!(!SEPS_NO_GROUPS.contains('}'));
        
        assert!(!SEPS_NO_FS.contains('/'));
        assert!(!SEPS_NO_FS.contains('\\'));
        
        assert!(TITLE_SEPS.contains('-'));
        assert!(TITLE_SEPS.contains('+'));
        assert!(TITLE_SEPS.contains('/'));
        assert!(TITLE_SEPS.contains('|'));
    }
    
    #[test]
    fn test_optional_pattern() {
        let pattern = optional("test");
        assert_eq!(pattern, "(?:test)?");
        
        let complex_pattern = optional(r"\d+");
        assert_eq!(complex_pattern, r"(?:\d+)?");
    }
    
    #[test]
    fn test_separator_checks() {
        assert!(is_separator(' '));
        assert!(is_separator('.'));
        assert!(is_separator('-'));
        assert!(!is_separator('a'));
        
        assert!(is_title_separator('-'));
        assert!(is_title_separator('+'));
        assert!(is_title_separator('/'));
        assert!(!is_title_separator(' '));
    }
    
    #[test]
    fn test_numeral_comprehensive() {
        // Test all numeral types in one go
        let test_cases = vec![
            ("123", 123),
            ("IV", 4),
            ("one", 1),
            ("twenty", 20),
            ("un", 1),
            ("vingt", 20),
            ("zero", 0),
            ("X", 10),
            ("MCMXCIV", 1994),
        ];
        
        for (input, expected) in test_cases {
            assert_eq!(parse_numeral_all(input), Ok(expected), 
                      "Failed to parse '{}' as {}", input, expected);
        }
    }
    
    #[test]
    fn test_date_comprehensive() {
        let test_cases = vec![
            ("2023-04-22", Some((2023, 4, 22))),
            ("22-04-2023", Some((2023, 4, 22))),
            ("04-22-2023", Some((2023, 4, 22))),
            ("1998-06-17", Some((1998, 6, 17))),
        ];
        
        for (input, expected) in test_cases {
            let result = search_date(&format!(" movie {} ", input), None, None);
            match expected {
                Some((year, month, day)) => {
                    assert!(result.is_some(), "Failed to parse date: {}", input);
                    let (_, _, date) = result.unwrap();
                    assert_eq!((date.year(), date.month(), date.day()), (year, month, day),
                             "Date mismatch for: {}", input);
                }
                None => {
                    assert!(result.is_none(), "Unexpectedly parsed date: {}", input);
                }
            }
        }
    }
    
    #[test]
    fn test_validators_comprehensive() {
        let test_string = "  test  ";
        
        // Test various validators
        assert!(seps_before(test_string, 2));
        assert!(seps_after(test_string, 6));
        assert!(seps_surround(test_string, 2, 6));
        assert!(seps_surround_no_groups(test_string, 2, 6));
        assert!(seps_surround_no_fs(test_string, 2, 6));
        
        assert!(int_coercable("123"));
        assert!(!int_coercable("abc"));
        
        assert!(at_beginning("test", 0, 4));
        assert!(at_end("test", 0, 4));
        assert!(word_boundary(" test ", 1, 5));
        assert!(standalone(" test ", 1, 5));
        
        // Test logical operators
        let validators = vec![seps_before, seps_after];
        assert!(and(&validators)(" test ", 1, 5));
        assert!(!and(&validators)("test", 0, 4));
        
        let validators = vec![at_beginning, at_end];
        assert!(or(&validators)("test", 0, 4));
        assert!(!or(&validators)(" test ", 1, 5));
    }
    
    #[test]
    fn test_formatters_comprehensive() {
        let test_cases = vec![
            ("Hello, World!", "Hello World"),
            ("  Multiple   spaces  ", "Multiple spaces"),
            ("Repeated---punctuation", "Repeated-punctuation"),
            ("Title-Case", "Title Case"),
            ("UPPERCASE", "UPPERCASE"),
            ("lowercase", "lowercase"),
        ];
        
        for (input, expected) in test_cases {
            assert_eq!(cleanup(input), expected);
        }
        
        // Test specific formatters
        assert_eq!(title_case("hello world"), "Hello World");
        assert_eq!(uppercase("test"), "TEST");
        assert_eq!(lowercase("TEST"), "test");
        assert_eq!(alphanumeric_only("Hello123!"), "Hello123");
        assert_eq!(alphabetic_only("Hello123"), "Hello");
        assert_eq!(numeric_only("Hello123"), "123");
        assert_eq!(normalize_dashes("Hello–World"), "Hello-World");
        assert_eq!(remove_duplicate_words("hello world hello"), "hello world");
    }
    
    #[test]
    fn test_quantity_comprehensive() {
        // Test size parsing
        let size = Size::parse("1.5GB").unwrap();
        assert_eq!(size.value, 1.5);
        assert_eq!(size.unit, "GB");
        assert!(size.to_bytes() > 0.0);
        
        // Test frame rate parsing
        let framerate = FrameRate::parse("23.976").unwrap();
        assert_eq!(framerate.value, 23.976);
        assert!(framerate.is_common());
        assert!(framerate.get_standard_name().is_some());
        
        // Test bit rate parsing
        let bitrate = BitRate::parse("320kbps").unwrap();
        assert_eq!(bitrate.value, 320.0);
        assert_eq!(bitrate.unit, "kbps");
        
        // Test duration parsing
        let duration = Duration::parse("5 min").unwrap();
        assert_eq!(duration.seconds, 300.0);
        
        // Test quantity parsing
        assert!(matches!(parse_quantity("1.5GB"), Some(Quantity::Size(_))));
        assert!(matches!(parse_quantity("30 fps"), Some(Quantity::FrameRate(_))));
        assert!(matches!(parse_quantity("320kbps"), Some(Quantity::BitRate(_))));
        assert!(matches!(parse_quantity("5 min"), Some(Quantity::Duration(_))));
    }
    
    #[test]
    fn test_words_comprehensive() {
        let text = "Hello World Test";
        
        // Test word iteration
        let words: Vec<&str> = WordIterator::new(text).collect();
        assert_eq!(words, vec!["Hello", "World", "Test"]);
        
        // Test word utilities
        assert_eq!(split_words(text), vec!["Hello", "World", "Test"]);
        assert_eq!(count_words(text), 3);
        assert_eq!(find_longest_word(text), Some("Hello"));
        assert!(contains_word(text, "world"));
        assert_eq!(get_unique_words("hello world hello"), vec!["hello", "world"]);
        
        // Test word classification
        assert!(is_word("hello"));
        assert!(is_alpha_word("hello"));
        assert!(is_numeric_word("123"));
        assert!(is_alphanumeric_word("test123"));
        assert!(is_numeric_like("test123"));
        assert!(is_abbreviation("USA"));
        assert!(is_title_case("Hello"));
        
        // Test word stats
        let stats = get_word_stats(text);
        assert_eq!(stats.total_words, 3);
        assert_eq!(stats.unique_words, 3);
        assert!(stats.average_word_length > 0.0);
    }
    
    #[test]
    fn test_comparators_comprehensive() {
        // Test basic comparators
        assert_eq!(case_insensitive_cmp("apple", "Banana"), std::cmp::Ordering::Less);
        assert_eq!(length_cmp("short", "longer"), std::cmp::Ordering::Less);
        assert_eq!(length_reverse_cmp("short", "longer"), std::cmp::Ordering::Greater);
        assert_eq!(numeric_or_lexical_cmp("10", "2"), std::cmp::Ordering::Greater);
        assert_eq!(word_count_cmp("one two", "one"), std::cmp::Ordering::Greater);
        
        // Test natural sorting
        assert_eq!(natural_cmp("file1.txt", "file10.txt"), std::cmp::Ordering::Less);
        assert_eq!(natural_cmp("version1.2", "version1.10"), std::cmp::Ordering::Less);
        
        // Test list position comparison
        let list = vec!["apple", "banana", "cherry"];
        assert_eq!(list_position_cmp(&list, &"apple", &"cherry"), std::cmp::Ordering::Less);
        
        // Test similarity comparison
        assert_eq!(similarity_cmp("hello", "hello", "world"), std::cmp::Ordering::Less);
        
        // Test pattern matching comparison
        let pattern = Regex::new(r"^\d+$").unwrap();
        assert_eq!(pattern_match_cmp(&pattern, "123", "abc"), std::cmp::Ordering::Less);
        
        // Test priority comparison
        let priority = vec!["high", "medium", "low"];
        assert_eq!(priority_cmp(&priority, "high", "low"), std::cmp::Ordering::Less);
    }
    
    #[test]
    fn test_expected_comprehensive() {
        // Test validators
        let exact_validator = ExpectedValidator::Exact("test".to_string());
        assert!(exact_validator.validate("test"));
        assert!(!exact_validator.validate("other"));
        
        let ignore_case_validator = ExpectedValidator::ExactIgnoreCase("test".to_string());
        assert!(ignore_case_validator.validate("test"));
        assert!(ignore_case_validator.validate("Test"));
        
        let one_of_validator = ExpectedValidator::OneOf(vec!["a".to_string(), "b".to_string()]);
        assert!(one_of_validator.validate("a"));
        assert!(!one_of_validator.validate("c"));
        
        let regex_validator = ExpectedValidator::Regex(Regex::new(r"^\d+$").unwrap());
        assert!(regex_validator.validate("123"));
        assert!(!regex_validator.validate("abc"));
        
        // Test expected builder
        let expected_fn = ExpectedBuilder::new()
            .add_exact("test1", "exact")
            .add_one_of("test2", vec!["a", "b", "c"])
            .build();
        
        assert!(expected_fn("exact"));
        assert!(expected_fn("b"));
        assert!(!expected_fn("invalid"));
        
        // Test convenience functions
        let exact_fn = exact_match("hello");
        assert!(exact_fn("hello"));
        
        let one_of_fn = one_of_match(vec!["a", "b", "c"]);
        assert!(one_of_fn("b"));
        
        let regex_fn = regex_match(Regex::new(r"^\d+$").unwrap());
        assert!(regex_fn("123"));
        
        let range_fn = numeric_range(10, 20);
        assert!(range_fn("15"));
        
        // Test built-in validators
        assert!(valid_year()("2023"));
        assert!(valid_season()("5"));
        assert!(valid_episode()("10"));
        assert!(valid_resolution()("1080p"));
        assert!(valid_source()("WEB-DL"));
        assert!(valid_codec()("H.264"));
        assert!(valid_audio()("AAC"));
        
        // Test cache
        let mut cache = ExpectedCache::new();
        let fn1 = cache.get_or_create("test", || exact_match("value"));
        let fn2 = cache.get_or_create("test", || exact_match("different"));
        assert!(fn1("value"));
        assert!(fn2("value"));
        assert_eq!(cache.len(), 1);
    }
    
    #[test]
    fn test_all_constants_and_utilities() {
        // Test that all constants are properly defined
        assert!(!SEPS.is_empty());
        assert!(!SEPS_NO_GROUPS.is_empty());
        assert!(!SEPS_NO_FS.is_empty());
        assert!(!TITLE_SEPS.is_empty());
        
        // Test dash constants
        assert_eq!(DASH.0, "-");
        assert_eq!(ALT_DASH.0, "@");
        
        // Test numeral constants
        assert!(!DIGITAL_NUMERAL.is_empty());
        assert!(!ROMAN_NUMERAL.is_empty());
        assert!(!ENGLISH_WORD_NUMERAL_LIST.is_empty());
        assert!(!FRENCH_WORD_NUMERAL_LIST.is_empty());
        assert!(!FRENCH_ALT_WORD_NUMERAL_LIST.is_empty());
        
        // Test date constants
        assert!(!DSEP.is_empty());
        assert!(!DSEP_BIS.is_empty());
    }
    
    #[test]
    fn test_error_handling_and_edge_cases() {
        // Test numeral parsing errors
        assert!(parse_numeral_all("invalid").is_err());
        assert!(parse_numeral("invalid", true, true, true, true).is_err());
        
        // Test date parsing edge cases
        assert!(search_date("no date here", None, None).is_none());
        assert!(extract_date("invalid date format").is_none());
        
        // Test quantity parsing errors
        assert!(Size::parse("invalid").is_none());
        assert!(FrameRate::parse("invalid").is_none());
        assert!(BitRate::parse("invalid").is_none());
        assert!(Duration::parse("invalid").is_none());
        assert!(parse_quantity("invalid").is_none());
        
        // Test word utilities edge cases
        assert_eq!(split_words(""), Vec::<&str>::new());
        assert_eq!(count_words(""), 0);
        assert_eq!(find_longest_word(""), None);
        assert!(!contains_word("", "anything"));
        
        // Test formatter edge cases
        assert_eq!(cleanup(""), "");
        assert_eq!(title_case(""), "");
        assert_eq!(remove_duplicate_words(""), "");
        
        // Test expected value edge cases
        let exact_fn = exact_match("");
        assert!(exact_fn(""));
        assert!(!exact_fn("non-empty"));
        
        let empty_one_of = one_of_match(vec![]);
        assert!(!empty_one_of("anything"));
    }
}
