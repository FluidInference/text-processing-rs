//! Whitelist TN tagger for Hindi (romanized).
//!
//! Lookup table for common abbreviations and titles with Hindi romanized output:
//! - "Dr." -> "daaktor"
//! - "Mr." -> "shri"
//! - "Mrs." -> "shreemati"
//! - "etc." -> "ityaadi"

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref WHITELIST: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        // Titles
        m.insert("Dr.", "daaktor");
        m.insert("Dr", "daaktor");
        m.insert("Mr.", "shri");
        m.insert("Mr", "shri");
        m.insert("Mrs.", "shreemati");
        m.insert("Mrs", "shreemati");
        m.insert("Ms.", "sushri");
        m.insert("Ms", "sushri");
        m.insert("Shri", "shri");
        m.insert("Smt.", "shreemati");
        m.insert("Prof.", "pradhyaapak");
        m.insert("St.", "sant");
        m.insert("Jr.", "kanishth");
        m.insert("Sr.", "varishth");

        // Common abbreviations
        m.insert("etc.", "ityaadi");
        m.insert("vs.", "banam");
        m.insert("vs", "banam");
        m.insert("No.", "sankhya");

        // Units
        m.insert("Km", "kilometre");

        // Currency
        m.insert("Rs.", "rupaye");
        m.insert("Rs", "rupaye");

        m
    };
}

/// Parse a whitelist abbreviation to its Hindi romanized spoken form.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // Direct lookup (case-sensitive)
    if let Some(&spoken) = WHITELIST.get(trimmed) {
        return Some(spoken.to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_titles() {
        assert_eq!(parse("Dr."), Some("daaktor".to_string()));
        assert_eq!(parse("Mr."), Some("shri".to_string()));
        assert_eq!(parse("Mrs."), Some("shreemati".to_string()));
        assert_eq!(parse("Ms."), Some("sushri".to_string()));
    }

    #[test]
    fn test_abbreviations() {
        assert_eq!(parse("etc."), Some("ityaadi".to_string()));
        assert_eq!(parse("vs."), Some("banam".to_string()));
        assert_eq!(parse("Rs."), Some("rupaye".to_string()));
    }

    #[test]
    fn test_hindi_specific() {
        assert_eq!(parse("Shri"), Some("shri".to_string()));
        assert_eq!(parse("Smt."), Some("shreemati".to_string()));
    }

    #[test]
    fn test_no_match() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("world"), None);
    }
}
