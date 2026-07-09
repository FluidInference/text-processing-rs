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

        // Misc special terms
        m.insert("tv", "TV");
        m.insert("ssn", "SSN");
        m.insert("401(k)", "four oh one k");

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

    // Dotted initials collapse to an acronym: "C. S." → "CS".
    if let Some(acronym) = merge_initials(trimmed) {
        return Some(acronym);
    }

    None
}

/// Collapse a run of dotted single upper-case initials into a bare acronym,
/// whether space- or dot-separated ("C. S." → "CS", "U.S.A." → "USA").
fn merge_initials(s: &str) -> Option<String> {
    if !s.contains('.') {
        return None;
    }
    let parts: Vec<&str> = s.split(['.', ' ']).filter(|p| !p.is_empty()).collect();
    if parts.len() < 2 {
        return None;
    }
    let mut letters = String::new();
    for part in &parts {
        let mut chars = part.chars();
        let c = chars.next()?;
        if chars.next().is_some() || !c.is_ascii_uppercase() {
            return None;
        }
        letters.push(c);
    }
    Some(letters)
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
