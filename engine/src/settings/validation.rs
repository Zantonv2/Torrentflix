use std::path::Path;
use url::Url;

/// Validation error type for settings
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Validate that a URL is a valid HTTP(S) URL
///
/// # Arguments
/// * `url` - The URL string to validate
/// * `field_name` - The name of the field for error messages
///
/// # Returns
/// * `Ok(())` if the URL is valid
/// * `Err(ValidationError)` if the URL is invalid
pub fn validate_url(url: &str, field_name: &str) -> Result<(), ValidationError> {
    if url.is_empty() {
        return Err(ValidationError::new(
            field_name,
            "URL cannot be empty",
        ));
    }

    match Url::parse(url) {
        Ok(parsed_url) => {
            match parsed_url.scheme() {
                "http" | "https" => Ok(()),
                _ => Err(ValidationError::new(
                    field_name,
                    "URL must use HTTP or HTTPS protocol",
                )),
            }
        }
        Err(_) => Err(ValidationError::new(
            field_name,
            "URL must be a valid HTTP(S) URL (e.g., http://localhost:8080)",
        )),
    }
}

/// Validate that a path is an absolute path and exists on the filesystem
///
/// # Arguments
/// * `path` - The path string to validate
/// * `field_name` - The name of the field for error messages
///
/// # Returns
/// * `Ok(())` if the path is valid
/// * `Err(ValidationError)` if the path is invalid
pub fn validate_path(path: &str, field_name: &str) -> Result<(), ValidationError> {
    if path.is_empty() {
        return Err(ValidationError::new(
            field_name,
            "Path cannot be empty",
        ));
    }

    let path_obj = Path::new(path);

    // Check if path is absolute
    if !path_obj.is_absolute() {
        return Err(ValidationError::new(
            field_name,
            "Path must be an absolute path",
        ));
    }

    // Check if path exists
    if !path_obj.exists() {
        return Err(ValidationError::new(
            field_name,
            "Path must exist on the system",
        ));
    }

    // Check if path is readable (is a directory)
    if !path_obj.is_dir() {
        return Err(ValidationError::new(
            field_name,
            "Path must be a directory",
        ));
    }

    Ok(())
}

/// Validate that a numeric value is within an acceptable range
///
/// # Arguments
/// * `value` - The numeric value to validate
/// * `min` - The minimum acceptable value (inclusive)
/// * `max` - The maximum acceptable value (inclusive)
/// * `field_name` - The name of the field for error messages
///
/// # Returns
/// * `Ok(())` if the value is within range
/// * `Err(ValidationError)` if the value is out of range
pub fn validate_numeric_range(
    value: u32,
    min: u32,
    max: u32,
    field_name: &str,
) -> Result<(), ValidationError> {
    if value < min || value > max {
        return Err(ValidationError::new(
            field_name,
            format!("Value must be between {} and {}", min, max),
        ));
    }
    Ok(())
}

/// Validate that a numeric value is within an acceptable range (u16 variant)
///
/// # Arguments
/// * `value` - The numeric value to validate
/// * `min` - The minimum acceptable value (inclusive)
/// * `max` - The maximum acceptable value (inclusive)
/// * `field_name` - The name of the field for error messages
///
/// # Returns
/// * `Ok(())` if the value is within range
/// * `Err(ValidationError)` if the value is out of range
pub fn validate_numeric_range_u16(
    value: u16,
    min: u16,
    max: u16,
    field_name: &str,
) -> Result<(), ValidationError> {
    if value < min || value > max {
        return Err(ValidationError::new(
            field_name,
            format!("Value must be between {} and {}", min, max),
        ));
    }
    Ok(())
}

/// Validate that a numeric value is within an acceptable range (u64 variant)
///
/// # Arguments
/// * `value` - The numeric value to validate
/// * `min` - The minimum acceptable value (inclusive)
/// * `max` - The maximum acceptable value (inclusive)
/// * `field_name` - The name of the field for error messages
///
/// # Returns
/// * `Ok(())` if the value is within range
/// * `Err(ValidationError)` if the value is out of range
pub fn validate_numeric_range_u64(
    value: u64,
    min: u64,
    max: u64,
    field_name: &str,
) -> Result<(), ValidationError> {
    if value < min || value > max {
        return Err(ValidationError::new(
            field_name,
            format!("Value must be between {} and {}", min, max),
        ));
    }
    Ok(())
}

/// Validate that an email address is in a valid format
///
/// # Arguments
/// * `email` - The email address to validate
/// * `field_name` - The name of the field for error messages
///
/// # Returns
/// * `Ok(())` if the email is valid
/// * `Err(ValidationError)` if the email is invalid
pub fn validate_email(email: &str, field_name: &str) -> Result<(), ValidationError> {
    if email.is_empty() {
        return Err(ValidationError::new(
            field_name,
            "Email address cannot be empty",
        ));
    }

    // Basic email validation: must contain @ and have text before and after
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(ValidationError::new(
            field_name,
            "Email address must be in valid format (e.g., user@example.com)",
        ));
    }

    // Check that domain part contains at least one dot
    if !parts[1].contains('.') {
        return Err(ValidationError::new(
            field_name,
            "Email domain must contain at least one dot (e.g., example.com)",
        ));
    }

    Ok(())
}

/// Validate that a required field is not empty when a feature is enabled
///
/// # Arguments
/// * `value` - The optional value to validate
/// * `field_name` - The name of the field for error messages
/// * `feature_name` - The name of the feature that requires this field
///
/// # Returns
/// * `Ok(())` if the value is present and not empty
/// * `Err(ValidationError)` if the value is missing or empty
pub fn validate_required_when_enabled(
    value: &Option<String>,
    field_name: &str,
    feature_name: &str,
) -> Result<(), ValidationError> {
    match value {
        Some(s) if !s.is_empty() => Ok(()),
        _ => Err(ValidationError::new(
            field_name,
            format!("{} is required when {} is enabled", field_name, feature_name),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_url_valid_http() {
        assert!(validate_url("http://localhost:8080", "qbittorrent_url").is_ok());
    }

    #[test]
    fn test_validate_url_valid_https() {
        assert!(validate_url("https://example.com", "qbittorrent_url").is_ok());
    }

    #[test]
    fn test_validate_url_invalid_protocol() {
        let result = validate_url("ftp://example.com", "qbittorrent_url");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().message,
            "URL must use HTTP or HTTPS protocol"
        );
    }

    #[test]
    fn test_validate_url_malformed() {
        let result = validate_url("not a url", "qbittorrent_url");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_url_empty() {
        let result = validate_url("", "qbittorrent_url");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().message, "URL cannot be empty");
    }

    #[test]
    fn test_validate_path_valid() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_string_lossy().to_string();
        assert!(validate_path(&path, "library_path").is_ok());
    }

    #[test]
    fn test_validate_path_relative() {
        let result = validate_path("./relative/path", "library_path");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().message, "Path must be an absolute path");
    }

    #[test]
    fn test_validate_path_nonexistent() {
        let result = validate_path("/nonexistent/path/that/does/not/exist", "library_path");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().message, "Path must exist on the system");
    }

    #[test]
    fn test_validate_path_empty() {
        let result = validate_path("", "library_path");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().message, "Path cannot be empty");
    }

    #[test]
    fn test_validate_numeric_range_valid() {
        assert!(validate_numeric_range(50, 0, 100, "search_limit").is_ok());
    }

    #[test]
    fn test_validate_numeric_range_min() {
        assert!(validate_numeric_range(0, 0, 100, "search_limit").is_ok());
    }

    #[test]
    fn test_validate_numeric_range_max() {
        assert!(validate_numeric_range(100, 0, 100, "search_limit").is_ok());
    }

    #[test]
    fn test_validate_numeric_range_below_min() {
        let result = validate_numeric_range(5, 10, 100, "search_limit");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_numeric_range_above_max() {
        let result = validate_numeric_range(150, 0, 100, "search_limit");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_numeric_range_u16_valid() {
        assert!(validate_numeric_range_u16(8080, 1024, 65535, "qbittorrent_port").is_ok());
    }

    #[test]
    fn test_validate_numeric_range_u16_invalid() {
        let result = validate_numeric_range_u16(100, 1024, 65535, "qbittorrent_port");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_numeric_range_u64_valid() {
        assert!(validate_numeric_range_u64(30, 1, 300, "search_timeout").is_ok());
    }

    #[test]
    fn test_validate_numeric_range_u64_invalid() {
        let result = validate_numeric_range_u64(500, 1, 300, "search_timeout");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_email_valid() {
        assert!(validate_email("user@example.com", "email").is_ok());
        assert!(validate_email("test.user@domain.co.uk", "email").is_ok());
    }

    #[test]
    fn test_validate_email_invalid_no_at() {
        let result = validate_email("userexample.com", "email");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_email_invalid_no_domain() {
        let result = validate_email("user@", "email");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_email_invalid_no_user() {
        let result = validate_email("@example.com", "email");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_email_invalid_no_dot() {
        let result = validate_email("user@example", "email");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_email_empty() {
        let result = validate_email("", "email");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_required_when_enabled_present() {
        let value = Some("test_value".to_string());
        assert!(validate_required_when_enabled(&value, "field", "feature").is_ok());
    }

    #[test]
    fn test_validate_required_when_enabled_none() {
        let value: Option<String> = None;
        let result = validate_required_when_enabled(&value, "field", "feature");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_required_when_enabled_empty() {
        let value = Some("".to_string());
        let result = validate_required_when_enabled(&value, "field", "feature");
        assert!(result.is_err());
    }
}
