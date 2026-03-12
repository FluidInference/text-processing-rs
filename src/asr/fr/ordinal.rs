//! Ordinal number tagger for French.
//!
//! Converts spoken French ordinal numbers to written form:
//! - "premier" → "1er"
//! - "première" → "1re"
//! - "deuxième" → "2e"
//! - "vingt et unième" → "21e"

use lazy_static::lazy_static;
use std::collections::HashMap;

use super::cardinal::words_to_number;

lazy_static! {
    /// French ordinal words mapping to value
    static ref ORDINAL_WORDS: HashMap<&'static str, i64> = {
        let mut m = HashMap::new();
        m.insert("premier", 1);
        m.insert("première", 1);
        m.insert("deuxième", 2);
        m.insert("second", 2);
        m.insert("seconde", 2);
        m.insert("troisième", 3);
        m.insert("quatrième", 4);
        m.insert("cinquième", 5);
        m.insert("sixième", 6);
        m.insert("septième", 7);
        m.insert("huitième", 8);
        m.insert("neuvième", 9);
        m.insert("dixième", 10);
        m.insert("onzième", 11);
        m.insert("douzième", 12);
        m.insert("treizième", 13);
        m.insert("quatorzième", 14);
        m.insert("quinzième", 15);
        m.insert("seizième", 16);
        m.insert("dix-septième", 17);
        m.insert("dix-huitième", 18);
        m.insert("dix-neuvième", 19);
        m.insert("vingtième", 20);
        m.insert("trentième", 30);
        m.insert("quarantième", 40);
        m.insert("cinquantième", 50);
        m.insert("soixantième", 60);
        m.insert("soixante-dixième", 70);
        m.insert("quatre-vingtième", 80);
        m.insert("quatre-vingt-dixième", 90);
        m.insert("centième", 100);
        m.insert("millième", 1000);
        m.insert("millionième", 1_000_000);
        m.insert("milliardième", 1_000_000_000);
        m
    };
}

/// Parse spoken French ordinal to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.trim().to_lowercase();

    // Check for direct ordinal word match
    if let Some(&value) = ORDINAL_WORDS.get(input_lower.as_str()) {
        return Some(format_ordinal(value, &input_lower));
    }

    // Check for compound ordinals like "vingt et unième"
    if let Some(result) = parse_compound_ordinal(&input_lower) {
        return Some(result);
    }

    None
}

/// Parse compound ordinals like "vingt et unième" → "21e"
fn parse_compound_ordinal(input: &str) -> Option<String> {
    // Look for ordinal suffix pattern
    if input.ends_with("ième") || input.ends_with("ème") {
        // Try to parse the whole thing as ordinal
        if let Some(&value) = ORDINAL_WORDS.get(input) {
            return Some(format!("{}e", value));
        }

        // Try removing "ième" and parsing as cardinal
        let cardinal_part = input
            .trim_end_matches("ième")
            .trim_end_matches("ème")
            .trim();

        // Special case: "unième" needs prefix
        if cardinal_part.ends_with(" et un") {
            let prefix = cardinal_part.trim_end_matches(" et un");
            if let Some(prefix_num) = words_to_number(prefix) {
                return Some(format!("{}e", prefix_num as i64 + 1));
            }
        }

        if let Some(num) = words_to_number(cardinal_part) {
            return Some(format!("{}e", num as i64));
        }
    }

    // Check for "premier" / "première" with cardinal prefix
    if input.ends_with(" premier") {
        let prefix = input.trim_end_matches(" premier");
        if let Some(num) = words_to_number(prefix) {
            return Some(format!("{}er", num as i64 + 1));
        }
    }

    if input.ends_with(" première") {
        let prefix = input.trim_end_matches(" première");
        if let Some(num) = words_to_number(prefix) {
            return Some(format!("{}re", num as i64 + 1));
        }
    }

    None
}

/// Format ordinal number with appropriate suffix
fn format_ordinal(value: i64, original: &str) -> String {
    if original.contains("première") || original.ends_with("première") {
        format!("{}re", value)
    } else if original.contains("premier") || original.ends_with("premier") {
        format!("{}er", value)
    } else {
        format!("{}e", value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_ordinals() {
        assert_eq!(parse("premier"), Some("1er".to_string()));
        assert_eq!(parse("première"), Some("1re".to_string()));
        assert_eq!(parse("deuxième"), Some("2e".to_string()));
        assert_eq!(parse("troisième"), Some("3e".to_string()));
        assert_eq!(parse("dixième"), Some("10e".to_string()));
    }

    #[test]
    fn test_compound_ordinals() {
        assert_eq!(parse("vingt et unième"), Some("21e".to_string()));
        assert_eq!(parse("cent unième"), Some("101e".to_string()));
    }

    #[test]
    fn test_large_ordinals() {
        assert_eq!(parse("centième"), Some("100e".to_string()));
        assert_eq!(parse("millième"), Some("1000e".to_string()));
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("cinq"), None); // cardinal, not ordinal
    }
}
