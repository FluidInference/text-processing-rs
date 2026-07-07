//! Fraction TN tagger.
//!
//! Converts written fractions to spoken form:
//! - "1/2" → "one half"
//! - "3/4" → "three quarters"
//! - "31/32" → "thirty one thirty seconds"
//! - "3 2/4" → "three and two quarters" (mixed)
//! - "2 1/2" → "two and a half" (mixed)
//! - "142/1" → "one hundred forty two over one"

use super::number_to_words;
use super::ordinal::number_to_ordinal_words;

/// Parse a written fraction to spoken form.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if !trimmed.contains('/') {
        return None;
    }

    // Collapse spaces immediately around the slash ("1795 / 1805" →
    // "1795/1805") so only a mixed fraction's whole/numerator gap remains as
    // a space.
    let collapsed = trimmed.replace(" /", "/").replace("/ ", "/");

    // Mixed fraction: "WHOLE NUMER/DENOM" (a space now separates the whole
    // number from the fraction, e.g. "3 2/4").
    if let Some((whole_str, frac_str)) = collapsed.split_once(' ') {
        let whole: i64 = whole_str.trim().parse().ok()?;
        let frac = parse_simple(frac_str.trim(), true)?;
        return Some(format!("{} and {}", number_to_words(whole), frac));
    }

    parse_simple(&collapsed, false)
}

/// Parse `NUMER/DENOM` (denominator may carry a written ordinal suffix like
/// `4th`). `mixed` selects the "a half" rendering used only after a whole
/// number ("2 1/2" → "two and a half", but standalone "1/2" → "one half").
fn parse_simple(input: &str, mixed: bool) -> Option<String> {
    let (numer_str, denom_raw) = input.split_once('/')?;
    let numer: i64 = numer_str.trim().parse().ok()?;

    // Drop a trailing ordinal suffix on the denominator (`4th`, `3RD`).
    let denom_trim = denom_raw.trim();
    let denom_digits = strip_ordinal_suffix(denom_trim);
    let denom: i64 = denom_digits.parse().ok()?;

    let plural = numer != 1;

    // Denominator of one reads as "over one" rather than an ordinal.
    if denom == 1 {
        return Some(format!("{} over one", number_to_words(numer)));
    }

    if mixed && numer == 1 && denom == 2 {
        return Some("a half".to_string());
    }

    let denom_word = denominator_words(denom, plural);
    Some(format!("{} {}", number_to_words(numer), denom_word))
}

/// Strip a trailing case-insensitive ordinal suffix (`st`/`nd`/`rd`/`th`),
/// leaving just the digits.
fn strip_ordinal_suffix(denom: &str) -> &str {
    for suffix in ["st", "nd", "rd", "th"] {
        if denom.len() > suffix.len() {
            let (head, tail) = denom.split_at(denom.len() - suffix.len());
            if tail.eq_ignore_ascii_case(suffix) && head.bytes().all(|b| b.is_ascii_digit()) {
                return head;
            }
        }
    }
    denom
}

/// The spoken denominator word: irregular half/third/quarter, otherwise the
/// ordinal reading. Pluralized (`quarters`, `thirty seconds`) when `plural`.
fn denominator_words(denom: i64, plural: bool) -> String {
    let (singular, plural_form) = match denom {
        2 => ("half", "halves"),
        3 => ("third", "thirds"),
        4 => ("quarter", "quarters"),
        _ => {
            let ordinal = number_to_ordinal_words(denom);
            return if plural {
                format!("{}s", ordinal)
            } else {
                ordinal
            };
        }
    };
    if plural { plural_form } else { singular }.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        assert_eq!(parse("1/2"), Some("one half".to_string()));
        assert_eq!(parse("1/3"), Some("one third".to_string()));
        assert_eq!(parse("1/4"), Some("one quarter".to_string()));
        assert_eq!(parse("22/3"), Some("twenty two thirds".to_string()));
        assert_eq!(
            parse("31/32"),
            Some("thirty one thirty seconds".to_string())
        );
    }

    #[test]
    fn test_ordinal_suffix_on_denominator() {
        assert_eq!(parse("1/4th"), Some("one quarter".to_string()));
        assert_eq!(parse("2/4TH"), Some("two quarters".to_string()));
        assert_eq!(parse("1/3RD"), Some("one third".to_string()));
    }

    #[test]
    fn test_large_denominators() {
        assert_eq!(
            parse("1/2007"),
            Some("one two thousand seventh".to_string())
        );
        assert_eq!(
            parse("12639/12640"),
            Some(
                "twelve thousand six hundred thirty nine twelve thousand six hundred fortieths"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_mixed() {
        assert_eq!(parse("3 2/4"), Some("three and two quarters".to_string()));
        assert_eq!(parse("2 1/2"), Some("two and a half".to_string()));
        assert_eq!(parse("3 5/2"), Some("three and five halves".to_string()));
    }

    #[test]
    fn test_over_one() {
        assert_eq!(
            parse("2 142/1"),
            Some("two and one hundred forty two over one".to_string())
        );
    }

    #[test]
    fn test_spaced_slash() {
        assert_eq!(
            parse("1795 / 1805"),
            Some(
                "one thousand seven hundred ninety five one thousand eight hundred fifths"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_not_a_fraction() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("12"), None);
        assert_eq!(parse("1/2/3"), None);
    }
}
