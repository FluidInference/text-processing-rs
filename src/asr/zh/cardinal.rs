//! Cardinal number tagger for Chinese.
//!
//! Converts Chinese numerals to Arabic numerals:
//! - "一百" → "100"
//! - "一万" → "1万"
//! - "九千九百九十九" → "9,999"
//! - "一億零一萬一千一百一十一" → "100,011,111"
//!
//! Handles both simplified and traditional characters.
//! Numbers with only 万/億 scale and no sub-units preserve the scale character.

/// Map a single Chinese digit to its value.
/// Handles standard, traditional, and financial (大写) forms.
pub fn zh_digit(c: char) -> Option<i64> {
    match c {
        '零' | '〇' => Some(0),
        '一' | '壹' => Some(1),
        '二' | '两' | '兩' | '贰' | '貳' => Some(2),
        '三' | '叁' | '參' => Some(3),
        '四' | '肆' => Some(4),
        '五' | '伍' => Some(5),
        '六' | '陆' | '陸' => Some(6),
        '七' | '柒' => Some(7),
        '八' | '捌' => Some(8),
        '九' | '玖' => Some(9),
        _ => None,
    }
}

/// Check if a character is a Chinese numeral (digit or scale).
pub fn is_zh_numeral(c: char) -> bool {
    zh_digit(c).is_some() || is_scale(c)
}

/// Check if a character is a scale multiplier.
fn is_scale(c: char) -> bool {
    matches!(
        c,
        '十' | '拾' | '百' | '佰' | '千' | '仟' | '万' | '萬' | '亿' | '億'
    )
}

/// Scale value for a scale character.
fn scale_value(c: char) -> Option<i64> {
    match c {
        '十' | '拾' => Some(10),
        '百' | '佰' => Some(100),
        '千' | '仟' => Some(1000),
        '万' | '萬' => Some(10_000),
        '亿' | '億' => Some(100_000_000),
        _ => None,
    }
}

/// Check if char is 万 or 萬.
fn is_wan(c: char) -> bool {
    c == '万' || c == '萬'
}

/// Check if char is 亿 or 億.
fn is_yi(c: char) -> bool {
    c == '亿' || c == '億'
}

/// Parse a Chinese number string to an integer.
///
/// Handles the full Chinese number system including:
/// - Standard digits: 一二三四五六七八九
/// - Traditional: 壹贰叁肆伍陆柒捌玖
/// - Scales: 十百千万億
/// - 零 as placeholder between non-adjacent scales
/// - 两/兩 as alternate for 2
pub fn zh_to_number(input: &str) -> Option<i64> {
    let chars: Vec<char> = input.chars().collect();
    if chars.is_empty() {
        return None;
    }

    // All characters must be Chinese numerals
    if !chars.iter().all(|&c| is_zh_numeral(c)) {
        return None;
    }

    // Reject if the input is solely 万/萬/亿/億 (which appear in formatted output)
    if chars.iter().all(|&c| is_wan(c) || is_yi(c)) {
        return None;
    }

    let mut result: i64 = 0;
    let mut i = 0;

    // Process 億 group
    if let Some(pos) = chars.iter().position(|&c| is_yi(c)) {
        let group = if pos == 0 {
            1
        } else {
            parse_sub_yi(&chars[..pos])?
        };
        result += group * 100_000_000;
        i = pos + 1;

        // Skip 零 after 億
        if i < chars.len() && chars[i] == '零' {
            i += 1;
        }
    }

    // Process 万 group
    let remaining = &chars[i..];
    if let Some(pos) = remaining.iter().position(|&c| is_wan(c)) {
        let group = if pos == 0 {
            1
        } else {
            parse_sub_wan(&remaining[..pos])?
        };
        result += group * 10_000;
        i += pos + 1;

        // Skip 零 after 万
        if i < chars.len() && chars[i] == '零' {
            i += 1;
        }
    }

    // Process remaining (0-9999)
    let remaining = &chars[i..];
    if !remaining.is_empty() {
        result += parse_sub_wan(remaining)?;
    }

    if result == 0 && !chars.iter().any(|&c| c == '零' || c == '〇') {
        if chars.is_empty() {
            return None;
        }
    }

    Some(result)
}

/// Parse a sub-万 number (0-9999): 千百十 scale.
fn parse_sub_wan(chars: &[char]) -> Option<i64> {
    if chars.is_empty() {
        return None;
    }

    // Handle single zero
    if chars.len() == 1 && chars[0] == '零' {
        return Some(0);
    }

    let mut result: i64 = 0;
    let mut i = 0;

    // Skip leading 零
    if i < chars.len() && chars[i] == '零' {
        i += 1;
    }

    // Process 千
    if let Some(pos) = chars[i..].iter().position(|&c| c == '千' || c == '仟') {
        let pos = pos + i;
        let multiplier = if pos == i {
            1 // bare 千
        } else if pos == i + 1 {
            zh_digit(chars[i])?
        } else {
            return None;
        };
        result += multiplier * 1000;
        i = pos + 1;

        // Skip 零
        if i < chars.len() && chars[i] == '零' {
            i += 1;
        }
    }

    // Process 百
    if i < chars.len() {
        if let Some(pos) = chars[i..].iter().position(|&c| c == '百' || c == '佰') {
            let pos = pos + i;
            let multiplier = if pos == i {
                1 // bare 百
            } else if pos == i + 1 {
                zh_digit(chars[i])?
            } else {
                return None;
            };
            result += multiplier * 100;
            i = pos + 1;

            // Skip 零
            if i < chars.len() && chars[i] == '零' {
                i += 1;
            }
        }
    }

    // Process 十
    if i < chars.len() {
        if let Some(pos) = chars[i..].iter().position(|&c| c == '十' || c == '拾') {
            let pos = pos + i;
            let multiplier = if pos == i {
                1 // bare 十
            } else if pos == i + 1 {
                zh_digit(chars[i])?
            } else {
                return None;
            };
            result += multiplier * 10;
            i = pos + 1;
        }
    }

    // Process remaining digit
    if i < chars.len() {
        if chars.len() - i == 1 {
            if chars[i] == '零' {
                // trailing zero, ignore
            } else {
                result += zh_digit(chars[i])?;
            }
        } else {
            return None; // unexpected extra characters
        }
    }

    Some(result)
}

/// Parse a sub-億 number (up to 9999万9999).
/// This handles the range above 万 but below 億.
fn parse_sub_yi(chars: &[char]) -> Option<i64> {
    if chars.is_empty() {
        return None;
    }

    let mut result: i64 = 0;
    let mut i = 0;

    // Process 万 group within the 億 group
    if let Some(pos) = chars.iter().position(|&c| is_wan(c)) {
        let group = if pos == 0 {
            1
        } else {
            parse_sub_wan(&chars[..pos])?
        };
        result += group * 10_000;
        i = pos + 1;

        // Skip 零 after 万
        if i < chars.len() && chars[i] == '零' {
            i += 1;
        }
    }

    // Process remaining sub-万
    let remaining = &chars[i..];
    if !remaining.is_empty() {
        result += parse_sub_wan(remaining)?;
    }

    Some(result)
}

/// Format a number with comma separators.
pub fn format_with_commas(n: i64) -> String {
    if n == 0 {
        return "0".to_string();
    }

    let negative = n < 0;
    let mut num = if negative { (n as i128).abs() as u64 } else { n as u64 };
    let mut groups: Vec<String> = Vec::new();

    while num > 0 {
        let group = num % 1000;
        groups.push(group.to_string());
        num /= 1000;
    }

    groups.reverse();

    if groups.is_empty() {
        return "0".to_string();
    }

    let mut result = groups[0].clone();
    for g in &groups[1..] {
        result.push(',');
        result.push_str(&format!("{:03}", g.parse::<u64>().unwrap()));
    }

    if negative {
        format!("-{}", result)
    } else {
        result
    }
}

/// Determine the output format for a Chinese number expression.
///
/// Chinese cardinal output follows these rules:
/// - If the number has only a 万/億-scale with no sub-units, preserve the scale char:
///   "一万" → "1万", "十万" → "10万", "一百万" → "100万"
///   "一億" → "1億", "十億" → "10億"
/// - If the number has sub-万 digits after 万, expand fully with commas:
///   "一万一千" → "11,000", "九十万五千八百二十五" → "905,825"
/// - If the number has sub-億 digits after 億 that go below 万, expand fully.
///
/// Returns the formatted string.
pub fn format_zh_cardinal(input: &str) -> Option<String> {
    let chars: Vec<char> = input.chars().collect();
    if chars.is_empty() {
        return None;
    }

    if !chars.iter().all(|&c| is_zh_numeral(c)) {
        return None;
    }

    // Reject if the input is solely 万/萬/亿/億 (which appear in formatted output)
    if chars.iter().all(|&c| is_wan(c) || is_yi(c)) {
        return None;
    }

    // Find 億 and 万 positions
    let yi_pos = chars.iter().position(|&c| is_yi(c));
    let wan_pos_after_yi = if let Some(yp) = yi_pos {
        chars[yp + 1..].iter().position(|&c| is_wan(c)).map(|p| p + yp + 1)
    } else {
        chars.iter().position(|&c| is_wan(c))
    };

    // Determine if we have sub-units after the highest scale
    let has_yi = yi_pos.is_some();
    let has_wan = wan_pos_after_yi.is_some();

    if has_yi {
        let yp = yi_pos.unwrap();
        let yi_char = chars[yp]; // preserve original 億/亿
        let yi_multiplier_chars = &chars[..yp];

        // Parse the 億 multiplier
        let yi_mult = if yi_multiplier_chars.is_empty() {
            1
        } else {
            parse_sub_wan(yi_multiplier_chars)?
        };

        // Check what comes after 億
        let after_yi_start = yp + 1;
        let mut after_yi = &chars[after_yi_start..];

        // Skip 零
        if !after_yi.is_empty() && after_yi[0] == '零' {
            after_yi = &after_yi[1..];
        }

        if after_yi.is_empty() {
            // Pure 億 number: N億 (with commas in multiplier if ≥1000)
            let mult_str = if yi_mult >= 1000 {
                format_with_commas(yi_mult)
            } else {
                yi_mult.to_string()
            };
            return Some(format!("{}{}", mult_str, yi_char));
        }

        // Check if after_yi contains 万
        if let Some(wp) = after_yi.iter().position(|&c| is_wan(c)) {
            let wan_char = after_yi[wp];
            let wan_mult_chars = &after_yi[..wp];
            let wan_mult = if wan_mult_chars.is_empty() {
                1
            } else {
                parse_sub_wan(wan_mult_chars)?
            };

            let after_wan_start = wp + 1;
            let mut after_wan = &after_yi[after_wan_start..];

            // Skip 零
            if !after_wan.is_empty() && after_wan[0] == '零' {
                after_wan = &after_wan[1..];
            }

            if after_wan.is_empty() {
                // 億 + 万 only, no sub-万: use mixed format like "N億" but with 万 in between
                // Actually looking at test data, patterns like 一億一千萬 → 110,000,000
                // So when there's 億 AND 万, we always expand fully
                let total = yi_mult * 100_000_000 + wan_mult * 10_000;
                return Some(format_with_commas(total));
            }

            // Has sub-万 digits
            let sub_wan = parse_sub_wan(after_wan)?;
            let total = yi_mult * 100_000_000 + wan_mult * 10_000 + sub_wan;
            return Some(format_with_commas(total));
        }

        // After 億 with no 万 — just sub-万 digits
        let sub_wan = parse_sub_wan(after_yi)?;
        let total = yi_mult * 100_000_000 + sub_wan;
        return Some(format_with_commas(total));
    }

    if has_wan {
        let wp = wan_pos_after_yi.unwrap();
        let wan_char = chars[wp]; // preserve original 万/萬
        let wan_mult_chars = &chars[..wp];

        let wan_mult = if wan_mult_chars.is_empty() {
            1
        } else {
            parse_sub_wan(wan_mult_chars)?
        };

        let after_wan_start = wp + 1;
        let mut after_wan = &chars[after_wan_start..];

        // Skip 零
        if !after_wan.is_empty() && after_wan[0] == '零' {
            after_wan = &after_wan[1..];
        }

        if after_wan.is_empty() {
            // Pure 万 number: N万 (with commas in multiplier if ≥1000)
            let mult_str = if wan_mult >= 1000 {
                format_with_commas(wan_mult)
            } else {
                wan_mult.to_string()
            };
            return Some(format!("{}{}", mult_str, wan_char));
        }

        // Has sub-万 digits — expand fully
        let sub_wan = parse_sub_wan(after_wan)?;
        let total = wan_mult * 10_000 + sub_wan;
        return Some(format_with_commas(total));
    }

    // No 万 or 億 — plain number
    let num = parse_sub_wan(&chars)?;
    Some(format_with_commas(num))
}

/// Format for ordinals: same as cardinal but no commas in expanded numbers,
/// and 万/億 multipliers are plain (no commas either).
pub fn format_zh_ordinal(input: &str) -> Option<String> {
    let chars: Vec<char> = input.chars().collect();
    if chars.is_empty() {
        return None;
    }

    if !chars.iter().all(|&c| is_zh_numeral(c)) {
        return None;
    }

    // Find 億 and 万 positions
    let yi_pos = chars.iter().position(|&c| is_yi(c));
    let wan_pos_after_yi = if let Some(yp) = yi_pos {
        chars[yp + 1..].iter().position(|&c| is_wan(c)).map(|p| p + yp + 1)
    } else {
        chars.iter().position(|&c| is_wan(c))
    };

    let has_yi = yi_pos.is_some();
    let has_wan = wan_pos_after_yi.is_some();

    if has_yi {
        let yp = yi_pos.unwrap();
        let yi_char = chars[yp];
        let yi_mult = if yp == 0 {
            1
        } else {
            parse_sub_wan(&chars[..yp])?
        };

        let mut after_yi = &chars[yp + 1..];
        if !after_yi.is_empty() && after_yi[0] == '零' {
            after_yi = &after_yi[1..];
        }

        if after_yi.is_empty() {
            return Some(format!("{}{}", yi_mult, yi_char));
        }

        // Has stuff after 億 — expand fully (no commas)
        let total = zh_to_number(input)?;
        return Some(total.to_string());
    }

    if has_wan {
        let wp = wan_pos_after_yi.unwrap();
        let wan_char = chars[wp];
        let wan_mult = if wp == 0 {
            1
        } else {
            parse_sub_wan(&chars[..wp])?
        };

        let mut after_wan = &chars[wp + 1..];
        if !after_wan.is_empty() && after_wan[0] == '零' {
            after_wan = &after_wan[1..];
        }

        if after_wan.is_empty() {
            return Some(format!("{}{}", wan_mult, wan_char));
        }

        // Has sub-万 digits — expand fully (no commas)
        let total = zh_to_number(input)?;
        return Some(total.to_string());
    }

    // No 万 or 億 — plain number
    let num = parse_sub_wan(&chars)?;
    Some(num.to_string())
}

/// Format for money: no commas, no 万-preservation. Plain number output.
pub fn format_zh_money(input: &str) -> Option<String> {
    let num = zh_to_number(input)?;
    Some(num.to_string())
}

/// Find and replace Chinese number spans in a string.
pub fn replace_zh_numbers(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut result = String::new();
    let mut i = 0;

    while i < chars.len() {
        if is_zh_numeral(chars[i]) {
            let start = i;
            while i < chars.len() && is_zh_numeral(chars[i]) {
                i += 1;
            }
            let span: String = chars[start..i].iter().collect();
            if let Some(formatted) = format_zh_cardinal(&span) {
                result.push_str(&formatted);
            } else {
                result.push_str(&span);
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(format_zh_cardinal("一百"), Some("100".to_string()));
        assert_eq!(format_zh_cardinal("两百"), Some("200".to_string()));
        assert_eq!(format_zh_cardinal("九百五十一"), Some("951".to_string()));
    }

    #[test]
    fn test_wan_preserved() {
        assert_eq!(format_zh_cardinal("一万"), Some("1万".to_string()));
        assert_eq!(format_zh_cardinal("十万"), Some("10万".to_string()));
        assert_eq!(format_zh_cardinal("一百万"), Some("100万".to_string()));
    }

    #[test]
    fn test_wan_expanded() {
        assert_eq!(
            format_zh_cardinal("一万一千"),
            Some("11,000".to_string())
        );
        assert_eq!(
            format_zh_cardinal("九千九百九十九"),
            Some("9,999".to_string())
        );
    }

    #[test]
    fn test_yi() {
        assert_eq!(format_zh_cardinal("一億"), Some("1億".to_string()));
        assert_eq!(
            format_zh_cardinal("一億一千萬"),
            Some("110,000,000".to_string())
        );
    }

    #[test]
    fn test_traditional() {
        assert_eq!(format_zh_cardinal("十萬"), Some("10萬".to_string()));
    }

    #[test]
    fn test_commas() {
        assert_eq!(format_with_commas(1), "1");
        assert_eq!(format_with_commas(1000), "1,000");
        assert_eq!(format_with_commas(905825), "905,825");
    }
}
