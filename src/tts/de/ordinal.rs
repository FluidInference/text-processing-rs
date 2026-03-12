//! Ordinal TN tagger for German.
//!
//! Converts written ordinal numbers to spoken German:
//! - "1." → "erste"
//! - "2." → "zweite"
//! - "3." → "dritte"
//! - "20." → "zwanzigste"

use super::number_to_words;

/// Special ordinal forms for 1-19 (irregular).
const SPECIAL_ORDINALS: &[(u32, &str)] = &[
    (1, "erste"),
    (2, "zweite"),
    (3, "dritte"),
    (4, "vierte"),
    (5, "fuenfte"),
    (6, "sechste"),
    (7, "siebte"),
    (8, "achte"),
];

/// Parse a written ordinal to spoken German words.
///
/// German ordinals are formed by adding a period after the number: "1.", "2.", "3."
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // German ordinals end with a period: "1.", "2.", "3."
    let num_str = trimmed.strip_suffix('.')?;

    if num_str.is_empty() || !num_str.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let n: u32 = num_str.parse().ok()?;
    if n == 0 {
        return None;
    }

    Some(ordinal_word(n))
}

/// Convert a number to its German ordinal word form.
fn ordinal_word(n: u32) -> String {
    // Check special forms first
    for &(num, word) in SPECIAL_ORDINALS {
        if n == num {
            return word.to_string();
        }
    }

    let cardinal = number_to_words(n as i64);

    if n < 20 {
        // 1-19: add "-te" suffix (special cases handled above)
        format!("{}te", cardinal)
    } else {
        // 20+: add "-ste" suffix
        format!("{}ste", cardinal)
    }
}

/// Convert a number to its German ordinal with "-ter" ending (for dates).
pub(crate) fn ordinal_word_ter(n: u32) -> String {
    if n == 0 {
        return "nullter".to_string();
    }

    // Check special forms
    let base = match n {
        1 => "erster".to_string(),
        2 => "zweiter".to_string(),
        3 => "dritter".to_string(),
        4 => "vierter".to_string(),
        5 => "fuenfter".to_string(),
        6 => "sechster".to_string(),
        7 => "siebter".to_string(),
        8 => "achter".to_string(),
        _ => {
            let cardinal = number_to_words(n as i64);
            if n < 20 {
                format!("{}ter", cardinal)
            } else {
                format!("{}ster", cardinal)
            }
        }
    };
    base
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_special_ordinals() {
        assert_eq!(parse("1."), Some("erste".to_string()));
        assert_eq!(parse("2."), Some("zweite".to_string()));
        assert_eq!(parse("3."), Some("dritte".to_string()));
        assert_eq!(parse("7."), Some("siebte".to_string()));
        assert_eq!(parse("8."), Some("achte".to_string()));
    }

    #[test]
    fn test_regular_ordinals() {
        assert_eq!(parse("9."), Some("neunte".to_string()));
        assert_eq!(parse("10."), Some("zehnte".to_string()));
        assert_eq!(parse("12."), Some("zwoelfte".to_string()));
        assert_eq!(parse("15."), Some("fuenfzehnte".to_string()));
    }

    #[test]
    fn test_ordinals_20_plus() {
        assert_eq!(parse("20."), Some("zwanzigste".to_string()));
        assert_eq!(parse("21."), Some("einundzwanzigste".to_string()));
        assert_eq!(parse("100."), Some("einhundertste".to_string()));
    }

    #[test]
    fn test_non_ordinals() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("0."), None);
        assert_eq!(parse("."), None);
    }
}
