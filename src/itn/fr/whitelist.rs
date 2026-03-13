//! Whitelist tagger for French.
//!
//! Converts specific French titles and words to their abbreviated forms with Unicode superscripts.

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    /// Mapping of French words to their abbreviated forms
    static ref WHITELIST: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        // Titles with Unicode superscripts
        m.insert("docteur", "Dʳ");
        m.insert("docteures", "Dʳᵉˢ");
        m.insert("monsieur", "M.");
        m.insert("messieurs", "MM.");
        m.insert("madame", "Mᵐᵉ");
        m.insert("mesdames", "Mᵐᵉˢ");
        m.insert("mademoiselle", "Mˡˡᵉ");
        m.insert("mademoiselles", "Mˡˡᵉˢ");
        m
    };
}

/// Convert whitelisted French words to their abbreviated forms.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trimmed = input_lower.trim();

    WHITELIST.get(input_trimmed).map(|&s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whitelist() {
        assert_eq!(parse("docteur"), Some("Dʳ".to_string()));
        assert_eq!(parse("madame"), Some("Mᵐᵉ".to_string()));
        assert_eq!(parse("monsieur"), Some("M.".to_string()));
    }

    #[test]
    fn test_not_whitelisted() {
        assert_eq!(parse("bonjour"), None);
        assert_eq!(parse("un"), None);
    }
}
