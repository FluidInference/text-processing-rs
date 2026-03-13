//! Punctuation tagger.
//!
//! Converts spoken punctuation words to their written symbols:
//! - "period" → "."
//! - "comma" → ","
//! - "question mark" → "?"
//! - "exclamation point" → "!"

use lazy_static::lazy_static;

lazy_static! {
    /// Spoken punctuation → written symbol mappings.
    /// Ordered longest-first so multi-word patterns match before single-word ones.
    static ref PUNCTUATION: Vec<(&'static str, &'static str)> = vec![
        // Multi-word patterns first
        ("exclamation point", "!"),
        ("exclamation mark", "!"),
        ("question mark", "?"),
        ("open parenthesis", "("),
        ("close parenthesis", ")"),
        ("left parenthesis", "("),
        ("right parenthesis", ")"),
        ("open bracket", "["),
        ("close bracket", "]"),
        ("left bracket", "["),
        ("right bracket", "]"),
        ("open brace", "{"),
        ("close brace", "}"),
        ("left brace", "{"),
        ("right brace", "}"),
        ("double quote", "\""),
        ("single quote", "'"),
        ("forward slash", "/"),
        ("back slash", "\\"),

        // Single-word patterns
        ("period", "."),
        ("dot", "."),
        ("comma", ","),
        ("colon", ":"),
        ("semicolon", ";"),
        ("hyphen", "-"),
        ("dash", "-"),
        ("ellipsis", "..."),
        ("ampersand", "&"),
        ("asterisk", "*"),
        ("at sign", "@"),
        ("hash", "#"),
        ("percent", "%"),
        ("plus", "+"),
        ("equals", "="),
        ("tilde", "~"),
        ("underscore", "_"),
        ("pipe", "|"),
        ("slash", "/"),
    ];
}

/// Try to parse spoken punctuation into its written symbol.
///
/// Returns `Some(symbol)` if the entire input matches a known punctuation word.
/// Only matches exact full input — does not replace within sentences.
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
        assert_eq!(parse("period"), Some(".".to_string()));
        assert_eq!(parse("comma"), Some(",".to_string()));
        assert_eq!(parse("colon"), Some(":".to_string()));
        assert_eq!(parse("semicolon"), Some(";".to_string()));
    }

    #[test]
    fn test_multi_word() {
        assert_eq!(parse("question mark"), Some("?".to_string()));
        assert_eq!(parse("exclamation point"), Some("!".to_string()));
        assert_eq!(parse("exclamation mark"), Some("!".to_string()));
        assert_eq!(parse("open parenthesis"), Some("(".to_string()));
        assert_eq!(parse("close parenthesis"), Some(")".to_string()));
        assert_eq!(parse("double quote"), Some("\"".to_string()));
        assert_eq!(parse("forward slash"), Some("/".to_string()));
    }

    #[test]
    fn test_case_insensitive() {
        assert_eq!(parse("Period"), Some(".".to_string()));
        assert_eq!(parse("COMMA"), Some(",".to_string()));
        assert_eq!(parse("Question Mark"), Some("?".to_string()));
    }

    #[test]
    fn test_symbols() {
        assert_eq!(parse("hyphen"), Some("-".to_string()));
        assert_eq!(parse("dash"), Some("-".to_string()));
        assert_eq!(parse("ampersand"), Some("&".to_string()));
        assert_eq!(parse("asterisk"), Some("*".to_string()));
        assert_eq!(parse("hash"), Some("#".to_string()));
        assert_eq!(parse("percent"), Some("%".to_string()));
        assert_eq!(parse("at sign"), Some("@".to_string()));
        assert_eq!(parse("ellipsis"), Some("...".to_string()));
    }

    #[test]
    fn test_no_match() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("the period was great"), None);
        assert_eq!(parse(""), None);
    }
}
