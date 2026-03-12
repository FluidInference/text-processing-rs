//! Ordinal TN tagger for French.
//!
//! Converts written ordinal numbers to spoken French:
//! - "1er" → "premier"
//! - "1re" → "premiere"
//! - "2e" → "deuxieme"
//! - "21e" → "vingt et unieme"

use super::number_to_words;

/// Parse a written ordinal to spoken French words.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // Detect French ordinal suffixes: er, ere, re, e, eme, ieme, ieme
    let (num_str, feminine) = if let Some(s) = trimmed.strip_suffix("ere") {
        (s, true)
    } else if let Some(s) = trimmed.strip_suffix("re") {
        (s, true)
    } else if let Some(s) = trimmed.strip_suffix("ieme") {
        (s, false)
    } else if let Some(s) = trimmed.strip_suffix("eme") {
        (s, false)
    } else if let Some(s) = trimmed.strip_suffix("er") {
        (s, false)
    } else if let Some(s) = trimmed.strip_suffix("nd") {
        (s, false)
    } else if let Some(s) = trimmed.strip_suffix("nde") {
        (s, true)
    } else if let Some(s) = trimmed.strip_suffix('e') {
        // Must check this is not just a word ending in 'e'
        if s.chars().all(|c| c.is_ascii_digit()) && !s.is_empty() {
            (s, false)
        } else {
            return None;
        }
    } else {
        return None;
    };

    if num_str.is_empty() || !num_str.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let n: i64 = num_str.parse().ok()?;
    if n <= 0 {
        return None;
    }

    if n == 1 {
        return Some(if feminine {
            "premiere".to_string()
        } else {
            "premier".to_string()
        });
    }

    if n == 2 && trimmed.ends_with("nd") {
        return Some("second".to_string());
    }
    if n == 2 && trimmed.ends_with("nde") {
        return Some("seconde".to_string());
    }

    let cardinal = number_to_words(n);
    Some(cardinal_to_ordinal(&cardinal))
}

/// Convert cardinal words to ordinal by adding -ieme suffix.
fn cardinal_to_ordinal(cardinal: &str) -> String {
    // Special transformations for the last word
    if let Some(prefix) = cardinal.strip_suffix("cinq") {
        format!("{}cinquieme", prefix)
    } else if let Some(prefix) = cardinal.strip_suffix("neuf") {
        format!("{}neuvieme", prefix)
    } else if cardinal.ends_with('e') {
        // Drop final 'e' before adding -ieme (quatre → quatrieme)
        format!("{}ieme", &cardinal[..cardinal.len() - 1])
    } else {
        format!("{}ieme", cardinal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_premier() {
        assert_eq!(parse("1er"), Some("premier".to_string()));
        assert_eq!(parse("1re"), Some("premiere".to_string()));
        assert_eq!(parse("1ere"), Some("premiere".to_string()));
    }

    #[test]
    fn test_basic() {
        assert_eq!(parse("2e"), Some("deuxieme".to_string()));
        assert_eq!(parse("3e"), Some("troisieme".to_string()));
        assert_eq!(parse("4e"), Some("quatrieme".to_string()));
        assert_eq!(parse("5e"), Some("cinquieme".to_string()));
        assert_eq!(parse("9e"), Some("neuvieme".to_string()));
    }

    #[test]
    fn test_teens() {
        assert_eq!(parse("11e"), Some("onzieme".to_string()));
        assert_eq!(parse("12e"), Some("douzieme".to_string()));
        assert_eq!(parse("13e"), Some("treizieme".to_string()));
    }

    #[test]
    fn test_compound() {
        assert_eq!(parse("21e"), Some("vingt et unieme".to_string()));
        assert_eq!(parse("22e"), Some("vingt-deuxieme".to_string()));
        assert_eq!(parse("99e"), Some("quatre-vingt-dix-neuvieme".to_string()));
    }

    #[test]
    fn test_large() {
        assert_eq!(parse("100e"), Some("centieme".to_string()));
        assert_eq!(parse("1000e"), Some("millieme".to_string()));
        assert_eq!(parse("101e"), Some("cent unieme".to_string()));
    }

    #[test]
    fn test_non_ordinals() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("0e"), None);
    }
}
