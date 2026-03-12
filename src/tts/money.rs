//! Money TN tagger.
//!
//! Converts written currency expressions to spoken form:
//! - "$5.50" → "five dollars and fifty cents"
//! - "$1" → "one dollar"
//! - "$0.01" → "one cent"
//! - "€100" → "one hundred euros"
//! - "$2.5 billion" → "two point five billion dollars"

use super::number_to_words;

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

    // Detect currency symbol
    let (currency, rest) = if let Some(r) = trimmed.strip_prefix('$') {
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
        if amount_str.contains('.') {
            let parts: Vec<&str> = amount_str.splitn(2, '.').collect();
            if parts.len() == 2 {
                let int_val: i64 = parts[0].parse().ok()?;
                let int_words = number_to_words(int_val);
                let frac_words = super::spell_digits(parts[1]);
                return Some(format!(
                    "{} point {} {} {}",
                    int_words, frac_words, scale_word, currency.plural
                ));
            }
        } else {
            let n: i64 = amount_str.parse().ok()?;
            let words = number_to_words(n);
            return Some(format!("{} {} {}", words, scale_word, currency.plural));
        }
    }

    // No scale — regular amount
    if amount_str.contains('.') {
        parse_dollars_cents(amount_str, currency)
    } else {
        // Strip commas
        let clean: String = amount_str.chars().filter(|c| *c != ',').collect();
        let n: i64 = clean.parse().ok()?;
        let words = number_to_words(n);
        let unit = if n == 1 {
            currency.singular
        } else {
            currency.plural
        };
        Some(format!("{} {}", words, unit))
    }
}

/// Parse "$X.YY" → "X dollars and Y cents"
fn parse_dollars_cents(amount: &str, currency: &Currency) -> Option<String> {
    let parts: Vec<&str> = amount.splitn(2, '.').collect();
    if parts.len() != 2 {
        return None;
    }

    let dollars: i64 = if parts[0].is_empty() {
        0
    } else {
        parts[0].parse().ok()?
    };
    let cents_str = parts[1];

    // Pad or truncate cents to 2 digits
    let cents: i64 = if cents_str.len() == 1 {
        cents_str.parse::<i64>().ok()? * 10
    } else if cents_str.len() == 2 {
        cents_str.parse().ok()?
    } else {
        // More than 2 decimal places — take first 2
        cents_str[..2].parse().ok()?
    };

    if dollars == 0 && cents == 0 {
        return Some(format!("zero {}", currency.plural));
    }

    if dollars == 0 {
        let cents_words = number_to_words(cents);
        let unit = if cents == 1 {
            currency.cent_singular
        } else {
            currency.cent_plural
        };
        return Some(format!("{} {}", cents_words, unit));
    }

    if cents == 0 {
        let dollar_words = number_to_words(dollars);
        let unit = if dollars == 1 {
            currency.singular
        } else {
            currency.plural
        };
        return Some(format!("{} {}", dollar_words, unit));
    }

    let dollar_words = number_to_words(dollars);
    let cents_words = number_to_words(cents);
    let dollar_unit = if dollars == 1 {
        currency.singular
    } else {
        currency.plural
    };
    let cent_unit = if cents == 1 {
        currency.cent_singular
    } else {
        currency.cent_plural
    };

    Some(format!(
        "{} {} and {} {}",
        dollar_words, dollar_unit, cents_words, cent_unit
    ))
}

/// Extract scale suffix from the amount string.
fn extract_scale(input: &str) -> (&str, Option<&str>) {
    for &scale in SCALE_SUFFIXES {
        if let Some(before) = input.strip_suffix(scale) {
            let before = before.trim_end();
            if !before.is_empty()
                && before
                    .chars()
                    .all(|c| c.is_ascii_digit() || c == '.' || c == ',')
            {
                return (before, Some(scale));
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
        assert_eq!(
            parse("$5.50"),
            Some("five dollars and fifty cents".to_string())
        );
        assert_eq!(
            parse("$1.01"),
            Some("one dollar and one cent".to_string())
        );
        assert_eq!(parse("$0.01"), Some("one cent".to_string()));
        assert_eq!(parse("$0.99"), Some("ninety nine cents".to_string()));
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
    }

    #[test]
    fn test_other_currencies() {
        assert_eq!(parse("€100"), Some("one hundred euros".to_string()));
        assert_eq!(parse("£1"), Some("one pound".to_string()));
        assert_eq!(parse("¥500"), Some("five hundred yen".to_string()));
    }

    #[test]
    fn test_non_money() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("123"), None);
        assert_eq!(parse("$"), None);
    }
}
