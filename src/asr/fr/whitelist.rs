//! Whitelist tagger for French.
//!
//! Pass-through specific French words/phrases without modification.

use lazy_static::lazy_static;
use std::collections::HashSet;

lazy_static! {
    /// Words that should pass through without modification
    static ref WHITELIST: HashSet<&'static str> = {
        let mut s = HashSet::new();
        // Common French words that might be confused with numbers
        // Note: "premier" and "première" are handled by ordinal parser
        s.insert("un");
        s.insert("une");
        s
    };
}

/// Pass through whitelisted words without modification.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trimmed = input_lower.trim();

    if WHITELIST.contains(input_trimmed) {
        Some(input.trim().to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whitelist() {
        assert_eq!(parse("un"), Some("un".to_string()));
        assert_eq!(parse("une"), Some("une".to_string()));
    }

    #[test]
    fn test_not_whitelisted() {
        assert_eq!(parse("bonjour"), None);
    }
}
