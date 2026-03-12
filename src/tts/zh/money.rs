//! Money TN tagger for Mandarin Chinese.
//!
//! Converts written currency expressions to spoken Mandarin pinyin:
//! - "¥100" -> "yi bai yuan"
//! - "¥5.50" -> "wu yuan wu jiao"
//! - "$100" -> "yi bai meiyuan"
//! - "€50" -> "wu shi ouyuan"
//! - "£20" -> "er shi yingbang"

use super::number_to_words;

/// Scale suffixes recognized after a currency amount.
/// yi = 亿 (hundred million), wan = 万 (ten thousand)
const SCALE_SUFFIXES: &[&str] = &["yi", "wan"];

struct Currency {
    /// Main unit name in pinyin
    unit: &'static str,
    /// Sub-unit at 0.1 level (jiao for RMB)
    sub_unit_tenth: Option<&'static str>,
    /// Sub-unit at 0.01 level (fen for RMB)
    _sub_unit_hundredth: Option<&'static str>,
}

const RMB: Currency = Currency {
    unit: "yuan",
    sub_unit_tenth: Some("jiao"),
    _sub_unit_hundredth: Some("fen"),
};

const DOLLAR: Currency = Currency {
    unit: "meiyuan",
    sub_unit_tenth: None,
    _sub_unit_hundredth: None,
};

const EURO: Currency = Currency {
    unit: "ouyuan",
    sub_unit_tenth: None,
    _sub_unit_hundredth: None,
};

const POUND: Currency = Currency {
    unit: "yingbang",
    sub_unit_tenth: None,
    _sub_unit_hundredth: None,
};

/// Parse a written money expression to spoken Mandarin pinyin.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Try prefix symbol: ¥100, $50, €20, £10
    if let Some(result) = parse_prefix_currency(trimmed) {
        return Some(result);
    }

    // Try suffix with Chinese character: 100元
    if let Some(result) = parse_chinese_suffix(trimmed) {
        return Some(result);
    }

    None
}

fn parse_prefix_currency(input: &str) -> Option<String> {
    let (currency, rest) = if let Some(r) = input.strip_prefix('\u{00A5}') {
        // ¥
        (&RMB, r)
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

    // Check for scale suffix: "$2.5 yi" (2.5 hundred million dollars)
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
        // With scale: "$2.5 yi" → "er dian wu yi meiyuan"
        if amount_str.contains('.') {
            let parts: Vec<&str> = amount_str.splitn(2, '.').collect();
            if parts.len() == 2 {
                let int_val: i64 = parts[0].parse().ok()?;
                let int_words = number_to_words(int_val);
                let frac_words = super::spell_digits(parts[1]);
                return Some(format!(
                    "{} dian {} {} {}",
                    int_words, frac_words, scale_word, currency.unit
                ));
            }
        } else {
            // No decimal: "$50 yi" → "wu shi yi meiyuan"
            let clean: String = amount_str.chars().filter(|c| c.is_ascii_digit()).collect();
            let n: i64 = clean.parse().ok()?;
            let words = number_to_words(n);
            return Some(format!("{} {} {}", words, scale_word, currency.unit));
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

fn parse_chinese_suffix(input: &str) -> Option<String> {
    // Handle 100元 format
    let amount_str = input.strip_suffix('\u{5143}')?; // 元
    let amount_str = amount_str.trim();

    if amount_str.is_empty() {
        return None;
    }

    if !amount_str.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return None;
    }

    parse_amount(amount_str, &RMB)
}

fn parse_amount(amount_str: &str, currency: &Currency) -> Option<String> {
    if amount_str.is_empty() {
        return None;
    }

    if amount_str.contains('.') {
        let parts: Vec<&str> = amount_str.splitn(2, '.').collect();
        if parts.len() == 2 {
            let int_clean: String = parts[0].chars().filter(|c| c.is_ascii_digit()).collect();
            let main_val: i64 = if int_clean.is_empty() {
                0
            } else {
                int_clean.parse().ok()?
            };

            let frac_str = parts[1].trim();

            // For RMB, handle jiao and fen
            if currency.sub_unit_tenth.is_some() {
                let jiao: i64;
                let fen: i64;
                if frac_str.len() >= 2 {
                    jiao = frac_str[..1].parse().ok()?;
                    fen = frac_str[1..2].parse().ok()?;
                } else if frac_str.len() == 1 {
                    jiao = frac_str.parse().ok()?;
                    fen = 0;
                } else {
                    jiao = 0;
                    fen = 0;
                }
                return Some(format_rmb(main_val, jiao, fen));
            }

            // For foreign currencies, say "N dian M M unit"
            if main_val == 0 && frac_str == "0" {
                return Some(format!("ling {}", currency.unit));
            }
            let cents: i64 = if frac_str.is_empty() {
                0
            } else if frac_str.len() == 1 {
                frac_str.parse::<i64>().ok()? * 10
            } else if frac_str.len() == 2 {
                frac_str.parse().ok()?
            } else {
                frac_str[..2].parse().ok()?
            };

            if cents == 0 {
                return Some(format!("{} {}", number_to_words(main_val), currency.unit));
            }

            return Some(format!(
                "{} dian {} {}",
                number_to_words(main_val),
                super::spell_digits(&format!("{:02}", cents)),
                currency.unit
            ));
        }
    }

    let clean: String = amount_str.chars().filter(|c| c.is_ascii_digit()).collect();
    let n: i64 = clean.parse().ok()?;
    Some(format!("{} {}", number_to_words(n), currency.unit))
}

fn format_rmb(yuan: i64, jiao: i64, fen: i64) -> String {
    let mut parts: Vec<String> = Vec::new();

    if yuan > 0 {
        parts.push(format!("{} yuan", number_to_words(yuan)));
    }

    if jiao > 0 {
        parts.push(format!("{} jiao", number_to_words(jiao)));
    }

    if fen > 0 {
        parts.push(format!("{} fen", number_to_words(fen)));
    }

    if parts.is_empty() {
        return "ling yuan".to_string();
    }

    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rmb() {
        assert_eq!(parse("\u{00A5}100"), Some("yi bai yuan".to_string()));
        assert_eq!(parse("\u{00A5}5.50"), Some("wu yuan wu jiao".to_string()));
        assert_eq!(
            parse("\u{00A5}3.25"),
            Some("san yuan er jiao wu fen".to_string())
        );
        assert_eq!(parse("\u{00A5}1"), Some("yi yuan".to_string()));
    }

    #[test]
    fn test_foreign_currencies() {
        assert_eq!(parse("$100"), Some("yi bai meiyuan".to_string()));
        assert_eq!(parse("\u{20AC}50"), Some("wu shi ouyuan".to_string()));
        assert_eq!(parse("\u{00A3}20"), Some("er shi yingbang".to_string()));
    }

    #[test]
    fn test_chinese_suffix() {
        assert_eq!(parse("100\u{5143}"), Some("yi bai yuan".to_string()));
    }

    #[test]
    fn test_dollars_and_cents() {
        assert_eq!(parse("$5.50"), Some("wu dian wu ling meiyuan".to_string()));
        assert_eq!(parse("$1.01"), Some("yi dian ling yi meiyuan".to_string()));
        assert_eq!(
            parse("$0.99"),
            Some("ling dian jiu jiu meiyuan".to_string())
        );
    }

    #[test]
    fn test_large_amounts() {
        assert_eq!(parse("$2.5 yi"), Some("er dian wu yi meiyuan".to_string()));
        assert_eq!(parse("$50 wan"), Some("wu shi wan meiyuan".to_string()));
    }

    #[test]
    fn test_trailing_dot() {
        assert_eq!(parse("$5."), Some("wu meiyuan".to_string()));
        assert_eq!(parse("$1."), Some("yi meiyuan".to_string()));
    }

    #[test]
    fn test_non_money() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("123"), None);
        assert_eq!(parse(""), None);
    }
}
