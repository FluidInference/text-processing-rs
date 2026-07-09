//! Money TN tagger.
//!
//! Converts written currency expressions to spoken form:
//! - "$5.50" → "five dollars fifty cents"
//! - "$1" → "one dollar"
//! - "$0.01" → "one cent"
//! - "€100" → "one hundred euros"
//! - "$2.5 billion" → "two point five billion dollars"

use super::{number_to_words, number_to_words_and, spell_digits};

/// Spell a non-negative money amount with NeMo's British "and": the multiplier
/// groups (thousand and up) read plainly, while the final group takes an "and"
/// — either inside the hundreds (`854` → "eight hundred and fifty four") or,
/// when it is a bare sub-100 value after a larger part, in front of it
/// (`18081` → "eighteen thousand and eighty one").
fn amount_words(n: i64) -> String {
    if n <= 0 {
        return number_to_words(n);
    }
    let n = n as u128;
    let units = (n % 1000) as u32;
    let higher = n - u128::from(units);

    if higher == 0 {
        // Single group: use the within-hundreds "and" only.
        return number_to_words_and(n);
    }

    let higher_words = number_to_words((higher) as i64);
    match units {
        0 => higher_words,
        // Bare sub-100 final group takes a leading "and".
        1..=99 => format!("{} and {}", higher_words, number_to_words(units as i64)),
        // A hundreds final group carries its own internal "and".
        _ => format!(
            "{} {}",
            higher_words,
            number_to_words_and(u128::from(units))
        ),
    }
}

/// Currency info: (symbol, singular, plural, cent_singular, cent_plural)
struct Currency {
    singular: &'static str,
    plural: &'static str,
    cent_singular: &'static str,
    cent_plural: &'static str,
}

const DOLLAR: Currency = Currency {
    singular: "dollar",
    plural: "dollars",
    cent_singular: "cent",
    cent_plural: "cents",
};

const US_DOLLAR: Currency = Currency {
    singular: "us dollar",
    plural: "us dollars",
    cent_singular: "cent",
    cent_plural: "cents",
};

const EURO: Currency = Currency {
    singular: "euro",
    plural: "euros",
    cent_singular: "cent",
    cent_plural: "cents",
};

const POUND: Currency = Currency {
    singular: "pound",
    plural: "pounds",
    cent_singular: "penny",
    cent_plural: "pence",
};

const YEN: Currency = Currency {
    singular: "yen",
    plural: "yen",
    cent_singular: "sen",
    cent_plural: "sen",
};

const WON: Currency = Currency {
    singular: "won",
    plural: "won",
    cent_singular: "jeon",
    cent_plural: "jeon",
};

/// Scale suffixes we recognize after a currency amount.
const SCALE_SUFFIXES: &[&str] = &["trillion", "billion", "million", "thousand"];

/// Parse a written money expression to spoken form.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Money per period: "$20/mo" → "twenty dollars per month".
    if let Some(result) = parse_per_period(trimmed) {
        return Some(result);
    }

    // Detect currency symbol ("US$" before the bare "$").
    let (currency, rest) = if let Some(r) = trimmed.strip_prefix("US$") {
        (&US_DOLLAR, r)
    } else if let Some(r) = trimmed.strip_prefix('$') {
        (&DOLLAR, r)
    } else if let Some(r) = trimmed.strip_prefix('€') {
        (&EURO, r)
    } else if let Some(r) = trimmed.strip_prefix('£') {
        (&POUND, r)
    } else if let Some(r) = trimmed.strip_prefix('¥') {
        (&YEN, r)
    } else if let Some(r) = trimmed.strip_prefix('₩') {
        (&WON, r)
    } else {
        return None;
    };

    let rest = rest.trim();
    if rest.is_empty() {
        return None;
    }

    // Check for scale suffix: "$2.5 billion"
    let (amount_str, scale) = extract_scale(rest);

    // Without a scale suffix, the amount must be purely numeric (digits, dots, commas).
    // This prevents "$10.20 on March 8" from matching as money and consuming trailing words.
    if scale.is_none()
        && !amount_str
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == ',')
    {
        return None;
    }

    // Parse the numeric amount
    if let Some(scale_word) = scale {
        // With scale: "$2.5 billion" → "two point five billion dollars"
        // The multiplier before a scale word is not a final group, so it takes
        // no "and" (`₩460 billion` → "four hundred sixty billion won").
        if amount_str.contains('.') {
            let parts: Vec<&str> = amount_str.splitn(2, '.').collect();
            if parts.len() == 2 {
                let int_val: i64 = parts[0].parse().ok()?;
                let frac_words = spell_digits(parts[1]);
                return Some(format!(
                    "{} point {} {} {}",
                    number_to_words(int_val),
                    frac_words,
                    scale_word,
                    currency.plural
                ));
            }
        } else {
            let clean: String = amount_str.chars().filter(|c| *c != ',').collect();
            let n: i64 = clean.parse().ok()?;
            return Some(format!(
                "{} {} {}",
                number_to_words(n),
                scale_word,
                currency.plural
            ));
        }
    }

    // No scale — regular amount
    if amount_str.contains('.') {
        parse_dollars_cents(amount_str, currency)
    } else {
        let clean: String = amount_str.chars().filter(|c| *c != ',').collect();
        let n: i64 = clean.parse().ok()?;
        let unit = if n == 1 {
            currency.singular
        } else {
            currency.plural
        };
        Some(format!("{} {}", amount_words(n), unit))
    }
}

/// Money over a period abbreviation: `$20/mo` → "twenty dollars per month".
fn parse_per_period(input: &str) -> Option<String> {
    let (money_part, period) = input.split_once('/')?;
    let period_word = match period.trim() {
        "mo" => "month",
        "yr" => "year",
        "wk" => "week",
        "d" => "day",
        "hr" => "hour",
        "sec" => "second",
        "min" => "minute",
        _ => return None,
    };
    let money = parse(money_part.trim())?;
    Some(format!("{} per {}", money, period_word))
}

/// Parse `$X.Y`. A fractional part that is a whole number of cents (≤ 2
/// significant digits after trimming trailing zeros) reads as cents with no
/// "and" — `$20.50` → "twenty dollars fifty cents". Otherwise it reads as a
/// decimal amount — `$20.506` → "twenty point five zero six dollars".
fn parse_dollars_cents(amount: &str, currency: &Currency) -> Option<String> {
    let (int_str, frac_str) = amount.split_once('.')?;
    let int_clean: String = int_str.chars().filter(|c| *c != ',').collect();

    // More than two significant fractional digits → spoken as a decimal.
    if frac_str.trim_end_matches('0').len() > 2 {
        let frac_words = spell_digits(frac_str);
        if int_clean.is_empty() {
            return Some(format!("point {} {}", frac_words, currency.plural));
        }
        let int_val: i64 = int_clean.parse().ok()?;
        return Some(format!(
            "{} point {} {}",
            amount_words(int_val),
            frac_words,
            currency.plural
        ));
    }

    // Whole cents.
    let dollars: i64 = if int_clean.is_empty() {
        0
    } else {
        int_clean.parse().ok()?
    };
    let cents: i64 = match frac_str.trim_end_matches('0') {
        "" => 0,
        one if one.len() == 1 => one.parse::<i64>().ok()? * 10,
        two => two.parse().ok()?,
    };

    let dollar_part = (dollars != 0).then(|| {
        let unit = if dollars == 1 {
            currency.singular
        } else {
            currency.plural
        };
        format!("{} {}", amount_words(dollars), unit)
    });
    let cent_part = (cents != 0).then(|| {
        let unit = if cents == 1 {
            currency.cent_singular
        } else {
            currency.cent_plural
        };
        format!("{} {}", amount_words(cents), unit)
    });

    match (dollar_part, cent_part) {
        (Some(d), Some(c)) => Some(format!("{} {}", d, c)),
        (Some(d), None) => Some(d),
        (None, Some(c)) => Some(c),
        (None, None) => Some(format!("zero {}", currency.plural)),
    }
}

/// Single-letter magnitude abbreviations after an amount (`¥30b`).
const SCALE_ABBREVS: &[(&str, &str)] = &[
    ("b", "billion"),
    ("m", "million"),
    ("k", "thousand"),
    ("t", "trillion"),
];

/// Extract a trailing scale word or single-letter magnitude abbreviation.
fn extract_scale(input: &str) -> (&str, Option<&str>) {
    let numeric = |s: &str| {
        !s.is_empty()
            && s.chars()
                .all(|c| c.is_ascii_digit() || c == '.' || c == ',')
    };

    for &scale in SCALE_SUFFIXES {
        if let Some(before) = input.strip_suffix(scale) {
            let before = before.trim_end();
            if numeric(before) {
                return (before, Some(scale));
            }
        }
    }
    // `30b` / `30 B` → billion. Case-insensitive; requires a numeric amount
    // immediately (optionally space-separated) before the letter.
    if let Some(last) = input.chars().last() {
        let lower = last.to_ascii_lowercase().to_string();
        if let Some(&(_, word)) = SCALE_ABBREVS.iter().find(|(a, _)| *a == lower) {
            let before = input[..input.len() - last.len_utf8()].trim_end();
            if numeric(before) {
                return (before, Some(word));
            }
        }
    }
    (input, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_dollars() {
        assert_eq!(parse("$1"), Some("one dollar".to_string()));
        assert_eq!(parse("$5"), Some("five dollars".to_string()));
        assert_eq!(parse("$100"), Some("one hundred dollars".to_string()));
    }

    #[test]
    fn test_dollars_and_cents() {
        // NeMo: no "and" between the dollar and cent parts.
        assert_eq!(parse("$5.50"), Some("five dollars fifty cents".to_string()));
        assert_eq!(parse("$1.01"), Some("one dollar one cent".to_string()));
        assert_eq!(parse("$0.01"), Some("one cent".to_string()));
        assert_eq!(parse("$0.99"), Some("ninety nine cents".to_string()));
        // Trailing zeros trimmed to whole cents.
        assert_eq!(
            parse("$20.500"),
            Some("twenty dollars fifty cents".to_string())
        );
    }

    #[test]
    fn test_british_and_in_amount() {
        assert_eq!(
            parse("$18854"),
            Some("eighteen thousand eight hundred and fifty four dollars".to_string())
        );
        // Bare sub-100 final group after a larger part takes a leading "and".
        assert_eq!(
            parse("$18081"),
            Some("eighteen thousand and eighty one dollars".to_string())
        );
    }

    #[test]
    fn test_more_than_two_decimals_reads_as_decimal() {
        assert_eq!(
            parse("$20.506"),
            Some("twenty point five zero six dollars".to_string())
        );
        assert_eq!(
            parse("$.506"),
            Some("point five zero six dollars".to_string())
        );
    }

    #[test]
    fn test_large_amounts() {
        assert_eq!(
            parse("$2.5 billion"),
            Some("two point five billion dollars".to_string())
        );
        assert_eq!(
            parse("$50 million"),
            Some("fifty million dollars".to_string())
        );
        // Multiplier before a scale word takes no "and".
        assert_eq!(
            parse("₩460 billion"),
            Some("four hundred sixty billion won".to_string())
        );
        // Single-letter magnitude abbreviation.
        assert_eq!(parse("¥30b"), Some("thirty billion yen".to_string()));
    }

    #[test]
    fn test_other_currencies() {
        assert_eq!(parse("€100"), Some("one hundred euros".to_string()));
        assert_eq!(parse("£1"), Some("one pound".to_string()));
        assert_eq!(parse("¥500"), Some("five hundred yen".to_string()));
    }

    #[test]
    fn test_trailing_dot() {
        // "$5." should not panic — empty cents treated as zero
        assert_eq!(parse("$5."), Some("five dollars".to_string()));
        assert_eq!(parse("$1."), Some("one dollar".to_string()));
        assert_eq!(parse("$0."), Some("zero dollars".to_string()));
    }

    #[test]
    fn test_non_money() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("123"), None);
        assert_eq!(parse("$"), None);
    }
}
