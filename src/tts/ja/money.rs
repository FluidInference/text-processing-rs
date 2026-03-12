//! Money TN tagger for Japanese (romaji output).
//!
//! Converts written currency expressions to spoken Japanese in romaji:
//! - "¥100" → "hyaku en"
//! - "¥1500" → "sen go hyaku en"
//! - "$5.50" → "go doru go juu sento"
//! - "€100" → "hyaku yuuro"

use super::number_to_words;

/// Scale suffixes recognized after a currency amount.
/// oku (億) = hundred million, man (万) = ten thousand
const SCALE_SUFFIXES: &[&str] = &["oku", "man"];

/// Japanese has no singular/plural distinction, so we use a single name per currency.
struct Currency {
    name: &'static str,
    cent_name: &'static str,
}

const YEN: Currency = Currency {
    name: "en",
    cent_name: "",
};

const DOLLAR: Currency = Currency {
    name: "doru",
    cent_name: "sento",
};

const EURO: Currency = Currency {
    name: "yuuro",
    cent_name: "sento",
};

const POUND: Currency = Currency {
    name: "pondo",
    cent_name: "pensu",
};

/// Parse a written money expression to spoken Japanese in romaji.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Try suffix symbol: "100円", "100 円"
    if let Some(result) = parse_suffix_currency(trimmed) {
        return Some(result);
    }

    // Try prefix symbol: "¥100", "$5.50", "€100", "£1"
    if let Some(result) = parse_prefix_currency(trimmed) {
        return Some(result);
    }

    None
}

fn parse_suffix_currency(input: &str) -> Option<String> {
    // Handle 円 suffix
    let amount_str = input.strip_suffix('\u{5186}')?; // 円
    let amount_str = amount_str.trim();
    parse_amount(amount_str, &YEN)
}

fn parse_prefix_currency(input: &str) -> Option<String> {
    let (currency, rest) = if let Some(r) = input.strip_prefix('\u{00A5}') {
        // ¥
        (&YEN, r)
    } else if let Some(r) = input.strip_prefix('$') {
        (&DOLLAR, r)
    } else if let Some(r) = input.strip_prefix('\u{20AC}') {
        // €
        (&EURO, r)
    } else if let Some(r) = input.strip_prefix('\u{00A3}') {
        // £
        (&POUND, r)
    } else {
        return None;
    };

    let rest = rest.trim();
    if rest.is_empty() {
        return None;
    }

    // Check for scale suffix: "¥2.5 man" (2.5万円)
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
        // With scale: "¥2.5 man" → "ni ten go man en"
        let sep = if amount_str.contains('.') {
            '.'
        } else if amount_str.contains(',') {
            ','
        } else {
            // No decimal: "¥50 man" → "go juu man en"
            let clean: String = amount_str.chars().filter(|c| c.is_ascii_digit()).collect();
            let n: i64 = clean.parse().ok()?;
            let words = number_to_words(n);
            return Some(format!("{} {} {}", words, scale_word, currency.name));
        };

        let parts: Vec<&str> = amount_str.splitn(2, sep).collect();
        if parts.len() == 2 {
            let int_val: i64 = parts[0].parse().ok()?;
            let int_words = number_to_words(int_val);
            let frac_words = super::spell_digits(parts[1]);
            return Some(format!(
                "{} ten {} {} {}",
                int_words, frac_words, scale_word, currency.name
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

    // Determine decimal separator
    let sep = if amount_str.contains('.') {
        '.'
    } else if amount_str.contains(',') {
        ','
    } else {
        // No decimal — whole amount only
        let clean: String = amount_str.chars().filter(|c| c.is_ascii_digit()).collect();
        let n: i64 = clean.parse().ok()?;
        return Some(format_currency(n, 0, currency));
    };

    let parts: Vec<&str> = amount_str.splitn(2, sep).collect();
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

    None
}

fn format_currency(main_amount: i64, cents: i64, currency: &Currency) -> String {
    let main_words = number_to_words(main_amount);

    // Yen has no sub-unit in modern usage
    if currency.cent_name.is_empty() {
        if main_amount == 0 {
            return format!("zero {}", currency.name);
        }
        return format!("{} {}", main_words, currency.name);
    }

    if main_amount == 0 && cents == 0 {
        return format!("zero {}", currency.name);
    }

    if main_amount == 0 {
        let cents_words = number_to_words(cents);
        return format!("{} {}", cents_words, currency.cent_name);
    }

    if cents == 0 {
        return format!("{} {}", main_words, currency.name);
    }

    let cents_words = number_to_words(cents);
    format!(
        "{} {} {} {}",
        main_words, currency.name, cents_words, currency.cent_name
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yen_prefix() {
        assert_eq!(parse("\u{00A5}100"), Some("hyaku en".to_string()));
        assert_eq!(parse("\u{00A5}1500"), Some("sen go hyaku en".to_string()));
        assert_eq!(parse("\u{00A5}1"), Some("ichi en".to_string()));
        assert_eq!(
            parse("\u{00A5}10000"),
            Some("ichi man en".to_string())
        );
    }

    #[test]
    fn test_yen_suffix() {
        assert_eq!(parse("100\u{5186}"), Some("hyaku en".to_string()));
        assert_eq!(parse("500\u{5186}"), Some("go hyaku en".to_string()));
    }

    #[test]
    fn test_dollar() {
        assert_eq!(parse("$100"), Some("hyaku doru".to_string()));
        assert_eq!(
            parse("$5.50"),
            Some("go doru go juu sento".to_string())
        );
        assert_eq!(parse("$1"), Some("ichi doru".to_string()));
    }

    #[test]
    fn test_euro() {
        assert_eq!(parse("\u{20AC}100"), Some("hyaku yuuro".to_string()));
        assert_eq!(
            parse("\u{20AC}2.50"),
            Some("ni yuuro go juu sento".to_string())
        );
    }

    #[test]
    fn test_pound() {
        assert_eq!(parse("\u{00A3}1"), Some("ichi pondo".to_string()));
        assert_eq!(
            parse("\u{00A3}3.99"),
            Some("san pondo kyuu juu kyuu pensu".to_string())
        );
    }

    #[test]
    fn test_dollars_and_cents() {
        assert_eq!(
            parse("$1.01"),
            Some("ichi doru ichi sento".to_string())
        );
        assert_eq!(
            parse("$0.99"),
            Some("kyuu juu kyuu sento".to_string())
        );
    }

    #[test]
    fn test_large_amounts() {
        assert_eq!(
            parse("\u{00A5}2.5 man"),
            Some("ni ten go man en".to_string())
        );
        assert_eq!(
            parse("$50 oku"),
            Some("go juu oku doru".to_string())
        );
    }

    #[test]
    fn test_trailing_dot() {
        assert_eq!(parse("$5."), Some("go doru".to_string()));
        assert_eq!(parse("$1."), Some("ichi doru".to_string()));
    }

    #[test]
    fn test_non_money() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("123"), None);
        assert_eq!(parse(""), None);
    }
}
