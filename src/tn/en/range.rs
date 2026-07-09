//! Range TN tagger.
//!
//! Reads `A<sep>B` where the separator implies a relation. A hyphen between
//! *typed* operands (years, decades, money, times, or a measured quantity)
//! reads as "to"; between bare cardinals it stays a spoken hyphen:
//! - "1980-1986" → "nineteen eighty to nineteen eighty six"
//! - "$250-$300" → "two hundred and fifty dollars to three hundred dollars"
//! - "2-5lb" → "two to five pounds"
//! - "2-5" → "two - five"
//! - "2+3" → "two plus three", "mid-1980s" → "mid nineteen eighties"

use super::{cardinal, date, measure, money, time};

/// Parse a range expression to spoken form.
pub fn parse(input: &str) -> Option<String> {
    let t = input.trim();

    // "mid-1980s" / "mid-80s" → "mid <decade>".
    if let Some(rest) = t.strip_prefix("mid-") {
        return date::parse(rest).map(|d| format!("mid {}", d));
    }

    // Addition reads literally.
    if let Some((a, b)) = split_binary(t, '+') {
        return Some(format!("{} plus {}", cardinal(a)?, cardinal(b)?));
    }

    // Dimensions: "2x8" → "two x eight" (both sides bare cardinals).
    if let Some((a, b)) = split_binary(t, 'x') {
        if let (Some(aw), Some(bw)) = (cardinal(a), cardinal(b)) {
            return Some(format!("{} x {}", aw, bw));
        }
    }

    // Hyphen or en-dash range.
    for sep in ['-', '\u{2013}'] {
        if let Some((a, b)) = split_binary(t, sep) {
            if let Some(result) = range_words(a, b) {
                return Some(result);
            }
        }
    }

    None
}

/// Split on the first `sep` into two non-empty trimmed operands.
fn split_binary(input: &str, sep: char) -> Option<(&str, &str)> {
    let (a, b) = input.split_once(sep)?;
    let (a, b) = (a.trim(), b.trim());
    if a.is_empty() || b.is_empty() {
        return None;
    }
    Some((a, b))
}

fn range_words(a: &str, b: &str) -> Option<String> {
    // Money — both sides carry a currency symbol.
    if starts_currency(a) && starts_currency(b) {
        return Some(format!("{} to {}", money::parse(a)?, money::parse(b)?));
    }

    // Time — both sides parse as a clock time.
    if let (Some(ta), Some(tb)) = (time::parse(a), time::parse(b)) {
        return Some(format!("{} to {}", ta, tb));
    }

    // Measured quantity — the right operand carries a unit ("5lb", "1970 kg").
    // Operands then read as plain cardinals ("two to five pounds").
    if let Some(measured_b) = measure::parse(b) {
        return Some(format!("{} to {}", cardinal(a)?, measured_b));
    }

    // Decade range ("1960s-80s" → "nineteen sixties to eighties").
    if is_decade(a) && is_decade(b) {
        return Some(format!("{} to {}", date::parse(a)?, date::parse(b)?));
    }

    // Year range — the left operand is a four-digit year. The right reads as a
    // year when it is also four digits, otherwise as a bare cardinal
    // ("1960-80" → "nineteen sixty to eighty").
    if let Some(ya) = year_words(a) {
        if is_all_digits(b) {
            let yb = year_words(b).or_else(|| cardinal(b))?;
            return Some(format!("{} to {}", ya, yb));
        }
    }

    // Bare cardinals keep the hyphen as a spoken " - ".
    Some(format!("{} - {}", cardinal(a)?, cardinal(b)?))
}

fn cardinal(s: &str) -> Option<String> {
    cardinal::parse(s.trim())
}

fn starts_currency(s: &str) -> bool {
    matches!(s.chars().next(), Some('$' | '€' | '£' | '¥' | '₩'))
}

fn is_all_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

fn is_decade(s: &str) -> bool {
    let core = s.strip_suffix('s').unwrap_or("");
    let core = core.strip_prefix('\'').unwrap_or(core);
    !core.is_empty() && core.bytes().all(|b| b.is_ascii_digit())
}

/// Year-style spelling for a four-digit token, else None.
fn year_words(s: &str) -> Option<String> {
    if s.len() == 4 && is_all_digits(s) {
        return date::verbalize_year(s.parse().ok()?);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn test_year_range() {
        assert_eq!(
            parse("1980-1986"),
            Some("nineteen eighty to nineteen eighty six".to_string())
        );
        assert_eq!(
            parse("1960-80"),
            Some("nineteen sixty to eighty".to_string())
        );
    }

    #[test]
    fn test_decade_range() {
        assert_eq!(
            parse("1960s-80s"),
            Some("nineteen sixties to eighties".to_string())
        );
        assert_eq!(
            parse("1960s-1980s"),
            Some("nineteen sixties to nineteen eighties".to_string())
        );
    }

    #[test]
    fn test_money_range() {
        assert_eq!(
            parse("$250-$300"),
            Some("two hundred and fifty dollars to three hundred dollars".to_string())
        );
    }

    #[test]
    fn test_measure_range() {
        assert_eq!(parse("2-5lb"), Some("two to five pounds".to_string()));
    }

    #[test]
    fn test_bare_and_ops() {
        assert_eq!(parse("2-5"), Some("two - five".to_string()));
        assert_eq!(parse("2+3"), Some("two plus three".to_string()));
        assert_eq!(
            parse("mid-1980s"),
            Some("mid nineteen eighties".to_string())
        );
    }

    #[test]
    fn test_not_a_range() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("-5"), None);
    }
}
