use std::collections::HashMap;

/// Function type for expected value validation
pub type ExpectedFunction = Box<dyn Fn(&str) -> bool + Send + Sync>;

/// Expected value configuration
#[derive(Debug, Clone)]
pub struct ExpectedValue {
    pub name: String,
    pub validator: ExpectedValidator,
}

impl ExpectedValue {
    pub fn new(name: impl Into<String>, validator: ExpectedValidator) -> Self {
        Self {
            name: name.into(),
            validator,
        }
    }
}

/// Different types of expected value validators
#[derive(Debug, Clone)]
pub enum ExpectedValidator {
    /// Exact string match
    Exact(String),
    /// Case-insensitive exact match
    ExactIgnoreCase(String),
    /// One of multiple possible values
    OneOf(Vec<String>),
    /// Regex pattern match
    Regex(regex::Regex),
    /// Custom function
    Custom(String), // Store function name for debugging
}

impl ExpectedValidator {
    pub fn validate(&self, value: &str) -> bool {
        match self {
            ExpectedValidator::Exact(expected) => value == expected,
            ExpectedValidator::ExactIgnoreCase(expected) => {
                value.to_lowercase() == expected.to_lowercase()
            }
            ExpectedValidator::OneOf(options) => {
                options.iter().any(|option| value == option)
            }
            ExpectedValidator::Regex(pattern) => pattern.is_match(value),
            ExpectedValidator::Custom(_) => {
                // In a real implementation, this would call the custom function
                // For now, return true as a placeholder
                true
            }
        }
    }
}

/// Builder for expected value functions
pub struct ExpectedBuilder {
    validators: Vec<ExpectedValue>,
}

impl ExpectedBuilder {
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }
    
    pub fn add_exact(mut self, name: impl Into<String>, expected: impl Into<String>) -> Self {
        self.validators.push(ExpectedValue::new(
            name,
            ExpectedValidator::Exact(expected.into()),
        ));
        self
    }
    
    pub fn add_exact_ignore_case(mut self, name: impl Into<String>, expected: impl Into<String>) -> Self {
        self.validators.push(ExpectedValue::new(
            name,
            ExpectedValidator::ExactIgnoreCase(expected.into()),
        ));
        self
    }
    
    pub fn add_one_of(mut self, name: impl Into<String>, options: Vec<impl Into<String>>) -> Self {
        let options: Vec<String> = options.into_iter().map(|s| s.into()).collect();
        self.validators.push(ExpectedValue::new(
            name,
            ExpectedValidator::OneOf(options),
        ));
        self
    }
    
    pub fn add_regex(mut self, name: impl Into<String>, pattern: regex::Regex) -> Self {
        self.validators.push(ExpectedValue::new(
            name,
            ExpectedValidator::Regex(pattern),
        ));
        self
    }
    
    pub fn add_custom(mut self, name: impl Into<String>) -> Self {
        let name_str = name.into();
        self.validators.push(ExpectedValue::new(
            name_str.clone(),
            ExpectedValidator::Custom(name_str),
        ));
        self
    }
    
    pub fn build(self) -> ExpectedFunction {
        Box::new(move |value: &str| -> bool {
            for validator in &self.validators {
                if validator.validator.validate(value) {
                    return true;
                }
            }
            false
        })
    }
}

impl Default for ExpectedBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Build an expected function from a configuration
pub fn build_expected_function(config: &ExpectedConfig) -> ExpectedFunction {
    let mut builder = ExpectedBuilder::new();
    
    for expected_value in &config.values {
        match &expected_value.validator {
            ExpectedValidator::Exact(expected) => {
                builder = builder.add_exact(&expected_value.name, expected);
            }
            ExpectedValidator::ExactIgnoreCase(expected) => {
                builder = builder.add_exact_ignore_case(&expected_value.name, expected);
            }
            ExpectedValidator::OneOf(options) => {
                builder = builder.add_one_of(&expected_value.name, options.clone());
            }
            ExpectedValidator::Regex(pattern) => {
                builder = builder.add_regex(&expected_value.name, pattern.clone());
            }
            ExpectedValidator::Custom(name) => {
                builder = builder.add_custom(name);
            }
        }
    }
    
    builder.build()
}

/// Configuration for expected values
#[derive(Debug, Clone)]
pub struct ExpectedConfig {
    pub values: Vec<ExpectedValue>,
}

impl ExpectedConfig {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
        }
    }
    
    pub fn add_value(mut self, value: ExpectedValue) -> Self {
        self.values.push(value);
        self
    }
    
    pub fn add_exact(mut self, name: impl Into<String>, expected: impl Into<String>) -> Self {
        self.values.push(ExpectedValue::new(
            name,
            ExpectedValidator::Exact(expected.into()),
        ));
        self
    }
    
    pub fn add_one_of(mut self, name: impl Into<String>, options: Vec<impl Into<String>>) -> Self {
        let options: Vec<String> = options.into_iter().map(|s| s.into()).collect();
        self.values.push(ExpectedValue::new(
            name,
            ExpectedValidator::OneOf(options),
        ));
        self
    }
    
    pub fn add_regex(mut self, name: impl Into<String>, pattern: regex::Regex) -> Self {
        self.values.push(ExpectedValue::new(
            name,
            ExpectedValidator::Regex(pattern),
        ));
        self
    }
}

impl Default for ExpectedConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate a value against multiple expected functions
pub fn validate_multiple(value: &str, validators: &[&ExpectedFunction]) -> bool {
    validators.iter().any(|validator| validator(value))
}

/// Create a simple exact match expected function
pub fn exact_match(expected: impl Into<String>) -> ExpectedFunction {
    let expected = expected.into();
    Box::new(move |value: &str| value == &expected)
}

/// Create a case-insensitive exact match expected function
pub fn exact_match_ignore_case(expected: impl Into<String>) -> ExpectedFunction {
    let expected = expected.into();
    Box::new(move |value: &str| {
        value.to_lowercase() == expected.to_lowercase()
    })
}

/// Create a one-of expected function
pub fn one_of_match(options: Vec<impl Into<String>>) -> ExpectedFunction {
    let options: Vec<String> = options.into_iter().map(|s| s.into()).collect();
    Box::new(move |value: &str| options.iter().any(|option| value == option))
}

/// Create a regex match expected function
pub fn regex_match(pattern: regex::Regex) -> ExpectedFunction {
    Box::new(move |value: &str| pattern.is_match(value))
}

/// Create a numeric range expected function
pub fn numeric_range(min: i64, max: i64) -> ExpectedFunction {
    Box::new(move |value: &str| {
        if let Ok(num) = value.parse::<i64>() {
            num >= min && num <= max
        } else {
            false
        }
    })
}

/// Create a year validation expected function
pub fn valid_year() -> ExpectedFunction {
    numeric_range(1900, 2030)
}

/// Create a season number validation expected function
pub fn valid_season() -> ExpectedFunction {
    numeric_range(1, 50)
}

/// Create an episode number validation expected function
pub fn valid_episode() -> ExpectedFunction {
    numeric_range(1, 999)
}

/// Create a resolution validation expected function
pub fn valid_resolution() -> ExpectedFunction {
    let resolutions = vec![
        "480p", "540p", "720p", "1080p", "1440p", "2160p", "4K", "8K"
    ];
    one_of_match(resolutions)
}

/// Create a source validation expected function
pub fn valid_source() -> ExpectedFunction {
    let sources = vec![
        "CAM", "TS", "TC", "SCR", "DVDSCR", "WP", "VODRIP", "DVDRIP", "DVDR", "DVD",
        "HDTV", "PDTV", "DSR", "TVRIP", "SATRIP", "DTHRIP", "BDSCR", "BDRIP", "BRRIP",
        "BLURAY", "BDMV", "BD25", "BD50", "UHD-BD", "UHD-BLURAY", "HDRIP", "WEB-DL",
        "WEBRIP", "WEB", "WEBRIP", "NF", "AMZN", "HULU", "DTV", "IPTV", "HDTS", "HDTC"
    ];
    one_of_match(sources)
}

/// Create a codec validation expected function
pub fn valid_codec() -> ExpectedFunction {
    let codecs = vec![
        "XVID", "DIVX", "H.264", "H.265", "X264", "X265", "AVC", "HEVC", "VC-1", "VP9",
        "AV1", "MPEG-2", "MPEG-4", "WMV", "FLV", "RMVB", "MKV", "MP4", "AVI"
    ];
    one_of_match(codecs)
}

/// Create an audio validation expected function
pub fn valid_audio() -> ExpectedFunction {
    let audio = vec![
        "MP3", "AAC", "AC3", "DTS", "FLAC", "OGG", "WMA", "PCM", "TRUEHD", "ATMOS",
        "EAC3", "DDP", "DD5.1", "DTS-HD", "DTS-MA", "LPCM"
    ];
    one_of_match(audio)
}

/// Cache for expected functions to avoid recreating them
pub struct ExpectedCache {
    cache: HashMap<String, ExpectedFunction>,
}

impl ExpectedCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
    
    pub fn get_or_create<F>(&mut self, key: &str, factory: F) -> &ExpectedFunction
    where
        F: FnOnce() -> ExpectedFunction,
    {
        if !self.cache.contains_key(key) {
            self.cache.insert(key.to_string(), factory());
        }
        self.cache.get(key).unwrap()
    }
    
    pub fn clear(&mut self) {
        self.cache.clear();
    }
    
    pub fn len(&self) -> usize {
        self.cache.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

impl Default for ExpectedCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    
    #[test]
    fn test_exact_validator() {
        let validator = ExpectedValidator::Exact("test".to_string());
        assert!(validator.validate("test"));
        assert!(!validator.validate("Test"));
        assert!(!validator.validate("other"));
    }
    
    #[test]
    fn test_exact_ignore_case_validator() {
        let validator = ExpectedValidator::ExactIgnoreCase("test".to_string());
        assert!(validator.validate("test"));
        assert!(validator.validate("Test"));
        assert!(validator.validate("TEST"));
        assert!(!validator.validate("other"));
    }
    
    #[test]
    fn test_one_of_validator() {
        let validator = ExpectedValidator::OneOf(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        assert!(validator.validate("a"));
        assert!(validator.validate("b"));
        assert!(validator.validate("c"));
        assert!(!validator.validate("d"));
    }
    
    #[test]
    fn test_regex_validator() {
        let validator = ExpectedValidator::Regex(Regex::new(r"^\d+$").unwrap());
        assert!(validator.validate("123"));
        assert!(validator.validate("456"));
        assert!(!validator.validate("abc"));
        assert!(!validator.validate("12a34"));
    }
    
    #[test]
    fn test_expected_builder() {
        let expected_fn = ExpectedBuilder::new()
            .add_exact("test1", "exact")
            .add_exact_ignore_case("test2", "ignore")
            .add_one_of("test3", vec!["a", "b", "c"])
            .add_regex("test4", Regex::new(r"^\d+$").unwrap())
            .build();
        
        assert!(expected_fn("exact"));
        assert!(expected_fn("IGNORE"));
        assert!(expected_fn("b"));
        assert!(expected_fn("123"));
        assert!(!expected_fn("invalid"));
    }
    
    #[test]
    fn test_build_expected_function() {
        let config = ExpectedConfig::new()
            .add_exact("exact", "test")
            .add_one_of("oneof", vec!["a", "b", "c"]);
        
        let expected_fn = build_expected_function(&config);
        
        assert!(expected_fn("test"));
        assert!(expected_fn("b"));
        assert!(!expected_fn("invalid"));
    }
    
    #[test]
    fn test_exact_match_function() {
        let expected_fn = exact_match("hello");
        assert!(expected_fn("hello"));
        assert!(!expected_fn("Hello"));
        assert!(!expected_fn("world"));
    }
    
    #[test]
    fn test_exact_match_ignore_case_function() {
        let expected_fn = exact_match_ignore_case("hello");
        assert!(expected_fn("hello"));
        assert!(expected_fn("Hello"));
        assert!(expected_fn("HELLO"));
        assert!(!expected_fn("world"));
    }
    
    #[test]
    fn test_one_of_match_function() {
        let expected_fn = one_of_match(vec!["a", "b", "c"]);
        assert!(expected_fn("a"));
        assert!(expected_fn("b"));
        assert!(expected_fn("c"));
        assert!(!expected_fn("d"));
    }
    
    #[test]
    fn test_regex_match_function() {
        let expected_fn = regex_match(Regex::new(r"^[A-Z]+$").unwrap());
        assert!(expected_fn("ABC"));
        assert!(expected_fn("XYZ"));
        assert!(!expected_fn("abc"));
        assert!(!expected_fn("123"));
    }
    
    #[test]
    fn test_numeric_range_function() {
        let expected_fn = numeric_range(10, 20);
        assert!(expected_fn("15"));
        assert!(expected_fn("10"));
        assert!(expected_fn("20"));
        assert!(!expected_fn("9"));
        assert!(!expected_fn("21"));
        assert!(!expected_fn("abc"));
    }
    
    #[test]
    fn test_valid_year_function() {
        let expected_fn = valid_year();
        assert!(expected_fn("2023"));
        assert!(expected_fn("1999"));
        assert!(!expected_fn("1800"));
        assert!(!expected_fn("2100"));
        assert!(!expected_fn("abc"));
    }
    
    #[test]
    fn test_valid_season_function() {
        let expected_fn = valid_season();
        assert!(expected_fn("1"));
        assert!(expected_fn("10"));
        assert!(expected_fn("25"));
        assert!(!expected_fn("0"));
        assert!(!expected_fn("51"));
    }
    
    #[test]
    fn test_valid_episode_function() {
        let expected_fn = valid_episode();
        assert!(expected_fn("1"));
        assert!(expected_fn("100"));
        assert!(expected_fn("999"));
        assert!(!expected_fn("0"));
        assert!(!expected_fn("1000"));
    }
    
    #[test]
    fn test_valid_resolution_function() {
        let expected_fn = valid_resolution();
        assert!(expected_fn("1080p"));
        assert!(expected_fn("720p"));
        assert!(expected_fn("4K"));
        assert!(!expected_fn("1080i"));
        assert!(!expected_fn("invalid"));
    }
    
    #[test]
    fn test_valid_source_function() {
        let expected_fn = valid_source();
        assert!(expected_fn("WEB-DL"));
        assert!(expected_fn("BLURAY"));
        assert!(expected_fn("CAM"));
        assert!(!expected_fn("invalid"));
    }
    
    #[test]
    fn test_valid_codec_function() {
        let expected_fn = valid_codec();
        assert!(expected_fn("H.264"));
        assert!(expected_fn("X265"));
        assert!(expected_fn("AVC"));
        assert!(!expected_fn("invalid"));
    }
    
    #[test]
    fn test_valid_audio_function() {
        let expected_fn = valid_audio();
        assert!(expected_fn("AAC"));
        assert!(expected_fn("AC3"));
        assert!(expected_fn("DTS"));
        assert!(!expected_fn("invalid"));
    }
    
    #[test]
    fn test_validate_multiple() {
        let fn1 = exact_match("a");
        let fn2 = exact_match("b");
        let fn3 = exact_match("c");
        let validators: Vec<&ExpectedFunction> = vec![&fn1, &fn2, &fn3];
        
        assert!(validate_multiple("a", &validators));
        assert!(validate_multiple("b", &validators));
        assert!(validate_multiple("c", &validators));
        assert!(!validate_multiple("d", &validators));
    }
    
    #[test]
fn test_expected_cache() {
        let mut cache = ExpectedCache::new();
        
        {
            let fn1 = cache.get_or_create("test1", || exact_match("value1"));
            assert!(fn1("value1"));
            assert!(!fn1("different"));
        }
        
        {
            let fn2 = cache.get_or_create("test1", || exact_match("different"));
            assert!(fn2("value1"));
            assert!(!fn2("different"));
        }
        
        assert_eq!(cache.len(), 1);
        
        cache.get_or_create("test2", || exact_match("value2"));
        assert_eq!(cache.len(), 2);
        
        cache.clear();
        assert_eq!(cache.len(), 0);
    }
}
