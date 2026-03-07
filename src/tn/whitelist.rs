//! Whitelist TN tagger.
//!
//! Lookup table for common abbreviations and special terms:
//! - "Dr." → "doctor"
//! - "Mrs." → "misses"
//! - "Mr." → "mister"
//! - "e.g." → "for example"

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref WHITELIST: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        // Titles
        m.insert("Dr.", "doctor");
        m.insert("Dr", "doctor");
        m.insert("Mrs.", "misses");
        m.insert("Mrs", "misses");
        m.insert("Mr.", "mister");
        m.insert("Mr", "mister");
        m.insert("Ms.", "miss");
        m.insert("Ms", "miss");
        m.insert("St.", "saint");
        m.insert("St", "saint");
        m.insert("Prof.", "professor");
        m.insert("Jr.", "junior");
        m.insert("Sr.", "senior");
        m.insert("Gen.", "general");
        m.insert("Gov.", "governor");
        m.insert("Sgt.", "sergeant");
        m.insert("Capt.", "captain");
        m.insert("Lt.", "lieutenant");
        m.insert("Rev.", "reverend");

        // Latin abbreviations
        m.insert("e.g.", "for example");
        m.insert("i.e.", "that is");
        m.insert("etc.", "et cetera");
        m.insert("vs.", "versus");
        m.insert("vs", "versus");

        // Units (when written as abbreviations)
        m.insert("ft.", "feet");
        m.insert("in.", "inches");
        m.insert("oz.", "ounces");
        m.insert("lb.", "pounds");
        m.insert("lbs.", "pounds");

        // Common
        m.insert("Ave.", "avenue");
        m.insert("Blvd.", "boulevard");
        m.insert("Dept.", "department");
        m.insert("Inc.", "incorporated");
        m.insert("Corp.", "corporation");
        m.insert("Ltd.", "limited");
        m.insert("Co.", "company");
        m.insert("No.", "number");
        m.insert("approx.", "approximately");

        m
    };
}

/// Parse a whitelist abbreviation to its spoken form.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // Direct lookup (case-sensitive first)
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
        assert_eq!(parse("Dr."), Some("doctor".to_string()));
        assert_eq!(parse("Mrs."), Some("misses".to_string()));
        assert_eq!(parse("Mr."), Some("mister".to_string()));
        assert_eq!(parse("St."), Some("saint".to_string()));
    }

    #[test]
    fn test_latin() {
        assert_eq!(parse("e.g."), Some("for example".to_string()));
        assert_eq!(parse("i.e."), Some("that is".to_string()));
        assert_eq!(parse("etc."), Some("et cetera".to_string()));
    }

    #[test]
    fn test_no_match() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("world"), None);
    }
}
