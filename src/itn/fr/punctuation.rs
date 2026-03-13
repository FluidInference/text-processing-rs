//! Punctuation tagger for French.
//!
//! Converts spoken French punctuation words to their written symbols:
//! - "point" → "."
//! - "virgule" → ","
//! - "point d'interrogation" → "?"

use lazy_static::lazy_static;

lazy_static! {
    /// Spoken French punctuation → written symbol mappings.
    static ref PUNCTUATION: Vec<(&'static str, &'static str)> = vec![
        // Multi-word patterns first
        ("point d'interrogation", "?"),
        ("point dinterrogation", "?"),
        ("point d'exclamation", "!"),
        ("point dexclamation", "!"),
        ("guillemet ouvrant", "«"),
        ("guillemet fermant", "»"),
        ("parenthèse ouvrante", "("),
        ("parenthèse fermante", ")"),
        ("crochet ouvrant", "["),
        ("crochet fermant", "]"),
        ("accolade ouvrante", "{"),
        ("accolade fermante", "}"),
        ("deux points", ":"),
        ("point virgule", ";"),
        ("trait d'union", "-"),
        ("barre oblique", "/"),

        // Single-word patterns
        ("point", "."),
        ("virgule", ","),
        ("tiret", "-"),
        ("arobase", "@"),
        ("dièse", "#"),
        ("pourcent", "%"),
        ("plus", "+"),
        ("égal", "="),
        ("astérisque", "*"),
        ("slash", "/"),
    ];
}

/// Try to parse spoken French punctuation into its written symbol.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trimmed = input_lower.trim();

    for (pattern, symbol) in PUNCTUATION.iter() {
        if input_trimmed == *pattern {
            return Some(symbol.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_punctuation() {
        assert_eq!(parse("point"), Some(".".to_string()));
        assert_eq!(parse("virgule"), Some(",".to_string()));
        assert_eq!(parse("deux points"), Some(":".to_string()));
        assert_eq!(parse("point virgule"), Some(";".to_string()));
    }

    #[test]
    fn test_multi_word() {
        assert_eq!(parse("point d'interrogation"), Some("?".to_string()));
        assert_eq!(parse("point d'exclamation"), Some("!".to_string()));
        assert_eq!(parse("parenthèse ouvrante"), Some("(".to_string()));
    }

    #[test]
    fn test_symbols() {
        assert_eq!(parse("tiret"), Some("-".to_string()));
        assert_eq!(parse("arobase"), Some("@".to_string()));
        assert_eq!(parse("dièse"), Some("#".to_string()));
        assert_eq!(parse("pourcent"), Some("%".to_string()));
    }
}
