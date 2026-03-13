//! Money tagger for Chinese.
//!
//! Converts Chinese currency expressions to symbolic form:
//! - "一千美元" → "US$1000"
//! - "一千元" → "¥1000"
//! - "一万美元" → "US$1万"
//! - "一点五万美元" → "US$1.5万"
//! - "一千万美元" → "US$1000万"

use super::cardinal;

/// Currency mapping: (Chinese name, symbol)
/// Order matters: longer names first to avoid partial matches.
/// "元" must be last since it's a suffix of "美元", "欧元", "日元", "韩元".
const CURRENCIES: &[(&str, &str)] = &[
    ("印度卢布", "₹"),
    ("美元", "US$"),
    ("欧元", "€"),
    ("英镑", "£"),
    ("韩元", "₩"),
    ("日元", "JPY¥"),
    ("元", "¥"),
];

/// Process money patterns in a string.
pub fn process(input: &str) -> String {
    let mut result = input.to_string();

    for &(name, symbol) in CURRENCIES {
        result = process_currency(&result, name, symbol);
    }

    result
}

/// Process a single currency: find Chinese number + currency name and replace.
fn process_currency(input: &str, currency_name: &str, symbol: &str) -> String {
    let mut result = String::new();
    let mut remaining = input;

    while let Some(pos) = remaining.find(currency_name) {
        let before = &remaining[..pos];
        let before_chars: Vec<char> = before.chars().collect();

        // For 元: skip if preceded by 公 or 纪 (公元, 公元前, 纪元)
        if currency_name == "元" {
            if before.ends_with('公') || before.ends_with('纪') {
                result.push_str(&remaining[..pos + currency_name.len()]);
                remaining = &remaining[pos + currency_name.len()..];
                continue;
            }
        }

        // Scan backwards for Chinese numerals or decimal point characters
        let mut num_start = before_chars.len();
        while num_start > 0 {
            let c = before_chars[num_start - 1];
            if cardinal::is_zh_numeral(c) || c == '点' || c == '點' {
                num_start -= 1;
            } else {
                break;
            }
        }

        if num_start < before_chars.len() {
            let prefix: String = before_chars[..num_start].iter().collect();
            let number_chars: String = before_chars[num_start..].iter().collect();

            result.push_str(&prefix);

            // Check if it contains a decimal point
            if number_chars.contains('点') || number_chars.contains('點') {
                if let Some(formatted) = format_money_decimal(&number_chars) {
                    result.push_str(&format!("{}{}", symbol, formatted));
                } else {
                    result.push_str(&format!("{}{}", symbol, number_chars));
                }
            } else {
                // Format for money: 万-preservation, no commas
                if let Some(formatted) = format_money_cardinal(&number_chars) {
                    result.push_str(&format!("{}{}", symbol, formatted));
                } else {
                    result.push_str(&format!("{}{}", symbol, number_chars));
                }
            }
        } else {
            result.push_str(before);
        }

        remaining = &remaining[pos + currency_name.len()..];
    }

    result.push_str(remaining);
    result
}

/// Format a cardinal number for money: 万-preservation, no commas.
/// - "一千" → "1000"
/// - "一万" → "1万"
/// - "一千万" → "1000万"
/// - "五十万" → "50万"
fn format_money_cardinal(input: &str) -> Option<String> {
    let chars: Vec<char> = input.chars().collect();
    if chars.is_empty() || !chars.iter().all(|&c| cardinal::is_zh_numeral(c)) {
        return None;
    }

    // Find 万 position
    let wan_pos = chars.iter().position(|&c| c == '万' || c == '萬');

    if let Some(wp) = wan_pos {
        let wan_char = chars[wp];
        let wan_mult = if wp == 0 {
            1
        } else {
            cardinal::zh_to_number(&chars[..wp].iter().collect::<String>())?
        };

        let mut after_wan = &chars[wp + 1..];
        if !after_wan.is_empty() && after_wan[0] == '零' {
            after_wan = &after_wan[1..];
        }

        if after_wan.is_empty() {
            // Pure 万: N万 (no commas in multiplier)
            return Some(format!("{}{}", wan_mult, wan_char));
        }

        // Has sub-万: expand fully without commas
        let total = cardinal::zh_to_number(input)?;
        return Some(total.to_string());
    }

    // No 万 — plain number without commas
    let num = cardinal::zh_to_number(input)?;
    Some(num.to_string())
}

/// Format a decimal number for money display.
/// e.g., "一点五万" → "1.5万"
fn format_money_decimal(input: &str) -> Option<String> {
    let dian_pos = input.find('点').or_else(|| input.find('點'))?;
    let dian_char = if input.contains('点') { '点' } else { '點' };

    let int_part = &input[..dian_pos];
    let after_dian = &input[dian_pos + dian_char.len_utf8()..];

    // Parse integer part
    let int_chars: Vec<char> = int_part.chars().collect();
    if int_chars.is_empty() || !int_chars.iter().all(|&c| cardinal::is_zh_numeral(c)) {
        return None;
    }
    let int_val = cardinal::zh_to_number(&int_chars.iter().collect::<String>())?;

    // Parse fractional part — check if it ends with 万/萬
    let after_chars: Vec<char> = after_dian.chars().collect();
    if after_chars.is_empty() {
        return None;
    }

    let last_char = *after_chars.last().unwrap();
    if last_char == '万' || last_char == '萬' {
        let frac_chars = &after_chars[..after_chars.len() - 1];
        let frac_digits: String = frac_chars
            .iter()
            .filter_map(|&c| cardinal::zh_digit(c).map(|d| d.to_string()))
            .collect();
        if frac_digits.is_empty() {
            return None;
        }
        Some(format!("{}.{}{}", int_val, frac_digits, last_char))
    } else {
        let frac_digits: String = after_chars
            .iter()
            .filter_map(|&c| cardinal::zh_digit(c).map(|d| d.to_string()))
            .collect();
        if frac_digits.is_empty() {
            return None;
        }
        Some(format!("{}.{}", int_val, frac_digits))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usd() {
        assert_eq!(process("一千美元"), "US$1000");
        assert_eq!(process("一万美元"), "US$1万");
        assert_eq!(process("一点五万美元"), "US$1.5万");
        assert_eq!(process("一千万美元"), "US$1000万");
    }

    #[test]
    fn test_cny() {
        assert_eq!(process("一千元"), "¥1000");
        assert_eq!(process("一万元"), "¥1万");
    }

    #[test]
    fn test_jpy() {
        assert_eq!(process("一千日元"), "JPY¥1000");
    }

    #[test]
    fn test_skip_gongyuan() {
        // 公元 should not match 元 currency
        assert_eq!(process("公元"), "公元");
    }
}
