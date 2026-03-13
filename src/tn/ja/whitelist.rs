//! Whitelist TN tagger for Japanese (romaji output).
//!
//! Lookup table for common abbreviations translated to Japanese romaji:
//! - "Dr." → "dokutaa"
//! - "Mr." → "misutaa"
//! - "etc." → "nado"
//! - "vs." → "tai"

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref WHITELIST: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        // Titles
        m.insert("Dr.", "dokutaa");
        m.insert("Dr", "dokutaa");
        m.insert("Mr.", "misutaa");
        m.insert("Mr", "misutaa");
        m.insert("Mrs.", "misizu");
        m.insert("Mrs", "misizu");
        m.insert("Ms.", "mizu");
        m.insert("Ms", "mizu");
        m.insert("Prof.", "kyouju");
        m.insert("St.", "seinto");
        m.insert("Jr.", "junia");
        m.insert("Sr.", "shinia");

        // Abbreviations
        m.insert("etc.", "nado");
        m.insert("vs.", "tai");
        m.insert("vs", "tai");
        m.insert("No.", "bangou");

        // Business
        m.insert("Inc.", "kabushiki gaisha");
        m.insert("Ltd.", "yuugen gaisha");
        m.insert("Co.", "gaisha");

        m
    };
}

/// Parse a whitelist abbreviation to its spoken Japanese romaji form.
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
        assert_eq!(parse("Dr."), Some("dokutaa".to_string()));
        assert_eq!(parse("Mr."), Some("misutaa".to_string()));
        assert_eq!(parse("Mrs."), Some("misizu".to_string()));
        assert_eq!(parse("Ms."), Some("mizu".to_string()));
    }

    #[test]
    fn test_abbreviations() {
        assert_eq!(parse("etc."), Some("nado".to_string()));
        assert_eq!(parse("vs."), Some("tai".to_string()));
        assert_eq!(parse("No."), Some("bangou".to_string()));
    }

    #[test]
    fn test_business() {
        assert_eq!(parse("Inc."), Some("kabushiki gaisha".to_string()));
        assert_eq!(parse("Ltd."), Some("yuugen gaisha".to_string()));
        assert_eq!(parse("Co."), Some("gaisha".to_string()));
    }

    #[test]
    fn test_no_match() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("world"), None);
    }
}
