//! Punctuation tagger for German.
//!
//! Converts spoken German punctuation words to symbols:
//! - "punkt" → "."
//! - "komma" → ","
//! - "fragezeichen" → "?"
//! - "ausrufezeichen" → "!"

use lazy_static::lazy_static;

lazy_static! {
    /// Punctuation mappings (longer patterns first)
    static ref PUNCTUATION: Vec<(&'static str, &'static str)> = vec![
        ("fragezeichen", "?"),
        ("ausrufezeichen", "!"),
        ("doppelpunkt", ":"),
        ("semikolon", ";"),
        ("punkt", "."),
        ("komma", ","),
        ("bindestrich", "-"),
        ("gedankenstrich", "—"),
        ("anführungszeichen", "\""),
    ];
}

/// Parse spoken German punctuation to symbol.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trim = input_lower.trim();

    for &(spoken, symbol) in PUNCTUATION.iter() {
        if input_trim == spoken {
            return Some(symbol.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_punctuation() {
        assert_eq!(parse("punkt"), Some(".".to_string()));
        assert_eq!(parse("komma"), Some(",".to_string()));
        assert_eq!(parse("fragezeichen"), Some("?".to_string()));
    }
}
