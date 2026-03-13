//! Cardinal number tagger for Japanese.
//!
//! Converts kanji numerals to Arabic numerals:
//! - "一" → "1"
//! - "五千億" → "500,000,000,000"
//! - "十一兆一" → "11,000,000,000,001"

/// Map a single kanji digit to its value.
pub fn kanji_digit(c: char) -> Option<i64> {
    match c {
        '零' | '〇' => Some(0),
        '一' => Some(1),
        '二' => Some(2),
        '三' => Some(3),
        '四' => Some(4),
        '五' => Some(5),
        '六' => Some(6),
        '七' => Some(7),
        '八' => Some(8),
        '九' => Some(9),
        _ => None,
    }
}

/// Check if a character is a kanji numeral (digit or scale).
pub fn is_kanji_numeral(c: char) -> bool {
    kanji_digit(c).is_some() || matches!(c, '十' | '百' | '千' | '万' | '億' | '兆')
}

/// Parse a kanji number string to an integer.
///
/// Handles the full Japanese number system:
/// - Scale: 兆(10^12), 億(10^8), 万(10^4)
/// - Within each group: 千(1000), 百(100), 十(10) + digits
///
/// Examples:
/// - "一" → 1
/// - "二十" → 20
/// - "百" → 100
/// - "千九百九十九" → 1999
/// - "五千億" → 500_000_000_000
/// - "一兆百万" → 1_000_001_000_000
pub fn kanji_to_number(input: &str) -> Option<i64> {
    let chars: Vec<char> = input.chars().collect();
    if chars.is_empty() {
        return None;
    }

    // All characters must be kanji numerals
    if !chars.iter().all(|&c| is_kanji_numeral(c)) {
        return None;
    }

    let mut result: i64 = 0;
    let mut i = 0;

    // Process 兆 group
    if let Some(pos) = chars.iter().position(|&c| c == '兆') {
        let group = if pos == 0 { 1 } else { parse_sub_man(&chars[..pos])? };
        result += group * 1_000_000_000_000;
        i = pos + 1;
    }

    // Process 億 group
    let remaining = &chars[i..];
    if let Some(pos) = remaining.iter().position(|&c| c == '億') {
        let group = if pos == 0 { 1 } else { parse_sub_man(&remaining[..pos])? };
        result += group * 100_000_000;
        i += pos + 1;
    }

    // Process 万 group
    let remaining = &chars[i..];
    if let Some(pos) = remaining.iter().position(|&c| c == '万') {
        let group = if pos == 0 { 1 } else { parse_sub_man(&remaining[..pos])? };
        result += group * 10_000;
        i += pos + 1;
    }

    // Process remaining (0-9999)
    let remaining = &chars[i..];
    if !remaining.is_empty() {
        result += parse_sub_man(remaining)?;
    }

    if result == 0 && !chars.iter().any(|&c| c == '零' || c == '〇') {
        // Didn't parse anything meaningful
        if chars.is_empty() {
            return None;
        }
    }

    Some(result)
}

/// Parse a sub-万 number (0-9999): 千百十 scale.
fn parse_sub_man(chars: &[char]) -> Option<i64> {
    if chars.is_empty() {
        return None;
    }

    let mut result: i64 = 0;
    let mut i = 0;

    // Process 千
    if let Some(pos) = chars[i..].iter().position(|&c| c == '千') {
        let pos = pos + i;
        let multiplier = if pos == i {
            1 // bare 千
        } else if pos == i + 1 {
            kanji_digit(chars[i])?
        } else {
            return None;
        };
        result += multiplier * 1000;
        i = pos + 1;
    }

    // Process 百
    if i < chars.len() {
        if let Some(pos) = chars[i..].iter().position(|&c| c == '百') {
            let pos = pos + i;
            let multiplier = if pos == i {
                1 // bare 百
            } else if pos == i + 1 {
                kanji_digit(chars[i])?
            } else {
                return None;
            };
            result += multiplier * 100;
            i = pos + 1;
        }
    }

    // Process 十
    if i < chars.len() {
        if let Some(pos) = chars[i..].iter().position(|&c| c == '十') {
            let pos = pos + i;
            let multiplier = if pos == i {
                1 // bare 十
            } else if pos == i + 1 {
                kanji_digit(chars[i])?
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
            result += kanji_digit(chars[i])?;
        } else {
            return None; // unexpected extra characters
        }
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

    // First group has no leading zeros
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

/// Find and replace kanji number spans in a string.
/// Returns the string with all kanji number sequences replaced by Arabic numerals.
pub fn replace_kanji_numbers(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut result = String::new();
    let mut i = 0;

    while i < chars.len() {
        if is_kanji_numeral(chars[i]) {
            // Find the end of the kanji numeral span
            let start = i;
            while i < chars.len() && is_kanji_numeral(chars[i]) {
                i += 1;
            }
            let kanji_span: String = chars[start..i].iter().collect();
            if let Some(num) = kanji_to_number(&kanji_span) {
                result.push_str(&format_with_commas(num));
            } else {
                result.push_str(&kanji_span);
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
        assert_eq!(kanji_to_number("一"), Some(1));
        assert_eq!(kanji_to_number("百"), Some(100));
        assert_eq!(kanji_to_number("十"), Some(10));
        assert_eq!(kanji_to_number("二十"), Some(20));
    }

    #[test]
    fn test_large() {
        assert_eq!(kanji_to_number("五千億"), Some(500_000_000_000));
        assert_eq!(kanji_to_number("五兆"), Some(5_000_000_000_000));
        assert_eq!(kanji_to_number("一兆百万"), Some(1_000_001_000_000));
    }

    #[test]
    fn test_commas() {
        assert_eq!(format_with_commas(1), "1");
        assert_eq!(format_with_commas(100), "100");
        assert_eq!(format_with_commas(1000), "1,000");
        assert_eq!(format_with_commas(50000), "50,000");
        assert_eq!(format_with_commas(500_000_000_000), "500,000,000,000");
    }

    #[test]
    fn test_replace() {
        assert_eq!(replace_kanji_numbers("そこに鳥一羽がいます"), "そこに鳥1羽がいます");
    }
}
