//! Money TN tagger for French.
//!
//! Converts written currency expressions to spoken French:
//! - "5,50 €" → "cinq euros et cinquante centimes"
//! - "€5.50" → "cinq euros et cinquante centimes"
//! - "$100" → "cent dollars"
//! - "£1" → "une livre"

use super::number_to_words;

struct Currency {
    singular: &'static str,
    plural: &'static str,
    cent_singular: &'static str,
    cent_plural: &'static str,
    /// Whether "un" becomes "une" for this currency
    feminine: bool,
}

const EURO: Currency = Currency {
    singular: "euro",
    plural: "euros",
    cent_singular: "centime",
    cent_plural: "centimes",
    feminine: false,
};

const DOLLAR: Currency = Currency {
    singular: "dollar",
    plural: "dollars",
    cent_singular: "cent",
    cent_plural: "cents",
    feminine: false,
};

const POUND: Currency = Currency {
    singular: "livre",
    plural: "livres",
    cent_singular: "penny",
    cent_plural: "pence",
    feminine: true,
};

const YEN: Currency = Currency {
    singular: "yen",
    plural: "yens",
    cent_singular: "sen",
    cent_plural: "sen",
    feminine: false,
};

/// Scale suffixes recognized after a currency amount.
const SCALE_SUFFIXES: &[&str] = &["billiard", "billion", "milliards", "milliard", "millions", "million", "mille"];

/// Parse a written money expression to spoken French.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Try suffix symbol: "5,50 €", "100 €"
    if let Some(result) = parse_suffix_currency(trimmed) {
        return Some(result);
    }

    // Try prefix symbol: "$5.50", "€100", "£1"
    if let Some(result) = parse_prefix_currency(trimmed) {
        return Some(result);
    }

    None
}

fn parse_suffix_currency(input: &str) -> Option<String> {
    let (amount_str, currency) = if let Some(s) = input.strip_suffix('€') {
        (s.trim(), &EURO)
    } else if let Some(s) = input.strip_suffix("EUR") {
        (s.trim(), &EURO)
    } else {
        return None;
    };

    parse_amount(amount_str, currency)
}

fn parse_prefix_currency(input: &str) -> Option<String> {
    let (currency, rest) = if let Some(r) = input.strip_prefix('$') {
        (&DOLLAR, r)
    } else if let Some(r) = input.strip_prefix('€') {
        (&EURO, r)
    } else if let Some(r) = input.strip_prefix('£') {
        (&POUND, r)
    } else if let Some(r) = input.strip_prefix('¥') {
        (&YEN, r)
    } else {
        return None;
    };

    let rest = rest.trim();
    if rest.is_empty() {
        return None;
    }

    // Check for scale suffix: "$2,5 milliards"
    let (amount_str, scale) = extract_scale(rest);

    // Without a scale suffix, the amount must be purely numeric
    if scale.is_none()
        && !amount_str
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == ',' || c == ' ')
    {
        return None;
    }

    if let Some(scale_word) = scale {
        // With scale: "$2,5 milliards" → "deux virgule cinq milliards de dollars"
        let decimal_sep = if amount_str.contains(',') {
            ','
        } else if amount_str.contains('.') {
            '.'
        } else {
            // No decimal: "$50 millions" → "cinquante millions de dollars"
            let clean: String = amount_str.chars().filter(|c| c.is_ascii_digit()).collect();
            let n: i64 = clean.parse().ok()?;
            let words = number_to_words(n);
            return Some(format!("{} {} de {}", words, scale_word, currency.plural));
        };

        let parts: Vec<&str> = amount_str.splitn(2, decimal_sep).collect();
        if parts.len() == 2 {
            let int_val: i64 = parts[0].parse().ok()?;
            let int_words = number_to_words(int_val);
            let frac_words = super::spell_digits(parts[1]);
            return Some(format!(
                "{} virgule {} {} de {}",
                int_words, frac_words, scale_word, currency.plural
            ));
        }
    }

    parse_amount(amount_str, currency)
}

/// Extract scale suffix from the amount string.
fn extract_scale(input: &str) -> (&str, Option<&str>) {
    for &scale in SCALE_SUFFIXES {
        if let Some(before) = input.strip_suffix(scale) {
            let before = before.trim_end();
            if !before.is_empty()
                && before
                    .chars()
                    .all(|c| c.is_ascii_digit() || c == '.' || c == ',' || c == ' ')
            {
                return (before, Some(scale));
            }
        }
    }
    (input, None)
}

fn parse_amount(amount_str: &str, currency: &Currency) -> Option<String> {
    if amount_str.is_empty() {
        return None;
    }

    // Determine decimal separator: French uses comma
    let sep = if amount_str.contains(',') { ',' } else { '.' };

    if amount_str.contains(sep) && sep != '.' || amount_str.contains('.') {
        let actual_sep = if amount_str.contains(',') { ',' } else { '.' };
        let parts: Vec<&str> = amount_str.splitn(2, actual_sep).collect();
        if parts.len() == 2 {
            let int_clean: String = parts[0].chars().filter(|c| c.is_ascii_digit()).collect();
            let dollars: i64 = if int_clean.is_empty() {
                0
            } else {
                int_clean.parse().ok()?
            };

            let cents_str = parts[1].trim();
            let cents: i64 = if cents_str.is_empty() {
                0
            } else if cents_str.len() == 1 {
                cents_str.parse::<i64>().ok()? * 10
            } else if cents_str.len() == 2 {
                cents_str.parse().ok()?
            } else {
                cents_str[..2].parse().ok()?
            };

            return Some(format_currency(dollars, cents, currency));
        }
    }

    let clean: String = amount_str.chars().filter(|c| c.is_ascii_digit()).collect();
    let n: i64 = clean.parse().ok()?;
    Some(format_currency(n, 0, currency))
}

fn format_currency(dollars: i64, cents: i64, currency: &Currency) -> String {
    let dollar_words = if dollars == 1 && currency.feminine {
        "une".to_string()
    } else {
        number_to_words(dollars)
    };

    if dollars == 0 && cents == 0 {
        return format!("zero {}", currency.plural);
    }

    if dollars == 0 {
        let cents_words = number_to_words(cents);
        let unit = if cents == 1 {
            currency.cent_singular
        } else {
            currency.cent_plural
        };
        return format!("{} {}", cents_words, unit);
    }

    if cents == 0 {
        let unit = if dollars == 1 {
            currency.singular
        } else {
            currency.plural
        };
        return format!("{} {}", dollar_words, unit);
    }

    let dollar_unit = if dollars == 1 {
        currency.singular
    } else {
        currency.plural
    };
    let cents_words = number_to_words(cents);
    let cent_unit = if cents == 1 {
        currency.cent_singular
    } else {
        currency.cent_plural
    };

    format!(
        "{} {} et {} {}",
        dollar_words, dollar_unit, cents_words, cent_unit
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_euro_suffix() {
        assert_eq!(parse("5 €"), Some("cinq euros".to_string()));
        assert_eq!(parse("1 €"), Some("un euro".to_string()));
        assert_eq!(
            parse("5,50 €"),
            Some("cinq euros et cinquante centimes".to_string())
        );
    }

    #[test]
    fn test_prefix_currencies() {
        assert_eq!(parse("$100"), Some("cent dollars".to_string()));
        assert_eq!(parse("£1"), Some("une livre".to_string()));
        assert_eq!(parse("€100"), Some("cent euros".to_string()));
    }

    #[test]
    fn test_dollars_and_cents() {
        assert_eq!(
            parse("$5.50"),
            Some("cinq dollars et cinquante cents".to_string())
        );
        assert_eq!(parse("$1.01"), Some("un dollar et un cent".to_string()));
        assert_eq!(parse("$0.99"), Some("quatre-vingt-dix-neuf cents".to_string()));
    }

    #[test]
    fn test_large_amounts() {
        assert_eq!(
            parse("$2,5 milliards"),
            Some("deux virgule cinq milliards de dollars".to_string())
        );
        assert_eq!(
            parse("$50 millions"),
            Some("cinquante millions de dollars".to_string())
        );
    }

    #[test]
    fn test_trailing_dot() {
        assert_eq!(parse("$5."), Some("cinq dollars".to_string()));
        assert_eq!(parse("$1."), Some("un dollar".to_string()));
    }

    #[test]
    fn test_non_money() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("123"), None);
    }
}
