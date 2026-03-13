//! Time tagger for Chinese.
//!
//! Converts Chinese time expressions to formatted form:
//! - "五点五分" → "05:05"
//! - "十三点五分十秒" → "13:05:10"
//! - "五点半" → "5点半"
//! - "五点一刻" → "5点1刻"
//! - "五分钟" → "5分钟"
//! - "五秒钟" → "5秒钟"
//!
//! Rules:
//! - X点Y分 → HH:MM (zero-padded)
//! - X点Y分Z秒 → HH:MM:SS (zero-padded)
//! - X点半 → N点半 (preserved, just convert digit)
//! - X点Y刻 → N点N刻 (preserved, just convert digit)
//! - X点 (alone) → N点 (preserved)
//! - X分钟 → N分钟, X秒钟 → N秒钟 (duration, just convert digit)

use super::cardinal;

/// Process time patterns in a string.
pub fn process(input: &str) -> String {
    let mut result = String::new();
    let mut remaining = input;

    while !remaining.is_empty() {
        if let Some((before, time_str, after)) = find_time_expr(remaining) {
            result.push_str(before);
            result.push_str(&time_str);
            remaining = after;
        } else {
            result.push_str(remaining);
            break;
        }
    }

    result
}

/// Find the next time expression in the string.
fn find_time_expr(input: &str) -> Option<(&str, String, &str)> {
    let chars: Vec<char> = input.chars().collect();
    let mut byte_pos = 0;

    for (i, &c) in chars.iter().enumerate() {
        // Look for 分钟 pattern (duration)
        if c == '分' && i > 0 {
            let after_fen = &input[byte_pos + c.len_utf8()..];
            if after_fen.starts_with('钟') {
                // X分钟 pattern
                let before_chars = &chars[..i];
                let mut num_start = before_chars.len();
                while num_start > 0 && cardinal::is_zh_numeral(before_chars[num_start - 1]) {
                    num_start -= 1;
                }
                if num_start < before_chars.len() {
                    let prefix_bytes: usize = chars[..num_start].iter().map(|c| c.len_utf8()).sum();
                    let kanji: String = before_chars[num_start..].iter().collect();
                    if let Some(num) = cardinal::zh_to_number(&kanji) {
                        let before = &input[..prefix_bytes];
                        let after = &input[byte_pos + c.len_utf8() + '钟'.len_utf8()..];
                        return Some((before, format!("{}分钟", num), after));
                    }
                }
            }
        }

        // Look for 秒钟 pattern (duration)
        if c == '秒' && i > 0 {
            let after_miao = &input[byte_pos + c.len_utf8()..];
            if after_miao.starts_with('钟') {
                let before_chars = &chars[..i];
                let mut num_start = before_chars.len();
                while num_start > 0 && cardinal::is_zh_numeral(before_chars[num_start - 1]) {
                    num_start -= 1;
                }
                if num_start < before_chars.len() {
                    let prefix_bytes: usize = chars[..num_start].iter().map(|c| c.len_utf8()).sum();
                    let kanji: String = before_chars[num_start..].iter().collect();
                    if let Some(num) = cardinal::zh_to_number(&kanji) {
                        let before = &input[..prefix_bytes];
                        let after = &input[byte_pos + c.len_utf8() + '钟'.len_utf8()..];
                        return Some((before, format!("{}秒钟", num), after));
                    }
                }
            }
        }

        // Look for 点 as time separator (X点Y分)
        if (c == '点' || c == '點') && i > 0 {
            // Check if preceded by Chinese numerals
            let before_chars = &chars[..i];
            let mut num_start = before_chars.len();
            while num_start > 0 && cardinal::is_zh_numeral(before_chars[num_start - 1]) {
                num_start -= 1;
            }

            if num_start < before_chars.len() {
                let hour_kanji: String = before_chars[num_start..].iter().collect();
                if let Some(hour) = cardinal::zh_to_number(&hour_kanji) {
                    let prefix_bytes: usize = chars[..num_start].iter().map(|c| c.len_utf8()).sum();
                    let after_dian = &chars[i + 1..];

                    // Check what follows 点
                    if let Some(time_result) = parse_after_dian(hour, after_dian) {
                        let before = &input[..prefix_bytes];
                        let consumed_bytes: usize = chars[num_start..i + 1 + time_result.1]
                            .iter()
                            .map(|c| c.len_utf8())
                            .sum();
                        let after = &input[prefix_bytes + consumed_bytes..];
                        return Some((before, time_result.0, after));
                    }
                }
            }
        }

        byte_pos += c.len_utf8();
    }

    None
}

/// Parse what comes after 点 in a time expression.
/// Returns (formatted_time, chars_consumed_after_dian).
fn parse_after_dian(hour: i64, after_dian: &[char]) -> Option<(String, usize)> {
    if after_dian.is_empty() {
        // X点 alone
        return Some((format!("{}点", hour), 0));
    }

    // Check for 半
    if after_dian[0] == '半' {
        return Some((format!("{}点半", hour), 1));
    }

    // Check for X刻
    let mut num_end = 0;
    while num_end < after_dian.len() && cardinal::is_zh_numeral(after_dian[num_end]) {
        num_end += 1;
    }

    if num_end > 0 && num_end < after_dian.len() && after_dian[num_end] == '刻' {
        let kanji: String = after_dian[..num_end].iter().collect();
        if let Some(quarter) = cardinal::zh_to_number(&kanji) {
            return Some((format!("{}点{}刻", hour, quarter), num_end + 1));
        }
    }

    // Check for Y分 (and optional Z秒)
    if num_end > 0 && num_end < after_dian.len() && after_dian[num_end] == '分' {
        let min_kanji: String = after_dian[..num_end].iter().collect();
        if let Some(minute) = cardinal::zh_to_number(&min_kanji) {
            let after_fen = &after_dian[num_end + 1..];

            // Check for seconds
            let mut sec_end = 0;
            while sec_end < after_fen.len() && cardinal::is_zh_numeral(after_fen[sec_end]) {
                sec_end += 1;
            }

            if sec_end > 0 && sec_end < after_fen.len() && after_fen[sec_end] == '秒' {
                let sec_kanji: String = after_fen[..sec_end].iter().collect();
                if let Some(second) = cardinal::zh_to_number(&sec_kanji) {
                    // HH:MM:SS
                    let total_consumed = num_end + 1 + sec_end + 1;
                    return Some((
                        format!("{:02}:{:02}:{:02}", hour, minute, second),
                        total_consumed,
                    ));
                }
            }

            // HH:MM only
            let total_consumed = num_end + 1;
            return Some((format!("{:02}:{:02}", hour, minute), total_consumed));
        }
    }

    // Check for 零Y分 pattern (e.g., 十三点零五分)
    if !after_dian.is_empty() && after_dian[0] == '零' {
        let rest = &after_dian[1..];
        let mut num_end2 = 0;
        while num_end2 < rest.len() && cardinal::is_zh_numeral(rest[num_end2]) {
            num_end2 += 1;
        }
        if num_end2 > 0 && num_end2 < rest.len() && rest[num_end2] == '分' {
            let min_kanji: String = rest[..num_end2].iter().collect();
            if let Some(minute) = cardinal::zh_to_number(&min_kanji) {
                let total_consumed = 1 + num_end2 + 1; // 零 + digits + 分
                return Some((format!("{:02}:{:02}", hour, minute), total_consumed));
            }
        }
    }

    // Check if what follows looks like decimal digits (not time)
    // If digits follow 点 without a time suffix, this is a decimal, not time
    if !after_dian.is_empty() && cardinal::zh_digit(after_dian[0]).is_some() {
        return None; // Let the decimal processor handle this
    }

    // X点 alone (no following digits or time suffixes)
    Some((format!("{}点", hour), 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hhmm() {
        assert_eq!(process("五点五分"), "05:05");
        assert_eq!(process("十三点五分"), "13:05");
    }

    #[test]
    fn test_hhmmss() {
        assert_eq!(process("一点五分十秒"), "01:05:10");
        assert_eq!(process("十三点五分十秒"), "13:05:10");
    }

    #[test]
    fn test_half() {
        assert_eq!(process("五点半"), "5点半");
    }

    #[test]
    fn test_quarter() {
        assert_eq!(process("五点一刻"), "5点1刻");
        assert_eq!(process("两点三刻"), "2点3刻");
    }

    #[test]
    fn test_alone() {
        assert_eq!(process("六点"), "6点");
        assert_eq!(process("十点"), "10点");
    }

    #[test]
    fn test_duration() {
        assert_eq!(process("五分钟"), "5分钟");
        assert_eq!(process("五秒钟"), "5秒钟");
    }
}
