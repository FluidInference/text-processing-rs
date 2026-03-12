//! Whitelist TN tagger for French.
//!
//! Lookup table for common French abbreviations and special terms:
//! - "M." → "monsieur"
//! - "Mme" → "madame"
//! - "Dr." → "docteur"
//! - "c.-à-d." → "c'est-a-dire"

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref WHITELIST: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        // Titles
        m.insert("M.", "monsieur");
        m.insert("M", "monsieur");
        m.insert("Mme", "madame");
        m.insert("Mme.", "madame");
        m.insert("Mlle", "mademoiselle");
        m.insert("Mlle.", "mademoiselle");
        m.insert("Dr", "docteur");
        m.insert("Dr.", "docteur");
        m.insert("Prof.", "professeur");
        m.insert("St", "saint");
        m.insert("St.", "saint");
        m.insert("Jr.", "junior");
        m.insert("Sr.", "senior");

        // French abbreviations
        m.insert("c.-\u{00e0}-d.", "c'est-a-dire");
        m.insert("c-\u{00e0}-d", "c'est-a-dire");
        m.insert("etc.", "et cetera");
        m.insert("p.ex.", "par exemple");

        // Common address and organization abbreviations
        m.insert("Av.", "avenue");
        m.insert("Bd.", "boulevard");
        m.insert("Cie", "compagnie");
        m.insert("Ste", "societe");
        m.insert("No", "numero");
        m.insert("no", "numero");

        m
    };
}

/// Parse a French whitelist abbreviation to its spoken form.
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
        assert_eq!(parse("M."), Some("monsieur".to_string()));
        assert_eq!(parse("Mme"), Some("madame".to_string()));
        assert_eq!(parse("Mlle"), Some("mademoiselle".to_string()));
        assert_eq!(parse("Dr."), Some("docteur".to_string()));
    }

    #[test]
    fn test_abbreviations() {
        assert_eq!(parse("etc."), Some("et cetera".to_string()));
        assert_eq!(parse("p.ex."), Some("par exemple".to_string()));
        assert_eq!(parse("Av."), Some("avenue".to_string()));
    }

    #[test]
    fn test_no_match() {
        assert_eq!(parse("bonjour"), None);
        assert_eq!(parse("monde"), None);
    }
}
