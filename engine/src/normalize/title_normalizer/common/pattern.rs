use std::collections::HashMap;

/// Whether a specific pattern is disabled.
/// 
/// The context object might define an inclusion list (includes) or an exclusion list (excludes)
/// A pattern is considered disabled if it's found in the exclusion list or
/// it's not found in the inclusion list and the inclusion list is not empty or not defined.
pub fn is_disabled(context: &Option<HashMap<String, Vec<String>>>, name: &str) -> bool {
    if context.is_none() {
        return false;
    }
    
    let ctx = context.as_ref().unwrap();
    
    // Check exclusion list
    if let Some(excludes) = ctx.get("excludes") {
        if excludes.contains(&name.to_string()) {
            return true;
        }
    }
    
    // Check inclusion list
    if let Some(includes) = ctx.get("includes") {
        if !includes.is_empty() && !includes.contains(&name.to_string()) {
            return true;
        }
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_disabled_no_context() {
        assert!(!is_disabled(&None, "any_pattern"));
    }
    
    #[test]
    fn test_is_disabled_empty_context() {
        let context = Some(HashMap::new());
        assert!(!is_disabled(&context, "any_pattern"));
    }
    
    #[test]
    fn test_is_disabled_in_exclusion_list() {
        let mut context = HashMap::new();
        context.insert("excludes".to_string(), vec!["disabled_pattern".to_string()]);
        assert!(is_disabled(&Some(context), "disabled_pattern"));
    }
    
    #[test]
    fn test_is_disabled_not_in_exclusion_list() {
        let mut context = HashMap::new();
        context.insert("excludes".to_string(), vec!["other_pattern".to_string()]);
        assert!(!is_disabled(&Some(context), "enabled_pattern"));
    }
    
    #[test]
    fn test_is_disabled_in_inclusion_list() {
        let mut context = HashMap::new();
        context.insert("includes".to_string(), vec!["enabled_pattern".to_string()]);
        assert!(!is_disabled(&Some(context), "enabled_pattern"));
    }
    
    #[test]
    fn test_is_disabled_not_in_inclusion_list() {
        let mut context = HashMap::new();
        context.insert("includes".to_string(), vec!["other_pattern".to_string()]);
        assert!(is_disabled(&Some(context), "disabled_pattern"));
    }
    
    #[test]
    fn test_is_disabled_empty_inclusion_list() {
        let mut context = HashMap::new();
        context.insert("includes".to_string(), vec![]);
        assert!(!is_disabled(&Some(context), "any_pattern"));
    }
}
