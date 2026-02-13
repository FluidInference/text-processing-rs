//! Whitelist tagger.
//!
//! Converts spoken abbreviations and special phrases to written form:
//! - "doctor smith" → "dr. smith"
//! - "misses jones" → "mrs. jones"
//! - "for example" → "e.g."
//! - "s and p five hundred" → "S&P 500"
//! - "r t x" → "RTX"

use lazy_static::lazy_static;

lazy_static! {
    /// Whitelist replacements: (spoken pattern, written form)
    /// Ordered from longest to shortest to match most specific first
    static ref REPLACEMENTS: Vec<(&'static str, &'static str)> = vec![
        // Tech terms with numbers
        ("l g a eleven fifty", "LGA 1150"),
        ("p c i e x eight", "PCIe x8"),
        ("s and p five hundred", "S&P 500"),
        ("seven eleven", "7-eleven"),
        ("cat five e", "CAT5e"),
        ("c u d n n", "cuDNN"),
        ("r t x", "RTX"),

        // Phrases
        ("for example", "e.g."),

        // Titles (must come after longer patterns)
        ("doctor", "dr."),
        ("misses", "mrs."),
        ("mister", "mr."),
        ("saint", "st."),
    ];
}

/// Patterns that should only match when they're the complete input
/// (abbreviations that might be part of larger alphanumeric codes)
fn is_exact_match_only(pattern: &str) -> bool {
    matches!(pattern, "r t x" | "p c i e x eight" | "cat five e" | "c u d n n")
}

/// Apply whitelist replacements to input text, preserving original casing where possible.
/// Returns Some if any replacement was made, None otherwise.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trimmed = input_lower.trim();
    let mut result = input.to_string(); // Keep original casing
    let mut made_replacement = false;

    for (pattern, replacement) in REPLACEMENTS.iter() {
        if is_exact_match_only(pattern) {
            // Only match if this is the complete input
            if input_trimmed == *pattern {
                return Some(replacement.to_string());
            }
        } else if input_lower.contains(pattern) {
            // Find the pattern case-insensitively and replace with case-aware replacement
            result = replace_preserve_case(&result, pattern, replacement);
            made_replacement = true;
        }
    }

    if made_replacement {
        Some(result)
    } else {
        None
    }
}

/// Replace pattern preserving the first letter's case from the original
fn replace_preserve_case(input: &str, pattern: &str, replacement: &str) -> String {
    let input_lower = input.to_lowercase();
    if let Some(start) = input_lower.find(pattern) {
        // Check if original starts with uppercase
        let orig_char = input.chars().nth(start);
        let replacement_adjusted = if orig_char.map(|c| c.is_uppercase()).unwrap_or(false) {
            // Capitalize the replacement
            let mut chars = replacement.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => replacement.to_string(),
            }
        } else {
            replacement.to_string()
        };

        // Replace in original string (case-insensitive position)
        let before = &input[..start];
        let after = &input[start + pattern.len()..];
        format!("{}{}{}", before, replacement_adjusted, after)
    } else {
        input.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_titles() {
        assert_eq!(parse("doctor dao"), Some("dr. dao".to_string()));
        assert_eq!(parse("misses smith"), Some("mrs. smith".to_string()));
        assert_eq!(parse("mister dao"), Some("mr. dao".to_string()));
        assert_eq!(parse("saint george"), Some("st. george".to_string()));
    }

    #[test]
    fn test_phrases() {
        assert_eq!(
            parse("i like for example ice cream"),
            Some("i like e.g. ice cream".to_string())
        );
    }

    #[test]
    fn test_tech_terms() {
        assert_eq!(parse("r t x"), Some("RTX".to_string()));
        assert_eq!(parse("s and p five hundred"), Some("S&P 500".to_string()));
        assert_eq!(parse("seven eleven stores"), Some("7-eleven stores".to_string()));
        assert_eq!(parse("cat five e"), Some("CAT5e".to_string()));
        assert_eq!(parse("c u d n n"), Some("cuDNN".to_string()));
        assert_eq!(parse("p c i e x eight"), Some("PCIe x8".to_string()));
        assert_eq!(parse("l g a eleven fifty"), Some("LGA 1150".to_string()));
    }

    #[test]
    fn test_no_match() {
        assert_eq!(parse("hello world"), None);
    }
}
