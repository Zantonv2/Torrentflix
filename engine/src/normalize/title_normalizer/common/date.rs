use regex::Regex;
use chrono::{NaiveDate, Datelike};

const DSEP: &str = r"[-/ \.]";
const DSEP_BIS: &str = r"[-/ \.x]";

lazy_static::lazy_static! {
    static ref DATE_REGEXPS: Vec<Regex> = vec![
        // 8-digit dates
        Regex::new(&format!(r"{}((\d{{8}})){}", DSEP, DSEP)).unwrap(),
        // 6-digit dates
        Regex::new(&format!(r"{}((\d{{6}})){}", DSEP, DSEP)).unwrap(),
        // 2-digit year formats (DD-MM-YY, MM-DD-YY)
        Regex::new(&format!(r"(?:^|[^\d])((\d{{2}}){}(\d{{1,2}}){}(\d{{1,2}}))(?:$|[^\d])", DSEP, DSEP)).unwrap(),
        // 2-digit year formats (DD-MM-YY, MM-DD-YY) reversed
        Regex::new(&format!(r"(?:^|[^\d])((\d{{1,2}}){}(\d{{1,2}}){}(\d{{2}}))(?:$|[^\d])", DSEP, DSEP)).unwrap(),
        // 4-digit year formats (YYYY-MM-DD, YYYY-DD-MM) - simplified boundary conditions
        Regex::new(&format!(r"((\d{{4}}){}(\d{{1,2}}){}(\d{{1,2}}))", DSEP_BIS, DSEP)).unwrap(),
        // 4-digit year formats (DD-MM-YYYY, MM-DD-YYYY) - simplified boundary conditions
        Regex::new(&format!(r"((\d{{1,2}}){}(\d{{1,2}}){}(\d{{4}}))", DSEP, DSEP_BIS)).unwrap(),
        // Text month formats
        Regex::new(&format!(r"(?:^|[^\d])((\d{{1,2}}(?:st|nd|rd|th)?{}(?:[a-z]{{3,10}}){}\d{{4}}))(?:$|[^\d])", DSEP, DSEP)).unwrap(),
    ];
    
    static ref CLEAN_RE: Regex = Regex::new(r"[^\d]*(\d+)[^\d]*").unwrap();
}

/// Check if number is a valid year
pub fn valid_year(year: i32) -> bool {
    1920 <= year && year < 2030
}

/// Check if number is a valid week
pub fn valid_week(week: i32) -> bool {
    1 <= week && week < 53
}

/// Check if the input string is an integer
fn is_int(string: &str) -> bool {
    string.parse::<i32>().is_ok()
}

/// If day_first is not defined, use some heuristic to fix it.
/// It helps to solve issues with python dateutils 2.5.3 parser changes.
fn guess_day_first_parameter(groups: &[&str]) -> Option<bool> {
    if groups.is_empty() {
        return None;
    }
    
    // If match starts with a long year, then day_first is forced to false.
    if let Ok(year) = groups[0].parse::<i32>() {
        if valid_year(year) && groups[0].len() >= 4 {
            return Some(false);
        }
    }
    
    // If match ends with a long year, the day_first is forced to true.
    if let Ok(year) = groups[groups.len() - 1].parse::<i32>() {
        if valid_year(year) && groups[groups.len() - 1].len() >= 4 {
            return Some(true);
        }
    }
    
    // If match starts with a short year, then day_first is forced to false.
    if let Ok(year) = groups[0].parse::<i32>() {
        if year > 31 && groups[0].len() == 2 {
            return Some(false);
        }
    }
    
    // If match ends with a short year, the day_first is forced to true.
    if let Ok(year) = groups[groups.len() - 1].parse::<i32>() {
        if year > 31 && groups[groups.len() - 1].len() == 2 {
            return Some(true);
        }
    }
    
    None
}

/// Search for date patterns in a string and return the date and span if found.
/// 
/// Year can be defined on two digit only. It will return the nearest possible
/// date from today.
/// 
/// Returns (start, end, date) if found, None otherwise
pub fn search_date(string: &str, year_first: Option<bool>, day_first: Option<bool>) -> Option<(usize, usize, NaiveDate)> {
    for date_re in DATE_REGEXPS.iter() {
        if let Some(search_match) = date_re.captures(string) {
            let start = search_match.get(1)?.start();
            let end = search_match.get(1)?.end();
            
            let groups: Vec<&str> = search_match.iter()
                .skip(2) // Skip the full match and outer group, use inner capture groups
                .filter_map(|m| m.map(|m| m.as_str()))
                .collect();
            
            if groups.is_empty() {
                continue;
            }
            
            let match_str = groups.join("-");
            
            // Determine day_first parameter
            let final_day_first = if day_first.is_some() {
                day_first
            } else if year_first == Some(true) {
                Some(false)
            } else {
                guess_day_first_parameter(&groups)
            };
            
            // Try different date parsing combinations
            let yearfirst_opts = if let Some(yf) = year_first {
                vec![yf]
            } else {
                vec![false, true]
            };
            
            let dayfirst_opts = if let Some(df) = final_day_first {
                vec![df]
            } else {
                vec![true, false]
            };
            
            for &df in dayfirst_opts.iter() {
                for &yf in yearfirst_opts.iter() {
                    // Simple date parsing logic (simplified version of dateutil parser)
                    if let Some(date) = parse_date_simple(&match_str, df, yf) {
                        if valid_year(date.year() as i32) {
                            return Some((start, end, date));
                        }
                    }
                }
            }
        }
    }
    
    None
}

/// Simple date parser (simplified version of dateutil.parser)
fn parse_date_simple(date_str: &str, day_first: bool, year_first: bool) -> Option<NaiveDate> {
    let parts: Vec<&str> = date_str.split('-').collect();
    
    if parts.len() != 3 {
        return None;
    }
    
    let (year, month, day) = if year_first {
        // Year-Month-Day format
        let year = parts[0].parse::<i32>().ok()?;
        let month = parts[1].parse::<u32>().ok()?;
        let day = parts[2].parse::<u32>().ok()?;
        (year, month, day)
    } else if day_first {
        // Day-Month-Year format
        let day = parts[0].parse::<u32>().ok()?;
        let month = parts[1].parse::<u32>().ok()?;
        let year = parts[2].parse::<i32>().ok()?;
        (year, month, day)
    } else {
        // Month-Day-Year format
        let month = parts[0].parse::<u32>().ok()?;
        let day = parts[1].parse::<u32>().ok()?;
        let year = parts[2].parse::<i32>().ok()?;
        (year, month, day)
    };
    
    // Handle 2-digit years
    let final_year = if year < 100 {
        // Convert to nearest year from current date (simplified)
        let current_year = chrono::Utc::now().year();
        let century = if year <= (current_year % 100) {
            current_year / 100
        } else {
            current_year / 100 - 1
        };
        century * 100 + year
    } else {
        year
    };
    
    NaiveDate::from_ymd_opt(final_year, month, day)
}

/// Extract date from string with default parameters
pub fn extract_date(string: &str) -> Option<NaiveDate> {
    search_date(string, None, None).map(|(_, _, date)| date)
}

/// Check if string contains a valid date
pub fn contains_date(string: &str) -> bool {
    search_date(string, None, None).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    
    #[test]
    fn test_valid_year() {
        assert!(valid_year(2023));
        assert!(valid_year(1999));
        assert!(!valid_year(1919));
        assert!(!valid_year(2030));
    }
    
    #[test]
    fn test_valid_week() {
        assert!(valid_week(1));
        assert!(valid_week(52));
        assert!(!valid_week(0));
        assert!(!valid_week(53));
    }
    
    #[test]
    fn test_is_int() {
        assert!(is_int("123"));
        assert!(is_int("-456"));
        assert!(!is_int("abc"));
        assert!(!is_int("12.34"));
    }
    
    #[test]
    fn test_guess_day_first_parameter() {
        // Long year at start - day_first should be false
        assert_eq!(guess_day_first_parameter(&["2023", "04", "22"]), Some(false));
        
        // Long year at end - day_first should be true
        assert_eq!(guess_day_first_parameter(&["22", "04", "2023"]), Some(true));
        
        // Short year > 31 at start - day_first should be false
        assert_eq!(guess_day_first_parameter(&["99", "04", "22"]), Some(false));
        
        // Short year > 31 at end - day_first should be true
        assert_eq!(guess_day_first_parameter(&["22", "04", "99"]), Some(true));
        
        // No clear pattern
        assert_eq!(guess_day_first_parameter(&["04", "22", "23"]), None);
    }
    
    #[test]
    fn test_parse_date_simple() {
        // Year-Month-Day
        let date = parse_date_simple("2023-04-22", false, true).unwrap();
        assert_eq!(date.year(), 2023);
        assert_eq!(date.month(), 4);
        assert_eq!(date.day(), 22);
        
        // Day-Month-Year
        let date = parse_date_simple("22-04-2023", true, false).unwrap();
        assert_eq!(date.year(), 2023);
        assert_eq!(date.month(), 4);
        assert_eq!(date.day(), 22);
        
        // Month-Day-Year
        let date = parse_date_simple("04-22-2023", false, false).unwrap();
        assert_eq!(date.year(), 2023);
        assert_eq!(date.month(), 4);
        assert_eq!(date.day(), 22);
        
        // 2-digit year
        let date = parse_date_simple("22-04-23", true, false).unwrap();
        assert!(date.year() >= 2000); // Should be converted to 2023
        assert_eq!(date.month(), 4);
        assert_eq!(date.day(), 22);
    }
    
    #[test]
    fn test_search_date() {
        // Standard date format
        let result = search_date(" This happened on 2002-04-22. ", None, None);
        assert!(result.is_some());
        let (start, end, date) = result.unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2002, 4, 22).unwrap());
        assert!(start > 0);
        assert!(end < 32);
        
        // Day-month-year format
        let result = search_date(" And this on 17-06-1998. ", None, None);
        assert!(result.is_some());
        let (_, _, date) = result.unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(1998, 6, 17).unwrap());
        
        // No date
        let result = search_date(" no date in here ", None, None);
        assert!(result.is_none());
    }
    
    #[test]
    fn test_extract_date() {
        assert_eq!(extract_date("Movie.2023.04.22.1080p"), 
                   Some(NaiveDate::from_ymd_opt(2023, 4, 22).unwrap()));
        assert_eq!(extract_date("no date here"), None);
    }
    
    #[test]
    fn test_contains_date() {
        assert!(contains_date("Release on 2023-04-22"));
        assert!(!contains_date("no date here"));
    }
}
