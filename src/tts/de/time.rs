//! Time TN tagger for German.
//!
//! Converts written time expressions to spoken German:
//! - "14:30" → "vierzehn uhr dreissig"
//! - "2:00" → "zwei uhr"
//! - "0:00" → "null uhr"
//! - "12:00" → "zwoelf uhr"

use super::number_to_words;

/// Parse a written time expression to spoken German.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // Try "14:30" format
    if let Some(result) = parse_colon_format(trimmed) {
        return Some(result);
    }

    // Try "14 Uhr 30" or "14 Uhr" format (German convention)
    if let Some(result) = parse_uhr_format(trimmed) {
        return Some(result);
    }

    None
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

fn parse_uhr_format(input: &str) -> Option<String> {
    let lower = input.to_lowercase();

    // Match "14 uhr 30" or "14 uhr"
    let uhr_pos = lower.find(" uhr")?;
    let hour_str = &lower[..uhr_pos].trim();
    let after_uhr = &lower[uhr_pos + 4..].trim();

    if !hour_str.chars().all(|c| c.is_ascii_digit()) || hour_str.is_empty() {
        return None;
    }

    let hour: u32 = hour_str.parse().ok()?;
    if hour > 23 {
        return None;
    }

    let minute: u32 = if after_uhr.is_empty() {
        0
    } else {
        if !after_uhr.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        let m: u32 = after_uhr.parse().ok()?;
        if m > 59 {
            return None;
        }
        m
    };

    Some(format_time(hour, minute))
}

fn format_time(hour: u32, minute: u32) -> String {
    // Special cases
    if hour == 0 && minute == 0 {
        return "mitternacht".to_string();
    }
    if hour == 12 && minute == 0 {
        return "mittag".to_string();
    }

    let hour_words = number_to_words(hour as i64);

    if minute == 0 {
        format!("{} uhr", hour_words)
    } else {
        format!(
            "{} uhr {}",
            hour_words,
            number_to_words(minute as i64)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colon_format() {
        assert_eq!(
            parse("14:30"),
            Some("vierzehn uhr dreissig".to_string())
        );
        assert_eq!(parse("2:00"), Some("zwei uhr".to_string()));
        assert_eq!(parse("8:15"), Some("acht uhr fuenfzehn".to_string()));
    }

    #[test]
    fn test_uhr_format() {
        assert_eq!(
            parse("14 Uhr 30"),
            Some("vierzehn uhr dreissig".to_string())
        );
        assert_eq!(parse("8 Uhr"), Some("acht uhr".to_string()));
    }

    #[test]
    fn test_special_hours() {
        assert_eq!(parse("0:00"), Some("mitternacht".to_string()));
        assert_eq!(parse("12:00"), Some("mittag".to_string()));
        assert_eq!(
            parse("0:30"),
            Some("null uhr dreissig".to_string())
        );
    }

    #[test]
    fn test_24h() {
        assert_eq!(
            parse("14:00"),
            Some("vierzehn uhr".to_string())
        );
        assert_eq!(
            parse("23:59"),
            Some("dreiundzwanzig uhr neunundfuenfzig".to_string())
        );
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("25:00"), None);
        assert_eq!(parse("12:60"), None);
    }
}
