//! Whitelist tagger for German.
//!
//! Maps spoken German titles and phrases to abbreviations:
//! - "doktor dao" → "Dr. dao"
//! - "mister dao" → "Mr. dao"
//! - "zum beispiel" → "z.B."

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    /// Whitelist mappings (spoken → abbreviated)
    static ref WHITELIST: Vec<(&'static str, &'static str)> = vec![
        // Multi-word entries first (longer match)
        ("zum beispiel", "z.B."),
        ("das heißt", "d.h."),
        ("das heisst", "d.h."),
        ("und so weiter", "usw."),
        ("beziehungsweise", "bzw."),
        // Titles
        ("doktor", "Dr."),
        ("professor", "Prof."),
        ("mister", "Mr."),
        ("miss", "Ms."),
        ("misses", "Mrs."),
        ("nummer", "Nr."),
    ];
}

/// Parse spoken German whitelist entry to abbreviated form.
/// Supports prefix matching: "doktor dao" → "Dr. dao"
/// and middle-of-sentence: "ich mag essen zum beispiel eis" → "ich mag essen z.B. eis"
/// and suffix matching: "Chanel nummer fünf" → "Chanel Nr. fünf"
pub fn parse(input: &str) -> Option<String> {
    let input_trim = input.trim();
    let input_lower = input_trim.to_lowercase();

    for &(spoken, abbreviated) in WHITELIST.iter() {
        // Exact match (case-insensitive)
        if input_lower == spoken {
            return Some(abbreviated.to_string());
        }

        // Prefix match: "doktor dao" → "Dr. dao"
        if input_lower.starts_with(spoken) {
            let after = &input_lower[spoken.len()..];
            if after.starts_with(' ') {
                // Use original case for the rest
                let rest = &input_trim[spoken.len()..].trim_start();
                return Some(format!("{} {}", abbreviated, rest));
            }
        }

        // Middle match: find spoken phrase with word boundaries in the middle
        let pattern = format!(" {} ", spoken);
        if let Some(pos) = input_lower.find(&pattern) {
            // Use original case for before/after
            let before = &input_trim[..pos];
            let after = &input_trim[pos + pattern.len()..];
            if after.is_empty() {
                return Some(format!("{} {}", before, abbreviated));
            } else {
                return Some(format!("{} {} {}", before, abbreviated, after));
            }
        }

        // End match with rest after: " spoken rest_word"
        // E.g., "Chanel nummer fünf" → pattern " nummer " found as middle match above
        // But also handle: "X spoken" at end
        let end_pattern = format!(" {}", spoken);
        if input_lower.ends_with(&end_pattern) {
            let before = &input_trim[..input_trim.len() - end_pattern.len()];
            return Some(format!("{} {}", before, abbreviated));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_titles() {
        assert_eq!(parse("doktor dao"), Some("Dr. dao".to_string()));
        assert_eq!(parse("mister dao"), Some("Mr. dao".to_string()));
        assert_eq!(parse("miss smith"), Some("Ms. smith".to_string()));
        assert_eq!(parse("misses smith"), Some("Mrs. smith".to_string()));
    }

    #[test]
    fn test_phrases() {
        assert_eq!(parse("zum beispiel"), Some("z.B.".to_string()));
    }

    #[test]
    fn test_contextual() {
        assert_eq!(
            parse("ich mag essen zum beispiel eis"),
            Some("ich mag essen z.B. eis".to_string())
        );
    }
}
