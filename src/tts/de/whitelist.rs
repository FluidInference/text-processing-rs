//! Whitelist TN tagger for German.
//!
//! Lookup table for common German abbreviations and special terms:
//! - "Dr." -> "doktor"
//! - "Hr." -> "herr"
//! - "z.B." -> "zum beispiel"
//! - "GmbH" -> "gesellschaft mit beschraenkter haftung"

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref WHITELIST: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        // Titles
        m.insert("Dr.", "doktor");
        m.insert("Dr", "doktor");
        m.insert("Hr.", "herr");
        m.insert("Hr", "herr");
        m.insert("Fr.", "frau");
        m.insert("Fr", "frau");
        m.insert("Prof.", "professor");
        m.insert("Prof", "professor");
        m.insert("St.", "sankt");
        m.insert("St", "sankt");
        m.insert("Jr.", "junior");
        m.insert("Sr.", "senior");

        // Common abbreviations
        m.insert("z.B.", "zum beispiel");
        m.insert("d.h.", "das heisst");
        m.insert("usw.", "und so weiter");
        m.insert("etc.", "et cetera");
        m.insert("bzw.", "beziehungsweise");
        m.insert("evtl.", "eventuell");
        m.insert("ca.", "circa");

        // Organizational
        m.insert("Nr.", "nummer");
        m.insert("Str.", "strasse");
        m.insert("GmbH", "gesellschaft mit beschraenkter haftung");
        m.insert("AG", "aktiengesellschaft");
        m.insert("Abt.", "abteilung");
        m.insert("Tel.", "telefon");

        m
    };
}

/// Parse a German whitelist abbreviation to its spoken form.
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
        assert_eq!(parse("Dr."), Some("doktor".to_string()));
        assert_eq!(parse("Hr."), Some("herr".to_string()));
        assert_eq!(parse("Fr."), Some("frau".to_string()));
        assert_eq!(parse("Prof."), Some("professor".to_string()));
    }

    #[test]
    fn test_abbreviations() {
        assert_eq!(parse("z.B."), Some("zum beispiel".to_string()));
        assert_eq!(parse("d.h."), Some("das heisst".to_string()));
        assert_eq!(parse("usw."), Some("und so weiter".to_string()));
        assert_eq!(parse("GmbH"), Some("gesellschaft mit beschraenkter haftung".to_string()));
    }

    #[test]
    fn test_no_match() {
        assert_eq!(parse("hallo"), None);
        assert_eq!(parse("welt"), None);
    }
}
