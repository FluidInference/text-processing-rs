//! Time TN tagger for Mandarin Chinese.
//!
//! Converts written time expressions to spoken Mandarin pinyin:
//! - "14:30" -> "shi si dian san shi fen"
//! - "3:05" -> "san dian ling wu fen"
//! - "12:00" -> "shi er dian zheng"
//!
//! Format: HOUR "dian" MINUTES "fen"

use super::number_to_words;

/// Parse a written time expression to spoken Mandarin pinyin.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // Try Chinese format: 14时30分 or 14点30分
    if let Some(result) = parse_chinese_format(trimmed) {
        return Some(result);
    }

    // Try "14:30" colon format
    if let Some(result) = parse_colon_format(trimmed) {
        return Some(result);
    }

    None
}

fn parse_chinese_format(input: &str) -> Option<String> {
    // 时 (shi, U+65F6) or 点 (dian, U+70B9) as hour marker
    let shi_char = '\u{65F6}'; // 时
    let dian_char = '\u{70B9}'; // 点
    let fen_char = '\u{5206}'; // 分

    let hour_sep_pos = input.find(shi_char).or_else(|| input.find(dian_char));
    let hour_sep_pos = hour_sep_pos?;

    let hour_str = &input[..hour_sep_pos];
    if hour_str.is_empty() || !hour_str.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let hour: u32 = hour_str.parse().ok()?;
    if hour > 23 {
        return None;
    }

    // Find the separator character to know its byte length
    let sep_char = input[hour_sep_pos..].chars().next()?;
    let after_sep = &input[hour_sep_pos + sep_char.len_utf8()..];

    let min_str = after_sep.trim_end_matches(fen_char).trim();
    let minute: u32 = if min_str.is_empty() {
        0
    } else {
        if !min_str.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        let m: u32 = min_str.parse().ok()?;
        if m > 59 {
            return None;
        }
        m
    };

    Some(format_time(hour, minute))
}

fn parse_colon_format(input: &str) -> Option<String> {
    if !input.contains(':') {
        return None;
    }

    let parts: Vec<&str> = input.splitn(2, ':').collect();
    if parts.len() != 2 {
        return None;
    }

    let hour_str = parts[0].trim();
    let min_str = parts[1].trim();

    if !hour_str.chars().all(|c| c.is_ascii_digit()) || hour_str.is_empty() {
        return None;
    }
    if !min_str.chars().all(|c| c.is_ascii_digit()) || min_str.is_empty() {
        return None;
    }

    let hour: u32 = hour_str.parse().ok()?;
    let minute: u32 = min_str.parse().ok()?;

    if hour > 23 || minute > 59 {
        return None;
    }

    Some(format_time(hour, minute))
}

fn format_time(hour: u32, minute: u32) -> String {
    let hour_words = number_to_words(hour as i64);

    if minute == 0 {
        format!("{} dian zheng", hour_words)
    } else if minute < 10 {
        // "ling" as placeholder for single-digit minutes: san dian ling wu fen
        format!(
            "{} dian ling {} fen",
            hour_words,
            number_to_words(minute as i64)
        )
    } else {
        format!("{} dian {} fen", hour_words, number_to_words(minute as i64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colon_format() {
        assert_eq!(parse("14:30"), Some("shi si dian san shi fen".to_string()));
        assert_eq!(parse("3:05"), Some("san dian ling wu fen".to_string()));
        assert_eq!(parse("12:00"), Some("shi er dian zheng".to_string()));
        assert_eq!(parse("0:00"), Some("ling dian zheng".to_string()));
    }

    #[test]
    fn test_chinese_format() {
        assert_eq!(
            parse("14\u{65F6}30\u{5206}"),
            Some("shi si dian san shi fen".to_string())
        );
        assert_eq!(
            parse("8\u{70B9}15\u{5206}"),
            Some("ba dian shi wu fen".to_string())
        );
    }

    #[test]
    fn test_24h() {
        assert_eq!(parse("14:00"), Some("shi si dian zheng".to_string()));
        assert_eq!(
            parse("23:59"),
            Some("er shi san dian wu shi jiu fen".to_string())
        );
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("25:00"), None);
        assert_eq!(parse("12:60"), None);
        assert_eq!(parse(""), None);
    }
}
