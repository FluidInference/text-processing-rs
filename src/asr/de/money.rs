//! Money tagger for German.
//!
//! Converts spoken German currency expressions to written form:
//! - "zwei dollar" → "$2"
//! - "zwei euro und zwanzig cent" → "€2,20"
//! - "zwei pfund und ein penny" → "£2,01"
//! - "zwei millionen euro" → "€2 millionen"

use super::cardinal;

struct Currency {
    names: &'static [&'static str],
    symbol: &'static str,
    prefix: bool, // true = $X, false = X €
    cent_names: &'static [&'static str],
    cent_singular: &'static str,
}

const CURRENCIES: &[Currency] = &[
    Currency {
        names: &["dollar", "dollars"],
        symbol: "$",
        prefix: true,
        cent_names: &["cent", "cents"],
        cent_singular: "cent",
    },
    Currency {
        names: &["euro", "euros"],
        symbol: "€",
        prefix: false,
        cent_names: &["cent", "cents"],
        cent_singular: "cent",
    },
    Currency {
        names: &["pfund"],
        symbol: "£",
        prefix: false,
        cent_names: &["pence", "penny"],
        cent_singular: "penny",
    },
];

/// Parse spoken German money expression to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trim = input_lower.trim();

    // Reject compound words without spaces (e.g., "zweidollarzwanzig")
    if !input_trim.contains(' ') {
        // Allow single-word currency amounts that are just "X dollar" etc.
        // Actually for no-space cases like "zweidollarzwanzig", reject
        for cur in CURRENCIES {
            for &name in cur.names {
                if input_trim.contains(name) && input_trim != name {
                    return None;
                }
            }
        }
        return None;
    }

    // Try scale patterns first: "zwei millionen euro" → "€2 millionen"
    if let Some(result) = parse_scale_money(input_trim) {
        return Some(result);
    }

    // Try decimal money: "zwei komma null null dollar" → "$2,00"
    if let Some(result) = parse_decimal_money(input_trim) {
        return Some(result);
    }

    // Try "X CURRENCY und Y SUBCURRENCY" pattern
    if let Some(result) = parse_with_subcurrency(input_trim) {
        return Some(result);
    }

    // Try "X CURRENCY Y" (implied cents)
    if let Some(result) = parse_implied_cents(input_trim) {
        return Some(result);
    }

    // Try simple "X CURRENCY"
    if let Some(result) = parse_simple_money(input_trim) {
        return Some(result);
    }

    // Try cent-only: "ein cent" → "€0,01"
    if let Some(result) = parse_cents_only(input_trim) {
        return Some(result);
    }

    None
}

/// Parse scale money: "zwei millionen euro" → "€2 millionen"
fn parse_scale_money(input: &str) -> Option<String> {
    let scale_words = ["millionen", "million", "milliarden", "milliarde",
                       "billionen", "billion"];

    for cur in CURRENCIES {
        for &cur_name in cur.names {
            if input.ends_with(cur_name) {
                let before = input[..input.len() - cur_name.len()].trim();
                // Check if before contains a scale word
                for &sw in &scale_words {
                    if before.ends_with(sw) {
                        let num_part = before[..before.len() - sw.len()].trim();

                        // Check for "komma" (decimal scale money)
                        if num_part.contains("komma") {
                            let parts: Vec<&str> = num_part.splitn(2, "komma").collect();
                            if parts.len() == 2 {
                                let int_part = parts[0].trim();
                                let dec_part = parts[1].trim();
                                let int_val = cardinal::words_to_number(int_part)?;
                                let dec_digits = parse_decimal_digits(dec_part)?;
                                return Some(format!("{}{}",
                                    format_with_symbol(cur, &format!("{},{} {}", int_val, dec_digits, sw)),
                                    ""
                                ));
                            }
                        }

                        let num = cardinal::words_to_number(num_part)?;
                        return Some(format_with_symbol(cur, &format!("{} {}", num, sw)));
                    }
                }
            }
        }
    }

    None
}

/// Parse decimal money: "zwei komma null null dollar" → "$2,00"
fn parse_decimal_money(input: &str) -> Option<String> {
    if !input.contains("komma") {
        return None;
    }

    for cur in CURRENCIES {
        for &cur_name in cur.names {
            if input.ends_with(cur_name) {
                let before = input[..input.len() - cur_name.len()].trim();
                let parts: Vec<&str> = before.splitn(2, "komma").collect();
                if parts.len() != 2 {
                    continue;
                }
                let int_part = parts[0].trim();
                let dec_part = parts[1].trim();

                let int_val = cardinal::words_to_number(int_part)?;
                let dec_digits = parse_decimal_digits(dec_part)?;

                return Some(format_with_symbol(cur, &format!("{},{}", int_val, dec_digits)));
            }
        }
    }

    None
}

/// Parse with subcurrency: "zwei euro und zwanzig cent" → "€2,20"
fn parse_with_subcurrency(input: &str) -> Option<String> {
    for cur in CURRENCIES {
        for &cent_name in cur.cent_names {
            // "X CURRENCY und Y SUBCURRENCY"
            if input.ends_with(cent_name) {
                let before_cent = input[..input.len() - cent_name.len()].trim();
                // Check for "und" separator
                if let Some(und_pos) = before_cent.rfind(" und ") {
                    let cent_part = before_cent[und_pos + 5..].trim();
                    let main_part = before_cent[..und_pos].trim();

                    // Parse cent amount
                    let cent_val = cardinal::words_to_number(cent_part)?;

                    // Check if cents >= 100 (special case)
                    if cent_val >= 100 {
                        // "zwei pfund und ein hundert penny" → "£2 und 100 penny"
                        for &cur_name in cur.names {
                            if main_part.ends_with(cur_name) {
                                let num_part = main_part[..main_part.len() - cur_name.len()].trim();
                                let main_val = cardinal::words_to_number(num_part)?;
                                return Some(format_with_symbol(cur, &format!("{} und {} {}", main_val, cent_val, cent_name)));
                            }
                        }
                        continue;
                    }

                    // Find main currency
                    for &cur_name in cur.names {
                        if main_part.ends_with(cur_name) {
                            let num_part = main_part[..main_part.len() - cur_name.len()].trim();
                            let main_val = cardinal::words_to_number(num_part)?;
                            return Some(format_with_symbol(cur, &format!("{},{:02}", main_val, cent_val)));
                        }
                    }
                }

                // "X CURRENCY Y SUBCURRENCY" (without "und")
                for &cur_name in cur.names {
                    let pattern = format!("{} ", cur_name);
                    if let Some(pos) = before_cent.find(&pattern) {
                        let num_part = before_cent[..pos].trim();
                        let cent_str = before_cent[pos + pattern.len()..].trim();

                        let main_val = cardinal::words_to_number(num_part)?;
                        let cent_val = cardinal::words_to_number(cent_str)?;

                        return Some(format_with_symbol(cur, &format!("{},{:02}", main_val, cent_val)));
                    }
                }
            }
        }
    }

    None
}

/// Parse implied cents: "zwei dollar zwanzig" → "$2,20"
fn parse_implied_cents(input: &str) -> Option<String> {
    for cur in CURRENCIES {
        for &cur_name in cur.names {
            let pattern = format!(" {} ", cur_name);
            if let Some(pos) = input.find(&pattern) {
                let num_part = &input[..pos];
                let cent_part = &input[pos + pattern.len()..];

                let main_val = cardinal::words_to_number(num_part)?;
                let cent_val = cardinal::words_to_number(cent_part)?;

                return Some(format_with_symbol(cur, &format!("{},{:02}", main_val, cent_val)));
            }
        }
    }

    None
}

/// Parse simple money: "zwei dollar" → "$2"
fn parse_simple_money(input: &str) -> Option<String> {
    for cur in CURRENCIES {
        for &cur_name in cur.names {
            if input.ends_with(cur_name) {
                let num_part = input[..input.len() - cur_name.len()].trim();
                if num_part.is_empty() {
                    continue;
                }
                let num = cardinal::words_to_number(num_part)?;
                return Some(format_with_symbol(cur, &num.to_string()));
            }
        }
    }

    None
}

/// Parse cents-only: "ein cent" → "€0,01"
/// Note: bare "cent" defaults to euro (€)
fn parse_cents_only(input: &str) -> Option<String> {
    // Check each currency's cent names
    // But bare "cent" (without matching a specific currency) defaults to euro
    for &cent_name in &["cent", "cents"] {
        if input.ends_with(cent_name) {
            let num_part = input[..input.len() - cent_name.len()].trim();
            if num_part.is_empty() {
                continue;
            }
            let num = cardinal::words_to_number(num_part)?;

            if num >= 100 {
                // "einhundert cent" → "100 cent"
                return Some(format!("{} {}", num, cent_name));
            }

            // Default to euro for bare cent
            return Some(format!("€0,{:02}", num));
        }
    }

    // Check for pence/penny → £
    for &cent_name in &["pence", "penny"] {
        if input.ends_with(cent_name) {
            let num_part = input[..input.len() - cent_name.len()].trim();
            if num_part.is_empty() {
                continue;
            }
            let num = cardinal::words_to_number(num_part)?;

            if num >= 100 {
                return Some(format!("{} {}", num, cent_name));
            }

            return Some(format!("£0,{:02}", num));
        }
    }

    None
}

/// Format amount with currency symbol
fn format_with_symbol(cur: &Currency, amount: &str) -> String {
    if cur.prefix {
        format!("{}{}", cur.symbol, amount)
    } else {
        format!("{}{}", cur.symbol, amount)
    }
}

/// Parse decimal digit words: "null null" → "00", "null eins" → "01"
fn parse_decimal_digits(input: &str) -> Option<String> {
    let digit_map = [
        ("null", "0"), ("eins", "1"), ("ein", "1"),
        ("zwei", "2"), ("drei", "3"), ("vier", "4"),
        ("fünf", "5"), ("sechs", "6"), ("sieben", "7"),
        ("acht", "8"), ("neun", "9"),
    ];

    let tokens: Vec<&str> = input.split_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }

    let mut result = String::new();
    for token in &tokens {
        let mut found = false;
        for &(word, digit) in &digit_map {
            if token == &word {
                result.push_str(digit);
                found = true;
                break;
            }
        }
        if !found {
            return None;
        }
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        assert_eq!(parse("zwei dollar"), Some("$2".to_string()));
        assert_eq!(parse("ein dollar"), Some("$1".to_string()));
    }

    #[test]
    fn test_with_cents() {
        assert_eq!(parse("zwei euro und zwanzig cent"), Some("€2,20".to_string()));
        assert_eq!(parse("zwei dollar und zwanzig cent"), Some("$2,20".to_string()));
    }

    #[test]
    fn test_cents_only() {
        assert_eq!(parse("ein cent"), Some("€0,01".to_string()));
        assert_eq!(parse("zwanzig cent"), Some("€0,20".to_string()));
    }

    #[test]
    fn test_scale() {
        assert_eq!(parse("eine million dollar"), Some("$1 million".to_string()));
        assert_eq!(parse("zwei millionen euro"), Some("€2 millionen".to_string()));
    }
}
