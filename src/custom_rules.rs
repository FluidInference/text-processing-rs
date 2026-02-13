//! Custom user-defined normalization rules.
//!
//! Allows callers to register spoken→written mappings at runtime.
//! These rules are checked with the highest priority in sentence mode,
//! before any built-in taggers.
//!
//! Example: ("linux", "Linux"), ("gee pee tee", "GPT")

use std::sync::RwLock;

use lazy_static::lazy_static;

lazy_static! {
    /// Global custom rules store. Entries are (lowercase_spoken, written).
    static ref CUSTOM_RULES: RwLock<Vec<(String, String)>> = RwLock::new(Vec::new());
}

/// Add a custom spoken→written mapping.
///
/// The spoken form is stored lowercased for case-insensitive matching.
/// If the same spoken form already exists, it is replaced.
pub fn add_rule(spoken: &str, written: &str) {
    let spoken_lower = spoken.to_lowercase();
    let mut rules = CUSTOM_RULES.write().unwrap();
    // Replace if exists
    if let Some(entry) = rules.iter_mut().find(|(s, _)| *s == spoken_lower) {
        entry.1 = written.to_string();
    } else {
        rules.push((spoken_lower, written.to_string()));
    }
}

/// Remove a custom rule by its spoken form.
///
/// Returns true if the rule was found and removed.
pub fn remove_rule(spoken: &str) -> bool {
    let spoken_lower = spoken.to_lowercase();
    let mut rules = CUSTOM_RULES.write().unwrap();
    let len_before = rules.len();
    rules.retain(|(s, _)| *s != spoken_lower);
    rules.len() < len_before
}

/// Clear all custom rules.
pub fn clear_rules() {
    let mut rules = CUSTOM_RULES.write().unwrap();
    rules.clear();
}

/// Try to match input against custom rules (exact match, case-insensitive).
///
/// Returns `Some(written_form)` if a rule matches, `None` otherwise.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trimmed = input_lower.trim();

    let rules = CUSTOM_RULES.read().unwrap();
    for (spoken, written) in rules.iter() {
        if input_trimmed == spoken {
            return Some(written.clone());
        }
    }

    None
}

/// Get the number of custom rules currently registered.
pub fn rule_count() -> usize {
    let rules = CUSTOM_RULES.read().unwrap();
    rules.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Single test to avoid parallel test races on shared global state.
    #[test]
    fn test_custom_rules() {
        clear_rules();

        // Add and parse
        add_rule("gee pee tee", "GPT");
        assert_eq!(parse("gee pee tee"), Some("GPT".to_string()));
        assert_eq!(parse("Gee Pee Tee"), Some("GPT".to_string()));
        assert_eq!(parse("unknown"), None);

        // Replace existing
        add_rule("gee pee tee", "GPT-4");
        assert_eq!(parse("gee pee tee"), Some("GPT-4".to_string()));
        assert_eq!(rule_count(), 1);

        // Remove
        assert!(remove_rule("gee pee tee"));
        assert_eq!(parse("gee pee tee"), None);
        assert!(!remove_rule("gee pee tee"));

        // Multiple rules + clear
        add_rule("alpha", "A");
        add_rule("bravo", "B");
        assert_eq!(rule_count(), 2);
        assert_eq!(parse("alpha"), Some("A".to_string()));
        assert_eq!(parse("bravo"), Some("B".to_string()));
        clear_rules();
        assert_eq!(rule_count(), 0);
        assert_eq!(parse("alpha"), None);
    }
}
