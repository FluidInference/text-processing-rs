//! Money TN tagger for Hindi (romanized).
//!
//! Converts written currency expressions to spoken romanized Hindi:
//! - "₹100" → "ek sau rupaye"
//! - "₹5.50" → "paanch rupaye aur pachaas paise"
//! - "$100" → "ek sau dollar"
//! - "€1" → "ek euro"

use super::number_to_words;

/// Scale suffixes recognized after a currency amount.
/// crore, lakh, hazaar (thousand)
const SCALE_SUFFIXES: &[&str] = &["crore", "lakh", "hazaar"];

struct Currency {
    singular: &'static str,
    plural: &'static str,
    cent_singular: &'static str,
    cent_plural: &'static str,
}

const RUPEE: Currency = Currency {
    singular: "rupaya",
    plural: "rupaye",
    cent_singular: "paisa",
    cent_plural: "paise",
};

const DOLLAR: Currency = Currency {
    singular: "dollar",
    plural: "dollar",
    cent_singular: "cent",
    cent_plural: "cents",
};

const EURO: Currency = Currency {
    singular: "euro",
    plural: "euro",
    cent_singular: "cent",
    cent_plural: "cents",
};

const POUND: Currency = Currency {
    singular: "pound",
    plural: "pound",
    cent_singular: "penny",
    cent_plural: "pence",
};

const YEN: Currency = Currency {
    singular: "yen",
    plural: "yen",
    cent_singular: "sen",
    cent_plural: "sen",
};

/// Parse a written money expression to spoken romanized Hindi.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Try suffix symbol: "100 ₹"
    if let Some(result) = parse_suffix_currency(trimmed) {
        return Some(result);
    }

    // Try prefix symbol: "₹100", "$5.50", "€100"
    if let Some(result) = parse_prefix_currency(trimmed) {
        return Some(result);
    }

    None
}

fn parse_suffix_currency(input: &str) -> Option<String> {
    let (amount_str, currency) = if let Some(s) = input.strip_suffix('\u{20B9}') {
        // ₹
        (s.trim(), &RUPEE)
    } else if let Some(s) = input.strip_suffix("INR") {
        (s.trim(), &RUPEE)
    } else if let Some(s) = input.strip_suffix('\u{20AC}') {
        // Euro sign
        (s.trim(), &EURO)
    } else if let Some(s) = input.strip_suffix("EUR") {
        (s.trim(), &EURO)
    } else {
        return None;
    };

    parse_amount(amount_str, currency)
}

fn parse_prefix_currency(input: &str) -> Option<String> {
    let (currency, rest) = if let Some(r) = input.strip_prefix('\u{20B9}') {
        // ₹
        (&RUPEE, r)
    } else if let Some(r) = input.strip_prefix("Rs") {
        // Rs or Rs.
        let r = r.strip_prefix('.').unwrap_or(r);
        (&RUPEE, r)
    } else if let Some(r) = input.strip_prefix('$') {
        (&DOLLAR, r)
    } else if let Some(r) = input.strip_prefix('\u{20AC}') {
        (&EURO, r)
    } else if let Some(r) = input.strip_prefix('\u{00A3}') {
        // £
        (&POUND, r)
    } else if let Some(r) = input.strip_prefix('\u{00A5}') {
        // ¥
        (&YEN, r)
    } else {
        return None;
    };

    let rest = rest.trim();
    if rest.is_empty() {
        return None;
    }

    // Check for scale suffix: "₹2.5 lakh" (2.5 lakh rupees)
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
        // With scale: "₹2.5 lakh" → "do dashmlav paanch lakh rupaye"
        if amount_str.contains('.') {
            let parts: Vec<&str> = amount_str.splitn(2, '.').collect();
            if parts.len() == 2 {
                let int_val: i64 = parts[0].parse().ok()?;
                let int_words = number_to_words(int_val);
                let frac_words = super::spell_digits(parts[1]);
                return Some(format!(
                    "{} dashmlav {} {} {}",
                    int_words, frac_words, scale_word, currency.plural
                ));
            }
        } else {
            // No decimal: "₹50 lakh" → "pachaas lakh rupaye"
            let clean: String = amount_str.chars().filter(|c| c.is_ascii_digit()).collect();
            let n: i64 = clean.parse().ok()?;
            let words = number_to_words(n);
            return Some(format!("{} {} {}", words, scale_word, currency.plural));
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

    // Check for decimal part
    let decimal_sep = '.';
    if amount_str.contains(decimal_sep) {
        let parts: Vec<&str> = amount_str.splitn(2, decimal_sep).collect();
        if parts.len() == 2 {
            let int_clean: String = parts[0].chars().filter(|c| c.is_ascii_digit()).collect();
            let main_amount: i64 = if int_clean.is_empty() {
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

            return Some(format_currency(main_amount, cents, currency));
        }
    }

    let clean: String = amount_str.chars().filter(|c| c.is_ascii_digit()).collect();
    let n: i64 = clean.parse().ok()?;
    Some(format_currency(n, 0, currency))
}

fn format_currency(main_amount: i64, cents: i64, currency: &Currency) -> String {
    let main_words = number_to_words(main_amount);

    if main_amount == 0 && cents == 0 {
        return format!("shunya {}", currency.plural);
    }

    if main_amount == 0 {
        let cents_words = number_to_words(cents);
        let unit = if cents == 1 {
            currency.cent_singular
        } else {
            currency.cent_plural
        };
        return format!("{} {}", cents_words, unit);
    }

    if cents == 0 {
        let unit = if main_amount == 1 {
            currency.singular
        } else {
            currency.plural
        };
        return format!("{} {}", main_words, unit);
    }

    let main_unit = if main_amount == 1 {
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
        "{} {} aur {} {}",
        main_words, main_unit, cents_words, cent_unit
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rupee_prefix() {
        assert_eq!(parse("\u{20B9}100"), Some("ek sau rupaye".to_string()));
        assert_eq!(parse("\u{20B9}1"), Some("ek rupaya".to_string()));
        assert_eq!(
            parse("\u{20B9}5.50"),
            Some("paanch rupaye aur pachaas paise".to_string())
        );
    }

    #[test]
    fn test_dollar_prefix() {
        assert_eq!(parse("$100"), Some("ek sau dollar".to_string()));
        assert_eq!(
            parse("$5.50"),
            Some("paanch dollar aur pachaas cents".to_string())
        );
    }

    #[test]
    fn test_euro_prefix() {
        assert_eq!(parse("\u{20AC}100"), Some("ek sau euro".to_string()));
    }

    #[test]
    fn test_dollars_and_cents() {
        assert_eq!(parse("$1.01"), Some("ek dollar aur ek cent".to_string()));
        assert_eq!(parse("$0.99"), Some("ninyaanbe cents".to_string()));
    }

    #[test]
    fn test_large_amounts() {
        assert_eq!(
            parse("\u{20B9}2.5 lakh"),
            Some("do dashmlav paanch lakh rupaye".to_string())
        );
        assert_eq!(parse("$50 crore"), Some("pachaas crore dollar".to_string()));
    }

    #[test]
    fn test_trailing_dot() {
        assert_eq!(parse("$5."), Some("paanch dollar".to_string()));
        assert_eq!(parse("$1."), Some("ek dollar".to_string()));
    }

    #[test]
    fn test_non_money() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("123"), None);
        assert_eq!(parse(""), None);
    }
}
